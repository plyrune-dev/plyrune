# Plyrune: one interface from source to checked change

**Implementation handover · 23 November 2026 · Reviewed design**

Plyrune unifies Tuft, Weft and Splicery around immutable, typed artifacts. An agent queries once, retains exact captures, inspects only the context it needs, and sends the same artifacts into an edit or a check. The identity and scope survive every step.

The product hypothesis is that this removes repeated discovery, transcription, reconstruction and recovery work. That hypothesis needs measurement. A more elaborate interface can cost more than a competent shell pipeline, and the historical pilots demonstrate that risk.

## 1. Product decisions

| Area | Decision | Reason |
|---|---|---|
| Umbrella | Plyrune; Tuft/Weft/Splicery remain internal names | Agents learn operations and types, not three products |
| Public surface | One library API, one CLI/stdio protocol, thin SDKs | Shared semantics across harnesses without a required orchestrator |
| Runtime | One binary and in-process modules first | Separate processes add coordination and lifecycle work before benefit is known |
| Semantic foundation | A small shared core with Lean claims over shipped Rust translations | Composition needs more than matching JSON fields |
| Initial acquisition | Explicit immutable fixtures and pinned committed Git trees | A scan of a changing working directory is not a coherent snapshot |
| Initial editing | Produce immutable candidates and diffs | Host publication needs a separate concurrency protocol |
| Initial query | Exact literal predicates over a declared finite inventory | An auditable starting point for completeness |
| Inspection | Optional bounded views over retained artifacts | Display limits must not silently reduce the selection |
| Requirements | Admitted separately from the proposed selection/change | Prevent vacuous success over a convenient subset |
| Evaluation | Competent native tools, scripting and batching available in every arm | Demonstrate an actual advantage, including interface overhead |
| Adoption | Tauten from the first build, with explicit scope and pin | Enforce engineering discipline while retaining domain obligations |

Unification is not a reason to discard existing proved kernels. Preserve Splicery's splice implementation and its evidence. Reuse suitable Weft finite-set/obligation machinery only where its semantics fit. Tuft's major evolution is complete, maintained selections with explicit query-domain dependencies.

## 2. What changes for each engine

### Tuft: from search output to retained selections

Keep language-neutral query/capture behavior. Return an immutable `Selection` whose members are exact `Capture`s and whose coverage describes the whole declared domain, including files with zero matches. Keep query parameters, matcher profile, exclusions, inventory and derivation inputs.

Inspection is a separate operation. Showing 5 of 500 captures does not change a 500-member selection into a 5-member result. A later candidate may use all 500 without asking the model to repeat their bytes.

Refresh creates a new selection and a relationship to the previous one. Positive results alone are insufficient dependencies: additions, deletions, negative matches and exclusion inputs can all change the answer. Start with conservative invalidation before attempting minimal incremental recomputation.

Literal/textual completeness means all matches of the stated predicate in the stated domain. It does not mean all semantic callers or all locations intended by a natural-language task.

### Splicery: the first practical delivery path

Expose exact replacement, wrapping and retained-source copies through shared capture handles. Familiar patch/front-end adapters may lower into the same candidate operation later. Do not make a new language the prerequisite for a useful edit.

Read dependencies and write targets are separate. A candidate can depend on an unmodified configuration file, a negative query or an independently admitted requirement. A changed target is not the only reason a candidate may be inapplicable.

Preserve current splice limits and refusal behavior until a reviewed change explicitly qualifies new limits. The inspected kernel's 4096-edit and 16 MiB constraints are real constraints; chunking them must not create an unproved global success or partial transaction. Existing proofs cover original-capture copies, disjoint targets, exact byte preservation and refusal conditions within their scope. Cross-file composition and domain mappings need their own claims.

### Weft: useful checks on concrete subjects

Start with exact coverage and preservation obligations over immutable artifacts. A check names the requirement, selection, candidate, premises and checker semantics. Refuse a report for different candidate bytes or a different requirement.

Separate a fact supplied by a host or a person from a proposition mechanically derived from bytes. Checking a formal statement does not establish that its natural-language premise is true. An ordinary edit should not require a manually authored proof ceremony; the common coverage check should be produced mechanically from the supported operation.

## 3. The unifying model

An artifact is immutable and addressable. An opaque wire handle resolves only within an explicit owner/store incarnation. A durable content identity is distinct from a session handle and from permission to read or write.

