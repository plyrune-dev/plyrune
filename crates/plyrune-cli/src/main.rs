//! The edge: the only place that touches the world. It acquires authority,
//! turns it into the capabilities the core asks for, and calls the core.
//! Keep it this thin.
//!
//! Nothing is exposed yet. M1 is candidate-only: the first operations will be
//! snapshot, requirement, query, inspect, prepare and check, and no operation
//! in M1 to M4 writes the working tree. The kernel's decisions are proved
//! (`proofs/claims.json`) before they are exposed here.

use std::process::ExitCode;

/// Exit status of a request this build cannot serve: no operation exists.
const NO_OPERATIONS: u8 = 2;

fn main() -> ExitCode {
    eprintln!("plyrune: no operation is available in this build (M0 bootstrap)");
    ExitCode::from(NO_OPERATIONS)
}
