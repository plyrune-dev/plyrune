# Plyrune engineering instructions

This is the proposed root `AGENTS.md` for the implementation repository. This handover contains design material and an offline website prototype, not an implemented Rust workspace. Do not report the commands below as executed until the implementation and toolchain exist and the commands have actually passed.

## Read and maintain the contract

Read `HANDOVER.md`, `MILESTONES.md`, `docs/CONTRACTS.md`, `docs/ASSURANCE.md`, `docs/TAUTEN.md` and the generated `AGENTS.tauten.md` when it exists. Apply the relevant local crate instructions. This profile adopts the engineering playbook at `tknawara/playbook@cca8e068d0843af5612522049f08d8d645252f7c`; changes to that pin are explicit migrations.

Explicit owner decisions remain authoritative. Select gates for the changed obligation; do not import every tool or optional gate from another repository. The playbook PDF regeneration rule governs edits to the playbook, not ordinary Plyrune changes.

Every change must identify the invariant, authoritative state, assumptions, effect boundary and evidence it affects. Distinguish a requirement, a source observation, an executed check and a proof. Tests, a command exiting zero, a comment and a type name are not interchangeable evidence.

## Architecture

- Keep semantic decisions in a small pure kernel. No ambient filesystem, environment, clock, network, random source or hidden policy inputs.
- Use focused modules and explicit responsibilities. Split crates for authority, dependency, proof or release reasons; avoid monoliths and arbitrary one-function crates.
- Dependencies point inward: edge → core → kernel. Do not move code outward or change a tier to silence a finding.
- Model protocols as pure state transitions with explicit commands; drivers perform the effects. Lifecycle, retries, cancellation and recovery are part of the protocol, including single-process async code.
- UI and renderers consume immutable views and emit typed intents. Rendering must not mutate semantic state or perform uncontrolled IO.
- Bound queues, buffers, reads, plan nodes, output, work and retained storage. A `no await` function can still block; measure relevant responsiveness.
- Keep the public API harness-agnostic. Writfold is an adapter/client, not a required runtime. Do not introduce a general-purpose evaluator inside Plyrune.

## Types and admission

- Use nominal types for owners, store incarnations, snapshots, resources, artifacts, requirements, candidates, requests, byte offsets and byte lengths. A type alias is not nominal separation.
- Make invalid construction private. Express static restrictions in Rust; check runtime facts at smart constructors and every trust boundary.
- Decode wire values into untrusted DTOs, then validate into domain types. `Deserialize`, `Default`, `From`, recovery, imports, FFI and undo paths must not bypass admission.
- Check owner, incarnation, artifact kind, schema, snapshot and content binding on handle resolution. A well-typed SDK handle is not sufficient wire validation and does not grant authority.
- Do not conflate byte offsets, character indices and line/column display coordinates. Define half-open bounds and encoding profiles explicitly. Cover empty ranges, Unicode boundaries, CRLF, overflow, oversized inputs and invalid encodings.
- Counters do not wrap or silently reset. Exhaustion and incarnation changes have typed behavior.
- Make error outcomes domain-specific. Preserve `Complete`, `Partial`, `Refused`, `Unknown`, `Cancelled` and `NoChange` distinctions where meaningful. Do not turn an error into an empty successful result.
- Release builds retain correctness, identity, authority and bounds checks. A debug assertion may supplement an invariant, never implement a required admission check.
- Use checked arithmetic unless saturation is the specified domain behavior. Do not use saturation to hide a failed invariant.

## Artifact semantics

- Artifacts are immutable. Refresh creates a new artifact and an explicit predecessor relation.
- An inspection view never becomes the authority for a full selection. Preserve source maps, omissions and completeness separately.
- Completeness names a requirement/query, scope, inventory and matcher profile. It accounts for zero-match resources and unsupported/skipped/error states. `requireComplete` is a request, not a witness.
- Requirements are admitted independently of proposed matches. Do not narrow a requirement to make a check pass.
- Dependencies include everything that can affect an answer: positive and negative inputs, exclusions, inventory membership, ordering and interpretation. Preserve dependencies through filtering, difference, joins and discarded branches.
- Keep historical evidence validity separate from current applicability. A fresh comparison is not a publication transaction.
- Report-to-subject binding includes candidate, requirement, premises and checker identity. Equal-looking output does not justify substituting a different subject.

## Proofs and correspondence

