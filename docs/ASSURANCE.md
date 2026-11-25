# Assurance and shared Lean foundation

This document specifies obligations. It does not assert that Plyrune's new claims have been proved.

## Evidence already available to preserve

Splicery's inspected README identifies `splicery-splice` as the production `no_std` crate translated with Charon/Aeneas. It describes exact admissibility/refusal, disjoint edits, original-source copies, untouched-gap preservation, order independence and bounded candidate behavior. Its domain-to-kernel mapping and candidate UTF-8 check are tested, not proved. The language calculus and TLA models have separate scopes.

Weft's inspected README identifies Aeneas proofs over `weft-core` finite-set operations, complete-trace classification and scope equality for reuse. Parsing, typing, provenance, budgets and reports are not thereby proved over the shipped whole program. Historical Writfold integration/authority is not inherited by Plyrune.

These are source-reported existing results at [pinned revisions](../SOURCES.md). This handover did not rerun them. M0 must reproduce the selected evidence before relying on it for a new release.

## Proposed first claim inventory

| Claim | Statement to establish | Production boundary | Required negative control |
|---|---|---|---|
| PR-C01 | Admitted capture resolves the named resource/snapshot range with exact bytes | Capture constructor/resolver | Omit owner or snapshot comparison |
| PR-C02 | Complete literal selection equals every supported match and carries a complete declared-inventory witness | Matcher + finite inventory traversal/witness | Skip a matching resource; separately omit a required no-match inventory witness |
| PR-C03 | Lowering retains target/source identity and range | Shared selection → splice input | Swap equal-looking captures across owners |
| PR-C04 | Candidate changes exactly admitted targets, preserving untouched bytes | Existing splice + manifest composition | Drop an untouched gap or file |
| PR-C05 | Accepted coverage answers the independently admitted requirement | Requirement/candidate coverage checker | Bind requirement to only returned matches |
| PR-C06 | Report refers to exact candidate, requirement, premises and checker profile | Check construction/verification | Substitute candidate or requirement |
| PR-C07 | Each supported operator preserves sufficient dependencies | Dependency algebra | Drop right operand of difference or a no-match resource dependency |
| PR-C08 | Composition preserves PR-C01–07 under named premises | Shared pipeline | Break an interface binding |

PR-C02 begins with an explicit immutable finite manifest. The claim must identify whether the scan is translated/proved or whether completeness uses a trusted supplied witness checked by a smaller kernel. Do not advertise end-to-end complete search if an unqualified producer can omit inventory entries. Acquire files through a separate qualified boundary.

Match-set equality alone does not detect omission of a file that contains no matches. The inventory-witness clause and PR-C07 dependency obligation cover that distinction. Calibrate each negative control against the property it actually violates; do not expect a pure match-set theorem to fail when its set remains correct.

Prove refusal correctness and totality where needed, not only the happy path. Non-vacuity controls must exhibit valid nonempty success, valid no-change and meaningful rejection. Never mark a broad theorem complete because the implementation refuses every input.

## Trusted bridges

### Proof-first sequencing (owner decision, 24 November)

| Claim | Target at the first relevant production boundary | If blocked |
|---|---|---|
| C01 | Prove store-bound text admission; then prove full capture binding. Test host ID assignment and acquisition separately. | Keep unresolved capture handles out of the shell. |
| C02 | Prove single-resource literal matching first; then inventory traversal or a complete-inventory witness checker. | Run differential controls but do not claim complete search. |
| C03 | Prove Plyrune lowering against the explicit cut/identity relation. Test the external Rust mapping. | Keep Splicery isolated; no qualified bridge claim. |
| C04 | Retain upstream splice proof; prove manifest composition and test integration with exact prepared order. | Report only the leaf and tested fixture results. |
| C05 | Prove checker success and refusal relative to the independently admitted requirement. | No production coverage verdict. |
| C06 | Prove report subject equality; separately test canonical bytes, decoding and replay. | No portable success report. |
| C07 | Prove negative inventory membership where required for completeness; operator reuse is conditional on measured demand. | Recompute on a new immutable input. |
| C08 | Document a composition argument listing proved decisions, tested host mappings and assumptions; no monolithic Lean theorem is promised. | Withhold end-to-end claims where a premise is unqualified. |

The failed cross-workspace extraction is not required for independent leaf proofs; it remains a blocker for claims depending on a single joint translation. TLA+ becomes useful if publication introduces concurrent writers or recovery choices, not for deciding equality or byte ranges.

### Single-resource matcher experiment (#34)

The isolated [Rust scan, Lean proposition and source control](../qualification/matcher-spike/) are a tractability experiment, **not** a new entrypoint or proof record for preview 1. `Spike.scan_spec` targets the actual extracted `scan` for arbitrary byte slices and either mode: reject an empty literal; otherwise produce exactly the greedy sequence of byte starts, stopping after the first hit in first mode. `Greedy` permits a one-byte skip only on a nonmatch and requires the entire literal to fit and agree at each hit. The skipped-match mutant starts at one instead of zero: on `abab` / `ab`, first changes from `[0]` to `[2]` and all from `[0,2]` to `[2]`. Near misses, complete-empty and refusal are separate controls.

From the repository root with the [pinned tool image](ADOPTION.md) built:

