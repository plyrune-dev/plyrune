# Independent review and disposition

Reviewer task: challenge shared types, materialization, Lean composition, dependencies, harness ergonomics, evaluation and publication. The reviewer was explicitly requested by the owner. Review date: 23 November 2026.

| Finding | Disposition in this handover |
|---|---|
| Unified transport alone misses composition errors | Shared identity, completeness, dependency and subject contracts are the center of M0/M1 |
| Dirty-file scans are not coherent snapshots | M1 uses immutable fixture manifests and committed Git trees; acquisition profiles are explicit |
| Selection completeness needs scope/inventory/matcher witness | Added full domain coverage, zero-match files, exclusions and unsupported/partial states |
| Coverage can be vacuous if the agent chooses its own requirement | Requirement admitted separately; narrowed and empty-scope controls included |
| Historical validity differs from live applicability | Separate Applicability judgment; immutable refresh objects |
| Report can be reused for a different subject | Candidate, requirement, premises and checker bindings required |
| Pre-write recheck leaves a TOCTOU gap | M1–M4 candidate-only; M5 qualifies an actual host mechanism |
| Three processes impose premature complexity | One binary/library with internal modules initially |
| Native baseline must include competent batching | All arms can script/batch; facade/shared/incremental ablations added |
| Whole-program proof claims would exceed inherited evidence | Bridge/trust table and production correspondence obligations added |
| First pipeline should be small and useful | Complete literal migration and exact coverage/preservation report selected |
| Framework growth could outrun benefit | Early M2 pilot and explicit stop/reshape decision |
| Final document review: skipping a no-match file need not falsify match-set equality | PR-C02 strengthened with complete inventory-witness coverage; matching-file omission and no-match witness/dependency omission have separate controls |

## Additional Tauten review

The reviewer independently inspected the pinned origin carrier. `Stamped<P>` stores a fact and revision, and `verify` compares only revision before returning evidence about the supplied subject. Distinct same-typed subjects at revision zero are therefore a source-level counterexample to subject binding. The reviewer confirmed this is distinct from rejected lifetime-brand expansion and not already covered by completed issue #9's acceptance controls.

The reviewer also confirmed current `docs/MAPPING.md` lags shipped evidence/effect/proof capabilities. Existing #86 covers stale adopter profile copies, not this current mapping table.

Both findings are source-inspected, not an executed Rust reproduction. Exact issue drafts and submission status are included under `issues/`.

## Accepted deferrals

Separate runtimes/distributed storage; automatic rebasing/semantic merging; general-purpose VM/DSL; broad semantic search/language-server integration; propositions automatically treated as true from prose; cross-repository transactions; arbitrary test-result reuse without environment/dependency closure; mandatory hand-authored proofs for ordinary edits; wholesale rewrites of proved kernels.

## Remaining implementation decisions

M0 must settle the canonical encoding/digest format, precise resource/path/encoding profile, initial budgets, shared-kernel dependency extraction, wire framing and exact SDK spelling. These are implementation decisions within the approved design direction, not reasons to pause all useful work. Record them with their consequences and controls.
