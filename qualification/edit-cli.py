"""Exercise the full release executable against private, non-Git Linux directories."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile

exe = str(Path(sys.argv[1]).resolve())

def run(*args, ok=True):
    proc = subprocess.run([exe, *map(str, args)], env={'PATH': ''}, capture_output=True, timeout=20)
    assert (proc.returncode == 0) == ok, (args, proc.returncode, proc.stdout, proc.stderr)
    return json.loads(proc.stdout) if proc.stdout else None

def sha(data):
    return hashlib.sha256(data).hexdigest()

with tempfile.TemporaryDirectory(prefix='plyrune-installed-') as work:
    tmp = Path(work)
    src = tmp / 'plain files'
    tree = src / 'conf'
    tree.mkdir(parents=True)
    original = { 'a': 'éold old\r\n'.encode(), 'b': b'old\n', 'keep': b'unchanged\n' }
    expected = {k: v.replace(b'old', b'new!') for k, v in original.items()}
    for name, data in original.items():
        (tree / name).write_bytes(data)
    (tree / 'a').chmod(0o640)
    outside = src / 'outside'
    outside.write_bytes(b'old')
    def image(folder):
        return {p.relative_to(folder).as_posix(): p.read_bytes() for p in folder.rglob('*') if p.is_file()}
    search = ['--root', src, 'conf/', 'old']
    task = [*search, 'new!']
    refs = run('refs', *search)
    assert (refs['matches'], refs['resources'][0]['ranges']) == (3, [[2, 5], [6, 9]])
    before = run('preview', *task)
    review = before['review']
    assert {r['path']: (r['base_sha256'], r['output_sha256']) for r in before['resources']} == {
        'conf/'+p: (sha(original[p]), sha(expected[p])) for p in expected}
    def apply(*dest, key=review, ok=True):
        return run('apply', *task, '--review', key, *dest, ok=ok)
    apply(ok=False)
    apply('--in-place', '--into', tmp / 'two', ok=False)
    apply('--into', tree / 'inside', ok=False)
    assert not (tree / 'inside').exists()
    assert image(tree) == original
    # Equal contents under another root must not authorize publication there.
    other = tmp / 'copy'
    shutil.copytree(src, other)
    run('apply', '--root', other, 'conf/', 'old', 'new!', '--review', review, '--in-place', ok=False)
    assert image(other / 'conf') == original
    occupied = tmp / 'occupied'
    occupied.mkdir()
    (occupied / 'mine').write_text('mine')
    apply('--into', occupied, ok=False)
    assert image(occupied) == {'mine': b'mine'}
    # A foreign review and a substituted no-hit member may never create an output.
    denied = tmp / 'denied'
    apply('--into', denied, key='0' * 64, ok=False)
    assert not denied.exists()
    for output, precreated in ((tmp / 'new dest', False), (tmp / 'my empty directory', True)):
        if precreated:
            output.mkdir()
        report = apply('--into', output)
        assert report['status'] == 'Complete' and image(output) == {'conf/'+p: data for p, data in expected.items()}
        assert stat.S_IMODE((output / 'conf/a').stat().st_mode) == 0o640
        assert image(tree) == original and outside.read_bytes() == b'old'
    (tree / 'keep').write_text('not reviewed')
    apply('--into', denied, ok=False)
    assert not denied.exists() and (tree / 'a').read_bytes() == original['a']
    (tree / 'keep').write_bytes(original['keep'])
    # A link inside scope is an unsupported member, even if it would never match.
    bad = tree / 'link'
    bad.symlink_to(outside)
    run('refs', *search, ok=False)
    bad.unlink()
    os.link(outside, bad)
    run('preview', *task, ok=False)
    bad.unlink()
    key = run('preview', *task)['review']
    unchanged_inode = (tree / 'keep').stat().st_ino
    report = apply('--in-place', key=key)
    assert report['status'] == 'Complete' and image(tree) == expected
    assert stat.S_IMODE((tree / 'a').stat().st_mode) == 0o640
    assert (tree / 'keep').stat().st_ino == unchanged_inode and outside.read_bytes() == b'old'
    apply('--in-place', key=key, ok=False)
    zero = run('preview', *task)
    assert (zero['disposition'], zero['matches']) == ('NoChange', 0)
    assert apply('--in-place', key=zero['review'])['status'] == 'NoChange'
print('Installed cycle: literal refs, reviewed hashes, two destination modes, stale/foreign/unsafe input refusals and NoChange passed without Git.')
