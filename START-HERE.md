# Plyrune 2.0 implementation handover

23 November 2026 · Working name: **Plyrune** (PLY-roon) · Design reviewed; implementation not started by this handover.

Build one harness-agnostic interface for finding source, retaining exact captures, inspecting bounded views, preparing candidates and checking declared obligations. Tuft, Weft and Splicery become internal engines behind shared artifact identities and contracts.

The first useful release is **candidate-only**. A successful check refers to an exact candidate and independently admitted requirement. It does not authorize filesystem writes or prove arbitrary program correctness.

## Read in this order

1. [HANDOVER.md](HANDOVER.md): decisions, architecture, use cases and tradeoffs.
2. [AGENTS.md](AGENTS.md): project coding and assurance rules, adapted from the pinned engineering playbook.
3. [MILESTONES.md](MILESTONES.md): ordered work, acceptance evidence and stop conditions.
4. [docs/CONTRACTS.md](docs/CONTRACTS.md): shared types, completeness, identity and wire boundaries.
5. [docs/ASSURANCE.md](docs/ASSURANCE.md): Lean composition claims and proof-to-runtime obligations.
6. [docs/TAUTEN.md](docs/TAUTEN.md): adoption from the first build, existing capabilities and gaps.
7. [docs/EVALUATION.md](docs/EVALUATION.md): fair baselines, ablations and website data.
8. [docs/REVIEW.md](docs/REVIEW.md): reviewer findings and their dispositions.
9. [website/DESIGN.md](website/DESIGN.md) and [website/index.html](website/index.html): visual direction and offline responsive prototype.

## First implementation assignment

Complete M0, then one M1 vertical slice: an immutable fixture manifest, an independently supplied literal-replacement requirement, a complete selection, the existing exact splice operation, and a report bound to the candidate. Include a wrong-owner refusal, a narrowed-scope refusal, a negative search and a view that shows fewer members than the selection retains.

Start with three Rust crates: `plyrune-kernel`, `plyrune-core`, `plyrune-cli`. Prefer modules inside the kernel to additional crates until dependency extraction and reuse are demonstrated. Keep one CLI/stdio process initially. The TypeScript SDK follows the working protocol; it is not an additional authority.

Do not start with a distributed runtime, a general-purpose language, automatic rebasing, language-specific semantic indexing or live workspace application. Ordinary harness scripts can compose calls.

## What this package does and does not establish

- The reviewer challenged the design and independently checked the two Tauten findings. Changes are incorporated.
- Playbook and Tauten source were inspected at exact revisions listed in [SOURCES.md](SOURCES.md).
- The website is a local design prototype. The images are generated concepts, not production vector assets.
- No Plyrune Rust implementation, benchmark run, Lean proof run, hosted site, domain registration or package publication was performed.
- [issues/STATUS.md](issues/STATUS.md) records the actual issue-submission outcome. Draft files are ready to submit; a draft is not a filed issue.

The public site must keep performance **unmeasured** and the new composition claims **planned** until exact evidence exists. Inherited engine proofs retain their original scope.
