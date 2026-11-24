# Scaled literal migration (development corpus v1)

Task: replace every exact `legacy_endpoint` under `config/` with `service_endpoint`. Keep every other byte, path and mode. There are 256 slots across 64 matching files, one in-scope negative, two excluded matches, near misses and mixed line endings. In the refresh variant the negative becomes a 257th slot. This is a synthetic correctness control, not a held-out trial.

From the repository root in the Compose dev container:

```sh
python3 qualification/scaled/corpus.py materialize /tmp/task
git -C /tmp/task init -q
# Give the editor only the task directory and requirement.
# After it finishes, the evaluator captures the full tree in this disposable repo:
git -C /tmp/task -c core.autocrlf=false add -f -A
python3 qualification/scaled/corpus.py grade /tmp/task "$(git -C /tmp/task write-tree)"
```

Use `--refreshed` before both subcommands for the changed input. Pin the corpus by repository revision. The grader constructs expected blobs from slots rather than running a replacement algorithm on the candidate. It checks the **entire Git file inventory**, modes and blob identities, independent of editor and checker; it assumes Git hash integrity. Keep the grader outside the agent checkout. Do not treat timings or pass rates from `check.sh` as model results.
