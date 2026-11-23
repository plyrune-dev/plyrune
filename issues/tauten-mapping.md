## Problem

The opening of [docs/MAPPING.md](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/docs/MAPPING.md) says its status column describes what exists today and that the website will reproduce the table. Several entries still describe earlier plans:

- Provenance/freshness is listed as `None / Not enforced`.
- Effect rows are described only as coarse tiers and as unavailable per function.
- The proof tier and several kernel-proof references remain `planned`.

At the same inspected revision, [origin.rs](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten/src/origin.rs), [lib.rs](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten/src/lib.rs), [the agent profile](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten-check/agents.md), README and STATUS describe shipped Evidence, Permit, Stamped, Bindings, effect annotations/TAU-F001, and proof gates.

This makes adoption planning unnecessarily inaccurate: it looks as though consumers must request features that already exist.

## Requested correction

1. Reconcile the table with current implementation and assurance records, or explicitly label and relocate it as historical design material if that is its intended purpose.
2. Separate borrowed evidence, exclusive permits, runtime version stamps and generated field comparisons. Preserve their actual assumptions and limitations, including the separately filed stamped-subject binding concern.
3. Describe declared capability/signature effect-row checks accurately. Do not imply full transitive inference, arbitrary captured-authority tracking, effect polymorphism or confinement.
4. Replace planned proof labels only for the exact functions/instances currently supported by claims and the proof gate. Keep source, translation, binding and external-world assumptions explicit.
5. Check any site copy sourced from this table and make README, STATUS, MAPPING and the agent profile consistent on these particular capabilities.

## Acceptance

A short source-to-row review table cites the deciding implementation, enforcement strength and named limitations. No new correctness/performance claim is inferred from lint counts or from the mere presence of an API.

Verification: source inspection at `bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42`, with independent review; no new runtime or proof gate was executed.

Related: #86 concerns detecting/refreshing stale *adopter copies* of AGENTS.tauten.md. This issue is the accuracy of Tauten's current capability mapping, not that refresh feature. Filed in later for owner triage.
