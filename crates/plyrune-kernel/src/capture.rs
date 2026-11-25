//! Store-local text captures into an immutable fixture resource.
use crate::binding::{StoreBinding, same_store};
use crate::range::{RangeRefusal, admit_text};
use crate::snapshot::{SnapshotKey, inspect_range};

/// Check store equality before considering offsets into the supplied bytes.
///
/// # Errors
///
/// `WrongStore` takes precedence over every coordinate error. Within the
/// same store, return the exact result of text-range admission.
pub fn admit_store_range(
    expected: &StoreBinding,
    presented: &StoreBinding,
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Result<(), RangeRefusal> {
    if same_store(expected, presented) {
        admit_text(bytes, start, end)
    } else {
        Err(RangeRefusal::WrongStore)
    }
}

/// Untrusted capture schema discriminant. Only v1 is supported.
#[derive(Clone, Copy)]
pub struct CaptureSchema(u16);

impl CaptureSchema {
    /// Schema for the fixture text-capture decision (not a wire version).
    pub const V1: Self = Self(1);

    /// Preserve unknown values so admission can refuse them.
    pub const fn from_raw(value: u16) -> Self {
        Self(value)
    }
}

/// Untrusted artifact kind discriminant; not a proof of the artifact's kind.
#[derive(Clone, Copy)]
pub struct CaptureKind(u16);

impl CaptureKind {
    /// A half-open text capture into a regular UTF-8 resource.
    pub const TEXT: Self = Self(1);

    /// Preserve unknown values so admission can refuse them.
    pub const fn from_raw(value: u16) -> Self {
        Self(value)
    }
}

/// One path's ordinal in the immutable fixture's sorted inventory.
#[derive(Clone, Copy)]
pub struct ResourceId(u64);

impl ResourceId {
    /// Only fixture admission establishes the association with a path.
    pub const fn from_ordinal(ordinal: u64) -> Self {
        Self(ordinal)
    }
}

/// A byte position, not a character index or a previously checked boundary.
#[derive(Clone, Copy)]
pub struct ByteOffset(usize);

impl ByteOffset {
    /// Keep a position for later admission against the actual resource image.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Position used for slicing after successful admission.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// The complete identity of a fixture resource. Neither a key nor equality
/// grants read authority.
#[derive(Clone, Copy)]
pub struct CaptureSubject {
    store: StoreBinding,
    resource: ResourceId,
    snapshot: SnapshotKey,
}

impl CaptureSubject {
    /// Group identity components without asserting that they have been issued.
    pub const fn new(store: StoreBinding, resource: ResourceId, snapshot: SnapshotKey) -> Self {
        Self {
            store,
            resource,
            snapshot,
        }
    }
}

/// Decide whether a presented v1 text capture resolves against `actual`.
/// The resolver must obtain `actual` and `bytes` from the same admitted
/// fixture entry, not from the capture. Equal bytes in two entries do not
/// imply equal subjects.
///
/// # Errors
///
/// Refuse unsupported schema, wrong kind, store, resource, or snapshot
/// (in that order), then reversed/out-of-bounds or nonboundary offsets.
pub fn admit_capture(
    schema: CaptureSchema,
    kind: CaptureKind,
    presented: &CaptureSubject,
    actual: &CaptureSubject,
    bytes: &[u8],
    start: ByteOffset,
    end: ByteOffset,
) -> Result<(), RangeRefusal> {
    if schema.0 != CaptureSchema::V1.0 {
        return Err(RangeRefusal::UnsupportedSchema);
    }
    if kind.0 != CaptureKind::TEXT.0 {
        return Err(RangeRefusal::WrongKind);
    }
    if !same_store(&presented.store, &actual.store) {
        return Err(RangeRefusal::WrongStore);
    }
    if presented.resource.0 != actual.resource.0 {
        return Err(RangeRefusal::WrongResource);
    }
    inspect_range(&presented.snapshot, &actual.snapshot, bytes, start.0, end.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{IncarnationId, OwnerId};

    #[test]
    fn wrong_store_wins_even_if_the_range_is_invalid() {
        let store = |i| StoreBinding::new(OwnerId::from_host(1), IncarnationId::from_host(i));
        let bytes = "é".as_bytes();
        assert_eq!(admit_store_range(&store(0), &store(0), bytes, 0, 2), Ok(()));
        assert_eq!(
            admit_store_range(&store(0), &store(1), bytes, 5, 3),
            Err(RangeRefusal::WrongStore)
        );
    }

    #[test]
    fn only_v1_text_accepts_matching_subjects() {
        let store = StoreBinding::new(OwnerId::from_host(4), IncarnationId::from_host(5));
        let resource = ResourceId::from_ordinal(12);
        let image = CaptureSubject::new(store, resource, SnapshotKey::from_halves(8, 9));
        let check = |schema, kind| {
            admit_capture(
                CaptureSchema::from_raw(schema),
                CaptureKind::from_raw(kind),
                &image,
                &image,
                "aé".as_bytes(),
                ByteOffset::new(1),
                ByteOffset::new(3),
            )
        };
        assert_eq!(check(1, 1), Ok(()));
        assert_eq!(check(2, 1), Err(RangeRefusal::UnsupportedSchema));
        assert_eq!(check(1, 2), Err(RangeRefusal::WrongKind));
        assert_eq!(check(2, 2), Err(RangeRefusal::UnsupportedSchema));
    }

    #[test]
    fn identical_bytes_do_not_bind_different_subjects() {
        let subject = |o, i, r, high, low| {
            CaptureSubject::new(
                StoreBinding::new(OwnerId::from_host(o), IncarnationId::from_host(i)),
                ResourceId::from_ordinal(r),
                SnapshotKey::from_halves(high, low),
            )
        };
        let actual = subject(4, 5, 6, 7, 8);
        let check = |presented: &CaptureSubject, lo, hi| {
            admit_capture(
                CaptureSchema::V1,
                CaptureKind::TEXT,
                presented,
                &actual,
                "aé".as_bytes(),
                ByteOffset::new(lo),
                ByteOffset::new(hi),
            )
        };
        for (other, refusal) in [
            (subject(0, 5, 6, 7, 8), RangeRefusal::WrongStore),
            (subject(4, 0, 6, 7, 8), RangeRefusal::WrongStore),
            (subject(4, 5, 0, 7, 8), RangeRefusal::WrongResource),
            (subject(4, 5, 6, 0, 8), RangeRefusal::WrongSnapshot),
            (subject(4, 5, 6, 7, 0), RangeRefusal::WrongSnapshot),
        ] {
            assert_eq!(check(&other, 1, 3), Err(refusal));
        }
        assert_eq!(check(&actual, 1, 3), Ok(()));
        assert_eq!(check(&actual, 3, 3), Ok(()));
        assert_eq!(check(&actual, 2, 3), Err(RangeRefusal::NotCharBoundary));
        assert_eq!(check(&actual, 2, 4), Err(RangeRefusal::OutOfBounds));
    }
}
