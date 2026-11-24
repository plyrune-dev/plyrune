//! Snapshot content binding for stateless fixture inspection.
use crate::range::{RangeRefusal, admit_text};

/// The two halves of one 256-bit digest, in network byte order.
pub struct SnapshotKey {
    high: u128,
    low: u128,
}

impl SnapshotKey {
    /// All bit patterns represent a digest; none grant authority.
    pub const fn from_halves(high: u128, low: u128) -> Self {
        Self { high, low }
    }
}

/// Check snapshot identity before admitting text offsets.
///
/// # Errors
/// A mismatch precedes any coordinate error. The caller resolves the path
/// in the same admitted inventory that produced `actual`.
pub fn inspect_range(
    expected: &SnapshotKey,
    actual: &SnapshotKey,
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Result<(), RangeRefusal> {
    if expected.high == actual.high && expected.low == actual.low {
        admit_text(bytes, start, end)
    } else {
        Err(RangeRefusal::WrongSnapshot)
    }
}
