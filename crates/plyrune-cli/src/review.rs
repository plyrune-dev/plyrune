//! A preview is a projection of a checked candidate; it never opens a destination.
use crate::hex;
use plyrune_core::fixture::{BoundFixture, Fixture, IncarnationId, OwnerId, StoreBinding};
use plyrune_core::replacement::{CHECKER_PROFILE, Disposition, ReviewedCandidate};
use serde::Serialize;
use sha2::{Digest, Sha256};

use std::io::{BufWriter, Write};
pub(super) fn bind(fixture: Fixture<'_>) -> BoundFixture<'_> {
    // Exactly one lexical store per one-shot CLI process; no captures cross processes.
    fixture.bind(StoreBinding::new(
        OwnerId::from_host(0),
        IncarnationId::from_host(0),
    ))
}

pub(super) fn digits(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(64);
    for &b in digest {
        s.push(char::from(HEX[usize::from(b >> 4)]));
        s.push(char::from(HEX[usize::from(b & 15)]));
    }
    s
}

#[derive(Serialize)]
struct FileReview<'a, R> {
    path: &'a str,
    base_sha256: String,
    output_sha256: String,
    ranges: R,
}

#[derive(Serialize)]
struct Preview<'a, S, R> {
    profile: &'static str,
    source: S,
    task_authority: &'static str,
    disposition: &'static str,
    prefix: &'a str,
    needle: &'a str,
    replacement: &'a str,
    snapshot: String,
    requirement: String,
    candidate: String,
    review: String,
    matches: usize,
    scoped_files: usize,
    changed_files: usize,
    resources: R,
}

pub(super) fn show(
    checked: &ReviewedCandidate<'_>,
    replacement: &str,
    review: &[u8; 32],
    source: impl Serialize,
) -> Result<(), String> {
    let selection = checked.selection();
    let mut files = Vec::with_capacity(checked.rows().len());
    let mut scoped = selection.rows().iter().peekable();
    for ((path, before), (out_path, after)) in selection.fixture().resources().zip(checked.rows()) {
        if path != out_path {
            return Err("review projection: inconsistent inventory".into());
        }
        let ranges = scoped
            .next_if(|row| row.path() == path)
            .map_or(&[][..], |row| row.ranges());
        let ranges = crate::references::Sequence(ranges.iter().map(|r| [r.start(), r.end()]));
        let base_sha256 = digits(&Sha256::digest(before.as_bytes()).into());
        let output_sha256 = if before == after {
            base_sha256.clone()
        } else {
            digits(&Sha256::digest(after.as_bytes()).into())
        };
        files.push(FileReview {
            path,
            base_sha256,
            output_sha256,
            ranges,
        });
    }
    let report = Preview {
        profile: CHECKER_PROFILE,
        source,
        task_authority: "caller-stated; no independent approval asserted",
        disposition: match checked.disposition() {
            Disposition::Changed => "Changed",
            Disposition::NoChange => "NoChange",
        },
        prefix: selection.prefix(),
        needle: selection.needle(),
        replacement,
        snapshot: hex(checked.base_digest()),
        requirement: digits(checked.requirement_digest()),
        candidate: digits(checked.candidate_digest()),
        review: digits(review),
        matches: checked.matched(),
        scoped_files: selection.rows().len(),
        changed_files: checked.changed(),
        resources: files,
    };
    let mut out = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut out, &report).map_err(|e| format!("output: {e}"))?;
    out.write_all(b"\n")
        .and_then(|()| out.flush())
        .map_err(|e| format!("output: {e}"))
}
