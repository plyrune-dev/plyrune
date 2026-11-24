//! Inspect an explicitly supplied fixture manifest. Nothing reads a workspace.
use plyrune_core::fixture::{Entry, Fixture, SnapshotDigest};
use serde::Deserialize;
use std::io::{Read, Write};
use std::process::ExitCode;

// CLI transport cap, independent of semantic budgets on decoded resources.
const MAX_INPUT: u64 = 1_048_576;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    status: String,
    snapshot: String,
    resources: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    path: String,
    mode: String,
    text: String,
}

const USAGE: &str = "plyrune demo | example | snapshot | inspect DIGEST PATH START END\nsnapshot and inspect read a fixture on stdin. Offsets are zero-based UTF-8 bytes; END is exclusive.";
const EXAMPLE: &[u8] = include_bytes!("../../../examples/input.json");

fn read_manifest() -> Result<Manifest, String> {
    let mut input = Vec::new();
    std::io::stdin()
        .lock()
        .take(MAX_INPUT + 1)
        .read_to_end(&mut input)
        .map_err(|e| format!("input: {e}"))?;
    if input.len() > 1_048_576 {
        return Err("fixture JSON exceeds 1 MiB".into());
    }
    serde_json::from_slice(&input).map_err(|e| format!("fixture JSON: {e}"))
}

fn admit(manifest: &Manifest) -> Result<Fixture<'_>, String> {
    if manifest.status != "illustrative-contract-fixture-not-a-wire-schema"
        || manifest.snapshot.is_empty()
    {
        return Err("expected the handover fixture format with a nonempty source label".into());
    }
    if manifest.resources.len() > 256 {
        return Err("fixture inventory exceeds 256 entries".into());
    }
    let entries: Vec<_> = manifest
        .resources
        .iter()
        .map(|r| Entry {
            path: &r.path,
            mode: &r.mode,
            bytes: r.text.as_bytes(),
        })
        .collect();
    Fixture::admit(&entries).map_err(|e| format!("fixture admission: {e:?}"))
}

fn hex(digest: SnapshotDigest) -> String {
    digest.bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_digest(input: &str) -> Result<SnapshotDigest, String> {
    let mut bytes = [0; 32];
    if input.len() != 64
        || !input
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("expected 64 lowercase hexadecimal snapshot digits".into());
    }
    for (out, pair) in bytes.iter_mut().zip(input.as_bytes().as_chunks::<2>().0) {
        let s = std::str::from_utf8(pair).map_err(|e| e.to_string())?;
        *out = u8::from_str_radix(s, 16).map_err(|e| e.to_string())?;
    }
    Ok(SnapshotDigest::from_bytes(bytes))
}

fn line(v: impl std::fmt::Display) -> Result<(), String> {
    writeln!(std::io::stdout().lock(), "{v}").map_err(|e| format!("output: {e}"))
}

fn inspect(
    fixture: &Fixture<'_>,
    expected: SnapshotDigest,
    path: &str,
    start: usize,
    end: usize,
) -> Result<serde_json::Value, String> {
    let text = fixture
        .inspect(expected, path, start, end)
        .map_err(|e| format!("inspect {path}: {e:?}"))?;
    Ok(
        serde_json::json!({"sha256":hex(expected), "path":path, "start":start, "end":end, "text":text}),
    )
}

fn demo() -> Result<(), String> {
    let mut manifest: Manifest = serde_json::from_slice(EXAMPLE).map_err(|e| e.to_string())?;
    let expected = {
        let fixture = admit(&manifest)?;
        let expected = fixture.digest();
        line(inspect(&fixture, expected, "config/server/api.env", 8, 23)?)?;
        expected
    };
    // Modify an unrelated file so the requested path and bytes still look right.
    let other = manifest
        .resources
        .iter_mut()
        .find(|r| r.path == "docs/notes.txt")
        .ok_or("demo fixture missing notes")?;
    other.text.push_str("Changed after snapshot.\n");
    let changed = admit(&manifest)?;
    match inspect(&changed, expected, "config/server/api.env", 8, 23) {
        Err(reason) => line(
            serde_json::json!({"changed":"docs/notes.txt", "newSnapshot":hex(changed.digest()), "refused":reason}),
        ),
        Ok(_) => Err("unexpected acceptance of a stale snapshot".into()),
    }
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os()
        .skip(1)
        .take(6)
        .map(|a| {
            a.into_string()
                .map_err(|e| format!("argument is not UTF-8: {e:?}"))
        })
        .collect::<Result<_, _>>()?;
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["--version"] => line(concat!("plyrune ", env!("CARGO_PKG_VERSION"))),
        ["--help"] => line(USAGE),
        ["demo"] => demo(),
        ["example"] => std::io::stdout()
            .lock()
            .write_all(EXAMPLE)
            .map_err(|e| e.to_string()),
        ["snapshot"] => {
            let manifest = read_manifest()?;
            let fixture = admit(&manifest)?;
            line(
                serde_json::json!({"profile":"plyrune/fixture/v1", "sha256":hex(fixture.digest()), "resources":fixture.resources().len()}),
            )
        }
        ["inspect", digest, path, lo, hi] => {
            let expected = parse_digest(digest)?;
            let start = lo
                .parse()
                .map_err(|e| format!("invalid start byte offset: {e}"))?;
            let end = hi
                .parse()
                .map_err(|e| format!("invalid end byte offset: {e}"))?;
            line(inspect(
                &admit(&read_manifest()?)?,
                expected,
                path,
                start,
                end,
            )?)
        }
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("plyrune: {reason}");
            ExitCode::from(2)
        }
    }
}
