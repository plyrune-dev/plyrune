# Shared semantic and interface contracts

Status: proposed 2.0 contract. The names below are design vocabulary; no wire compatibility is promised until M0 admits a versioned schema.

## Identity is layered

Use separate nominal identifiers for repository/source owner, store incarnation, snapshot, resource, artifact, requirement, candidate and request. A transient handle contains or resolves through an owner/incarnation and artifact kind. A content digest identifies a canonical representation under declared assumptions. Neither a handle nor a digest confers authorization.

Keep path identity, resource identity and blob identity distinct. Two paths may contain identical bytes; one path may name different content across snapshots. Explicitly define rename semantics. M1 can conservatively model a rename as remove plus add.

A proposed immutable artifact envelope contains schema version, artifact kind, owner/context identity, immutable subject identity, canonical payload identity and derivation dependencies. Exact storage encoding is an M0 decision. Do not put secret authorization tokens inside the content digest or public evidence bundle.

## Snapshot and acquisition profile

`Snapshot` is a validated immutable manifest of resources, content and relevant metadata. The profile records how it was acquired:

- `FixtureManifest`: explicit supplied bytes, finite inventory and provenance label.
- `GitTree`: exact committed tree identity, repository/source identity, entry paths/modes and blob identities. No dirty or untracked working-tree content is implied.
- Later `FilesystemSnapshot`: only with a qualified host mechanism.
- Later `CapturedManifest`: individually read immutable blobs assembled into a manifest. Useful, but it must not claim simultaneous capture of a changing source.

Before/after stat or metadata sampling can detect some races; it is not a universal coherence proof. Lazy inspection resolves immutable captured content, never rereads whatever currently occupies the original path.

M1 specifies path validation, Unicode/byte behavior, case sensitivity, symlink/submodule handling, file types, input-size limits and error precedence. Do not traverse symlinks by default. A skipped required resource prevents complete coverage. A resource excluded by the independently admitted requirement is a different case and remains recorded in scope semantics.

## Requirement

An admitted requirement is a separate immutable input with:

- accepted source/domain and path scope;
- predicate and matcher profile;
- exclusions and their meaning;
- allowed dispositions per required occurrence;
- expected replacement where applicable;
- optional explicit postconditions;
- admission provenance and version.

The host supplies/adopts it before candidate-specific selection. The model may propose it, but the tool must not infer independent authority merely from that fact. Do not introduce a mandatory human approval click for every edit: the host's existing authorized workflow can admit a precise requirement.

For M1, a literal is nonempty UTF-8 with no normalization. Search exact encoded bytes on valid UTF-8 resource content. Enumerate non-overlapping occurrences left-to-right in deterministic resource order. Matching inside comments or strings is intentional. Other matching semantics require different profiles and evidence.

If all occurrences must be replaced with fixed bytes, no arbitrary skip disposition is permitted. If exclusions are allowed, their reason and rule must predate the candidate. Define whether extra edits are forbidden; the M1 coverage check forbids them.

## Capture and selection

`Capture` binds snapshot, resource, half-open byte range `[start,end)` and exact content identity. Range admission checks owner/snapshot, bounds, overflow, encoding boundaries and supported resource kind. A `keep` operation reads its source from the original captured bytes.

`Selection` binds the query/requirement, domain, matching profile, exact capture membership, ordering and duplicate semantics, derivation and coverage state. Complete selections require an inventory witness, including files with no matches. Their construction is controlled by runtime validation and the qualified query implementation, not a caller's `complete: true` boolean.

Coverage is at least `Complete`, `Partial(reason, observedDomain)` or `Refused(reason)`. Timeouts, unsupported types, unreadable required files and search limits must be reflected. A 5-member display limit belongs to a view, not query completeness. A 5-match search limit makes the query partial unless completeness is independently established.

For set operations, define exact equality keys. Preserve dependencies of every operand. Difference and anti-join depend on excluded sets, including when the result is empty. Sorting and slicing depend on membership and ordering of discarded elements. A narrower selection cannot inherit the wider requirement's coverage automatically.

