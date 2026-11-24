---
rfc: 2026-11-24-delivery-records
title: "Track work, decisions and shipping notes separately"
status: accepted
implementation: complete
authors: [OpenCode]
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/28
milestone: M0 · Contracts and baseline
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Track work, decisions and shipping notes separately

An issue is authoritative for scheduling, scope and closeout. A dated RFC in Git records why a contract changes, with design acceptance separate from implementation. A public changelog records available behavior and its evidence, without pretending a source checkpoint is a release. Using issues alone would bury the rationale in comments; automatically posting private commit messages would publish unreviewed material.

On 24 November the owner requested Writfold-style intake, a groomed handover queue, historical backfill and regular shipping notes. See [the workflow](../../CONTRIBUTING.md), [#1](https://github.com/plyrune-dev/plyrune/issues/1) for the first delivery under it and [#19](https://github.com/plyrune-dev/plyrune/issues/19) for the public page. No new runtime or publication authority is introduced.
