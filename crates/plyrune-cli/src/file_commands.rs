//! Linux-only live inputs. Never discover a repository, invoke Git or infer a write target.
use crate::{
    directory::Directory,
    publish::{Destination, Status, publish},
    references, review,
};
use plyrune_core::replacement::preview;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

fn filesystem_key(origin: &[u8; 32], checked: &[u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"plyrune/linux-directory-review/v1\0");
    hash.update(origin);
    hash.update(checked);
    hash.finalize().into()
}

#[derive(Serialize)]
struct Origin<'a, F> {
    root: &'a Path,
    directory_metadata: &'a std::collections::BTreeMap<String, crate::directory::FileMeta>,
    file_metadata: F,
}

#[derive(Serialize)]
struct FileInfo<'a> {
    path: &'a str,
    #[serde(flatten)]
    meta: &'a crate::directory::FileMeta,
}
pub(super) fn refs(root: &str, prefix: &str, needle: &str) -> Result<(), String> {
    let source = Directory::read(Path::new(root), prefix)?;
    references::show(&source.fixture()?, prefix, needle)
}

pub(super) fn edit(
    root: &str,
    prefix: &str,
    needle: &str,
    replacement: &str,
    effect: Option<(&str, Destination<'_>)>,
) -> Result<(), String> {
    let approved = effect
        .map(|(key, dest)| {
            super::parse_digest(key)
                .map(|digest| (*digest.bytes(), dest))
                .map_err(|e| format!("--review: {e}; use the exact ID from preview"))
        })
        .transpose()?;
    let source = Directory::read(Path::new(root), prefix)?;
    let bound = review::bind(source.fixture()?);
    let checked =
        preview(&bound, prefix, needle, replacement).map_err(|e| format!("preview: {e:?}"))?;
    let key = filesystem_key(&source.identity(), checked.review_id());
    let Some((approved, dest)) = approved else {
        let origin = Origin {
            root: &source.root,
            directory_metadata: &source.dirs,
            file_metadata: references::Sequence(source.files.iter().map(|(path, file)| FileInfo {
                path,
                meta: &file.meta,
            })),
        };
        return review::show(&checked, replacement, &key, origin);
    };
    let rows: Vec<_> = checked.rows().collect();
    let report = publish(&source, &rows, &key, &approved, dest);
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, &report)
        .map_err(|e| format!("output: {e}; reconcile before retry"))?;
    out.write_all(b"\n")
        .map_err(|e| format!("output: {e}; reconcile before retry"))?;
    match report.status {
        Status::Complete | Status::NoChange => Ok(()),
        status @ (Status::Refused | Status::Partial | Status::Unknown) => {
            Err(format!("apply {status:?}; see the publication result"))
        }
    }
}
