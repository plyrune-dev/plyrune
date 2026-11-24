# Plyrune

[Changelog](https://plyrune-dev.github.io/changelog.html) · [Milestones](https://github.com/plyrune-dev/plyrune/milestones) · [Issues and RFC process](CONTRIBUTING.md)

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
entire manifest to the checked-in expected result.

### Try the first Plyrune command

```sh
docker compose run --rm -T dev cargo +1.98.0 run --quiet --locked -p plyrune-cli -- snapshot < examples/input.json
```

`snapshot` validates the fixture on stdin and returns its resource count
and canonical content digest. The example has five resources and digest
`6ba5a5e50f91714f5a6d96a5ad3a07e525f375f95b6a2b86b2594673319b9d5c`.
It rejects malformed or oversized inputs with exit status 2. This first
command does not retain handles or serve edit requests.
