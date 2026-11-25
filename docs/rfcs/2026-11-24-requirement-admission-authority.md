---
rfc: 2026-11-24-requirement-admission-authority
title: "Say who admits a requirement and what the report can claim"
status: accepted
implementation: not-started
authors: [outside-review]
created: 2026-11-24
updated: 2026-11-25
issue: https://github.com/plyrune-dev/plyrune/issues/31
milestone: Next · Proved subject binding
supersedes: []
superseded_by: []
blocked_on: []
---
# RFC: Who admitted this requirement?

A checker can find every occurrence under `config/server/` without noticing that the user asked for all of `config/`. The check is about the admitted task, not the request in the user's head. The [paired framing RFC](2026-11-24-requirement-framing.md) identifies the bytes; the admission record identifies who fixed them and when.

## Required admission and report modes

Before candidate-specific selection, the host records the exact requirement, base snapshot, source and owner, a **required mode**, and an immutable evidence reference. The verifier obtains the expected record from the authorized host workflow, not from the candidate, its author, or the report. Echo the mode and admission identity for every report with an admitted subject, including no-change and negative results. An absent or unknown mode, unresolvable evidence, changed requirement, or attempted mode upgrade refuses verification; it is not a fallback to `host-authored`.

| Recorded mode (closed set) | Evidence supplied by host | Strongest wording for a complete satisfying candidate |
|---|---|---|
| `host-authored` | Requirement fixed outside the editing agent's turn, e.g. a protected CI policy revision. | “Candidate satisfies the host-authored requirement on this snapshot.” |
| `model-proposed, human-confirmed` | Host recorded an authorized person's confirmation of this exact rendered requirement and base before selection. | “Candidate satisfies the requirement the reviewer confirmed on this snapshot.” |
| `model-authored` | Host recorded the agent's own statement before selection; nobody independently approved its scope. | “Candidate is consistent with the requirement the agent stated.” **Not** unqualified “coverage complete” or “the requested migration is complete.” |

The same narrow requirement and same candidate give the same mechanical answer in all three modes; changing the mode changes the premise and therefore the report subject and wording. Neither a model-supplied mode string nor evidence from a previous base elevates a report. A mode cannot turn incomplete coverage or an incorrect candidate into success. A policy requiring independent approval cannot accept `model-authored` as a pass for that policy. Reports do not authorize applying edits.

## Review surface and named workflow

Show one sentence during admission and with the result. For the example manifest: **Replace every `legacy_endpoint` with `service_endpoint` under `config/` (4 regular UTF-8 files in scope; 1 manifest file outside scope).** The narrowed `config/server/` version would read **1 in scope; 4 outside**. Counts come from the complete admitted base inventory, including no-match resources, *not* the number of touched files, hits, repository files on disk, or a model estimate. Escape control characters in a one-line summary and make the full exact values inspectable; don't accept confirmation of a truncated or ambiguous rendering. With unknown inventory, don't invent counts or confirm this summary.

**Example design, not a reported deployment: “Endpoint migration PR check.”** A maintainer commits the reviewed `config/` task to protected base-branch CI configuration before assigning the edit. The check opens the expected task from that protected revision and the base manifest from the authorized source, records `host-authored` with a reference to that policy revision, then evaluates the candidate supplied by the editing agent. Never read the authoritative policy from the candidate branch. If a rebase changes the snapshot, resolve the policy again and create a new snapshot-specific admission. This needs no per-edit approval click; the assumption is that the agent cannot rewrite the protected policy or impersonate the maintainer. If the agent supplied the policy without independent review, use `model-authored` instead.

## Evidence, alternatives and decision log

Leaving provenance implicit would make the scope trap disappear into a formally correct but self-chosen requirement. Requiring a human click every time would preclude existing host workflows and hide the model-only case. The proposed modes preserve both uses while distinguishing the conclusions. PR-C05/06 would bind the formal task and report premises, not prove a person's intent, the authenticity of CI configuration or the host's chronology.

Owner decision, recorded 25 November: require the three host-recorded modes and qualified wording for model-authored tasks ([#31](https://github.com/plyrune-dev/plyrune/issues/31), paired with [#3](https://github.com/plyrune-dev/plyrune/issues/3)). This is a contract decision, not a production admission or report implementation. The [pilot plan](../EVALUATION.md#requirement-admission-in-the-six-task-pilot) separates these modes, including a model-authored scope trap. Validate with an actual workflow that can fix scope independently and measure its cost against reviewing the diff before making an adoption claim.
