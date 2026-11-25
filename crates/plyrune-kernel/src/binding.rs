//! Store-local provenance. Comparison does not establish permission to read.

/// Identity assigned to a source owner by the host.
#[derive(Clone, Copy)]
pub struct OwnerId(u64);

/// Identity assigned to one lifetime of the owner's artifact store.
#[derive(Clone, Copy)]
pub struct IncarnationId(u64);

/// A host-supplied pair; neither component is an authorization token.
#[derive(Clone, Copy)]
pub struct StoreBinding {
    owner: OwnerId,
    incarnation: IncarnationId,
}

impl OwnerId {
    /// Every bit pattern is representable; uniqueness is the host's obligation.
    pub const fn from_host(value: u64) -> Self {
        Self(value)
    }
}

impl IncarnationId {
    /// The host must never recycle this value for the same owner.
    pub const fn from_host(value: u64) -> Self {
        Self(value)
    }
}

impl StoreBinding {
    /// Associate two identifiers without conflating their roles.
    pub const fn new(owner: OwnerId, incarnation: IncarnationId) -> Self {
        Self { owner, incarnation }
    }
}

/// Exact store comparison. Callers must still check artifact kind, snapshot
/// and subject. Used as a prerequisite, never as sufficient handle admission.
#[must_use]
pub fn same_store(expected: &StoreBinding, presented: &StoreBinding) -> bool {
    expected.owner.0 == presented.owner.0 && expected.incarnation.0 == presented.incarnation.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_store_matches_but_equal_revision_in_another_store_does_not() {
        let store = |o, i| StoreBinding::new(OwnerId::from_host(o), IncarnationId::from_host(i));
        assert!(same_store(&store(1, 0), &store(1, 0)));
        assert!(!same_store(&store(1, 0), &store(2, 0)));
        assert!(!same_store(&store(1, 0), &store(1, 1)));
    }
}
