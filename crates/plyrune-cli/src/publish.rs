//! Explicit Linux publication on a trusted local namespace. The visibility point
//! is a successful rename for ONE file, not a transaction or durability claim.
use crate::directory::{Directory, FileMeta, check_profile, open_directory};
use plyrune_core::replacement::same_review;
use rustix::fs::{self, AtFlags, Mode, OFlags, RenameFlags};
use rustix::io::Errno;
use rustix::process::{Gid, geteuid};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

mod authority;
#[cfg(test)]
mod tests;

use authority::{below, same_directory};

#[derive(Clone, Copy)]
pub(super) enum Destination<'a> {
    InPlace,
    Into(&'a Path),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum Status {
    Complete,
    NoChange,
    Refused,
    Partial,
    Unknown,
}

/// Paths are relative to the selected destination; `.` means its root. Files
/// are recorded only after rename succeeds. No automatic rollback or retry.
#[derive(Debug, Serialize)]
pub(super) struct Publication {
    pub(super) status: Status,
    installed_files: Vec<String>,
    created_directories: Vec<String>,
    staging_directories_remaining: BTreeSet<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery: Option<&'static str>,
}

const RECONCILE: &str = "Stop writers. Missing completion (including process death) means Unknown. Use the saved preview, not a fresh preview of a mixed tree. At each sorted scoped path compare the current regular file's bytes (SHA-256) with BOTH reviewed base and candidate; record base, candidate, both, absent, or conflict. Compare the complete path inventory, modes/ownership and unexpected entries too. Inspect private .plyrune-stage~PID directories separately; remove only identified staging entries after inspection. Do not roll back or replay over conflicts. For --into, preserve the partial output and use a new empty destination. For --in-place, independently reconcile remaining edits and preview the resulting source again before a newly authorized apply.";

fn err(e: impl Display) -> String {
    e.to_string()
}

fn identity(fd: &File) -> Result<(u64, u64), String> {
    let meta = fd.metadata().map_err(err)?;
    Ok((meta.dev(), meta.ino()))
}

impl Publication {
    fn new() -> Self {
        Self {
            status: Status::Refused,
            installed_files: Vec::new(),
            created_directories: Vec::new(),
            staging_directories_remaining: BTreeSet::new(),
            diagnostic: None,
            recovery: None,
        }
    }

    fn mutation<T>(
        &mut self,
        result: Result<T, Errno>,
        verb: &str,
        path: &str,
    ) -> Result<T, String> {
        result.map_err(|e| {
            if matches!(e, Errno::IO | Errno::INTR) {
                self.status = Status::Unknown;
            }
            format!("{verb} {path}: {e}")
        })
    }

    fn failed(&mut self, error: String) {
        if self.status != Status::Unknown
            && !(self.installed_files.is_empty()
                && self.created_directories.is_empty()
                && self.staging_directories_remaining.is_empty())
        {
            self.status = Status::Partial;
        }
        if matches!(self.status, Status::Partial | Status::Unknown) {
            self.recovery = Some(RECONCILE);
        }
        self.diagnostic = Some(error);
    }
}

fn private_directory(
    parent: &File,
    name: &Path,
    relative: &str,
    staging: bool,
    effects: &mut Publication,
) -> Result<File, String> {
    effects.mutation(fs::mkdirat(parent, name, Mode::RWXU), "mkdir", relative)?;
    if staging {
        effects
            .staging_directories_remaining
            .insert(relative.to_owned());
    } else {
        effects.created_directories.push(relative.to_owned());
    }
    let fd = below(parent, name)?;
    if fd.metadata().map_err(err)?.uid() != geteuid().as_raw() {
        return Err(format!("unexpected directory owner: {relative}"));
    }
    fs::fchmod(&fd, Mode::RWXU).map_err(err)?;
    check_profile(&fd)?;
    Ok(fd)
}

fn parts(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

fn relative_dir(path: &str) -> &Path {
    Path::new(if path.is_empty() { "." } else { path })
}

struct Planned<'a> {
    path: &'a str,
    text: &'a str,
    meta: &'a FileMeta,
}