| Type | Carries | Does not imply |
|---|---|---|
| `Snapshot` | Identified immutable resource manifest and acquisition profile | A sequential dirty-tree scan was simultaneous |
| `Requirement` | Accepted scope, predicate, matching semantics, permitted dispositions, exclusions | The tool inferred the user's full intent |
| `Capture` | Snapshot, resource identity, byte range, exact content binding | Identical bytes elsewhere have the same provenance |
| `Selection` | Query, domain, exact members, ordering/duplicate rules and coverage | A request for complete results is itself evidence |
| `View` | Projection, source mapping, displayed subset and omissions | Displayed text is an exact editable source |
| `Change` | Base, operations, dependency closure and candidate identity | Prepared bytes may be published now |
| `Check` | Proposition, subject identities, premises, checker version and outcome | Arbitrary code correctness or execution authority |
| `Applicability` | Comparison of dependencies with a chosen current context | A host has atomically published the candidate |

Historical validity and live applicability are different. An old report can stay valid about its original candidate while becoming inapplicable to a changed working context. Refresh does not mutate or retroactively invalidate the original immutable object.

The authoritative details are in [CONTRACTS.md](docs/CONTRACTS.md).

## 4. A useful first example

The admitted task is: in all regular UTF-8 files under `config/` in a fixed manifest, replace every exact occurrence of `legacy_endpoint` with `service_endpoint`. Comments and string literals count because the predicate is textual. If the owner intends a semantic rename, this requirement is insufficient and must be changed explicitly.

The host admits this requirement before the implementation chooses its matches. The following is **proposed API notation, not runnable shipped code**:

```typescript
const base = await p.snapshot({ kind: "git-tree", repository, tree });
const req = await p.requirement(admittedRequirement);
const hits = await p.query({ snapshot: base, requirement: req });

// Optional. Retains the full selection; shows only a bounded view.
const view = await p.inspect({ artifact: hits, maxItems: 5, maxBytes: 4096 });

const change = await p.prepare({
  snapshot: base,
  targets: hits,
  replaceWith: "service_endpoint"
});
const report = await p.check({ requirement: req, selection: hits, change });
return { change, report };
```

No captures are reconstructed from the view. If the decision depends on what the model sees, return the view to the model and resume with retained handles in a later call. If replacement is fully specified, the harness can execute the deterministic steps in a single script without an intermediate model turn.

The check establishes, within the declared profile, that every required occurrence received the permitted replacement, no extra edit was introduced, untouched bytes were preserved, and the report refers to the exact candidate. A separate postcondition such as “the resulting files contain no forbidden literal” is evaluated explicitly; it is not inferred automatically from coverage, since edits can create new matches across boundaries.

An empty selection is `NoChange` with its complete-domain witness. It is not “migration performed.” If an unreadable or unsupported file prevents completeness, return partial/refused coverage and the blocking resources; do not award full coverage.

## 5. Architecture and module boundaries

Start with a small workspace:

| Component | Tauten tier | Responsibility |
|---|---|---|
| `plyrune-kernel` | kernel | Pure identities, admitted spans, selection/set decisions, edit admission and exact coverage decisions |
| `plyrune-core` | core | Immutable object store, validated DTO conversion, bounded plan evaluation, projection and adapter composition |
| `plyrune-cli` | edge | Stdio/CLI, manifest acquisition, bounded file reads, output and process lifecycle |
| `sdk/typescript` | outside Rust checker | Typed convenience handles and wire encoding; runtime still validates |

Within the kernel use focused modules such as `binding`, `capture`, `selection`, `change`, `coverage` and `dependency`. Add crate boundaries only for a real dependency, proof, authority or release boundary. Existing engine crates may remain separate dependencies if translation and proof composition actually work; M0 must demonstrate this before a new shared-kernel abstraction is committed.

All frontend routes lower to one checked representation. Reject malformed ranges, mixed snapshots, unknown object kinds, unknown schema versions and unbound results at admission. Serialization, recovery and imports must not bypass checked constructors.

The plan representation is a finite acyclic graph of known operations with bounded resources. It has typed references to earlier results, explicit returns and optional inspection outputs. No arbitrary code evaluator, loops, ambient filesystem access or plugin code execution lives in the semantic kernel. The harness already provides general-purpose orchestration.

