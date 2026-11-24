---
rfc: 2026-11-24-portable-check-reports
title: "Portable candidate evidence for standalone CI and optional Writfold gates"
status: draft
implementation: in-progress
issue: https://github.com/plyrune-dev/plyrune/issues/39
---
# Portable candidate evidence

**Decision in scope:** one producer format, two independent consumers. A reviewer can ask whether a native edit satisfies a previously admitted migration. Plain CI and Writfold must be able to read the *same bytes*. The [Writfold companion](https://github.com/writfold-dev/writfold/issues/106) owns gate configuration and policy. No shared runtime, automatic install, receipt import or workflow engine.

**Proposed contract, not a shipped schema:** bind an explicit proposition to stable source/resource identities, the complete base and candidate manifests, the requirement and its admission provenance (#31), matcher/checker profiles and inventory including zero-match files. Carry completeness, limitations and a typed outcome. Session IDs, duration, timestamps, local paths, and Writfold run IDs belong only to invocation records. Retain immutable input bytes or references with an explicit retention policy; an unresolved digest cannot be replayed. Never execute artifact-supplied commands.

**Verification:** the consumer supplies the intended subject independently. Recompute the canonical report from retained inputs using the pinned checker; require equality of proposition, inputs, profile and result. Only complete satisfaction of that proposition can pass the configured command gate. A complete no-change may satisfy a stated requirement, not an obligation to modify a particular file. Partial, malformed, unsupported, unavailable and timeout are non-success. A source digest does not grant authority. If the current tree differs, the historical report remains valid for its old tree; rerun for the new one.

**Controls before freeze:** same semantic bytes across fresh processes; different equal-content path, extra in-scope file, changed requirement/profile, stale candidate, absent dependency, refused or partial result all fail the relevant check. Keep PR-C05/06 subject binding distinct from tested serialization, host IO and the configured checker’s provenance. Negotiate schema IDs/framing with #106 and #3 before freezing vectors. A Writfold receipt records *its own* invocation; neither receipt nor report establishes application.

**Implementation order:** [snapshot gate example](../../examples/writfold/) exercises a real producer, checked-in expected identity and separate artifact directory first. It is a transport control only. Next pair the #30 coverage decision with the #31 admission record; qualify the standalone report before feeding it through this same gate mechanism. Combined two-binary qualification remains open. Owner authorized the integration direction on 24 November; exact report format is pending review.
