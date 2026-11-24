---
rfc: 2026-11-24-range-and-store
title: "Check byte ranges and store-local provenance in the kernel"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/23
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Check byte ranges and store-local provenance in the kernel

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Require ordered in-bounds offsets, UTF-8 character boundaries for admitted text, and equality of both owner and store incarnation.

## Evidence, alternatives and limits

A revision-only stamp cannot distinguish unrelated stores. Four production-translation claims passed with six mutants. This is not proof of snapshot acquisition, host ID uniqueness or complete handle resolution.

## Decision log

24 November follow-up ([#41](https://github.com/plyrune-dev/plyrune/issues/41)): compose the two guards as a pure function. Store mismatch takes precedence over range/UTF-8 refusal, including when both facts are invalid. Otherwise preserve the exact text-range decision. This is a prerequisite of capture resolution, not a grant of authority or evidence of snapshot/resource identity. Require an audited total theorem over the extracted function and a direct bypass-store mutant before exposing it.

Landed in [`df8d5b7`](https://github.com/plyrune-dev/plyrune/commit/df8d5b7), [`c163c30`](https://github.com/plyrune-dev/plyrune/commit/c163c30), [`5cf92c1`](https://github.com/plyrune-dev/plyrune/commit/5cf92c1), [`5c7fc80`](https://github.com/plyrune-dev/plyrune/commit/5c7fc80), [`0d88d9d`](https://github.com/plyrune-dev/plyrune/commit/0d88d9d). Tracking issue: [#23](https://github.com/plyrune-dev/plyrune/issues/23).