## View

`View` carries source artifact identity, projection specification, explicit omissions, source map and bounded display data. Support byte-budget and item-budget limits independently. Return enough metadata to explain whether omitted data exists and how to inspect it.

A view can be exact for its displayed slice while partial for its parent artifact. It cannot silently become an exact whole-source object. Transformations such as summaries, escaped paths or line-number decoration are presentation; edits still resolve through original captures.

## Change

`Change` binds base snapshot, admitted operations, original-source read dependencies, write targets and immutable candidate manifest. Candidate generation does not modify the source.

Validate all operations against one base. Reject mixed snapshots and overlapping targets. Preserve original semantics for zero-width inserts and tie ordering; if the inherited kernel refuses an ambiguous case, keep the refusal until an explicit extension is qualified. Do not guess an offset after previous edits; all original targets refer to the original snapshot.

Within M1, each file's splice operation is qualified independently; the manifest composition must establish that all required file candidates are present and unmodified resources/modes are retained. No multi-file publication is implied. Cross-file `keep` is outside the initial splice scope.

## Check and applicability

A `Check` identifies its proposition, exact requirement, selection, candidate, premises, matching/checker semantics and verdict. Use explicit outcome categories and reasons. Separate a proven checker algorithm from an individual execution report and from the truth/admission of its inputs.

The M1 report verifies: domain/matcher agreement with the requirement; exact occurrence coverage; permitted replacement; absence of unintended edits; preservation outside targets; exact candidate identity. Optional semantic or textual postconditions are separately named and evaluated. Do not infer arbitrary program correctness.

`Applicability` compares immutable dependencies to a specified current context. It yields applicable, inapplicable with named differences, or unknown. The operation is observational. Publication requires a host protocol that maintains the relevant relationship through its effect linearization point.

## Dependency rules

| Operation | Required dependencies |
|---|---|
| Query | Source inventory, bytes/metadata that affect matching, scope/exclusions, matcher/profile |
| Inspect | Parent identity, projection, display/mapping rules |
| Filter/project | Parent derivation and every input used by the predicate/projection |
| Union/intersection/difference | Both operands and equality/order semantics |
| Sort/take | All ordering keys and membership, including discarded members |
| Prepare | Base, targets, retained-copy sources, replacement data, all supporting selection/requirement inputs |
| Check | Candidate, requirement, premises, checker/profile and all evaluated inputs |
| Refresh | Previous derivation, new context/inventory and operation implementation/profile |

Dependencies should initially over-approximate safely. Minimality is a separate optimization claim. New-file detection needs inventory/domain evidence; retaining only previously read positive files is unsound for complete-query reuse.

## Proposed operation vocabulary

`snapshot`, `requirement`, `query`, `inspect`, `prepare`, `check`, `compare`, `refresh`, and explicit artifact export/import when supported. `apply` is absent from M1–M4.

Do not flatten them into a giant string command parser. Each request has a versioned tagged operation, a request ID, typed arguments, resource budget and explicit returned artifacts. Results have tagged success/partial/refusal/error forms. Bind responses to request IDs and context. Stdout is protocol only; diagnostic logs go to stderr with redaction.

A finite plan may reference prior node outputs. Validate DAG shape, reference kind, node limits and total budgets before evaluation. Deterministic nodes can execute without a model roundtrip; a judgment based on inspection needs a real suspend/resume boundary. No implicit privileged retries or arbitrary code nodes.

For long-lived stdio, define framing, maximum message size, cancellation, request overlap, process shutdown and handle lifetime. Bound retained bytes and reject/expire explicitly. Report whether an outcome is available after cancellation; never disguise a failed request as an empty result.

## Design fixture

The files in `examples/` are illustrative contract inputs, not a wire schema. Their fixture labels are human-readable IDs, not production digests. They demonstrate the exact small migration used in M1. A check must reject a query narrowed to `config/server/`, even if every match it did return is replaced.
