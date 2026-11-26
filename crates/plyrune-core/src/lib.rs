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
//! The fixture adapter validates explicit bytes only.

pub mod fixture;
pub mod replacement;
pub mod selection;
