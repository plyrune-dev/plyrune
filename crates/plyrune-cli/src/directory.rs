//! Linux acquisition of the fixture-v1 path and text profile. Only the explicit prefix is
//! enumerated. All data is read through no-follow descriptors; no lock file or writable source
//! is needed. An acquisition is not a filesystem transaction. The local mount and hierarchy
//! must be host-controlled, without hidden privileged metadata or hostile namespace changes.
use plyrune_core::fixture::{Entry, Fixture};
use rustix::fs::{self, AtFlags, Dir, FileType, Mode, OFlags, ResolveFlags};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, Metadata};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

const MAX_PATH: usize = 255;
const MAX_DEPTH: usize = 32;
const MAX_DIRECTORIES: usize = 256;
const MAX_FILES: usize = 256;
const MAX_ENTRIES: usize = 512;
const MAX_FILE: u64 = 8_388_608;
const MAX_TOTAL: u64 = 33_554_432;

#[derive(Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct FileMeta {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
}

impl FileMeta {
    fn from_stat(stat: &Metadata) -> Self {
        Self {
            dev: stat.dev(),
            ino: stat.ino(),
            mode: stat.mode() & 0o7777,
            uid: stat.uid(),
            gid: stat.gid(),
        }
    }

    fn hash(&self, hash: &mut Sha256) {
        hash.update(self.dev.to_be_bytes());
        hash.update(self.ino.to_be_bytes());
        hash.update(self.mode.to_be_bytes());
        hash.update(self.uid.to_be_bytes());
        hash.update(self.gid.to_be_bytes());
    }
}

// Access time is deliberately absent: reading the input may itself advance it.
#[derive(Debug, Eq, PartialEq)]
struct Version {
    size: u64,
    mtime: (i64, i64),
    ctime: (i64, i64),
}

impl Version {
    fn from_stat(stat: &Metadata) -> Self {
        Self {
            size: stat.len(),
            mtime: (stat.mtime(), stat.mtime_nsec()),
            ctime: (stat.ctime(), stat.ctime_nsec()),
        }
    }
}

#[derive(Debug)]
pub(super) struct SourceFile {
    pub bytes: Vec<u8>,
    pub meta: FileMeta,
    version: Version,
}

#[derive(Debug)]
pub(super) struct Directory {
    pub root: PathBuf,
    pub handle: File,
    pub prefix: String,
    pub files: BTreeMap<String, SourceFile>,
    // All prefix ancestors and scoped directories, including empty ones. No trailing slash.
    pub dirs: BTreeMap<String, FileMeta>,
    root_meta: FileMeta,
}

fn dir_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW
}

/// Do not `canonicalize` the caller's spelling before this open: a link, even in a root
/// ancestor, is not an authorized spelling. `NO_SYMLINKS` also excludes procfs magic links.
pub(super) fn open_directory(path: &Path) -> Result<File, String> {
    if path.components().any(|p| p.as_os_str() == ".git") {
        return Err("Git metadata is outside the filesystem edit profile".into());
    }
    fs::openat2(
        fs::CWD,
        path,
        dir_flags(),
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS,
    )
    .map(File::from)
    .map_err(|e| format!("open directory {path:?} without symlinks: {e}"))
}

fn open_child(parent: &File, name: &str, flags: OFlags) -> Result<File, String> {
    fs::openat2(
        parent,
        name,
        flags | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_XDEV,
    )
    .map(File::from)
    .map_err(|e| format!("open scoped entry {name:?}: {e}"))
}

