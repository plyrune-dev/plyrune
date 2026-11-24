---
rfc: 2026-11-24-fixture-snapshot
title: "Admit a finite fixture manifest and expose its canonical snapshot digest"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/25
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Admit a finite fixture manifest and expose its canonical snapshot digest

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Use sorted, length-framed SHA-256 over validated regular UTF-8 resources; snapshot reads bounded stdin and returns resource count and digest.

## Evidence, alternatives and limits

Hashing JSON serialization would bind field order and whitespace. The installed consumer matched the independent fixture vector; canonicalization is tested, not proved. No retained session handle is issued.

## Decision log

Landed in [`dc05b7b`](https://github.com/plyrune-dev/plyrune/commit/dc05b7b), [`d148a15`](https://github.com/plyrune-dev/plyrune/commit/d148a15). Tracking issue: [#25](https://github.com/plyrune-dev/plyrune/issues/25).
