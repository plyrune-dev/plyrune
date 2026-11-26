//! Wire projection only: discovery has already admitted the complete selection.
use plyrune_core::{fixture::Fixture, selection::refs};
use serde::{Serialize, Serializer};
use std::io::{BufWriter, Write};

/// Serialize the borrowed rows directly, without constructing a second match list.
struct Sequence<I>(I);

impl<I> Serialize for Sequence<I>
where
    I: Iterator + Clone,
    I::Item: Serialize,
{
    fn serialize<S: Serializer>(&self, output: S) -> Result<S::Ok, S::Error> {
        output.collect_seq(self.0.clone())
    }
}

#[derive(Serialize)]
struct Resource<'a, R> {
    path: &'a str,
    ranges: R,
}

#[derive(Serialize)]
struct References<'a, R> {
    profile: &'static str,
    snapshot: String,
    prefix: &'a str,
    needle: &'a str,
    coverage: &'static str,
    matches: usize,
    resources: R,
}

pub(super) fn show(fixture: &Fixture<'_>, prefix: &str, needle: &str) -> Result<(), String> {
    let selected = refs(fixture, prefix, needle).map_err(|e| format!("refs: {e:?}"))?;
    let response = References {
        profile: "plyrune/refs/literal/v1",
        snapshot: super::hex(fixture.digest()),
        prefix,
        needle,
        coverage: "Complete",
        matches: selected.match_count(),
        resources: Sequence(selected.rows().iter().map(|row| Resource {
            path: row.path(),
            ranges: Sequence(row.ranges().iter().map(|r| [r.start(), r.end()])),
        })),
    };
    let mut out = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut out, &response).map_err(|e| format!("output: {e}"))?;
    out.write_all(b"\n")
        .and_then(|()| out.flush())
        .map_err(|e| format!("output: {e}"))
}