/// Local Linux filesystems with ordinary inode, rename and advisory-lock semantics. A true
/// empty xattr list is required; absence of access to privileged namespaces is a host premise.
/// Physical allocation, extents and the directory index are storage details, not user flags.
pub(super) fn check_profile(file: &File) -> Result<(), String> {
    const EXT: fs::FsWord = 0xef53;
    const TMPFS: fs::FsWord = 0x0102_1994;
    const OVERLAY: fs::FsWord = 0x794c_7630;
    let fs_type = fs::fstatfs(file)
        .map_err(|e| format!("identify filesystem: {e}"))?
        .f_type;
    if !matches!(fs_type, EXT | TMPFS | OVERLAY) {
        return Err(format!("unsupported filesystem type {fs_type:#x}"));
    }
    if fs::flistxattr(file, &mut [0_u8; 0]).map_err(|e| format!("list extended attributes: {e}"))?
        != 0
    {
        return Err("extended attributes (including ACLs) are unsupported".into());
    }
    const EXTENTS: u32 = 0x0008_0000;
    const DIR_INDEX: u32 = 0x0000_1000;
    let flags = fs::ioctl_getflags(file)
        .map_err(|e| format!("inspect inode flags: {e}"))?
        .bits();
    if flags & !(EXTENTS | DIR_INDEX) != 0 {
        return Err(format!("unsupported inode flags {flags:#x}"));
    }
    Ok(())
}

fn valid_component(name: &str) -> bool {
    !matches!(name, "" | "." | ".." | ".git")
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}

fn scope(prefix: &str) -> Result<&str, String> {
    let path = prefix
        .strip_suffix('/')
        .ok_or("prefix requires a trailing slash")?;
    if prefix.len() > MAX_PATH || !path.split('/').all(valid_component) {
        return Err("prefix outside fixture-v1 relative path profile".into());
    }
    if path.split('/').count() > MAX_DEPTH {
        return Err("prefix exceeds directory depth budget".into());
    }
    Ok(path)
}

fn metadata(file: &File) -> Result<Metadata, String> {
    file.metadata()
        .map_err(|e| format!("read inode metadata: {e}"))
}

fn directory_meta(file: &File, root: bool) -> Result<FileMeta, String> {
    let meta = FileMeta::from_stat(&metadata(file)?);
    if !root && meta.mode & 0o7000 != 0 {
        return Err("special directory permission bits are unsupported".into());
    }
    check_profile(file)?;
    Ok(meta)
}

impl Directory {
    pub fn read(root: &Path, prefix: &str) -> Result<Self, String> {
        let path = scope(prefix)?;
        root.to_str().ok_or("source root is not UTF-8")?;
        let handle = open_directory(root)?;
        let root_meta = directory_meta(&handle, true)?;
        let root = root
            .canonicalize()
            .map_err(|e| format!("canonical source root: {e}"))?;
        root.to_str().ok_or("canonical source root is not UTF-8")?;
        if FileMeta::from_stat(&metadata(&open_directory(&root)?)?) != root_meta {
            return Err("source root changed during acquisition".into());
        }
        let mut dirs = BTreeMap::new();
        let mut files = BTreeMap::new();
        Walk::new(
            Inventory::Capture {
                dirs: &mut dirs,
                files: &mut files,
            },
            &root_meta,
        )
        .scan(&handle, path)?;
        Ok(Self {
            root,
            handle,
            prefix: prefix.into(),
            files,
            dirs,
            root_meta,
        })
    }

