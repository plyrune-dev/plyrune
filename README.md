# Plyrune

Plyrune is an open-source toolkit for checking an agent’s candidate edits against a requirement fixed before the edit. For example, a migration scoped to all of `config/` should not pass just because every occurrence under `config/server/` was replaced. The first preview identifies a fixture and inspects text from an exact snapshot; the migration checker is still in development.

[Changelog](https://plyrune-dev.github.io/changelog.html) · [Milestones](https://github.com/plyrune-dev/plyrune/milestones) · [Issues and RFC process](CONTRIBUTING.md)

See the [adoption record](docs/ADOPTION.md) for executed checks, the [proof record](proofs/claims.json) for the precise kernel claims, and the [Writfold gate example](examples/writfold/) for optional integration. Plyrune is [MIT licensed](LICENSE); upstream components retain their own notices.

## Reproduce the current build

**To play with the preview** on Linux x86-64 or WSL2, no Rust or Docker is required:

```sh
curl -fsSL https://plyrune-dev.github.io/install.sh | sh
"$HOME/.local/bin/plyrune" demo
```

The [installer](website/install.sh) verifies the pinned release asset's SHA-256, replaces only the `plyrune` executable in `~/.local/bin` (or `PLYRUNE_INSTALL_DIR`), and does not edit your shell profile. Remove that binary to uninstall.

`demo` inspects `config/server/api.env[8..23)`, then changes an unrelated file and repeats the same request. The text is still there, but its old snapshot no longer matches. To drive the real commands yourself:

```sh
P="$HOME/.local/bin/plyrune"
"$P" example > fixture.json
"$P" snapshot < fixture.json
"$P" inspect 6ba5a5e50f91714f5a6d96a5ad3a07e525f375f95b6a2b86b2594673319b9d5c \
  config/server/api.env 8 23 < fixture.json
```

The view contains `legacy_endpoint`. Edit any resource's text in `fixture.json` and repeat the last command: it exits 2 with `WrongSnapshot`. Offsets are half-open UTF-8 **byte** offsets, not character indices.

### Build from source

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