## 6. Harness independence

Ship a library and a long-lived stdio service with the same semantics. An exec-only harness can use the CLI and explicit artifact bundle export/import. An MCP adapter can later expose the same small operation set. None of these transports becomes a separate truth source.

Session handles may expire; persistent bundles need an explicit format, schema version, binding verification and resource policy. Do not promise resume or replay before those paths are implemented and qualified. Handle expiry is a typed result with a safe recovery path, not an invitation to select similar-looking text.

The host controls which sources enter the store, resource budgets and authority. A handle is not a capability grant. An immutable store does not excuse leaking arbitrary repository content into a model response. Projection honors the host's authorized source scope.

Writfold is a possible consumer of Plyrune. It is not required and its historical worktree/authority protocol must not be assumed to be the current product contract.

## 7. Formal foundation

Use one shared set of identity, span, inventory, dependency and report contracts. Prove composition through each supported operation. Keep separate proof modules for the internal engines and explicit bridges between them.

The first composition claim is deliberately narrow:

> Given the identified immutable inputs and validated operations, the candidate uses the selected source bytes, preserves untouched bytes, and the coverage report refers to that exact candidate and independently admitted requirement.

Prove over the shipped production Rust via the existing Charon/Aeneas route where supported. A Lean design model can help specify a new contract, but it does not qualify a different Rust algorithm. Do not relabel tested canonicalization, acquisition or DTO mapping as proved because the leaf kernel is proved.

Hashes are concrete choices with assumptions. An injective encoding theorem and an abstract unique-ID theorem do not prove collision resistance. Scope and classify each assumption in [ASSURANCE.md](docs/ASSURANCE.md).

## 8. Live application is a separate milestone

The sequence “compare read-set hashes, then write” still has a race. A cooperative lock constrains only participants that honor it. A Rust permit does not lock an external file against another process.

M5 may qualify one host mechanism: a controlled private workspace with an enforced writer model, or publication of a fully specified candidate Git tree using an atomic reference comparison/update. A Git ref transaction does not update a dirty working tree or include untracked files automatically.

State precisely what is atomic, visible, durable and recoverable. Multi-file worktree publication is not implied by single-file rename. Cancellation, timeout or process death after an effect can leave an unknown outcome; recovery must inspect authoritative state. Do not advertise “prevents stale edits” until the chosen host contract warrants it.

## 9. Measurement and shipping decision

Measure full successful task delivery, not just kernel speed or output bytes. Include model calls, cached/uncached token use, setup, failure recovery, source IO, candidate errors and false refusals. Separate cold and warm runs.

Use three initial arms: competent native tools; one unified facade with ordinary text handoffs; shared artifacts and captures. Add incremental maintenance as a fourth arm only after the previous comparison is useful. Keep the model, harness, instructions, budget, corpus and independent grader aligned. Every arm can script and batch.

The early M2 pilot is a go/no-go checkpoint, not a marketing study. If success is at ceiling and the interface costs more, narrow the intended use case or simplify it. Do not make the baseline deliberately worse or move to a weaker model solely to produce a win. Publish negative results.

The website separates three things: currently available operations, exact assurance claims, and measured task outcomes. No score combines them into a “verified” badge.

## 10. Brand and design

Plyrune suggests several strands forming one precise mark. Use the **Join** direction from the identity board as the working preference. The warm ivory site is the recommended public face; the dark workbench is an alternative/documentation mode.

The name availability check was preliminary on 23 November 2026: no RDAP registration record for `plyrune.dev` or `plyrune.com`, no exact published npm/crates.io/PyPI package, and no exact GitHub repository-name result in the checks performed. Nothing was reserved. Recheck at registration; do not treat this as trademark clearance.

The website prototype has truthful pending states and local navigation. Generated designs are visual references. Before launch, create/approve production vectors and qualify favicon legibility, font licensing, accessibility, responsiveness and every public claim.

## 11. Finish criteria

The first release should let a fresh agent complete a real finite literal migration using retained captures, inspect a bounded view, obtain an exact candidate/report, and explain every refusal through one interface. Its source and proof pins must be reproducible, and its measured interaction costs must be visible.

Only then decide whether retained evidence and incremental maintenance justify expanding the scope. Preserve what is valuable in the three engines; make the path through them cheaper and easier to trust.
