"""Exercise the retained Rust dependency on the five-resource golden fixture."""
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[2]
load = lambda name: json.loads((root / "examples" / name).read_bytes())
requirement = load("requirement.json")
assert requirement["predicate"]["value"] == "legacy_endpoint"
assert requirement["disposition"]["replacement"] == "service_endpoint"
result = []
for row in load("input.json")["resources"]:
    if row["path"].startswith(requirement["scope"]["prefix"]):
        binary = Path(os.environ["CARGO_TARGET_DIR"]) / "release/plyrune-splice-probe"
        out = subprocess.run([binary], input=row["text"].encode(), capture_output=True,
                             check=True, timeout=5)
        row = {**row, "text": out.stdout.decode()}
    result.append(row)
if result != load("expected-candidate.json")["resources"]:
    raise SystemExit("fixture candidate mismatch")
print("Exact dependency probe: all five candidate resources match the golden fixture.")
