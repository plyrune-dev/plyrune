# Tauten adoption from the first build

Inspection date: 23 November 2026. Source inspected at `tauten-dev/tauten@bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42`. The repository workspace reports version 0.2.0; this exact development revision is distinct from the documented 0.2.0 release pin `370e7050f2adaba1e0a1c7351ab686d5f1238c47`.

Use one deliberately selected revision for CLI/runtime/templates. The default proposal below pins the inspected development revision. M0 may choose the released pin instead after checking the differences; record the decision and rerun affected gates. Never silently mix latest CLI behavior with older runtime/templates.

## What already exists

| Capability | Adoption use | Boundary |
|---|---|---|
| Kernel/core/edge profile, compiler and Clippy integration | Enforce workspace layering and selected coding restrictions | Discipline, not confinement; dependencies/FFI are not fully controlled |
| Nominal/refined wrappers and always-on contracts | Checked construction where the macro's supported shape fits | Runtime predicates are not arbitrary static refinement proofs |
| Evidence and exclusive Permit | Keep local in-memory checks tied to borrowed state | No lock against external filesystem mutation; interior mutability/subject design matters |
| Stamped evidence | Only with an adequate project subject binding | Inspected carrier compares revision only; see issue draft |
| Derived Bindings | Compare explicitly recorded fields | Cannot discover omitted dependencies for the caller |
| Effect annotations/TAU-F001 | Check declared rows against supported capability signatures | Name-based classification has unresolved cases; not full transitive inference/polymorphism |
| Machine + Lean induction helper | Structure real state machines where useful | Adopter must prove its invariant and map effects |
| Proof records, audits, mutants, freshness/subsumption | Track exact qualified kernel claims | A lint pass is not a proof; native gate output remains a trust boundary |
| Bounded ReadDir reads | Bound source acquisition IO where it fits | Does not create a coherent snapshot |

`docs/MAPPING.md` is stale on several of these capabilities. Prefer current source, README, STATUS and the proof records, reconcile conflicts, and do not infer an absent feature solely from that table.

## Bootstrap recipe to execute in M0

The authoring environment has no Rust compiler; these are adoption instructions, not executed results.

```sh
rustup toolchain install 1.98.0 --profile minimal --component clippy --component rustfmt
cargo +1.98.0 install --locked \
  --git https://github.com/tauten-dev/tauten \
  --rev bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42 cargo-tauten
cargo +1.98.0 tauten init
```

Review the proposed tiers, then run `cargo +1.98.0 tauten init --write` in the new implementation workspace. This writes tiers/profile and may set release overflow checking. The Plyrune profile chooses release overflow checking as defense in depth; public operations still return typed errors for admitted bounds failures rather than relying on a panic.

Every crate records metadata such as:

```toml
[package.metadata.tauten]
tier = "kernel" # core / edge in the appropriate other crates
```

Where runtime helpers are used, pin the same inspected revision in the workspace dependency and lock it. The pure kernel may use private checked structs directly rather than importing macro machinery. Verify the supported kernel-dependency allowlist and extraction before adding a dependency just for nominal wrappers.

```toml
[workspace.dependencies]
tauten = { git = "https://github.com/tauten-dev/tauten", rev = "bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42", default-features = false }

[profile.release]
overflow-checks = true
```

The edge may explicitly enable supported `std`/`fs` features after inspecting the selected dependency manifest. Record feature unification; a workspace-wide build must not silently change the kernel extraction environment.

Run the fast gate and repair findings. Greenfield crates should not begin with imported debt. If retaining legacy engine code requires a baseline, review each accepted finding, keep it scoped, and track it explicitly. Never regenerate a baseline in CI; Tauten's shrink-only ratchet is a guard, not permission to expand it casually.

## Proof path

After the kernel and toolchain are real, the documented template path is:

```sh
cargo +1.98.0 tauten init --proofs plyrune-kernel --write
sh proofs/refresh.sh
cargo +1.98.0 tauten proofs
cargo +1.98.0 tauten obligations
```

These commands are not a substitute for choosing and proving the domain claims. Inspect generated files, pin extraction/image inputs and retain existing qualified Splicery/Weft gates until the unified gate actually covers them. Tauten's default proof run operates on a clean committed snapshot; follow its documented live/snapshot mode instead of assuming dirty edits were included.

The published proof-toolchain reference in ADOPTING uses Lean 4.35.0, Aeneas nightly `2026.11.07-c5eb4a2`, Charon `8cae8fd` and image tag `ghcr.io/tauten-dev/proofs-toolchain:lean-4.35.0-aeneas-c5eb4a2-charon-8cae8fd`. Resolve and record an immutable image digest in M0. Do not invent it here or assume another engine's extraction is automatically compatible.

Keep `cargo tauten check` and the proof gate separate. Totality subsumption requires a current audited total claim for the exact function. Conditional splice theorems do not automatically remove every arithmetic finding.

## Gaps and upstream work

Two new focused drafts were prepared after source inspection and independent review:

1. [Stamp subject binding](../issues/tauten-stamp-binding.md): distinguish subject/incarnation from revision. Do not use a bare version stamp as Plyrune capture/report identity.
2. [Capability mapping accuracy](../issues/tauten-mapping.md): correct stale current-status documentation without overstating origin/effect guarantees.

See [submission status](../issues/STATUS.md) for actual creation outcome. An unsuccessful API attempt is not a filed issue.

Existing upstream issues already cover several useful extensions; do not create duplicates or make them blanket blockers:

| Existing issue | Relevance | Plyrune can proceed by |
|---|---|---|
| [#24](https://github.com/tauten-dev/tauten/issues/24) spans/edit maps | Shared span binding, kernel dependency extraction | Reuse/qualify current Splicery span logic; supply a concrete adopter case |
| [#27](https://github.com/tauten-dev/tauten/issues/27) digest framing | Domain-separated canonical artifact identity | Specify project framing and assumptions; no invented hash proof |
| [#19](https://github.com/tauten-dev/tauten/issues/19) project effect labels | Future project capability rows | Use supported fixed rows and explicit capability interfaces |
| [#22](https://github.com/tauten-dev/tauten/issues/22) record invariants | Cross-field capture/manifest constraints | Private checked constructors with domain proofs |
| [#20](https://github.com/tauten-dev/tauten/issues/20) stale/dedup admission | Session lifecycle and request reuse | Small project protocol; no claim about at-most-once host effects |
| [#86](https://github.com/tauten-dev/tauten/issues/86) profile drift | Generated AGENTS.tauten.md freshness | Record producer/digest and compare pinned generated output |
| [#90](https://github.com/tauten-dev/tauten/issues/90) reusable total audit | Existing native proof gate integration | Preserve the native audit with explicit provenance |
| [#79](https://github.com/tauten-dev/tauten/issues/79), [#91](https://github.com/tauten-dev/tauten/issues/91) proof-reuse input closure | Avoid unsound cached proof reuse | Prefer full admitted gates until reuse inputs are qualified |

A requirement for complete query inventory, independent task coverage or host atomicity belongs to Plyrune's semantics. Do not ask Tauten to infer those arbitrary domain facts from Rust syntax.

## Adoption record to commit

Record the selected source and release status; Rust/components; CLI/runtime/template pins; features/cfg; image digests; crate tiers; selected effects; generated-profile digest; baseline and reasons; claim registry; actual gate commands and resource limits; proof input closure; exception policy; known upstream dependencies; and each verification result with its exact source identity.

Do not claim Tauten reduces escaped defects from these materials. Its current STATUS explicitly reports that no such benefit has been established on its benchmark.
