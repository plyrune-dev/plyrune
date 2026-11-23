# M0 adoption and design findings

Build and check on linux/amd64 with `docker compose build tools` and
`docker compose run --rm gate`. The latter checks a Git archive of HEAD;
commit pending changes before using it. Rust 1.98.0, rustfmt and Clippy
are installed inside the image alongside the development Tauten revision
in `source-pins.json`. The parent digest and install commands are in
`docker/Dockerfile`. The 8 CPU / 8 GiB / 1024 process defaults are in
`compose.yaml`; CI limits the job to 40 minutes. No host Cargo is required.
The three workspace members have no external runtime dependencies, no
optional features, and no lint exceptions or legacy baseline.

## What would make the proposal give a wrong answer?

* **Binding two paths by blob alone:** Splicery compares a 32-byte original
  image identity. Two different paths can have identical bytes. Admit the
  full owner, incarnation, snapshot and resource before lowering to a cut.
  Test two equal-content paths with swapped handles, even when the resulting
  patch bytes would look correct.
* **Calling render with a plausible order:** `render` rechecks byte ranges
  but its order argument can be empty. Even perfect preservation of gaps
  doesn't establish coverage. Retain the exact `prepare` output together
  with its immutable edits; forbid construction of that pair by the caller.
* **Confusing no matches with no byte changes:** replacing a literal by
  itself can satisfy six obligations while producing an identical candidate.
  Report required occurrences, satisfied dispositions and changed bytes
  separately. An empty inventory also needs its own complete scope witness.
* **Bounding each file alone:** 4096 edits per resource is not a whole-plan
  memory or time budget. Charge total inventory bytes, matches and retained
  replacement bytes before allocating candidate outputs. Refuse before
  handing any partial result to the checker.
* **Measuring an advantage:** `tools/native_reference.py` exercises six
  batched fixture tasks against independently stored candidate bytes. This
  is a correctness reference. Only a paired consumer trial with the same
  inputs and grading can establish an interaction-cost improvement.

`docs/ASSURANCE.md` lists the proposed obligations. Range bounds are only
one premise of PR-C01. Neither the fast gate nor inherited leaf proofs
establish report binding, acquisition correctness or complete migration.