- State the obligation before implementation. For new semantic decision logic, implement the smallest production kernel function, extract it and establish its selected Lean claims before exposing it through the shell.
- Use the established Rust → Charon/Aeneas → Lean path where it fits. A separate reference model may guide design but cannot stand in for the production algorithm.
- Preserve existing proved splice behavior and explicit limits. Reuse via a qualified dependency or a provenance-preserving extraction; do not rewrite for stylistic uniformity.
- Record exact source, features/cfg, translation, toolchain, theorem, callers/preconditions and trust assumptions. Check theorem coverage of shipped entrypoints, including refusal paths.
- Keep canonicalization, decoding, acquisition and host IO bridges classified independently as proved, tested or assumed. Do not inherit a leaf proof across an unqualified mapping.
- Lean claims use only the admitted standard axioms. No `sorry`, `admit`, hidden axioms or runtime-decider shortcuts in release assurance. Follow the selected gate's exact audit policy.
- A conditional correctness theorem is not an unconditional totality theorem. Never claim panic freedom or Tauten subsumption through a premise that callers have not established.
- For an assurance-critical guard, add a focused negative control or source mutant and a positive companion. Calibrate the intended failure; a mutant failing for unrelated parse/build reasons is not meaningful proof sensitivity.
- Do not weaken a theorem, remove a mutant, expand assumptions or change an evidence selector just to pass. Treat those changes as semantic-policy changes and report their implications.

## Tauten from the first build

- Follow the pinned development adoption in `docs/TAUTEN.md`. Record Rust and CLI/runtime revisions and lock dependencies. Do not silently follow `main` or upgrade to whatever is latest.
- Every Rust crate declares its tier. Kernel crates are `no_std`, deterministic and exception-free. Core receives explicit capability parameters. Edge creates authority and owns external IO.
- Declare supported effect rows where useful; record unresolved/custom capability classifications as limitations. Tauten discipline is not OS confinement or complete transitive effect inference.
- `cargo tauten check` must pass. It includes the structural/compiler/Clippy profile; it is not the proof gate. Run the selected proof gate separately.
- Start greenfield code without a legacy debt baseline. A genuinely reused engine may carry a narrowly reviewed baseline with owners and reasons. Never regenerate or enlarge a baseline in CI to pass.
- Use the smallest justified `#[expect(..., reason = "...")]` where the selected rule permits it. No kernel exceptions, stubs or unsafe escape. The narrow reasoned `dead_code` allowance for shared test helpers is governed by the pinned Tauten rule, not a general `allow` permission.
- Do not use bare `Stamped<P>` as a complete owner/snapshot binding. At the inspected pin it checks only a revision. Use explicit project bindings until the upstream contract is fixed and qualified.
- `AGENTS.tauten.md` is generated guidance. Record its producer and digest; check drift locally until upstream #86 provides the supported flow. Do not hand-edit it into a contradictory second profile.

## Selected gates

M0 must commit an adoption record with exact commands, toolchain/image pins, features, timeout/resource policy, input closure and evidence locations. The intended baseline after workspace bootstrap is:

```sh
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 tauten check --format json
cargo +1.98.0 test --locked --workspace
```

Use `cargo +1.98.0 test --release --locked -p plyrune-kernel -p plyrune-core` for the admitted release-behavior controls. Once proof integration is present, `cargo +1.98.0 tauten proofs` is a distinct required semantic gate. Preserve the established native proof gate if adapting it is not qualified yet; lack of integration is not a successful proof run.

Run targeted changed-path checks while iterating and all required gates at the agreed admission boundary. Broaden testing only for new failures or unresolved risks. Do not add tests that merely restate a low-impact presentation edit.

Relevant additional gates are: fuzz/property checks for untrusted decoders and range admission; proof audits and calibrated mutants for semantic decisions; installed CLI/stdio consumer tests for adapters; protocol exploration and real fault/race injection when live publication is introduced. Do not impose Verus, TLA+, TLAPS, Loom and Lean simultaneously without a distinct obligation for each.

## Effects, recovery and delivery

- M1–M4 prepare candidates; they do not write the user's working tree.
- A future apply adapter must name its concurrency model and actual linearization point. A last-minute recheck or advisory lock is not enough against unrelated writers.
- Distinguish atomic visibility, durability, multi-file composition and recovery. Process-kill testing does not prove power-loss behavior.
- Cancellation does not prove a physical effect stopped. Preserve partial and unknown outcomes; reconcile against authoritative state before retrying.
- Test a packaged consumer through the production entrypoint before calling an adapter shipped. Internal unit tests alone do not qualify packaging, permissions or filesystem behavior.
- Publishing, releases and migrations follow the owner's task authorization. This handover authorizes implementation planning, not automatic public release.

## Review and completion report

Explain the behavior change and why it is needed; identify exact checks run and material limits. Retain failures and blocked checks. Never report missing tools, skipped proofs, source inspection or cached historical results as a fresh pass.

Update public examples, schema and relevant assurance/evaluation records when their semantics change. Keep historical measurements tied to their original inputs. Public claims must be generated or checked against current admitted evidence; no manually improved benchmark numbers.
