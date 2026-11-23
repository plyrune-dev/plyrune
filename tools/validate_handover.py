"""Check handover consistency. This is not a Plyrune product/proof test."""
from pathlib import Path
import json
import re
from html.parser import HTMLParser

ROOT = Path(__file__).resolve().parents[1]
errors = []
for path in ROOT.rglob('*.json'):
    try:
        json.loads(path.read_text())
    except Exception as exc:
        errors.append(f'{path.relative_to(ROOT)}: {exc}')
for path in ROOT.rglob('*.md'):
    for target in re.findall(r'\]\(([^)]+)\)', path.read_text()):
        if target.startswith(('http:', 'https:', '#', 'mailto:')):
            continue
        target = target.split('#', 1)[0]
        if target and not (path.parent / target).exists():
            errors.append(f'{path.relative_to(ROOT)}: missing link {target}')

source = json.loads((ROOT/'examples/input.json').read_text())
req = json.loads((ROOT/'examples/requirement.json').read_text())
expected = json.loads((ROOT/'examples/expected-candidate.json').read_text())
domain = [r for r in source['resources'] if r['path'].startswith(req['scope']['prefix'])]
count = sum(r['text'].count(req['predicate']['value']) for r in domain)
assert count == expected['requiredOccurrenceCount'] == 6
assert len(domain) == expected['domainResourceCount'] == 4
before = {r['path']: r['text'] for r in source['resources']}
after = {r['path']: r['text'] for r in expected['resources']}
for path, content in before.items():
    want = content.replace(req['predicate']['value'], req['disposition']['replacement']) if path.startswith(req['scope']['prefix']) else content
    assert after[path] == want, path
for path in expected['unchangedResources']:
    assert before[path] == after[path], path

class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = []
        self.anchors = []
        self.scripts = []
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if 'id' in a:
            self.ids.append(a['id'])
        if tag == 'a' and a.get('href', '').startswith('#'):
            self.anchors.append(a['href'][1:])
        if tag == 'script' and a.get('src'):
            self.scripts.append(a['src'])

html = (ROOT/'website/index.html').read_text()
page = Page()
page.feed(html)
assert len(page.ids) == len(set(page.ids)), 'duplicate HTML ids'
assert set(page.anchors) <= set(page.ids), 'broken HTML anchors'
assert not page.scripts, 'unexpected external script'
assert 'not a running Plyrune engine' in html
assert 'Not measured yet' in html
metrics = json.loads((ROOT/'website/measurements.json').read_text())
assert metrics['status'] == 'unmeasured' and metrics['trials'] == [] and metrics['metrics'] == []
assert metrics['assurance']['newComposition'] == 'planned'

if errors:
    raise SystemExit('\n'.join(errors))
print('PASS: JSON, local links, example counts/candidate preservation, HTML anchors and honest measurement state.')
print('Not checked: Rust, Lean, live API, browser layout, accessibility conformance or product benchmarks.')
