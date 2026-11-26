"""Native behavior of the actual source mutations selected by the Lean gate."""
from pathlib import Path
import subprocess
import tempfile
import tomllib

root = Path(__file__).resolve().parents[1]
registry = tomllib.loads((root / 'proofs/claims.toml').read_text())
file = 'crates/plyrune-kernel/src/literal.rs'
source = (root / file).read_text()
control = '''
fn main() {
    let mode = std::env::args().nth(1).unwrap();
    // Only the named fault changes each companion's expected native result.
    let first = if mode == "skip-initial-match" { Some(2) } else { Some(0) };
    assert_eq!(first_match(b"ababX", b"ab"), first);
    assert_eq!(first_match(b"@ab", b"ab"), if mode == "reject-exact-fit" { None } else { Some(1) });
    assert_eq!(first_match(b"@ac", b"ab"), None);
    assert_eq!(inventory_matches(&[2, 4], &[2]), mode == "accept-missing-zero-member");
    assert!(inventory_matches(&[2, 4], &[2, 4]));
    assert!(!inventory_matches(&[2, 4], &[4, 2]));
    assert!(!inventory_matches(&[2], &[2, 4]));
}
'''
mutants = [m for m in registry['mutant'] if m['file'] == file]
assert {m['name'] for m in mutants} == {'skip-initial-match', 'reject-exact-fit', 'accept-missing-zero-member'}
with tempfile.TemporaryDirectory() as temp:
    work = Path(temp)
    for mutant in [None, *mutants]:
        name = mutant['name'] if mutant else 'baseline'
        text = source
        if mutant:
            assert text.count(mutant['before']) == 1, name
            text = text.replace(mutant['before'], mutant['after'])
        (work / 'literal.rs').write_text(text + control)
        subprocess.run(['rustc', '+1.98.0', '--edition=2024', '-O', '-C', 'overflow-checks=yes',
                        'literal.rs', '-o', 'controls'], cwd=work, check=True)
        subprocess.run([work / 'controls', name], check=True)
        print('native literal:', name, flush=True)