// Source paths have already passed path and count admission. Still require a
// one-for-one mapping at this last edge, before even creating an output root.
fn plan<'a>(
    base: &'a Directory,
    rows: &[(&'a str, &'a str)],
    into: bool,
) -> Result<Vec<Planned<'a>>, String> {
    if rows.len() != base.files.len() {
        return Err("candidate inventory length mismatch".into());
    }
    let mut different = false;
    let mut files = Vec::new();
    for ((path, source), &(key, text)) in base.files.iter().zip(rows) {
        if path != key {
            return Err("candidate path mismatch".into());
        }
        let changed = source.bytes.as_slice() != text.as_bytes();
        different |= changed;
        if into || changed {
            files.push(Planned {
                path: key,
                text,
                meta: &source.meta,
            });
        }
    }
    if !different {
        return Ok(Vec::new());
    }
    let owner = geteuid().as_raw();
    for file in &files {
        if file.meta.uid != owner {
            return Err(format!(
                "refusing to replace or copy file owned by a different user: {}",
                file.path
            ));
        }
    }
    Ok(files)
}

struct Materialized<'a> {
    path: &'a Path,
    root: File,
    dirs: BTreeMap<&'a str, File>,
}

impl<'a> Materialized<'a> {
    fn create(
        path: &'a Path,
        root: File,
        base: &'a Directory,
        effects: &mut Publication,
    ) -> Result<Self, String> {
        let mut tree = Self {
            path,
            root,
            dirs: BTreeMap::new(),
        };
        for key in base.dirs.keys() {
            let (parent, name) = parts(key);
            let fd = private_directory(
                tree.directory(parent)?,
                Path::new(name),
                key,
                false,
                effects,
            )?;
            tree.dirs.insert(key.as_str(), fd);
        }
        Ok(tree)
    }

    fn directory(&self, relative: &str) -> Result<&File, String> {
        if relative.is_empty() {
            Ok(&self.root)
        } else {
            self.dirs
                .get(relative)
                .ok_or_else(|| format!("missing output parent: {relative}"))
        }
    }

    fn current_parent(&self, relative: &str) -> Result<&File, String> {
        same_directory(&self.root, &open_directory(self.path)?)?;
        let pinned = self.directory(relative)?;
        same_directory(pinned, &below(&self.root, relative_dir(relative))?)?;
        Ok(pinned)
    }
}

struct Stage {
    parent: File,
    dir: File,
    relative: String,
}

struct Prepared<'a> {
    path: &'a str,
    text: &'a str,
    meta: &'a FileMeta,
    file: File,
}

fn prepare_file(stage: &File, name: &str, text: &str, meta: &FileMeta) -> Result<File, String> {
    // The inaccessible staging directory keeps even final, world-readable
    // permission bits private until rename. Nothing ever truncates a live name.
    let mut file = File::from(
        fs::openat(
            stage,
            name,
            OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(err)?,
    );
    file.write_all(text.as_bytes()).map_err(err)?;
    if file.metadata().map_err(err)?.gid() != meta.gid {
        fs::fchown(&file, None, Some(Gid::from_raw(meta.gid))).map_err(err)?;
    }
    fs::fchmod(&file, Mode::from_raw_mode(meta.mode)).map_err(err)?;
    file.sync_all().map_err(err)?;
    file.rewind().map_err(err)?;
    staged_metadata(&file, name, text, meta)?;
    Ok(file)
}

fn staged_metadata(file: &File, name: &str, text: &str, meta: &FileMeta) -> Result<(), String> {
    check_profile(file)?;
    let actual = file.metadata().map_err(err)?;
    if !actual.is_file()
        || actual.nlink() != 1
        || actual.uid() != meta.uid
        || actual.gid() != meta.gid
        || actual.mode() & 0o7777 != meta.mode
        || actual.len() != u64::try_from(text.len()).map_err(err)?
    {
        return Err(format!("staged file metadata mismatch: {name}"));
    }
    Ok(())
}

fn stage_all<'a>(
    base: &Directory,
    rows: Vec<Planned<'a>>,
    tree: Option<&Materialized<'_>>,
    stage_name: &str,
    stages: &mut BTreeMap<&'a str, Stage>,
    effects: &mut Publication,
) -> Result<Vec<Prepared<'a>>, String> {
    let mut prepared = Vec::with_capacity(rows.len());
    for Planned { path, text, meta } in rows {
        let (parent, name) = parts(path);
        let stage = match stages.entry(parent) {
            std::collections::btree_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::btree_map::Entry::Vacant(e) => {
                let fd = match tree {
                    Some(tree) => tree.current_parent(parent)?.try_clone().map_err(err)?,
                    None => base.parent_for(path)?,
                };
                let relative = if parent.is_empty() {
                    stage_name.to_owned()
                } else {
                    format!("{parent}/{stage_name}")
                };
                let dir = private_directory(&fd, Path::new(stage_name), &relative, true, effects)?;
                e.insert(Stage {
                    parent: fd,
                    dir,
                    relative,
                })
            }
        };
        let file = prepare_file(&stage.dir, name, text, meta)
            .map_err(|e| format!("prepare {path}: {e}"))?;
        prepared.push(Prepared {
            path,
            text,
            meta,
            file,
        });
    }
    Ok(prepared)
}

