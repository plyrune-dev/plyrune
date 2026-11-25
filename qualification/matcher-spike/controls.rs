use matcher_spike::scan;

fn main() {
    let skipped = std::env::args().nth(1).as_deref() == Some("skipped");
    // Direct semantic calibration: the mutant omits offset zero only. An
    // empty output, refusal or crash here is not the expected counterexample.
    let first: &[usize] = if skipped { &[2] } else { &[0] };
    let all: &[usize] = if skipped { &[2] } else { &[0, 2] };
    assert_eq!(scan(b"abab", b"ab", true).as_deref(), Some(first));
    assert_eq!(scan(b"abab", b"ab", false).as_deref(), Some(all));
    for mode in [true, false] {
        assert_eq!(scan(b"aaab", b"aab", mode), Some(vec![1]));
        assert_eq!(scan(b"aaac", b"aab", mode), Some(vec![]));
        assert_eq!(scan(b"", b"x", mode), Some(vec![]));
        assert_eq!(scan(b"x", b"", mode), None);
    }
    if !skipped {
        assert_eq!(scan(b"aaaaa", b"aa", false), Some(vec![0, 2]));
        assert_eq!(scan("éé".as_bytes(), "é".as_bytes(), false), Some(vec![0, 2]));
    }
}
