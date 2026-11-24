#!/bin/sh
# Standalone consumer. The caller provides a fresh artifact directory and trusted binary.
set -eu
out=${1:?supply the artifact directory}
set -C
plyrune snapshot < examples/input.json > "$out/snapshot.json"
python3 - "$out/snapshot.json" <<'PY'
import json
from pathlib import Path
import sys
expected = json.loads(Path("qualification/fixture-vectors.json").read_bytes())["fixture"]
record = json.loads(Path(sys.argv[1]).read_bytes())
if (record != {"profile": "plyrune/fixture/v1", "resources": 5, "sha256": expected}
        or type(record["resources"]) is not int):
    sys.exit("snapshot output does not identify the expected five-resource fixture")
PY
