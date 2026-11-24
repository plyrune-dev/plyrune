# Optional Writfold command gate: snapshot identity

This example works with the existing `plyrune snapshot`. It answers “did the configured binary report the expected fixture identity?” It does not check a candidate or implement the draft portable report. A host that accepts changes to both the fixture and its oracle has not independently fixed the subject.

Run in a clean source checkout with Docker Compose:

```sh
docker compose run --rm -T dev sh -ec '
  tools=$(mktemp -d)
  cargo +1.98.0 install --locked --path crates/plyrune-cli --root "$tools"
  out=$(mktemp -d)
  PATH="$tools/bin:$PATH" sh examples/writfold/check-snapshot.sh "$out"
  cat "$out/snapshot.json"'
```

To opt in from a Writfold run for this repository, install a trusted, revision-pinned `plyrune` on the gate runner's PATH, provide Python 3, and copy `plyrune-snapshot.toml` into `.writfold/gates/`. Run `writfold kb gate run plyrune-snapshot` in that run. The command stores the original stdout in Writfold's fresh `{artifacts}` directory; nonzero exit is not a pass. Scope pins the fixture, oracle and adapter sources, **not** external tools on PATH: pin those in the runner image and bump gate version when they change. Remove the gate definition to opt out. Normal Plyrune builds do not install or invoke Writfold.

Definition follows the [pinned Writfold guide](https://github.com/writfold-dev/writfold/blob/4708ebfbdcaa087e6bc02540bb83ac46fd9b612a/docs/guide/gates.md). On 24 November a Compose smoke run used the Writfold `v0.6.0-dev.18` Linux binary (SHA-256 `ec14296ff1cb615aa4df734df9e3cb6f114d2177cac6261a4fc462c020c6b846`) and the release-mode Plyrune CLI. In a temporary Git repository, `kb init`, `kb checkpoint --new-run` and `kb gate run` produced a satisfied receipt and local `artifacts/snapshot.json`; replacing `examples/input.json` with malformed bytes produced a failed receipt (exit 2). Writfold's artifact directory is local and untracked. This smoke does not establish durable cross-clone replay.
