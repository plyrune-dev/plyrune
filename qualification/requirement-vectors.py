"""Check frozen #3/#31 vectors, not a wire decoder or production coverage checker.

Trusted review data only. No Plyrune imports or CLI, no filesystem mutation.
The base fields stand for resolved candidate base and selection requirement.
Budget examples exercise arithmetic on sizes, not claimed allocation limits.
"""
import hashlib
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODES = ("host-authored", "model-proposed, human-confirmed", "model-authored")
FIELDS = ("owner", "source", "snapshot", "requirement", "mode", "evidenceRef")
MIB = 1 << 20


def frame(value):
    return len(value).to_bytes(8, "big") + value


def framed_hash(tag, values):
    return hashlib.sha256(frame(tag.encode()) + b"".join(map(frame, values))).hexdigest()


def snapshot(files):
    content = hashlib.sha256(frame(b"plyrune/fixture/v1") + len(files).to_bytes(8, "big"))
    for path, text in sorted(files.items()):
        content.update(frame(path.encode()) + frame(b"regular") + frame(text.encode()))
    return content.hexdigest()


def requirement(fields):
    prefix, needle, _ = fields
    if not re.fullmatch(r"(?:[A-Za-z0-9._-]+/)+", prefix) or any(
        part in (".", "..") for part in prefix.split("/")
    ):
        return "InvalidPrefix"
    if not needle:
        return "EmptyNeedle"
    return framed_hash("plyrune/requirement/literal/v1", (v.encode() for v in fields))


def admission(record):
    values = (bytes.fromhex(record[k]) if k in ("snapshot", "requirement")
              else record[k].encode() for k in FIELDS)
    return framed_hash("plyrune/admission/fixture-literal/v1", values)


def binding(host, offered, base):
    # Host lookup/authentication and local handle resolution are premises, not simulated.
    for record in (host, offered):
        if record.get("mode") not in MODES:
            return "InvalidMode"
        if not record.get("evidenceRef"):
            return "MissingEvidence"
    errors = ("WrongOwner", "WrongSource", "WrongSnapshot", "WrongRequirement",
              "WrongMode", "WrongEvidence")
    for expected, actual, fields in ((host, offered, FIELDS), (offered, base, FIELDS[:4])):
        for key, error in zip(fields, errors):
            if expected[key] != actual[key]:
                return error
    return "Bound"


def witness(fields, files, claim):
    prefix, needle, _ = fields
    domain = sorted(p for p in files if p.startswith(prefix))
    if domain != sorted(claim):
        return "WrongInventory"
    literal = re.compile(re.escape(needle.encode()))
    for path in domain:
        spans = [list(m.span()) for m in literal.finditer(files[path].encode())]
        if spans != claim[path]:
            return "WrongOccurrences"
    return "ExactWitness" if any(claim.values()) else "WitnessedZero"


def budget(case):
    inputs, matches, outputs = (case.get(k, []) for k in ("inputs", "matches", "outputs"))
    limits = (
        (case.get("resources", 0), 256, "InventoryLimit"),
        (case.get("path", 0), 255, "PathLimit"),
        (max(inputs, default=0), 8 * MIB, "InputLimit"),
        (sum(inputs), 32 * MIB, "TotalInputLimit"),
        (case.get("prefix", 0), 255, "PrefixLimit"),
        (case.get("needle", 0), 8 * MIB, "NeedleLimit"),
        (case.get("replacement", 0), 16 * MIB, "ReplacementLimit"),
        (max(case.get(k, 0) for k in ("owner", "source", "evidenceRef")), 255, "IdentityLimit"),
        (max(matches, default=0), 4096, "MatchLimit"),
        (sum(matches), 8192, "TotalMatchLimit"),
        (max(outputs, default=0), 16 * MIB, "OutputLimit"),
        (sum(outputs), 32 * MIB, "TotalOutputLimit"),
    )
    return next((error for size, cap, error in limits if size > cap), "WithinLimits")


def check(label, actual, expected):
    if actual != expected:
        raise SystemExit(f"{label}: expected {expected!r}, got {actual!r}")


def main():
    vectors = json.loads((HERE / "requirement-vectors.json").read_bytes())
    rows = json.loads((HERE.parent / "examples/input.json").read_bytes())["resources"]
    files = {r["path"]: r["text"] for r in rows}
    check("fixture paths distinct", len(files), len(rows))
    check("fixture modes", {r["mode"] for r in rows}, {"regular"})
    tasks = vectors["requirements"]
    fixture_task = json.loads((HERE.parent / "examples/requirement.json").read_bytes())
    check("fixture task fields", tasks["fixture"][0], [
        fixture_task["scope"]["prefix"], fixture_task["predicate"]["value"],
        fixture_task["disposition"]["replacement"],
    ])
    for name, (fields, expected) in tasks.items():
        check(name, requirement(fields), expected)

    record = vectors["hostRecord"]
    check("host snapshot", snapshot(files), record["snapshot"])
    check("host task", requirement(tasks["fixture"][0]), record["requirement"])
    for mode in MODES:
        check(mode, admission(record | {"mode": mode}), vectors["admissionHashes"][mode])

    # S2 changes only an *excluded* file. Named substitutions denote actual hashes.
    updated = snapshot(files | {"docs/notes.txt": files["docs/notes.txt"] + "!"})
    def resolve(overrides):
        values = record | overrides
        if values["snapshot"] == "updated-fixture":
            values["snapshot"] = updated
        if values["requirement"] in tasks:
            values["requirement"] = requirement(tasks[values["requirement"]][0])
        return values

    for case in vectors["bindings"]:
        subjects = (resolve(case.get(k, {})) for k in ("host", "admission", "base"))
        check(case["name"], binding(*subjects), case["expect"])
    for case in vectors["witnesses"]:
        inventory = files if case["files"] == "fixture" else case["files"]
        check(case["name"], witness(tasks[case["task"]][0], inventory, case["claim"]), case["expect"])
    for case in vectors["budgetCases"]:
        check(case["name"], budget(case), case["expect"])
    print("Fixture-v1 requirement framing, admission, witness and budget vectors match.")


if __name__ == "__main__":
    main()
