---
rfc: 2026-11-24-requirement-framing
title: "Bind literal migration requirements independently of matches"
status: draft
implementation: not-started
authors: [OpenCode]
created: 2026-11-24
updated: 2026-11-24
issue: https://github.com/plyrune-dev/plyrune/issues/3
milestone: Next · Proved subject binding
supersedes: []
superseded_by: []
blocked_on: [owner-review]
---
# RFC: Bind literal migration requirements independently of matches

The caller admits a requirement before the matcher sees candidate edits. A digest over returned matches alone would let `config/server/` silently replace the broader `config/` obligation.

**Proposal:** use the same length framing as the fixture profile but a distinct `plyrune/requirement/literal/v1` domain. Encode the exact directory prefix, nonempty needle and fixed replacement in that order. The tag commits to regular UTF-8 resources, byte-exact non-overlapping matching, no normalization, no exclusions and no extra edits. Reject unsupported options rather than discarding them. Owner admission and source scope are separate verified bindings; the wire's human label does not create authority.

**Open decision:** bind the snapshot digest inside the requirement digest, or bind the reusable requirement profile to a specific snapshot in a separate admission record? The latter supports refresh without changing the task, but the checker must then validate *both* records.

**Controls before implementation:** changing `config/` to `config/server/`, changing replacement bytes, or swapping the admitted snapshot must defeat report reuse. Zero occurrences on a complete inventory must still have a witness. Freeze independent positive and negative vectors after owner review. No format or proof is admitted by this draft.
