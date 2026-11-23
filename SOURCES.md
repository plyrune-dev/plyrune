# Sources and inspection record

Prepared 23 November 2026. Exact revisions are recorded in `source-pins.json`. Repository content was read through the connected GitHub service. This is source inspection, not a rerun of the repositories' build/proof gates.

## Engineering and Tauten

- [Playbook](https://github.com/tknawara/playbook/tree/cca8e068d0843af5612522049f08d8d645252f7c), private: README, catalog, AGENTS, architecture, types/boundaries, verification, quality gates and review/evolution. Adapted as a project-specific profile; the playbook is not redistributed wholesale.
- [Tauten](https://github.com/tauten-dev/tauten/tree/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42): README, AGENTS, Cargo manifests, ADOPTING, STATUS, MAPPING, proof README, agent profile, ROADMAP, accepted five-primitives RFC, origin.rs, lib.rs, macro source and behavior tests.
- [Stamp carrier](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten/src/origin.rs#L194-L258) and [generated minting](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten-macros/src/lib.rs#L610-L623): source-level wrong-subject finding, independently reviewed.
- [Existing stamp fixture](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten/tests/behaviour.rs#L140-L202): basis for the proposed regression test.
- Tauten open/related issue bodies were checked before drafting. Relevant links are in `docs/TAUTEN.md`; #9 is the originating completed carrier feature and #30 is the parked/rejected register.

## Existing engines

- [Splicery README](https://github.com/writfold-dev/splicery/blob/632a90fd96e920fbac7ee5818f989739c1617062/README.md): production splice proof boundary, exact limits, candidate-only CLI and tested mappings. Historical Writfold owner descriptions are not treated as current Plyrune authority.
- [Weft README](https://github.com/writfold-dev/weft/blob/118c1bf3de82002e3070267a54f28eedaa1ad678/README.md): finite-set/trace/reuse kernels, local development status and unproved whole-program/integration boundaries.
- [Tuft archive](https://github.com/writfold-dev/writfold-archive/tree/c79f2dfca763d9b0eba6031d3f0437bda75b08ca/public-split-2026-11-16), private: historical architecture and measurement context. Not a current independently shipped Tuft 2.0 product.
- [Projection pilot 2](https://github.com/writfold-dev/writfold-archive/blob/c79f2dfca763d9b0eba6031d3f0437bda75b08ca/public-split-2026-11-16/bench/2026-11-10-projection-pilot-2/README.md): exact historical figures and limitations cited in the evaluation plan. No transcripts or raw private source are included in public website assets.

## Naming

The preceding naming exploration checked exact repository/package names and authoritative domain records on 23 November 2026. The `.dev` check used `pubapi.registry.google/rdap/domain/plyrune.dev`; `.com` used `rdap.verisign.com/com/v1/domain/plyrune.com`. No domain record was returned in those checks. Exact npm/crates.io/PyPI package checks returned no package; repository-name searches found no exact match. These are transient availability observations, not reservations or trademark clearance.

## Evidence boundaries

- Design review and source inspection were performed.
- The local handover/fixture checks and PDF visual review are recorded in `VALIDATION.md` after execution.
- The images are generated design concepts, not live product screenshots.
- No Rust/Lean/product benchmark execution or website deployment was performed.
- Tauten issue creation was attempted and blocked; see `issues/STATUS.md`.
