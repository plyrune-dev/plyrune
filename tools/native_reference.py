"""Six batched native development tasks on the frozen M0 fixture (no writes).

This is the native comparison arm, not a Plyrune engine or a held-out trial.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "examples"


def load(name):
    return json.loads((ROOT / name).read_bytes())


def inventory(resources):
    result = {r["path"]: (r["mode"], r["text"].encode()) for r in resources}
    if len(resources) != len(result):
        raise ValueError("duplicate fixture path")
    return result


def main():
    base = inventory(load("input.json")["resources"])
    requirement = load("requirement.json")
    expected = load("expected-candidate.json")
    prefix = requirement["scope"]["prefix"]
    needle = requirement["predicate"]["value"].encode()
    replacement = requirement["disposition"]["replacement"].encode()
    domain = sorted(p for p in base if p.startswith(prefix))
    hits = []
    for path in domain:
        data = base[path][1]
        start = 0
        while (pos := data.find(needle, start)) >= 0:
            hits.append((path, pos))
            start = pos + len(needle)
    candidate = dict(base)
    for path in domain:
        mode, data = base[path]
        candidate[path] = mode, data.replace(needle, replacement)
    oracle = inventory(expected["resources"])
    first = hits[0]
    path, offset = first
    mode, data = base[path]
    one = data[:offset] + replacement + data[offset + len(needle):]
    narrowed = {p: candidate[p] if p.startswith("config/server/") else base[p]
                for p in base}
    previous_absences = [p for p in domain if needle not in base[p][1]]
    changed = dict(base)
    changed[previous_absences[0]] = mode, needle
    checks = {
        "single lookup": first == ("config/client/app.env", 8),
        "single patch": one == b"PRIMARY=service_endpoint\nSECONDARY=legacy_endpoint\n",
        "repeated inspection": len(hits[:2]) == 2 and len(hits) == 6,
        "broad migration": candidate == oracle and len(domain) == 4,
        "narrowed scope trap": narrowed != oracle,
        "negative dependency recovery": previous_absences == ["config/unchanged.env"]
        and sum(changed[p][1].count(needle) for p in domain) == len(hits) + 1,
    }
    print(json.dumps(checks, indent=2))
    return 0 if all(checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
