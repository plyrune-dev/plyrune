//! Admission of half-open byte ranges into one immutable resource image.
//!
//! A capture is a half-open byte range `[start, end)` into the bytes of one
//! resource of one snapshot. Its coordinates are decided here and nowhere
//! else: every path that turns wire or host numbers into a range calls
//! [`admit`] or [`admit_text`], so a reversed or out-of-image range is
//! refused instead of sliced. Byte offsets are not character indices or
//! display coordinates; a text profile adds the boundary rule below.
//!
//! Obligation PR-C01 (`docs/ASSURANCE.md`): an admitted range resolves to a
//! range of the named image with exact bytes. That a boundary is a
//! character boundary of *well-formed* UTF-8 is a property of the image: it
//! is checked where the image is admitted, and assumed, not proved, here.
//!
//! Shape rules for the translation, each recorded where it applies: `get`
//! rather than indexing, explicit `match` (no `?`), no iterators, no derives
//! reached from a translated function.

/// Why a half-open byte range was not admitted into a resource image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeRefusal {
    /// `start` is past `end`, or `end` lies beyond the image.
    OutOfBounds,
    /// Under the text profile, an end that does not fall on a character
    /// boundary of the image.
    NotCharBoundary,
    /// The store-bound caller presented a different owner or incarnation.
    WrongStore,
    /// The request names a different snapshot.
    WrongSnapshot,
    /// The capture does not use the supported schema.
    UnsupportedSchema,
    /// The artifact is not a text capture.
    WrongKind,
    /// Another resource was requested, even if it contains identical bytes.
    WrongResource,
}

/// Whether `byte` continues the UTF-8 character before it (`10xxxxxx`).
#[must_use]
pub const fn is_continuation(byte: u8) -> bool {
    byte >= 0x80 && byte <= 0xBF
}

/// Whether `at` is a character boundary of `bytes`: the end of the image, or
/// a position whose byte does not continue a character. A position past the
/// end is not a boundary.
///
/// For well-formed UTF-8 this is exactly `str::is_char_boundary`.
#[must_use]
pub fn char_boundary(bytes: &[u8], at: usize) -> bool {
    match bytes.get(at) {
        Some(byte) => !is_continuation(*byte),
        None => at == bytes.len(),
    }
}

/// Admit the half-open byte range `[start, end)` of an image of `len` bytes:
/// accepted exactly when `start <= end <= len`. The image's length is given
/// rather than its bytes because the range is bound to the image by the
/// caller, which holds the snapshot and resource identities.
///
/// # Errors
///
/// [`RangeRefusal::OutOfBounds`] when `start > end` or `end > len`.
pub fn admit(len: usize, start: usize, end: usize) -> Result<(), RangeRefusal> {
    if start > end || end > len {
        Err(RangeRefusal::OutOfBounds)
    } else {
        Ok(())
    }
}

/// Admit the half-open byte range `[start, end)` of `bytes` under the text
/// profile: [`admit`] accepts it, and both ends are character boundaries of
/// `bytes` ([`char_boundary`]). A refusal names the first rule that fails.
///
/// # Errors
///
/// [`RangeRefusal::OutOfBounds`] as for [`admit`], else
/// [`RangeRefusal::NotCharBoundary`] when an end splits a character.
pub fn admit_text(bytes: &[u8], start: usize, end: usize) -> Result<(), RangeRefusal> {
    match admit(bytes.len(), start, end) {
        Ok(()) => {}
        Err(refusal) => return Err(refusal),
    }
    if char_boundary(bytes, start) && char_boundary(bytes, end) {
        Ok(())
    } else {
        Err(RangeRefusal::NotCharBoundary)
    }
}

#[cfg(test)]
mod tests {
    use super::{RangeRefusal, admit, admit_text, char_boundary, is_continuation};

    #[test]
    fn admits_empty_partial_and_full_ranges() {
        assert_eq!(admit(5, 0, 0), Ok(()));
        assert_eq!(admit(5, 5, 5), Ok(()));
        assert_eq!(admit(5, 0, 5), Ok(()));
        assert_eq!(admit(5, 2, 4), Ok(()));
        assert_eq!(admit(0, 0, 0), Ok(()));
    }

    #[test]
    fn refuses_reversed_and_out_of_image_ranges() {
        assert_eq!(admit(5, 3, 2), Err(RangeRefusal::OutOfBounds));
        assert_eq!(admit(5, 0, 6), Err(RangeRefusal::OutOfBounds));
        assert_eq!(admit(5, 6, 6), Err(RangeRefusal::OutOfBounds));
        assert_eq!(admit(0, 0, 1), Err(RangeRefusal::OutOfBounds));
        assert_eq!(admit(usize::MAX, 1, 0), Err(RangeRefusal::OutOfBounds));
        assert_eq!(admit(usize::MAX, usize::MAX, usize::MAX), Ok(()));
    }

    #[test]
    fn boundary_is_the_end_or_a_non_continuation_byte() {
        // 'é' is encoded 0xC3 0xA9.
        let text = "aé";
        assert!(char_boundary(text.as_bytes(), 0));
        assert!(char_boundary(text.as_bytes(), 1));
        assert!(!char_boundary(text.as_bytes(), 2));
        assert!(char_boundary(text.as_bytes(), 3));
        assert!(!char_boundary(text.as_bytes(), 4));
        assert!(!char_boundary(text.as_bytes(), usize::MAX));
        assert!(char_boundary(&[], 0));
        assert!(!char_boundary(&[], 1));
        assert!(is_continuation(0x80));
        assert!(is_continuation(0xBF));
        assert!(!is_continuation(0x7F));
        assert!(!is_continuation(0xC0));
    }

    #[test]
    fn text_profile_requires_both_ends_on_boundaries() {
        let text = "aé";
        assert_eq!(admit_text(text.as_bytes(), 0, 3), Ok(()));
        assert_eq!(admit_text(text.as_bytes(), 0, 1), Ok(()));
        assert_eq!(admit_text(text.as_bytes(), 1, 3), Ok(()));
        assert_eq!(admit_text(text.as_bytes(), 3, 3), Ok(()));
        assert_eq!(admit_text(&[], 0, 0), Ok(()));
        assert_eq!(
            admit_text(text.as_bytes(), 1, 2),
            Err(RangeRefusal::NotCharBoundary)
        );
        assert_eq!(
            admit_text(text.as_bytes(), 2, 3),
            Err(RangeRefusal::NotCharBoundary)
        );
        assert_eq!(
            admit_text(text.as_bytes(), 2, 2),
            Err(RangeRefusal::NotCharBoundary)
        );
        assert_eq!(
            admit_text(text.as_bytes(), 0, 4),
            Err(RangeRefusal::OutOfBounds)
        );
        assert_eq!(
            admit_text(text.as_bytes(), 3, 1),
            Err(RangeRefusal::OutOfBounds)
        );
    }
}