    pub fn fixture(&self) -> Result<Fixture<'_>, String> {
        let entries: Vec<_> = self
            .files
            .iter()
            .map(|(path, file)| Entry {
                path,
                bytes: &file.bytes,
                mode: "regular",
            })
            .collect();
        Fixture::admit(&entries).map_err(|e| format!("fixture admission: {e:?}"))
    }

    /// The review separately binds the fixture digest. This domain binds the source root,
    /// scoped topology and inode metadata independently of enumeration order. Paths cannot
    /// contain NUL; record tags, NUL-terminated paths and fixed-width integers are unambiguous.
    pub fn identity(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"plyrune/linux-scoped-source/v1\0");
        hash.update(self.root.as_os_str().as_bytes());
        hash.update([0]);
        self.root_meta.hash(&mut hash);
        hash.update(self.prefix.as_bytes());
        hash.update([0]);
        for (path, meta) in &self.dirs {
            hash.update(b"d");
            hash.update(path.as_bytes());
            hash.update([0]);
            meta.hash(&mut hash);
        }
        for (path, file) in &self.files {
            hash.update(b"f");
            hash.update(path.as_bytes());
            hash.update([0]);
            file.meta.hash(&mut hash);
            hash.update(file.version.size.to_be_bytes());
            hash.update(file.version.mtime.0.to_be_bytes());
            hash.update(file.version.mtime.1.to_be_bytes());
            hash.update(file.version.ctime.0.to_be_bytes());
            hash.update(file.version.ctime.1.to_be_bytes());
        }
        hash.finalize().into()
    }

    fn named_root(&self) -> Result<File, String> {
        let root = open_directory(&self.root)?;
        if directory_meta(&root, true)? != self.root_meta {
            return Err("stale source root".into());
        }
        Ok(root)
    }

    /// Under the publisher's lock, check the current root name, every scoped inode, bytes and
    /// inventory membership. Directory timestamps do not describe scoped membership (and the
    /// root may have unrelated out-of-scope edits), so enumerate rather than comparing them.
    pub fn recheck(&self) -> Result<(), String> {
        let handle = self.named_root()?;
        Walk::new(Inventory::Verify(self), &self.root_meta).scan(&handle, scope(&self.prefix)?)
    }

    pub fn parent_for(&self, path: &str) -> Result<File, String> {
        if !self.files.contains_key(path) {
            return Err("target absent from source manifest".into());
        }
        let (parent, _) = path
            .rsplit_once('/')
            .ok_or("target outside requested prefix")?;
        let mut dir = self.named_root()?;
        let mut relative = String::new();
        for name in parent.split('/') {
            push_name(&mut relative, name);
            dir = open_child(&dir, name, dir_flags())?;
            if self.dirs.get(&relative) != Some(&directory_meta(&dir, false)?) {
                return Err(format!("stale source directory {relative:?}"));
            }
        }
        Ok(dir)
    }

    pub fn recheck_file(&self, parent: &File, path: &str) -> Result<(), String> {
        let expected = self
            .files
            .get(path)
            .ok_or("target absent from source manifest")?;
        let (_, name) = path
            .rsplit_once('/')
            .ok_or("target outside requested prefix")?;
        // Staging may hold the parent descriptor across earlier per-file visibility points.
        // Verify that the reviewed name still leads to that parent, not just that the old
        // descriptor still contains a file with the expected basename.
        let current = self.parent_for(path)?;
        if FileMeta::from_stat(&metadata(parent)?) != FileMeta::from_stat(&metadata(&current)?) {
            return Err(format!("stale target parent for {path:?}"));
        }
        let observed = stat_child(parent, name)?;
        let (mut file, meta, version) = open_regular(parent, name, &observed)?;
        compare_file(&mut file, &meta, &version, expected).map_err(|e| format!("{path:?}: {e}"))?;
        unchanged(&mut file, &meta, &version)
    }
}

fn stat_child(parent: &File, name: &str) -> Result<fs::Stat, String> {
    fs::statat(
        parent,
        name,
        AtFlags::SYMLINK_NOFOLLOW | AtFlags::NO_AUTOMOUNT,
    )
    .map_err(|e| format!("stat scoped entry {name:?}: {e}"))
}

fn regular_meta(file: &File) -> Result<(FileMeta, Version), String> {
    let stat = metadata(file)?;
    if !stat.is_file() || stat.nlink() != 1 || stat.mode() & 0o7000 != 0 {
        return Err("expected a single-link regular file without special permission bits".into());
    }
    if stat.len() > MAX_FILE {
        return Err("file exceeds the 8 MiB source budget".into());
    }
    Ok((FileMeta::from_stat(&stat), Version::from_stat(&stat)))
}

