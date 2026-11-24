# Working on Plyrune

Start with a [GitHub issue](https://github.com/plyrune-dev/plyrune/issues). Report a reproducible defect or describe a task, its scope, dependencies and the observable result that would satisfy it. Trivial documentation corrections may go directly to a commit.

**Grooming:** new issues get `needs-triage`. Assign an area, size and milestone-relative priority. Split large items before implementation; list prerequisite issue numbers. `accepted` means the scope is authorized by the owner or reviewed handover, not that a new design has been approved. Park speculative work outside the active queue and identify its trigger. The [milestones](https://github.com/plyrune-dev/plyrune/milestones) are the live schedule.

**Design:** before changing a public verb/schema, semantic decision, authority boundary, proof premise or costly dependency, label the issue `needs-rfc`. Copy [the template](docs/rfcs/0000-template.md) to `docs/rfcs/YYYY-MM-DD-short-title.md`, link it both ways, and add it to [the registry](docs/rfcs/README.md). Keep the filename on revision. Status is `draft`, `accepted`, `rejected` or `superseded`; implementation is separately `not-started`, `in-progress`, `partial`, `complete`, `removed` or `n/a`. The owner decides new designs; record who decided, when, why and what remains open. No RFC overrides AGENTS.md.

**Delivery:** link the issue in the change. Run the applicable Docker gates in [README.md](README.md), attach the commit, evidence and remaining limits, then close the scoped issue. For a user-visible or assurance change, add a dated note to [the changelog](website/changelog.html) in the same change. Say whether it is available from source, live on Pages, or a tagged release. Publish only the staged allowlist and compare the live page with those bytes. Editorial changes need no product release.

## Queue after the initial build

Finish M0: [native traces #1](https://github.com/plyrune-dev/plyrune/issues/1), [source and adoption closure #2](https://github.com/plyrune-dev/plyrune/issues/2), [requirement framing and adversarial vectors #3](https://github.com/plyrune-dev/plyrune/issues/3). The source inventory and requirement RFC can proceed independently after the baseline.

Then M1: capture binding **#4** → full literal inventory **#5** → bounded view **#6** and retained splice bridge **#7** → independent report **#8**. Pinned Git acquisition **#9** follows the fixture adapter. Stdio sessions and packaging wait for M2; fine-grained reuse and publication are outside this queue.
