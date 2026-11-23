"""Independent snapshot-preimage vectors for the revision-1 fixture profile.

Trusted development fixtures only, not an implementation of wire admission.
"""
import hashlib
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def frame(data):
    return struct.pack(">Q", len(data)) + data


def digest(resources):
    preimage = bytearray(frame(b"plyrune/fixture/v1"))
    preimage.extend(struct.pack(">Q", len(resources)))
    for path, mode, content in sorted(resources):
        for item in (path, mode, content):
            preimage.extend(frame(item))
    return hashlib.sha256(preimage).hexdigest()


def vectors():
    manifest = json.loads((ROOT / "examples/input.json").read_bytes())
    rows = [(r["path"].encode(), r["mode"].encode(), r["text"].encode())
            for r in manifest["resources"]]
    actual = digest(rows)
    if digest(list(reversed(rows))) != actual:
        raise ValueError("ordering changed content identity")
    return {
        "empty_inventory": digest([]),
        "empty_file": digest([(b"a", b"regular", b"")]),
        "fixture": actual,
        "same_bytes_other_path": digest([(b"b", b"regular", b"")]),
        "unicode_crlf": digest([(b"a", b"regular", "é\r\n".encode())]),
    }


if __name__ == "__main__":
    import sys

    actual = vectors()
    if sys.argv[1:] == ["--print"]:
        print(json.dumps(actual, indent=2))
    elif actual != json.loads((ROOT / "qualification/fixture-vectors.json").read_bytes()):
        raise SystemExit("fixture framing drift")
    else:
        print("Canonical fixture vectors match.")