fn open_regular(
    parent: &File,
    name: &str,
    observed: &fs::Stat,
) -> Result<(File, FileMeta, Version), String> {
    // Do not open sockets, devices or FIFOs. NONBLOCK also prevents blocking on a FIFO in a
    // name race; no hostile change of namespace is a precondition, not a property of statat.
    if FileType::from_raw_mode(observed.st_mode) != FileType::RegularFile {
        return Err(format!("unsupported source entry {name:?}"));
    }
    let file = open_child(
        parent,
        name,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOCTTY,
    )?;
    let (meta, version) = regular_meta(&file)?;
    if meta.dev != observed.st_dev || meta.ino != observed.st_ino {
        return Err(format!("source entry {name:?} changed while opening"));
    }
    check_profile(&file)?;
    Ok((file, meta, version))
}

fn unchanged(file: &mut File, meta: &FileMeta, version: &Version) -> Result<(), String> {
    if file
        .read(&mut [0_u8; 1])
        .map_err(|e| format!("verify source EOF: {e}"))?
        != 0
    {
        return Err("source data exceeds its reported length".into());
    }
    let (after, stamp) = regular_meta(file)?;
    if after != *meta || stamp != *version {
        return Err("source file changed while reading".into());
    }
    Ok(())
}

fn compare_file(
    file: &mut File,
    meta: &FileMeta,
    version: &Version,
    expected: &SourceFile,
) -> Result<(), String> {
    if meta != &expected.meta || version != &expected.version {
        return Err("stale source file metadata".into());
    }
    let mut buf = [0_u8; 16_384];
    for chunk in expected.bytes.chunks(buf.len()) {
        let buf = buf
            .get_mut(..chunk.len())
            .ok_or("invalid comparison chunk length")?;
        file.read_exact(buf)
            .map_err(|e| format!("read source file: {e}"))?;
        if buf != chunk {
            return Err("stale source file bytes".into());
        }
    }
    Ok(())
}

fn read_bytes(file: &mut File, size: u64) -> Result<Vec<u8>, String> {
    let length = usize::try_from(size).map_err(|e| format!("source file size: {e}"))?;
    let mut bytes = Vec::with_capacity(length);
    file.take(size)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read source file: {e}"))?;
    if bytes.len() != length {
        return Err("source file shortened during acquisition".into());
    }
    std::str::from_utf8(&bytes).map_err(|e| format!("source is not UTF-8: {e}"))?;
    Ok(bytes)
}

fn push_name(relative: &mut String, name: &str) {
    if !relative.is_empty() {
        relative.push('/');
    }
    relative.push_str(name);
}

enum Inventory<'a> {
    Capture {
        dirs: &'a mut BTreeMap<String, FileMeta>,
        files: &'a mut BTreeMap<String, SourceFile>,
    },
    Verify(&'a Directory),
}

struct Walk<'a> {
    inventory: Inventory<'a>,
    // Includes the root. Never revisit an inode, even via a surprising alias.
    seen: BTreeSet<(u64, u64)>,
    directories: usize,
    files: usize,
    total_bytes: u64,
}

impl<'a> Walk<'a> {
    fn new(inventory: Inventory<'a>, root: &FileMeta) -> Self {
        Self {
            inventory,
            seen: BTreeSet::from([(root.dev, root.ino)]),
            directories: 0,
            files: 0,
            total_bytes: 0,
        }
    }

    fn scan(mut self, root: &File, prefix: &str) -> Result<(), String> {
        let mut dir = root
            .try_clone()
            .map_err(|e| format!("pin source root: {e}"))?;
        let mut relative = String::new();
        for name in prefix.split('/') {
            push_name(&mut relative, name);
            dir = open_child(&dir, name, dir_flags())?;
            self.record_dir(&relative, &dir)?;
        }
        self.visit(&dir, &mut relative, prefix.split('/').count())?;
        if let Inventory::Verify(base) = self.inventory
            && (self.files != base.files.len() || self.directories != base.dirs.len())
        {
            return Err("stale source: missing scoped entries".into());
        }
        Ok(())
    }

