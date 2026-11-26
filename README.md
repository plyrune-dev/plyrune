# Plyrune

Plyrune is a local CLI for finding and replacing **literal text** across a directory. Find every match, preview the exact replacement and then choose where to write it. It doesn't require Git, stage files, make commits or push. It does not resolve programming-language symbols.

[Changelog](https://plyrune-dev.github.io/changelog.html) · [Milestones](https://github.com/plyrune-dev/plyrune/milestones) · [Issues and RFC process](CONTRIBUTING.md)

See the [adoption record](docs/ADOPTION.md) for executed checks, the [proof record](proofs/claims.json) for the precise kernel claims, and the [Writfold gate example](examples/writfold/) for optional integration. Plyrune is [MIT licensed](LICENSE); upstream components retain their own notices.

## Reproduce the current build

**To play with the preview** on Linux x86-64 or WSL2, no Rust or Docker is required:

```sh
curl -fsSL https://plyrune-dev.github.io/install.sh | sh
"$HOME/.local/bin/plyrune" demo
```

The [installer](website/install.sh) verifies the pinned release asset's SHA-256, replaces only the `plyrune` executable in `~/.local/bin` (or `PLYRUNE_INSTALL_DIR`), and does not edit your shell profile. Remove that binary to uninstall.

### Edit files without a repository

With `config/` under your chosen root:

```sh
p="$HOME/.local/bin/plyrune"
"$p" refs --root . config/ old_host
"$p" preview --root . config/ old_host new_host > review.json
# Inspect the paths, ranges and per-file hashes in review.json first.
id=$(jq -er .review review.json)
"$p" apply --root . config/ old_host new_host --review "$id" --into ../edited
```

`../edited` must be separate and either nonexistent or empty. It receives the **scoped tree**, not a clone of everything under `.`. Use `--in-place` instead of `--into ../edited` to replace the changed originals. Nothing writes unless you choose. A changed source or task needs a new review. Use a supported local Linux filesystem under your control, with one authorized writer. Multiple renames are **not one transaction**: save the original preview outside the scope and [reconcile interrupted work](docs/rfcs/2026-11-26-filesystem-edits.md#recovery-after-interruption) before retrying.

### Try a read-only stale-input demonstration

`demo` inspects `config/server/api.env[8..23)`, then changes an unrelated file and repeats the same request. The text is still there, but its old snapshot no longer matches. To drive the real commands yourself:

```sh
P="$HOME/.local/bin/plyrune"
"$P" example > fixture.json
"$P" snapshot < fixture.json
"$P" inspect 6ba5a5e50f91714f5a6d96a5ad3a07e525f375f95b6a2b86b2594673319b9d5c \
  config/server/api.env 8 23 < fixture.json
```

The view contains `legacy_endpoint`. Edit any resource's text in `fixture.json` and repeat the last command: it exits 2 with `WrongSnapshot`. Offsets are half-open UTF-8 **byte** offsets, not character indices.

### Retain an exact capture (Rust library, not preview 1)

For an in-process harness, bind an admitted `Fixture` to a host-issued `StoreBinding`, then use `BoundFixture::open(path)` to select the intended resource. `resource.capture(ByteOffset::new(start), ByteOffset::new(end))` retains its identity and byte coordinates. Resolve with `resource.resolve(&capture)`; supplying a different resource with identical text is an error, not a successful read. No bytes are copied into the capture, and no handle crosses a process boundary. The [external library consumer](crates/plyrune-core/tests/fixture_captures.rs) shows the complete calls. See the [capture RFC](docs/rfcs/2026-11-25-fixture-captures.md) for host identity assumptions and refusal order.

### Find references in a fixture instead of a directory

`plyrune refs config/ legacy_endpoint < fixture.json` finds **literal text**, not a programming-language symbol. In the bundled example it returns six locations in three files plus a fourth row, `config/unchanged.env`, with no matches. `docs/notes.txt` is outside the prefix. On a refused query there is no partial selection to mistake for a successful search. The [literal references RFC](docs/rfcs/2026-11-26-literal-references.md) describes exactly what gets counted.

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
