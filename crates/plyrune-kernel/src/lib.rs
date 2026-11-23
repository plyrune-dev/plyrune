//! Plyrune's pure decision kernel: total functions over plain data.
//!
//! Every semantic decision — whether a byte range may be admitted as a
//! capture, which occurrences of a literal a declared inventory contains,
//! whether a candidate satisfies an independently admitted requirement, what
//! a report is bound to — is made here, as a plain function that returns for
//! every input. Nothing in this crate does IO or reads a clock, a file, an
//! environment variable or shared mutable state: the crate is `no_std` with
//! no dependencies, so the compiler enforces what the profile (TAU-K001,
//! TAU-K003) and the proof gate (TAU-K004) also check.
//!
//! The functions named in `[package.metadata.charon] start_from` are
//! translated to Lean 4 by Charon and Aeneas; the theorems under `proofs/`
//! are stated over that translation, and `cargo tauten proofs` records one
//! outcome per claim in `proofs/claims.json`. A decision is *proved* only
//! when that record says so for the current source; until then it is
//! implemented and tested, nothing more. The obligations the claims discharge
//! are PR-C01 to PR-C08 in `docs/ASSURANCE.md`.
//!
//! The code is written for translation as tauten's own kernel is: no derives
//! are reached from a translated function (enumerations are compared with
//! `match`), no integer casts, no iterators or closures (explicit `while`
//! loops over `get`), no nested borrows, and no parameter shares the name of
//! its module.
#![no_std]

pub mod binding;
pub mod range;
