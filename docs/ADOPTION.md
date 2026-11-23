# M0 adoption and design findings

Build and check on linux/amd64 with `docker compose build tools` and
`docker compose run --rm gate`. The latter checks a Git archive of HEAD;
commit pending changes before using it. Rust 1.98.0, rustfmt and Clippy
are installed inside the image alongside the development Tauten revision
in `source-pins.json`. The parent digest and install commands are in
`docker/Dockerfile`. The 8 CPU / 8 GiB / 1024 process defaults are in
`compose.yaml`; CI limits the job to 40 minutes. No host Cargo is required.
The pure kernel has no dependencies. Core hashes with `sha2`; the edge
decodes untrusted JSON with `serde` (derive) and `serde_json`. `Cargo.lock`
pins their closure. There are no lint exceptions or legacy baseline.

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

## Executed spikes

* `docker compose run --rm gate` passed at `c163c30`. The pinned CLI's
  `tauten init --write` generated `AGENTS.tauten.md`; the gate verifies
  its digest via `scripts/profile.sha256`. The production kernel declares
  no external dependencies or feature switches.
* `docker compose run --rm -T proofs` on `d95276a` proved
  `Proofs.Range.admit_spec`; both `reversed-accepted` and `past-end-accepted`
  broke that theorem directly. The committed record is `proofs/claims.json`.
  An earlier run completed translation but failed because Lake could not
  access image-owned files. Its outcome is retained in
  `qualification/first-proof-attempt.json`. The proof service now clones
  the read-only source into the container. The check concerns half-open
  bounds only; the UTF-8 controls are runtime tests.
* The [consumer probe](../qualification/splice-probe/) built with
  `--release --locked`, passed Tauten, and produced all five expected
  fixture resources using the pinned upstream `prepare`/`render`. This
  demonstrates dependency use in Rust, not proof composition.
* `docker compose run --rm dev sh qualification/splice-probe/extract.sh`
  **failed** before translation: Splicery inherits workspace lints, but
  Tauten's shadow workspace copies only inherited package keys. A valid
  Cargo dependency is therefore not yet a qualified extraction route.
  Keep the splice kernel independent, retain its native gate, and qualify
  the narrow identity-to-cut mapping separately. Do not rewrite splice.
* Splicery's **independent native proof gate passed** at `632a90f` in the
  pinned Tauten image. It regenerated the translation, compared it against
  the upstream committed copy, built Lean, audited axioms, and rejected all
  23 calibrated mutants (750 seconds reported by the gate). To repeat the
  same container command and inputs, run
  `sh qualification/splice-probe/native-proof.sh`. This qualifies the
  upstream leaf at that revision; it does not prove the Plyrune bridge.