```sh
PLYRUNE_CPUS=4 PLYRUNE_MEMORY=8g docker compose run --rm -T --entrypoint timeout gate \
  --kill-after=30s 1200s python3 qualification/matcher-spike/run.py
```

One run gets 20 minutes, 4 CPUs, 8 GiB and 1024 processes. It copies just the probe and existing proof scaffold to scratch, compiles the Rust controls on both bodies, extracts with Charon/Aeneas, builds and audits Lean, and requires the re-extracted mutant to break `Spike.scan_spec` **directly**. Timeout, missing tool, build failure, missing audit or uncalibrated mutant is not a pass. The fresh translation digest and elapsed seconds are printed. The ephemeral drift comparison is a repeat extraction, not an independently committed baseline.

Code-shape cost: two explicit index loops (window comparison and cursor scan), one allocation-backed offset vector, no iterators or opaque slice equality. The obligations include bounds and arithmetic, prefix agreement, decreasing loops, output room and the fact that every skipped position is a miss. Complexity is worst-case $O(nm)$ byte comparisons for resource and literal lengths $n,m$, and $O(k)$ output for $k$ hits. No benchmark or production budget is implied. Rust allocation/physical exhaustion, UTF-8 admission, owner/snapshot/capture binding, inventory traversal, zero-match resource dependencies and the proposed 4096-hit limit are outside this spike. No matcher result may be advertised for the released binary on this basis.

Effort: initial agent investigation and authoring, approximately 0.1 wall-clock hours (22:03–22:09 UTC, 24 November), separate from extraction/proof time. This is not a human-effort estimate.

**Executed result:** the native baseline and mutant controls passed, and Charon/Aeneas generated `MatcherSpike.lean` (SHA-256 `80e4ca9ce7bcf3ed446c3e2b720719a66b287d3426754665b98bb49c419d05c9`). The first two attempts stopped at Lean build: the branch tactic split the nested first-hit branch too early (22.6 s), then the decreasing-cursor goal remained open (12.0 s). With the same Rust, statement and control, explicit branch selection and a natural-number arithmetic step closed the proof. The final run (45.0 s) passed build, audit and direct mutant calibration: `skip-initial-match` broke `Spike.scan_spec`. Proceed to the inventory witness obligation in [#5](https://github.com/plyrune-dev/plyrune/issues/5); do not count the experiment as a seventh claim in the preview.

### External bridges

| Bridge/assumption | Initial treatment | What it does not establish |
|---|---|---|
| Rust → Charon/Aeneas → Lean | Pinned translation trust base and drift gate | Correctness of unextracted Rust |
| Lean kernel and admitted axioms | Exact gate audit | Truth of external premises |
| Canonical artifact encoding | Specified format; tests/property checks; prove framing where useful | Cryptographic collision resistance |
| Digest equality | Named cryptographic assumption with version/domain separation | Authority or origin by itself |
| Wire decode/domain mapping | Production-path checked constructors and adversarial tests; prove selected mappings later | Validity of arbitrary serialized objects |
| Git/fixture inventory acquisition | Profile-specific validation and tests | Coherent dirty-working-tree snapshots |
| Source/requirement admission | Host-owned contract | Natural-language truth or complete user intent |
| Filesystem publication | Deferred to M5 host protocol | Atomicity merely from pre-write validation |

An abstract unique identifier can simplify a Lean model, but the mapping to concrete framed bytes and digests remains explicit. Avoid importing a cryptographic assumption as an untracked Lean axiom; make external assumptions parameters/premises of the claim.

## Proof organization

Use modules for `Binding`, `Capture`, `Selection`, `Change`, `Coverage`, `Dependency` and `Composition`. Reuse existing leaf theorems. The shared core should be small enough that callers can understand each premise.

For each admitted claim record: exact production function/body, source revision and input closure, features/cfg, translation revision, theorem names, preconditions and caller evidence, assumptions, deciding gate, positive controls, calibrated mutants, status and exclusions. Use Tauten's supported claim format when available; do not invent a parallel badge system.

The extraction must correspond to the shipped function. Any source move or dependency change needs translation/proof qualification, not just equal function names. Where two kernels compose, show that the upstream guarantee supplies the downstream premise. No circular “both are validated” argument.

Use independent status fields for source inspection, tests, model exploration and proofs. Missing tool, timeout, stale inputs, failed audit and surviving mutant are non-success outcomes. An old proof can remain historically valid while the current build has no fresh evidence.

## Protocol assurance

M2 needs bounded transport and lifecycle behavior, but no filesystem publication proof. M3 needs dependency/revalidation semantics. M5 chooses its concurrency model and proof/model tools based on the actual host protocol.

A model transition called atomic needs a real mapping. A single-file publication theorem does not imply a multi-file or cross-repository transaction. Process death and storage power loss have distinct failure relations. A host lock's exclusivity assumptions must include the relevant writers or its guarantee is cooperative only.

## Wording for the website

Before implementation: “Planned composition claims.” After qualification: “These listed properties are proved over these translated functions at this revision, under these assumptions.” Link the claim record and reproduction steps.

Avoid “verified agent,” “correct edits,” “safe at any time,” “all callers,” or “prevents stale edits” unless the exact corresponding scope is established. Performance and assurance are separate pages and separate evidence.
