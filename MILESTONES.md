# Delivery milestones

These are the **original design boundaries**, not calendar promises or live shipping status. For current delivery use the [issue milestones](https://github.com/plyrune-dev/plyrune/milestones). The owner selected a short path to real-file editing: [literal references](docs/rfcs/2026-11-26-literal-references.md) → [checked replacement](docs/rfcs/2026-11-26-reviewed-replacement.md) → [explicit Linux filesystem destinations](docs/rfcs/2026-11-26-filesystem-edits.md). It recomputes on every apply instead of assuming M3 reuse, does not require Git, and makes **no** M4 performance or general workflow claims. The broader retained-session, evaluation and portable-report obligations remain separate.

| Milestone | Depends on | Outcome | Exit evidence |
|---|---|---|---|
| M0 · Contracts and baseline | — | Source pins, Tauten profile, shared contracts and native-tool reference tasks | Reproducible bootstrap, adoption record, admitted fixtures, baseline traces |
| M1 · Pure composition | M0 | Immutable input → literal selection → exact candidate → coverage report | Claims over shipped kernel, bridge tests, adversarial controls, no workspace writes |
| M2 · Agent interface | M1 | One CLI/stdio surface with retained handles and bounded inspection | Packaged consumer, handle refusals, real harness trials, early cost comparison |
| M3 · Revalidation | M2 | Sound conservative reuse and immutable refresh | Inventory/negative-dependency controls and measured recovery cost |
| M4 · Controlled evaluation | M2; M3 for incremental arm | Reproducible paired study and evidence-driven website | Independent grading, complete raw records, uncertainty and negative results |
| M5 · Optional publication | M1–M4 plus host protocol | One qualified live-application adapter | Concurrency mapping, fault/race tests and explicit recovery guarantees |

## M0 — Establish the contract before expanding code

Deliver:

1. Pin playbook, Tauten, Splicery, Weft and the Tuft archive sources. Inventory code, licenses, proofs, generated translations and trusted mappings.
2. Complete the project-adoption record: selected gates, commands, toolchain/image digests, feature matrix, evidence paths and exceptions. Bootstrap three crates with Tauten tiers; keep the initial baseline empty.
3. Specify the M1 literal matcher, immutable manifest, independent requirement and canonical artifact encoding. Admit positive/negative fixtures before implementation.
4. Run one extraction spike involving the retained Splicery kernel and the proposed shared identities. Decide whether existing crates can compose through qualified dependencies or need a minimal source-preserving move. Do not assume kernel-to-kernel extraction support.
5. Select six development tasks: simple lookup, simple patch, repeated inspection, broad literal migration, narrowed-scope trap and stale/negative dependency recovery. Run competent native reference workflows with batching available.

Exit: another agent can reproduce the bootstrap and understand every assumption; a concrete end-to-end M1 fixture is agreed. If extraction fails, keep existing kernels independent and qualify the narrow bridge. Do not solve it by silently proving a different algorithm.

## M1 — One pure, useful vertical slice

Deliver in order:

1. Checked nominal bindings and immutable fixture manifest. Support a pinned Git-tree adapter after the fixture path works; classify its parsing/acquisition boundary separately.
2. Nonempty literal matching with exact scope, deterministic non-overlapping match ordering and full inventory coverage. No implicit normalization or semantic-source claim.
3. Bounded `View` over retained `Selection`. A view of five members leaves every member available downstream.
4. Exact candidate preparation with the reused splice kernel. Group operations by resource; refuse overlaps/unsupported cross-file copies. A candidate manifest preserves unmodified resources and relevant modes.
5. Independent requirement check: exact selection scope, allowed dispositions, no unintended edits, preservation and exact candidate/report binding.

Acceptance controls: wrong owner, wrong snapshot, changed requirement, narrowed scope, empty/no-change, no-match file, duplicate members, overlapping spans, UTF-8 boundary error, limit exceeded, mixed input snapshots, altered report subject, partial view versus complete selection.

Evidence: use the [per-claim targets and blocked behavior](docs/ASSURANCE.md#proof-first-sequencing-owner-decision-24-november). Prove the selected Plyrune decisions over their own translated functions; retain Splicery's independent proof and test the identity/lowering bridge. C08 is a documented argument about those results and their premises, not a joint Lean theorem. Classify acquisition and canonical encoding separately.

Exit: deterministic golden fixture plus real production-path checks, with the specified proofs and calibrated controls passing at the recorded revisions. An upstream fix for joint extraction is **not** an M1 dependency. A blocked leaf withholds its dependent guarantee; tests or the single-resource C02 spike cannot stand in for inventory-wide completeness. No workspace mutation endpoint exists in this slice.

## M2 — Make the interface cheaper to use

Deliver one library API, one long-lived stdio process, compact JSON envelopes, retained handles, batch/finite-plan evaluation, typed refusals, inspect and preview commands. Add a thin TypeScript client after wire semantics are stable. Export/import only if a real exec-only consumer requires it; otherwise make process-lifetime scope explicit.

Use exact operation names in [CONTRACTS.md](docs/CONTRACTS.md) as draft vocabulary; settle wire spelling/version in a small RFC before release. SDKs may provide ergonomic helpers but cannot create trusted domain objects locally.

Acceptance: packaged binary on a fresh fixture; unknown/mixed/expired handles refuse; no retyping of retained source; output budgets bound responses; cancellation and resource exhaustion have explicit outcomes; stdout contains only protocol data in stdio mode. Separate source-data strings from executable instructions.

Run an early paired pilot on the M0 tasks. Record total task cost, correctness and recovery. If the common workflow is more expensive with no meaningful benefit, simplify the interface before M3. Preserve the native lookup/patch fast path; do not force proof ceremony on tasks outside the supported contract.

## M3 — Revalidation before clever incrementality

First compare complete conservative dependency closures. Then add fine-grained invalidation only where traces show worthwhile reuse. Refresh emits a new snapshot/selection/change/report family; old evidence stays immutable.

Required scenarios: a new matching file; deletion; rename; an unchanged positive match plus changed no-match file; changed exclusion/ignore policy; empty search becoming nonempty; difference/anti-join inputs changing; matcher version change; ordering-sensitive slice changing; owner/store restart; requirement change. Unsupported operations force recomputation.

Exit: no unjustified reuse in controls, known false-refusal/recompute costs measured, scope/applicability returned explicitly. Do not claim minimal dependencies or exact invalidation unless proved for each supported operator.

## M4 — Publish measured outcomes

Pre-register the experiment described in [EVALUATION.md](docs/EVALUATION.md) before held-out runs. Start with the native/facade/shared-artifact comparison. Add M3 incrementality as a separate ablation, with cold and warm traces and explicit changes between tasks.

Exit: every trial, failure and intervention is accounted for; independent oracles check candidate correctness and task coverage; raw usage and wall time are reproducible; uncertainty and supported scope are shown; site data is rendered from versioned results. A ceiling result is published as a ceiling result.

Stop/reshape: if no useful task family improves without unacceptable cost, keep the valuable kernel/library pieces and reconsider the broader product. Do not keep adding framework machinery to manufacture a benchmark win.

## M5 — Optional live publication

Pick one host contract only after a concrete adopter needs it. Preferred first research options: a fully controlled private workspace, or a candidate Git tree published through compare-and-swap of a specific reference. Neither is a general dirty-worktree transaction.

Specify authority, read-set applicability, actual linearization point, external-writer behavior, modes/symlinks, dirty/untracked files, partial/unknown outcomes, retry identity and recovery. Map any model-level atomic action to real host operations.

Acceptance: a planted recheck/write race is detected or excluded by an enforced assumption; noncooperating writers are handled honestly; failure between each effect and acknowledgement is tested; crash versus power-loss evidence is distinct. Inherited single-file rename proofs do not qualify multi-file publication.

## First three changes to land

1. **Contract and bootstrap:** adoption record, pins, three crate boundaries, admitted fixtures, Tauten fast gate, extraction spike result.
2. **Pure path:** checked bindings and complete literal selection, reused splice, exact candidate and independently scoped report; first composition evidence.
3. **Consumer path:** retained stdio session, bounded inspection, packaged SDK/CLI example and early paired pilot.

Each change reports what was executed and what remains planned. No issue count, proof count or line count is a substitute for these outcomes.
