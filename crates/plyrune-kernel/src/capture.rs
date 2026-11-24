//! Store check before text-coordinate admission. This leaf neither resolves
//! a handle nor establishes that the supplied bytes belong to a snapshot.
use crate::binding::{StoreBinding, same_store};
use crate::range::{RangeRefusal, admit_text};

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
}
