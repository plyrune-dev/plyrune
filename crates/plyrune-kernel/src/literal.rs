//! Exact byte search and complete inventory witnesses for literal/v1.

/// The candidate window must fit before indexing either slice. The totality
/// claim covers all positions, not just those reached by `first_match`.
fn matches_at(bytes: &[u8], needle: &[u8], pos: usize) -> bool {
    if pos > bytes.len() {
        return false;
    }
    if needle.len() > bytes.len() - pos {
        return false;
    }
    let mut j = 0;
    while j < needle.len() {
        if bytes[pos + j] != needle[j] {
            return false;
        }
        j += 1;
    }
    true
}

/// The least byte offset of a nonempty literal in the supplied suffix.
/// Empty needles have no selectable occurrence. A caller enumerates greedy
/// nonoverlapping hits by continuing at the exclusive end of each hit.
pub fn first_match(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let mut pos = 0;
    while pos < bytes.len() {
        if matches_at(bytes, needle, pos) {
            return Some(pos);
        }
        pos += 1;
    }
    None
}

/// Compare independently obtained canonical inventory ordinals, not just
/// the members reporting hits. Missing, extra and reordered rows all fail.
pub fn inventory_matches(expected: &[usize], returned: &[usize]) -> bool {
    if expected.len() != returned.len() {
        return false;
    }
    let mut i = 0;
    while i < returned.len() {
        if expected[i] != returned[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{first_match, inventory_matches};

    #[test]
    fn earliest_complete_byte_window() {
        assert_eq!(first_match(b"abab", b"ab"), Some(0));
        assert_eq!(first_match(b"aaab", b"aab"), Some(1));
        assert_eq!(first_match(b"aaac", b"aab"), None);
        assert_eq!(first_match(b"a", b"ab"), None);
        assert_eq!(first_match(b"", b"a"), None);
        assert_eq!(first_match(b"a", b""), None);
    }

    #[test]
    fn witnesses_include_zero_hit_members_in_order() {
        assert!(inventory_matches(&[], &[]));
        assert!(inventory_matches(&[2, 4], &[2, 4]));
        assert!(!inventory_matches(&[2, 4], &[2]));
        assert!(!inventory_matches(&[2, 4], &[4, 2]));
        assert!(!inventory_matches(&[2], &[2, 4]));
    }
}
