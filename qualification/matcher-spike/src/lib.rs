#![no_std]

extern crate alloc;
use alloc::vec::Vec;

// The caller established pos < bytes.len(). After the fit test the entire
// window exists, so pos + j cannot overflow and both indices are in bounds.
fn matches_at(bytes: &[u8], needle: &[u8], pos: usize) -> bool {
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

fn scan_from(bytes: &[u8], needle: &[u8], first_only: bool, start: usize) -> Vec<usize> {
    let mut hits = Vec::new();
    let mut pos = start;
    while pos < bytes.len() {
        if matches_at(bytes, needle, pos) {
            hits.push(pos);
            if first_only {
                return hits;
            }
            pos += needle.len();
        } else {
            pos += 1;
        }
    }
    hits
}

/// One immutable byte image. None refuses an empty needle; Some(empty)
/// means a complete miss. Each returned start has end = start + needle.len().
pub fn scan(bytes: &[u8], needle: &[u8], first_only: bool) -> Option<Vec<usize>> {
    if needle.len() == 0 {
        return None;
    }
    Some(scan_from(bytes, needle, first_only, 0))
}
