//! A one-resource, fixed-fixture probe for the exact upstream dependency.
//! Paths, modes and independently computed expectations are checked by the driver.
use splicery_splice::{Cut, Edit, Piece, prepare, render};
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const OLD: &[u8] = b"legacy_endpoint";
    const NEW: &[u8] = b"service_endpoint";
    let mut bytes = Vec::new();
    std::io::stdin().take(8193).read_to_end(&mut bytes)?;
    if bytes.len() > 8192 {
        return Err("fixture exceeds probe budget".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    // One immutable input per process. This local token is deliberately
    // not presented as a digest, a session handle or a provenance check.
    let token = [0; 32];
    let mut edits = Vec::new();
    for (lo, matched) in text.match_indices(std::str::from_utf8(OLD)?) {
        edits.push(Edit {
            target: Cut {
                capture: token,
                lo,
                hi: lo.checked_add(matched.len()).ok_or("offset overflow")?,
            },
            parts: vec![Piece::Literal(NEW.to_vec())],
        });
    }
    let order = prepare(&bytes, true, &token, &edits).map_err(|e| format!("{e:?}"))?;
    let result = render(&bytes, &edits, &order).map_err(|e| format!("{e:?}"))?;
    std::io::stdout().lock().write_all(&result.bytes)?;
    Ok(())
}
