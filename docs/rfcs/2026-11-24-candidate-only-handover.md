---
rfc: 2026-11-24-candidate-only-handover
title: "Import the reviewed candidate-only handover"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/21
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Import the reviewed candidate-only handover

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Keep source, requirement and candidate distinct. No working-tree write operation belongs in M0–M4.

## Evidence, alternatives and limits

Preserve the original manifest rather than retrospectively updating the historical evidence. The handover describes future milestones, not a released implementation.

## Decision log

Landed in [`c375f59`](https://github.com/plyrune-dev/plyrune/commit/c375f59). Tracking issue: [#21](https://github.com/plyrune-dev/plyrune/issues/21).
