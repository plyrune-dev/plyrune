---
rfc: 2026-11-24-pages-site
title: "Deploy the allowlisted design preview to the public organization site"
status: accepted
implementation: complete
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/27
milestone: M0 · Contracts and baseline
authors: [OpenCode]
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Deploy the allowlisted design preview to the public organization site

**Retrospective.** Recorded on 24 November after the scoped change landed; this is not a claim of prior RFC review.

## Decision

Serve static files from the organization github.io origin while keeping the implementation private. Preserve planned/unmeasured states.

## Evidence, alternatives and limits

A separate public tree avoids exposing private design materials. The merge includes all seven website-worktree commits (98f1da9 through 8570a88). Published site revisions: bba2cf3, 8cfba6e, 171abf3, bc14aee. Browser and accessibility evidence and its limits are in website/DESIGN.md.

## Decision log

Landed in [`ef68525`](https://github.com/plyrune-dev/plyrune/commit/ef68525). Tracking issue: [#27](https://github.com/plyrune-dev/plyrune/issues/27).
