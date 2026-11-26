---
rfc: 2026-11-26-literal-references
status: accepted
implementation: complete
issue: https://github.com/plyrune-dev/plyrune/issues/5
milestone: Literal references
---
# Find exact literal references

Use the existing fixture/v1 input and frozen literal/v1 semantics. `plyrune refs PREFIX NEEDLE` reads a fixture manifest on stdin. Its JSON output identifies the snapshot, prefix and needle, and lists in-scope resource paths (including zero matches) with half-open UTF-8 byte ranges in canonical order. Complete-empty is not an error. Empty needles, invalid prefixes, unsupported inputs and budget excesses refuse without outputting a partial selection. This is exact byte matching, not symbol resolution, normalized Unicode or a knowledge-base query. No filesystem reads other than the explicitly supplied input; no writes. Real file acquisition for editing belongs to the publication adapter, not to the kernel.

Authority is the immutable admitted inventory, not a caller-supplied hit list. Compose a qualified literal scanner with an independent inventory witness check. Bind every selected range back to an actual fixture resource; retain both positive and negative dependencies via the snapshot. Limit matches to 4096 per resource and 8192 total; bound matching work explicitly. Do not infer complete search from a successful producer loop alone. Translate and audit production decisions; independently calibrate a skipped hit and a missing zero-hit witness. Core mapping, prefix filtering and transport stay separately tested. No shell until selected proofs pass.

Milestone exit requires a real CLI invocation, proof/controls, and a fresh reading of the Writfold site against this scope. At planning, Writfold v0.6.0-dev.20 searches knowledge-base records and tracks citations; Plyrune searches supplied source bytes. We will not import KB indexing, run plans or receipt policy. Source: https://writfold.dev/status.html .

**Exit:** `docker compose run --rm -T proofs` proved ten claims on `665db68` (17 killed source mutants). The initial fast gate rejected unassociated arithmetic/index findings and a discarded insertion index; the audited, unconditional same-function records now discharge those rules. `docker compose run --rm -T gate` passed on `4da8665`. A statically linked package passed both CLI consumers; the bundled example returned six matches, four scoped rows. Filesystem acquisition is not part of this milestone. [Writfold checkpoint](../WRITFOLD-CHECKPOINTS.md#literal-references--milestone-9).
