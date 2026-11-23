//! Plyrune's library tier.
//!
//! This crate owns the immutable object store, the conversion of untrusted
//! wire values into domain types, bounded plan evaluation and the composition
//! of adapters. Decisions come from the kernel ([`plyrune_kernel`]); the world
//! reaches this crate only through parameters (a capability, a writer, the
//! bytes of a fixture), never through the ambient filesystem, environment,
//! clock or standard streams, which the tauten profile forbids here
//! (TAU-A001, TAU-O001).
//!
//! No operation is exposed yet. The kernel's first decisions are being proved
//! (`proofs/claims.json`) before any of them is exposed through this crate, as
//! `AGENTS.md` requires.
