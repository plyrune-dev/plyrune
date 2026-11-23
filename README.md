# Plyrune

Plyrune is developing a candidate-only editing interface for agents. Start
with the [handover](START-HERE.md) for the design and [adoption record](docs/ADOPTION.md)
for implementation evidence and open gaps.

## Reproduce the current build

Use Docker Compose on linux/amd64. The tools image includes Tauten's pinned
proof toolchain (about 20 GB before the stable Rust tools are added).

```sh
docker compose build tools
docker compose run --rm gate
docker compose run --rm -T proofs
```

Both gates check committed HEAD. The fast gate runs format, build, Tauten,
fixture controls and debug/release tests. The separate proof gate emits a
JSON record on stdout only after successful extraction, drift comparison,
Lean build, axiom audit and source-mutant calibration.

To try the exact upstream splice engine against the migration fixture:

```sh
docker compose run --rm dev sh -ec 'cd qualification/splice-probe &&
  cargo +1.98.0 build --release --locked && python3 check.py'
```

This developer probe prepares candidate bytes in memory and compares the
entire manifest to the checked-in expected result. The `plyrune` executable
does not yet serve edit requests.
