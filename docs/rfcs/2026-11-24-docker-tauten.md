---
rfc: 2026-11-24-docker-tauten
title: "Build and qualify every crate in Docker with pinned Tauten"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/22
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Build and qualify every crate in Docker with pinned Tauten

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Use a combined Tauten/Rust/Lean image and a read-only checkout. Run Lake in a disposable clone so it can write dependency metadata without touching host sources.

## Evidence, alternatives and limits

Separate tools images would save space on lint-only runs but complicate proof snapshot paths. Hosted CI exhausted disk and rejected the eight-CPU request; the two-core workflow passed in run 45310624502.

## Decision log

Landed in [`df8d5b7`](https://github.com/plyrune-dev/plyrune/commit/df8d5b7), [`bdede3b`](https://github.com/plyrune-dev/plyrune/commit/bdede3b), [`d95276a`](https://github.com/plyrune-dev/plyrune/commit/d95276a), [`1ea793a`](https://github.com/plyrune-dev/plyrune/commit/1ea793a), [`79edf8d`](https://github.com/plyrune-dev/plyrune/commit/79edf8d). Tracking issue: [#22](https://github.com/plyrune-dev/plyrune/issues/22).
