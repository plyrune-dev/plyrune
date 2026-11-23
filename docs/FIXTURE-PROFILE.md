# M1 fixture profile, revision 1 (implementation target)

The three JSON files under `examples/` are fixed development inputs. The
decoder must not trust the `status`, `id`, or `admittedBeforeCandidate`
strings as receipts. The host supplies the manifest and admits the
requirement in separate operations.

* **Inventory:** each entry is a regular file with exact UTF-8 bytes. Path
  segments are ASCII letters, digits, `.`, `_` or `-`; slash separates
  segments. Reject empty, `.` and `..` segments, absolute paths, duplicates,
  unsupported modes and malformed text. Do not traverse a filesystem.
  Preserve line endings, trailing newlines and case. This deliberately
  narrow fixture adapter places no restrictions on future Git-tree paths.
* **Scope:** require a directory prefix ending in `/`; compare whole byte
  prefixes. `config2/` is outside `config/`. Sort resources by raw path
  bytes; within a resource enumerate matches left-to-right with the next
  start at or after the previous end. Empty search text refuses. A request
  for a subset must not satisfy the original requirement.
* **Budget:** up to 256 manifest entries, 255 bytes per path, 8 MiB per
  input and 32 MiB aggregate input. Up to 4096 occurrences per resource
  and 8192 across the manifest; 16 MiB per candidate and 32 MiB total.
  Check sums before allocating; exceeding any search budget prevents a
  complete result. Do not split the edit batch to bypass a refusal.
* **First canonical digest:** SHA-256 of framed binary input. `frame(x)`
  is an unsigned 64-bit big-endian byte length followed by exactly those
  bytes. A snapshot preimage begins with `frame("plyrune/fixture/v1")`
  (ASCII), then the unsigned 64-bit big-endian resource count, then
  `frame(path) || frame("regular") || frame(bytes)` per resource in sorted
  order. JSON whitespace, field order and fixture labels do not enter the
  preimage. Do not reuse this domain tag for requirements or candidates.
  A digest assumes collision resistance and does not encode ownership.
* **Fixture oracle:** six required replacements across three touched files;
  four resources in scope including the no-match `config/unchanged.env`.
  All five output paths, modes and bytes must equal the independently
  supplied expected manifest. The out-of-scope `docs/notes.txt` still
  contains the old literal. A two-member view must retain all six captures.

The [probe](../qualification/splice-probe/) exercises the last byte comparison.
`plyrune snapshot` implements input path/mode/encoding admission, inventory
and byte limits, and digest framing; the transport adds a 1 MiB JSON limit.
Canonicalization is tested against independent vectors, not proved. Match
and candidate limits, requirement/report admission, and a stable wire
schema remain obligations for the product implementation.
