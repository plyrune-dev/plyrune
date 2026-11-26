//! The decisions between a complete literal selection and a reviewed candidate.
//!
//! The caller supplies the authoritative resource independently of the capture.
//! For each independently required path it feeds `consume` every original gap,
//! each fixed replacement, and the final original suffix, in that order. It
//! starts at zero, threads the returned cursor, and requires `finished` at the
//! end. Even a no-hit file has one required term: its entire original image.
//! Neither digest equality nor a displayed subset is a coverage witness.

use crate::capture::{ByteOffset, CaptureKind, CaptureSchema, CaptureSubject, admit_capture};
use crate::literal::matches_at;
use crate::range::RangeRefusal;

/// Lower the sealed v1 text capture to a splice cut on the supplied resource.
/// Neither byte equality nor coincident offsets stand in for resource identity.
///
/// # Errors
///
/// Returns the capture admission refusal, without producing a cut.
pub fn lower_cut(
    presented: &CaptureSubject,
    actual: &CaptureSubject,
    source: &[u8],
    start: ByteOffset,
    end: ByteOffset,
) -> Result<(usize, usize), RangeRefusal> {
    match admit_capture(
        CaptureSchema::V1,
        CaptureKind::TEXT,
        presented,
        actual,
        source,
        start,
        end,
    ) {
        Ok(()) => Ok((start.get(), end.get())),
        Err(why) => Err(why),
    }
}

/// Consume exactly the required raw term at this candidate position. Empty
/// terms may be consumed at a valid cursor (including EOF), not beyond EOF.
#[must_use]
pub fn consume(candidate: &[u8], cursor: usize, required: &[u8]) -> Option<usize> {
    if matches_at(candidate, required, cursor) {
        Some(cursor + required.len())
    } else {
        None
    }
}

/// Exhaustion is equality, not just fitting within the image or inventory.
#[must_use]
pub const fn finished(consumed: usize, total: usize) -> bool {
    consumed == total
}

/// Exact equality for whole path bytes, source literals or reviewed digest
/// bytes. If these are digests, framing and collision resistance are external.
#[must_use]
pub fn same_bytes(expected: &[u8], presented: &[u8]) -> bool {
    expected.len() == presented.len() && matches_at(presented, expected, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{IncarnationId, OwnerId, StoreBinding};
    use crate::capture::ResourceId;
    use crate::snapshot::SnapshotKey;

    fn subject(owner: u64) -> CaptureSubject {
        CaptureSubject::new(
            StoreBinding::new(OwnerId::from_host(owner), IncarnationId::from_host(2)),
            ResourceId::from_ordinal(0),
            SnapshotKey::from_halves(3, 4),
        )
    }

    #[test]
    fn cut_is_bound_to_owner_and_preserves_both_offsets() {
        let start = ByteOffset::new(1);
        let end = ByteOffset::new(3);
        assert_eq!(
            lower_cut(&subject(1), &subject(1), b"abcd", start, end),
            Ok((1, 3))
        );
        assert_eq!(
            lower_cut(&subject(1), &subject(5), b"abcd", start, end),
            Err(RangeRefusal::WrongStore)
        );
    }

    #[test]
    fn complete_terms_including_deletion_and_unchanged_file() {
        let candidate = b"aXXb";
        assert_eq!(consume(candidate, 0, b"a"), Some(1));
        assert_eq!(consume(candidate, 1, b"XX"), Some(3));
        assert_eq!(consume(candidate, 1, b"XY"), None);
        assert_eq!(consume(candidate, 3, b"b"), Some(4));
        assert_eq!(consume(candidate, 4, b""), Some(4));
        assert_eq!(consume(candidate, usize::MAX, b""), None);
        assert!(finished(4, candidate.len()));
        assert!(!finished(4, b"aXXb!".len()));
        assert_eq!(consume(b"unchanged", 0, b"unchanged"), Some(9));
    }

    #[test]
    fn whole_subject_not_only_a_prefix_or_length() {
        assert!(same_bytes(b"review:a", b"review:a"));
        assert!(!same_bytes(b"review:a", b"review:b"));
        assert!(!same_bytes(b"review:a", b"review:a!"));
    }
}
