"""Run inside the pinned Compose gate image. Never modify the release registry."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib

SPIKE = Path(__file__).resolve().parent
ROOT = SPIKE.parent.parent
CRATE = SPIKE.relative_to(ROOT)


def controls(work, mode, text):
    src = work / 'lib.rs'
    src.write_text(text)
    compiler = ['rustc', '+1.98.0', '--edition=2024', '-C', 'overflow-checks=yes']
    subprocess.run([*compiler, '--crate-name=matcher_spike', '--crate-type=rlib', src],
                   cwd=work, check=True)
    subprocess.run([*compiler, SPIKE / 'controls.rs', '--extern',
                    'matcher_spike=libmatcher_spike.rlib', '-o', 'controls'], cwd=work, check=True)
    subprocess.run(['./controls', mode], cwd=work, check=True)
    print(f'native {mode}: exact first/all, near miss, complete-empty and refusal controls passed', flush=True)


def run(work):
    crate = work / CRATE
    proof = work / 'proofs'
    shutil.copytree(SPIKE / 'src', crate / 'src')
    shutil.copy(SPIKE / 'Cargo.toml', crate)
    shutil.copy(ROOT / 'Cargo.toml', work)
    shutil.copy(ROOT / 'Cargo.lock', work)
    (proof / 'Proofs').mkdir(parents=True)
    for name in ['extract.sh', 'gate.py', 'Audit.lean', 'lean-toolchain', 'lake-manifest.json', 'lakefile.toml']:
        shutil.copy(ROOT / 'proofs' / name, proof)
    shutil.copy(ROOT / 'proofs/Proofs/TautenAssumption.lean', proof / 'Proofs')
    shutil.copy(SPIKE / 'Matcher.lean', proof / 'Proofs')
    shutil.copy(SPIKE / 'claims.toml', proof)
    (proof / 'Proofs.lean').write_text('import Proofs.TautenAssumption\nimport Proofs.Matcher\n')
    (proof / '.lake').mkdir()
    (proof / '.lake/packages').symlink_to('/work/proofs/.lake/packages')

    mutant, = tomllib.loads((proof / 'claims.toml').read_text())['mutant']
    text = (work / mutant['file']).read_text()
    if text.count(mutant['before']) != 1:
        raise SystemExit('ambiguous control site')
    controls(work, 'baseline', text)
    controls(work, 'skipped', text.replace(mutant['before'], mutant['after']))

    # Seed the drift check with a fresh translation, not a handwritten model.
    # The release template then re-extracts both baseline and source mutant.
    subprocess.run(['sh', proof / 'extract.sh', crate, proof / 'Extracted', '--', '-abort-on-error'],
                   cwd=work, check=True)
    generated = proof / 'Extracted/MatcherSpike.lean'
    print('extracted MatcherSpike.lean sha256:', hashlib.sha256(generated.read_bytes()).hexdigest(), flush=True)
    records = []
    with subprocess.Popen([sys.executable, proof / 'gate.py'], cwd=work, stdout=subprocess.PIPE,
                          text=True) as gate:
        for line in gate.stdout:
            print(line, end='', flush=True)
            if line.startswith('TAUTEN-PROOFS '):
                records.append(json.loads(line.removeprefix('TAUTEN-PROOFS ')))
        if gate.wait() != 0:
            print(generated.read_text(), file=sys.stderr)
            raise SystemExit('isolated Lean gate failed')
    if not any(r.get('total') == 'Spike.scan_spec' and r.get('function') == 'matcher_spike.scan'
               for r in records):
        raise SystemExit('missing audited, total scan theorem')
    if not any(r.get('mutant') == mutant['name'] and r.get('status') == 'killed' and
               r.get('direct') == mutant['direct'] for r in records):
        raise SystemExit('missing directly calibrated scan theorem failure')


if __name__ == '__main__':
    started = time.monotonic()
    try:
        with tempfile.TemporaryDirectory(prefix='plyrune-matcher-') as tmp:
            run(Path(tmp))
    finally:
        print(f'spike elapsed: {time.monotonic() - started:.1f} s', flush=True)
