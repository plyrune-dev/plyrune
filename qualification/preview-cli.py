"""Compare the actual preview command against the separately supplied expected candidate."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

exe = str(Path(sys.argv[1]).resolve())
examples = Path(__file__).resolve().parents[1] / 'examples'
base = (examples / 'input.json').read_bytes()
expected = json.loads((examples / 'expected-candidate.json').read_bytes())
args = [exe, 'preview', 'config/', 'legacy_endpoint']

def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()

def preview(replacement):
    return json.loads(subprocess.check_output([*args, replacement], input=base, env={'PATH': ''}, timeout=10))

report = preview('service_endpoint')
assert (report['disposition'], report['matches'], report['changed_files']) == ('Changed', 6, 3)
assert {r['path']: r['output_sha256'] for r in report['resources']} == {
    r['path']: sha(r['text']) for r in expected['resources']}
assert {r['path']: r['base_sha256'] for r in report['resources']} == {
    r['path']: sha(r['text']) for r in json.loads(base)['resources']}
assert sum(len(r['ranges']) for r in report['resources']) == 6
unchanged = preview('legacy_endpoint')
assert (unchanged['disposition'], unchanged['matches'], unchanged['changed_files']) == ('NoChange', 6, 0)
assert unchanged['review'] != report['review']
print('Candidate CLI: every base/output hash matches independent fixtures; six hits, unchanged scope and no-op task passed.')
