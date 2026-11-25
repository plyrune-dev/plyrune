---
rfc: 2026-11-24-requirement-framing
title: "Bind literal migration requirements independently of matches"
status: accepted
implementation: not-started
authors: [OpenCode]
created: 2026-11-24
updated: 2026-11-25
issue: https://github.com/plyrune-dev/plyrune/issues/3
milestone: Next · Proved subject binding
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Bind literal migration requirements independently of matches

## What has an identity?

The task is not the set of matches the candidate author chose. For example, the `config/server/` edits can satisfy a narrower task but cannot discharge a task over `config/`. The verifier needs the intended task **and** the intended base from outside the candidate. A digest identifies bytes under a collision-resistance assumption; it does not admit them.

Give a literal task a reusable content identity `R`. A host record `A` admits `R` on one expected fixture snapshot `S` in one source and owner context, with the [admission mode and evidence](2026-11-24-requirement-admission-authority.md). A new snapshot needs a new admission even when `R` stays the same. Including `S` inside `R` was rejected because it changes the identity of an unchanged task on every rebase.

## Fixture-v1 encoding

`F(x)` is `u64be(len(x)) || x`; lengths count **bytes**. Hash with SHA-256. In a preimage, digests are exactly 32 raw bytes; hex is display only. Concatenation below is in the listed order:

| Identity | Preimage |
|---|---|
| Snapshot `S` | [fixture/v1](../FIXTURE-PROFILE.md): framed tag, unframed u64be inventory count, then framed path, mode and bytes per sorted resource, including out-of-scope and no-match files |
| Requirement `R` | `F("plyrune/requirement/literal/v1") || F(prefix) || F(needle) || F(replacement)` |
| Admission `A` | `F("plyrune/admission/fixture-literal/v1") || F(owner) || F(source) || F(S) || F(R) || F(mode) || F(evidenceRef)` |

Tags and mode names are their exact ASCII spellings. `owner`, `source` and `evidenceRef` are nonempty host-resolved opaque byte identifiers, at most 255 bytes each. `source` identifies the inventory's origin within the owner context, **not** its bytes. `evidenceRef` identifies immutable admission evidence resolvable by this verifier; it is neither a secret nor an arbitrary claimant-supplied URL. No timestamp, JSON layout, human fixture ID, candidate, match, display limit or `admittedBeforeCandidate` boolean enters `R`. The host verifies the origin and ordering of admission independently of these hashes. This is not a portable report schema ([#39](https://github.com/plyrune-dev/plyrune/issues/39)) or a candidate hash.

The `R` tag fixes recursive, case-sensitive prefix matching on the regular UTF-8 fixture inventory; literal UTF-8 bytes, no normalization, left-to-right non-overlapping matches, all must receive the fixed replacement, no extra edits, exclusions or postconditions. Prefixes follow fixture path-segment rules, with **one final slash**; `config2/` is outside `config/`. The needle is nonempty; an empty replacement is deletion. Unsupported options are refusals, never ignored defaults. Changing any of these semantics requires a new profile, not new bytes under this tag.

## Refusal order and budgets

These stages constrain the future fixture checker, not the existing snapshot CLI. Stop at the first failing stage; never turn an incomplete scan into an empty success.

1. **Acquire bounded inputs.** Apply transport and manifest admission before interpreting the task. Inclusive limits: 256 resources, 255 bytes per path, 8 MiB per input resource, 32 MiB aggregate. The current snapshot JSON transport has a separate 1 MiB cap, not a promise of a requirement transport. Cap prefix at 255 bytes including `/`, needle at 8 MiB and replacement at 16 MiB. Reject impossible sizes/overflow before allocation or hashing.
2. **Decode the supported profile and admission.** Require the fixed options, valid prefix, valid UTF-8 and nonempty needle; refuse absent/unknown modes and missing evidence. Resolve the record through the trusted host. A valid hash of an untrusted record is not a credential.
3. **Bind subjects before searching.** Compare owner, source, `S`, `R`, mode and evidence with the independently expected record, in that order. Check the resolved selection/candidate base against the same owner, source and `S`, and the selection requirement against `R`. Resolve any local handles in their store incarnation and kind first. Never take the expected base from the report itself.
4. **Enumerate the admitted domain.** Retain one witness per in-scope resource in raw path order, even for zero hits. Enforce 4096 occurrences per resource, then 8192 total. Refuse over limit, never truncate or chunk into a success. A narrower selection, omitted zero-hit resource, duplicate or overlapping edit is not coverage. Only a fully witnessed empty result on this domain can justify `NoChange` for an unchanged candidate.
5. **Bound and compare the candidate.** Count output bytes including untouched resources: 16 MiB per resource and 32 MiB total. Check projected sizes with checked arithmetic before materializing; a limit failure cannot yield a partial edit batch. Exact byte/mode preservation and candidate/report binding remain distinct checker obligations. No filesystem effects.

## Independent vectors and decision log

Run `python3 qualification/requirement-vectors.py` from the repository root. The [frozen vectors](../../qualification/requirement-vectors.json) cover exact framing, identity swaps, missing authority, scope/witnesses, overlap, UTF-8 byte offsets and inclusive budget boundaries. They are independent development oracles; they do not test production admission or prove scanning. The fixture task has four in-scope files (one no-match), one outside the prefix and six hits; dropping the no-match witness defeats coverage.

Owner decision, recorded 25 November: retain the task identity across snapshots, require a separately checked snapshot admission, and freeze this fixture-v1 framing, budgets and design vectors for M1 ([#3](https://github.com/plyrune-dev/plyrune/issues/3), [#31](https://github.com/plyrune-dev/plyrune/issues/31)). A future production decoder must agree with the independent vectors; PR-C05/06 require separate production-path evidence and selected proofs. No new public command.
