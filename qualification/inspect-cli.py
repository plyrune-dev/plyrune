"""Consumer qualification: pass the path to the actual preview executable."""
import json
import subprocess
import sys

exe = sys.argv[1]


def run(args, data=b"", refusal=None):
    p = subprocess.run([exe, *args], input=data, capture_output=True, timeout=10)
    if refusal is not None:
        assert (p.returncode, p.stdout) == (2, b""), (args, p)
        assert refusal in p.stderr.decode(), p.stderr
    else:
        assert p.returncode == 0, (args, p.stderr)
        return p.stdout


view, stale = map(json.loads, run(["demo"]).splitlines())
assert view["text"] == "legacy_endpoint" and "WrongSnapshot" in stale["refused"]
data = run(["example"])
key = json.loads(run(["snapshot"], data))["sha256"]
assert key == "6ba5a5e50f91714f5a6d96a5ad3a07e525f375f95b6a2b86b2594673319b9d5c"
path = "config/client/app.env"
assert json.loads(run(["inspect", key, path, "0", "7"], data))["text"] == "PRIMARY"
run(["inspect", key, "missing", "0", "0"], data, "UnknownPath")
for at in (0, 63):
    wrong = key[:at] + ("1" if key[at] == "0" else "0") + key[at + 1:]
    run(["inspect", wrong, path, "55", "2"], data, "WrongSnapshot")
run(["inspect", key, path, "55", "2"], data, "OutOfBounds")
manifest = json.loads(data)
manifest["resources"][1]["text"] = "é" + "a" * 4096
changed = json.dumps(manifest).encode()
new = json.loads(run(["snapshot"], changed))["sha256"]
run(["inspect", key, path, "0", "2"], changed, "WrongSnapshot")
run(["inspect", new, path, "0", "1"], changed, "NotCharBoundary")
run(["inspect", new, path, "0", "4097"], changed, "DisplayBudget")
assert json.loads(run(["inspect", new, path, "0", "2"], changed))["text"] == "é"
assert json.loads(run(["inspect", new, path, "2", "2"], changed))["text"] == ""
print("Installed CLI: exact subject, path, UTF-8, range precedence and display cap passed.")
