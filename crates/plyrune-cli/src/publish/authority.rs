//! Acquire closed, non-blocking lock sets before inspecting or creating output.
//! Flocks coordinate this driver, not arbitrary writers or namespace changes.
use super::{Destination, Publication, err, identity, private_directory};
use crate::directory::{Directory, check_profile, open_directory};
use rustix::fs::{self, AtFlags, Mode, OFlags, ResolveFlags, StatxFlags};
use rustix::io::Errno;
use std::collections::BTreeMap;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

const MAX_ANCESTORS: usize = 256;

fn dir_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}

pub(super) fn below(parent: &File, path: &Path) -> Result<File, String> {
    fs::openat2(
        parent,
        path,
        dir_flags(),
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::BENEATH | ResolveFlags::NO_XDEV,
    )
    .map(File::from)
    .map_err(err)
}

fn ancestry(fd: &File) -> Result<Vec<File>, String> {
    // Never lock a dup of the caller's open file description: that could
    // survive return, and concurrent calls could accidentally share its lock.
    let mut here = below(fd, Path::new("."))?;
    let mut chain = Vec::new();
    for _ in 0..MAX_ANCESTORS {
        let parent = File::from(fs::openat(&here, "..", dir_flags(), Mode::empty()).map_err(err)?);
        let done = identity(&parent)? == identity(&here)?;
        chain.push(here);
        if done {
            return Ok(chain);
        }
        here = parent;
    }
    Err("directory ancestry exceeds lock budget".into())
}

fn mount_id(fd: &File) -> Result<u64, String> {
    let info = fs::statx(fd, "", AtFlags::EMPTY_PATH, StatxFlags::MNT_ID).map_err(err)?;
    if info.stx_mask & StatxFlags::MNT_ID.bits() == 0 {
        return Err("mount identity unavailable".into());
    }
    Ok(info.stx_mnt_id)
}

fn includes(chain: &[File], fd: &File) -> Result<bool, String> {
    let target = identity(fd)?;
    for ancestor in chain {
        if identity(ancestor)? == target {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) enum Output<'a> {
    Existing {
        path: &'a Path,
        dir: File,
    },
    New {
        path: &'a Path,
        parent_path: PathBuf,
        parent: File,
    },
}

impl<'a> Output<'a> {
    fn locate(path: &'a Path) -> Result<Self, String> {
        if path.components().any(|p| p.as_os_str() == ".git") {
            return Err("Git metadata is not an output directory".into());
        }
        match fs::openat2(
            fs::CWD,
            path,
            dir_flags(),
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        ) {
            Ok(fd) => Ok(Self::Existing {
                path,
                dir: fd.into(),
            }),
            Err(Errno::NOENT) => {
                if path.file_name().is_none() {
                    return Err(
                        "new destination must name one directory in an existing parent".into(),
                    );
                }
                let parent_path = path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));
                let parent = open_directory(parent_path)?;
                Ok(Self::New {
                    path,
                    parent_path: parent_path.to_owned(),
                    parent,
                })
            }
            Err(e) => Err(format!("open destination without links: {e}")),
        }
    }

    fn anchor(&self) -> &File {
        match self {
            Self::Existing { dir, .. } => dir,
            Self::New { parent, .. } => parent,
        }
    }

    fn validate(&self) -> Result<(), String> {
        check_profile(self.anchor())?;
        match self {
            Self::Existing { path, dir } => {
                same_directory(dir, &open_directory(path)?)?;
                for entry in fs::Dir::read_from(dir).map_err(err)? {
                    let entry = entry.map_err(err)?;
                    if !matches!(entry.file_name().to_bytes(), b"." | b"..") {
                        return Err("destination is not empty".into());
                    }
                }
                Ok(())
            }
            Self::New {
                path,
                parent_path,
                parent,
            } => {
                same_directory(parent, &open_directory(parent_path)?)?;
                let name = path.file_name().ok_or("missing destination name")?;
                match fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW) {
                    Err(Errno::NOENT) => Ok(()),
                    Ok(_) => Err("new destination name now exists".into()),
                    Err(e) => Err(err(e)),
                }
            }
        }
    }

    pub(super) fn enter(self, effects: &mut Publication) -> Result<(&'a Path, File), String> {
        match self {
            Self::Existing { path, dir } => Ok((path, dir)),
            Self::New { path, parent, .. } => {
                let name = path.file_name().ok_or("missing destination name")?;
                let dir = private_directory(&parent, Path::new(name), ".", false, effects)?;
                // The parent reservation covers other cooperating entrants until
                // return; lock the new root too, including callers using its inode.
                dir.try_lock().map_err(err)?;
                Ok((path, dir))
            }
        }
    }
}

pub(super) fn same_directory(expected: &File, observed: &File) -> Result<(), String> {
    if identity(expected)? != identity(observed)? || mount_id(expected)? != mount_id(observed)? {
        return Err("directory name no longer identifies the pinned directory".into());
    }
    Ok(())
}

type LockSet = BTreeMap<(u64, u64), (File, bool)>;

fn reserve(locks: &mut LockSet, fd: File, exclusive: bool) -> Result<(), String> {
    locks
        .entry(identity(&fd)?)
        .and_modify(|(_, mode)| *mode |= exclusive)
        .or_insert((fd, exclusive));
    Ok(())
}

pub(super) struct Authority<'a> {
    // Owning the independent open descriptions keeps locks across the entire publication.
    _locks: LockSet,
    pub(super) output: Option<Output<'a>>,
}

pub(super) fn acquire<'a>(
    base: &Directory,
    destination: Destination<'a>,
) -> Result<Authority<'a>, String> {
    let input = ancestry(&base.handle)?;
    let mut locks = LockSet::new();
    let output = match destination {
        Destination::InPlace => None,
        Destination::Into(path) => {
            let output = Output::locate(path)?;
            let anchor = output.anchor();
            let chain = ancestry(anchor)?;
            if includes(&chain, &base.handle)?
                || (matches!(output, Output::Existing { .. }) && includes(&input, anchor)?)
                || (base.handle.metadata().map_err(err)?.dev()
                    == anchor.metadata().map_err(err)?.dev()
                    && mount_id(&base.handle)? != mount_id(anchor)?)
            {
                return Err(
                    "source and destination overlap or have ambiguous mount aliases".into(),
                );
            }
            for (i, fd) in chain.into_iter().enumerate() {
                // Missing root: reserve its existing parent before mkdir. Sibling
                // operations may conservatively conflict while this call runs.
                reserve(&mut locks, fd, i == 0)?;
            }
            Some(output)
        }
    };
    for (i, fd) in input.into_iter().enumerate() {
        reserve(&mut locks, fd, i == 0)?;
    }
    // Also protect admitted directory inodes reached through bind aliases whose
    // namespace ancestry may not lead back to this invocation's source root.
    for path in base.dirs.keys() {
        reserve(&mut locks, below(&base.handle, Path::new(path))?, true)?;
    }
    for (fd, exclusive) in locks.values() {
        let result = if *exclusive {
            fd.try_lock()
        } else {
            fd.try_lock_shared()
        };
        result.map_err(|e| format!("cooperating writer lock unavailable: {e}"))?;
    }
    if let Some(output) = &output {
        output.validate()?;
    }
    Ok(Authority {
        _locks: locks,
        output,
    })
}
