---
rfc: 2026-11-24-retained-splice
title: "Reuse the exact pinned splice engine behind a qualified bridge"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/24
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Reuse the exact pinned splice engine behind a qualified bridge

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Keep the original splice dependency and its native gate. Run prepare/render against the golden five-resource manifest.

## Evidence, alternatives and limits

The shared shadow extractor fails on inherited workspace lints. The independent native gate passed 23 mutant controls, but that does not establish identity-preserving lowering (#7).

## Decision log

Landed in [`27978b1`](https://github.com/plyrune-dev/plyrune/commit/27978b1), [`d344af0`](https://github.com/plyrune-dev/plyrune/commit/d344af0). Tracking issue: [#24](https://github.com/plyrune-dev/plyrune/issues/24).