    fn record_dir(&mut self, path: &str, file: &File) -> Result<(), String> {
        if self.directories >= MAX_DIRECTORIES {
            return Err("directory budget exceeded".into());
        }
        let meta = directory_meta(file, false).map_err(|e| format!("{path:?}: {e}"))?;
        self.record_inode(&meta)?;
        self.directories = self
            .directories
            .checked_add(1)
            .ok_or("directory count overflow")?;
        match &mut self.inventory {
            Inventory::Capture { dirs, .. } => {
                if dirs.insert(path.into(), meta).is_some() {
                    return Err(format!("duplicate source directory {path:?}"));
                }
            }
            Inventory::Verify(base) => {
                if base.dirs.get(path) != Some(&meta) {
                    return Err(format!("stale source directory {path:?}"));
                }
            }
        }
        Ok(())
    }

    fn record_inode(&mut self, meta: &FileMeta) -> Result<(), String> {
        if !self.seen.insert((meta.dev, meta.ino)) {
            return Err("scoped inode appears more than once".into());
        }
        Ok(())
    }

    fn visit(&mut self, dir: &File, relative: &mut String, depth: usize) -> Result<(), String> {
        let before = Version::from_stat(&metadata(dir)?);
        let entries =
            Dir::read_from(dir).map_err(|e| format!("open listing for {relative:?}: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read directory {relative:?}: {e}"))?;
            let name = entry
                .file_name()
                .to_str()
                .map_err(|e| format!("non-UTF-8 name in {relative:?}: {e}"))?;
            if matches!(name, "." | "..") {
                continue;
            }
            if self.seen.len() > MAX_ENTRIES {
                return Err("visited entry budget exceeded".into());
            }
            if !valid_component(name) {
                return Err(format!("invalid path component {name:?} in {relative:?}"));
            }
            let previous = relative.len();
            push_name(relative, name);
            if relative.len() > MAX_PATH {
                return Err("relative path exceeds fixture-v1 length budget".into());
            }
            let observed = stat_child(dir, name)?;
            if FileType::from_raw_mode(observed.st_mode) == FileType::Directory {
                if depth >= MAX_DEPTH {
                    return Err("directory depth budget exceeded".into());
                }
                let child = open_child(dir, name, dir_flags())?;
                self.record_dir(relative, &child)?;
                self.visit(
                    &child,
                    relative,
                    depth.checked_add(1).ok_or("depth overflow")?,
                )?;
            } else {
                self.visit_file(dir, name, relative, &observed)
                    .map_err(|e| format!("{relative:?}: {e}"))?;
            }
            relative.truncate(previous);
        }
        let after = metadata(dir)?;
        if after.nlink() == 0 || Version::from_stat(&after) != before {
            return Err(format!(
                "source directory changed during listing: {relative:?}"
            ));
        }
        Ok(())
    }

    fn visit_file(
        &mut self,
        dir: &File,
        name: &str,
        path: &str,
        observed: &fs::Stat,
    ) -> Result<(), String> {
        if self.files >= MAX_FILES {
            return Err("file count budget exceeded".into());
        }
        let (mut file, meta, version) = open_regular(dir, name, observed)?;
        self.record_inode(&meta)?;
        self.files = self.files.checked_add(1).ok_or("file count overflow")?;
        self.total_bytes = self
            .total_bytes
            .checked_add(version.size)
            .ok_or("source byte count overflow")?;
        if self.total_bytes > MAX_TOTAL {
            return Err("source exceeds the 32 MiB aggregate budget".into());
        }
        match &mut self.inventory {
            Inventory::Capture { files, .. } => {
                let bytes = read_bytes(&mut file, version.size)?;
                unchanged(&mut file, &meta, &version)?;
                if files
                    .insert(
                        path.into(),
                        SourceFile {
                            bytes,
                            meta,
                            version,
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate scoped path".into());
                }
            }
            Inventory::Verify(base) => {
                let expected = base
                    .files
                    .get(path)
                    .ok_or("unexpected file in source inventory")?;
                compare_file(&mut file, &meta, &version, expected)?;
                unchanged(&mut file, &meta, &version)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
