"""Development corpus v1. Grade an immutable Git tree, not a working directory."""
import argparse
import hashlib
from pathlib import Path
import subprocess


def manifest(refreshed, expected):
    # Construct the oracle from designated slots, without searching base bytes.
    slot = "service_endpoint" if expected else "legacy_endpoint"
    files = {}
    for i in range(64):
        ending = "\r\n" if i % 3 == 0 else "\n"
        lines = [f"# shard {i}: legacy-endpoint is a near miss",
                 *(f"{name}={slot}" for name in ("PRIMARY", "SECONDARY", "UP", "DOWN"))]
        files[f"config/group{i % 8}/service{i:02}.env"] = ending.join(lines) + ending
    files["config/quiet.env"] = "URL=" + (slot if refreshed else "modern_endpoint") + "\n"
    files["config2/outside.env"] = "URL=legacy_endpoint\n"
    files["docs/notes.txt"] = "legacy_endpoint\n"
    return {p: text.encode() for p, text in files.items()}


def grade(repo, tree, files):
    def git(*args):
        return subprocess.check_output(["git", "-C", repo, *args])

    algorithm = git("rev-parse", "--show-object-format").decode().strip()
    oid = git("rev-parse", "--verify", "--end-of-options", tree + "^{tree}").decode().strip()
    observed = {}
    for entry in git("ls-tree", "-rz", oid).split(b"\0"):
        if entry:
            meta, path = entry.split(b"\t", 1)
            observed[path] = meta.split()
    oracle = {}
    for path, data in files.items():
        blob = b"blob " + str(len(data)).encode() + b"\0" + data
        oracle[path.encode()] = [b"100644", b"blob", hashlib.new(algorithm, blob).hexdigest().encode()]
    return sorted(p for p in observed.keys() | oracle.keys() if observed.get(p) != oracle.get(p))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--refreshed", action="store_true")
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("materialize").add_argument("new_directory", type=Path)
    check = sub.add_parser("grade")
    check.add_argument("repo")
    check.add_argument("tree")
    args = parser.parse_args()
    files = manifest(args.refreshed, expected=args.cmd == "grade")
    if args.cmd == "materialize":
        args.new_directory.mkdir()  # Must be new; never overwrite a user's tree.
        for path, data in files.items():
            dest = args.new_directory / path
            dest.parent.mkdir(exist_ok=True, parents=True)
            dest.write_bytes(data)
            dest.chmod(0o644)
        return 0
    differences = grade(args.repo, args.tree, files)
    for p in differences:
        print("different:", repr(p))
    return int(bool(differences))


if __name__ == "__main__":
    raise SystemExit(main())