fn recheck_staged(item: &Prepared<'_>) -> Result<(), String> {
    let mut bytes = [0_u8; 8192];
    let mut file = &item.file;
    for chunk in item.text.as_bytes().chunks(bytes.len()) {
        let read = bytes
            .get_mut(..chunk.len())
            .ok_or("stage comparison exceeds buffer")?;
        file.read_exact(read).map_err(err)?;
        if read != chunk {
            return Err(format!("staged bytes differ from review: {}", item.path));
        }
    }
    staged_metadata(&item.file, item.path, item.text, item.meta)
}

fn install(
    base: &Directory,
    tree: Option<&Materialized<'_>>,
    stage_name: &str,
    stage: &Stage,
    item: &Prepared<'_>,
    effects: &mut Publication,
) -> Result<(), String> {
    let (parent, name) = parts(item.path);
    let source_parent = base.parent_for(item.path)?;
    let target = match tree {
        Some(tree) => tree.current_parent(parent)?,
        None => &source_parent,
    };
    same_directory(&stage.parent, target)?;
    same_directory(&stage.dir, &below(target, Path::new(stage_name))?)?;
    recheck_staged(item)?;
    let named = fs::statat(&stage.dir, name, AtFlags::SYMLINK_NOFOLLOW).map_err(err)?;
    if (named.st_dev, named.st_ino) != identity(&item.file)? || named.st_nlink != 1 {
        return Err(format!("staged name or link count changed: {}", item.path));
    }
    base.recheck_file(&source_parent, item.path)?;
    let renamed = if tree.is_some() {
        fs::renameat_with(&stage.dir, name, target, name, RenameFlags::NOREPLACE)
    } else {
        fs::renameat(&stage.dir, name, target, name)
    };
    effects.mutation(renamed, "rename", item.path)?;
    effects.installed_files.push(item.path.to_owned());
    Ok(())
}

fn run(
    base: &Directory,
    rows: &[(&str, &str)],
    computed_review: &[u8; 32],
    approved_review: &[u8; 32],
    destination: Destination<'_>,
    effects: &mut Publication,
    mut after_install: impl FnMut(&str) -> Result<(), String>,
) -> Result<Status, String> {
    let mut authority = authority::acquire(base, destination)?;
    if !same_review(approved_review, computed_review) {
        return Err("review identity mismatch; preview the current source and request".into());
    }
    // The reviewed candidate came from this immutable capture. Reacquiring the
    // complete subject while locked (including negative witnesses and metadata)
    // is essential: checking only changed files would trust a stale review.
    base.recheck()?;
    let files = plan(base, rows, authority.output.is_some())?;
    if files.is_empty() {
        return Ok(Status::NoChange);
    }
    let tree = if let Some(output) = authority.output.take() {
        let (path, root) = output.enter(effects)?;
        Some(Materialized::create(path, root, base, effects)?)
    } else {
        None
    };
    let stage_name = format!(".plyrune-stage~{}", std::process::id());
    let mut stages = BTreeMap::new();
    let prepared = stage_all(
        base,
        files,
        tree.as_ref(),
        &stage_name,
        &mut stages,
        effects,
    )?;
    for item in &prepared {
        let stage = stages
            .get(parts(item.path).0)
            .ok_or("missing pinned staging directory")?;
        install(base, tree.as_ref(), &stage_name, stage, item, effects)?;
        after_install(item.path)?;
    }
    for stage in stages.values() {
        same_directory(&stage.dir, &below(&stage.parent, Path::new(&stage_name))?)?;
        effects.mutation(
            fs::unlinkat(&stage.parent, stage_name.as_str(), AtFlags::REMOVEDIR),
            "rmdir",
            &stage.relative,
        )?;
        effects
            .staging_directories_remaining
            .remove(&stage.relative);
    }
    Ok(Status::Complete)
}

pub(super) fn publish(
    base: &Directory,
    candidate: &[(&str, &str)],
    computed_review: &[u8; 32],
    approved_review: &[u8; 32],
    destination: Destination<'_>,
) -> Publication {
    let mut effects = Publication::new();
    match run(
        base,
        candidate,
        computed_review,
        approved_review,
        destination,
        &mut effects,
        |_| Ok(()),
    ) {
        Ok(status) => effects.status = status,
        Err(e) => effects.failed(e),
    }
    effects
}
