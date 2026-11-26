"""Black-box literal/v1 qualification. Use an installed executable, not the Rust library."""
import json
from pathlib import Path
import subprocess
import sys

cli = str(Path(sys.argv[1]).resolve())
manifest = json.loads(subprocess.check_output([cli, 'example']))
manifest['resources'] = [
    {'path': p, 'text': t, 'mode': 'regular'} for p, t in
    [('scope/no-hit', 'b'), ('scope/sub/hits', 'éaaaaa\r\n'), ('scope2/other', 'aa')]
]

def refs(prefix, needle):
    return subprocess.run([cli, 'refs', prefix, needle], input=json.dumps(manifest).encode(),
                          capture_output=True, timeout=10, env={'PATH': ''})

def success(prefix, needle):
    p = refs(prefix, needle)
    assert p.returncode == 0, p.stderr
    result = json.loads(p.stdout)
    assert result['coverage'] == 'Complete'
    return result

found = success('scope/', 'aa')
assert (found['resources'], found['matches']) == ([{'path': 'scope/no-hit', 'ranges': []},
    {'path': 'scope/sub/hits', 'ranges': [[2, 4], [4, 6]]}], 2)
assert success('missing/', 'aa')['resources'] == []
assert success('scope/', 'x')['matches'] == 0
for p, n in [('scope/', ''), ('scope', 'aa'), ('../scope/', 'aa')]:
    denied = refs(p, n)
    assert denied.returncode == 2 and not denied.stdout, (denied, p, n)
manifest['resources'][0]['text'] = 'a' * 4097
r = refs('scope/', 'a')
assert r.returncode == 2 and not r.stdout and b'ResourceMatchLimit' in r.stderr, r
print('Literal CLI: scoped inventory, no-hit witnesses, UTF-8 non-overlap, empty scope and budget refusal passed.')
