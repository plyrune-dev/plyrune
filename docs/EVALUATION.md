# Evaluation and website measurements

Plyrune 2.0 has no measured results yet. This is a proposed experiment design. Version and freeze the final protocol before held-out trials.

## Questions worth answering

1. Does sharing exact artifacts reduce cost or error versus ordinary text handoffs through the same interface?
2. Does it help beyond a competent native workflow with scripts, patches, batching and retained files available?
3. Does dependency-aware reuse make repeated work cheaper without accepting stale conclusions?
4. Which failures are prevented, and what are the costs of false refusals and recovery?

Pure engine speed, fewer returned bytes or more proofs are not answers to all four.

## Arms

| Arm | Available workflow | Purpose |
|---|---|---|
| A · Native | Harness read/search/patch plus shell or code-mode scripting and batching | Real competitive baseline |
| B · Unified facade | Same advertised operations and batching, ordinary serialized text handoffs | Isolate benefit of a single interface |
| C · Shared artifacts | Retained exact captures, views, candidate/report identities | Isolate artifact/type composition |
| D · Incremental | C plus admitted dependency-aware reuse | Measure maintenance/recovery, after M3 |

Avoid artificially disabling a native tool or useful shell post-processing in A. All arms receive adequate short documentation. Measure its token cost. Record arm differences and interventions rather than hiding them.

## Task families and independent oracles

- Ordinary single lookup and simple patch: control tasks where native tools should remain strong.
- Broad finite literal replacement: missed/extra occurrences and untouched-byte comparison against independent fixture expectations.
- Repeated inspect/prepare/check workflow: reuse exact captured values without transcription.
- Large retained set with small display: show a few members but edit/check the full required set.
- Stale-support recovery: supporting file changed while target bytes stayed unchanged.
- Inventory changes: new file, removal, rename, previously empty query becomes nonempty.
- Scope traps: accepted requirement covers a wider domain than the agent selected.
- Difference/exclusion dependencies: a discarded operand changes the required outcome.

Use real tasks where possible and label adversarial mechanism tests separately. A planted guard violation demonstrates that a mechanism catches it; it does not estimate everyday agent defect reduction. Keep task specification quality constant and audit ambiguities before grading.

The grader must not just call the same production checker. Use independently authored exact expected candidates/sets and untouched-file checks for the finite fixtures. With real code tasks, separate build/test success, requested-behavior correctness, coverage and unintended changes. Hidden grading material stays out of agent workspaces and tool hints.

### Requirement admission in the six-task pilot

Proposed for [#12](https://github.com/plyrune-dev/plyrune/issues/12), using the [three accepted admission modes](rfcs/2026-11-24-requirement-admission-authority.md). These are assignments for a future paired protocol, not results of the native fixture trace or evidence of an adopter. Production proof gates precede comparative trials.

| Development task | Who fixes the scope before selection? |
|---|---|
| Single lookup | `host-authored`: experimenter fixes the lookup target. No migration report required. |
| Single patch | `host-authored`: experimenter fixes the target and replacement. Keep the native fast path. |
| Repeated inspection | `model-proposed, human-confirmed`: reviewer confirms the task and inventory counts before the first selection; subsequent views do not alter it. |
| Broad literal migration | `host-authored`: use a protected base-revision requirement, not a policy supplied in the candidate. |
| Narrowed-scope trap | **`model-authored` primary trial**: agent receives the broad natural-language request and writes its own formal scope. Also run separate paired `host-authored` and `model-proposed, human-confirmed` controls on the same task. |
| Stale/negative dependency recovery | `host-authored`: keep the task fixed; on the changed manifest record a fresh snapshot-specific admission. |

Keep mode, authority and review opportunity the same across competing arms in each pair. For each scope-trap trial, retain the original request, proposed and finally admitted scope, inventory counts, reviewer intervention/time (if any), and the candidate. Grade independently against **both** the hidden broad task and the exact stated task. A self-narrowed requirement can legitimately pass the latter and fail the former. Do not give the model the hidden formal scope or repair its statement in the model-authored arm. Record failed/refused confirmations, model drafting and human review cost; report by mode, not pooled as independent coverage successes. A separate fixed-input control can vary just the mode for identical narrow bytes to check the report qualification without estimating a user benefit.

## Trial design

Begin with six development tasks and repeated paired trials to discover workflow problems, not headline statistics. Use at least two real harness adapters before claiming broad harness independence. For held-out evaluation, use variance/failure rates from the pilot to choose task count and repeat count, then pre-register them. No arbitrary small sample guarantees power.

Keep model/provider revision, harness revision, budgets, task prompts, corpus commits, machine limits and cache policy aligned. Randomize paired arm order. Isolate workspaces. Record concurrent contention; prefer uncontended timing runs. Do not silently rerun failures. Define retries, timeouts, exclusions and missing-usage handling in advance.

Report cold setup and warm retained-state costs separately, including index/store/proof-cache setup and amortization assumptions. Report a complete task cost as well as per-step breakdown. Include unsuccessful trials in the denominators and aggregate cost; avoid success-only latency bias.

## Primary and diagnostic measures

| Measure | Definition / caveat |
|---|---|
| Task success | Independent oracle, all trials included |
| Time to correct result | End-to-end wall time including retries/setup as specified; timeouts retained explicitly |
| Model usage/cost | Input, cache read/write, output tokens and actual billed cost if available; price/version dated |
| Model roundtrips | Distinct from internal tool operations and reported harness calls |
| Unintended/missed changes | Exact candidate comparison or independent task oracle |
| False refusals | Valid operations refused under the supported contract, including recovery cost |
| Incorrect reuse | Stale/inapplicable artifacts accepted in controlled scenarios |
| Source bytes/model bytes | Source IO and returned context measured separately |
| Runtime overhead | Cold/warm execution, retained storage, peak memory and relevant tail latency |

Use paired differences and uncertainty intervals appropriate to the task/repeat structure. Show raw trial records and per-family results; do not promote a subgroup chosen after seeing outcomes into the primary claim.

A possible shipping objective is equal-or-better task correctness with a useful cost/recovery benefit on a declared task family. Choose material thresholds from actual adopter costs before the held-out run. Report any correctness/cost tradeoff explicitly rather than averaging it into a single score.

## Historical pilot: keep separate

The 10 November projection debug pilot is not a Plyrune 2.0 result. Its README reports both arms at 12/12 correct, native total tokens 424,110 versus Tuft 437,387 (+3.1%), and model-facing output 43,435 versus 40,994 bytes (−5.6%). Tool calls excluding hand-back were 21 versus 30. Trials ran three at a time, so its wall times contain contention. The report concludes no token win.

That result motivates measuring full workflow cost and shared handoffs. It does not establish that 2.0 will win. The archive is private; do not publish private transcripts, source or report extracts without the owner's publication decision. Public website copy can simply say the new design is unmeasured until an approved public report exists.

## Website data contract

`website/measurements.json` is an explicit empty state, not fabricated results. It has no sample count, effect size or success percentage. The offline prototype renders this same pending state.

Once evaluation exists, generate site data from versioned admitted results. Each record links source commit, model/harness/corpus versions, method, complete denominators, failure policy, estimate/interval, raw data and reproduction steps. A presentation build must not silently drop failed trials or unsupported metrics.

Assurance records are separate from performance records. “X listed functions have fresh proofs” is not “X% fewer defects.” Keep unmeasured, unavailable and zero distinct in rendering. Allow negative results and regressions to display without manual editing.
