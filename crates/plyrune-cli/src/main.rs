//! Inspect an explicitly supplied fixture manifest. Nothing reads a workspace.
use plyrune_core::fixture::{Entry, Fixture};
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

fn snapshot() -> Result<(), String> {
    let mut input = Vec::new();
    std::io::stdin()
        .lock()
        .take(MAX_INPUT + 1)
        .read_to_end(&mut input)
        .map_err(|e| format!("input: {e}"))?;
    if input.len() > 1_048_576 {
        return Err("fixture JSON exceeds 1 MiB".into());
    }
    let manifest: Manifest =
        serde_json::from_slice(&input).map_err(|e| format!("fixture JSON: {e}"))?;
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
    let fixture = Fixture::admit(&entries).map_err(|e| format!("fixture admission: {e:?}"))?;
    let digest: String = fixture
        .digest()
        .bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    writeln!(std::io::stdout().lock(), "{}", serde_json::json!({
        "profile": "plyrune/fixture/v1", "sha256": digest, "resources": fixture.resources().len()
    })).map_err(|e| format!("output: {e}"))
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let result = match (args.next(), args.next()) {
        (Some(cmd), None) if cmd == "snapshot" => snapshot(),
        _ => Err("usage: plyrune snapshot < fixture.json".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("plyrune: {reason}");
            ExitCode::from(2)
        }
    }
}
