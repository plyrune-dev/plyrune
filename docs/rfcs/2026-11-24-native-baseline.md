---
rfc: 2026-11-24-native-baseline
title: "Introduce six native task controls before benchmarking"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/26
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Introduce six native task controls before benchmarking

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Use a batched native reference and independent expected candidate for six task families, including negative dependency recovery.

## Evidence, alternatives and limits

These are development controls. Passing booleans are not retained task traces (#1), nor evidence of a performance improvement (#12).

## Decision log

Landed in [`df8d5b7`](https://github.com/plyrune-dev/plyrune/commit/df8d5b7), [`27978b1`](https://github.com/plyrune-dev/plyrune/commit/27978b1). Tracking issue: [#26](https://github.com/plyrune-dev/plyrune/issues/26).
