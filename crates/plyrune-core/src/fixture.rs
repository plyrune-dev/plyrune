//! Admission and canonical hashing for `docs/FIXTURE-PROFILE.md`.
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Finite input supplied by the host. Neither mode nor encoding is trusted.
pub struct Entry<'a> {
    /// Repository-relative path.
    pub path: &'a str,
    /// The supported mode is `regular`.
    pub mode: &'a str,
    /// Exact bytes, including line endings.
    pub bytes: &'a [u8],
}

/// Snapshot content identity, independent of store-local ownership.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SnapshotDigest([u8; 32]);

impl SnapshotDigest {
    /// SHA-256 bytes in digest order.
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// No successful partial snapshot is returned for an invalid inventory.
#[derive(Debug, Eq, PartialEq)]
pub enum Refusal {
    /// Resource count, individual byte size, or aggregate size exceeds the profile.
    Budget,
    /// Path is not in the fixture path grammar.
    InvalidPath,
    /// Two entries have the same path.
    Duplicate,
    /// Non-regular fixture entry.
    UnsupportedMode,
    /// Content is not well-formed UTF-8.
    Encoding(std::str::Utf8Error),
    /// A canonical frame length was not representable in 64 bits.
    Length(std::num::TryFromIntError),
}

/// Validated immutable inventory; the immutable source borrow pins its bytes.
pub struct Fixture<'a> {
    resources: BTreeMap<&'a str, &'a str>,
    digest: SnapshotDigest,
}

fn frame(hash: &mut Sha256, bytes: &[u8]) -> Result<(), Refusal> {
    let n = u64::try_from(bytes.len()).map_err(Refusal::Length)?;
    hash.update(n.to_be_bytes());
    hash.update(bytes);
    Ok(())
}

impl<'a> Fixture<'a> {
    /// Admit an entire inventory before exposing it or hashing its contents.
    ///
    /// # Errors
    /// Returns [`Refusal`] for a profile or budget violation.
    pub fn admit(entries: &[Entry<'a>]) -> Result<Self, Refusal> {
        if entries.len() > 256 {
            return Err(Refusal::Budget);
        }
        let mut resources = BTreeMap::new();
        let mut total = 0usize;
        for entry in entries {
            if entry.path.len() > 255
                || !entry.path.split('/').all(|part| {
                    !matches!(part, "" | "." | "..")
                        && part
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
                })
            {
                return Err(Refusal::InvalidPath);
            }
            if entry.mode != "regular" {
                return Err(Refusal::UnsupportedMode);
            }
            total = total
                .checked_add(entry.bytes.len())
                .ok_or(Refusal::Budget)?;
            if entry.bytes.len() > 8_388_608 || total > 33_554_432 {
                return Err(Refusal::Budget);
            }
            let text = std::str::from_utf8(entry.bytes).map_err(Refusal::Encoding)?;
            if resources.insert(entry.path, text).is_some() {
                return Err(Refusal::Duplicate);
            }
        }
        let mut hash = Sha256::new();
        frame(&mut hash, b"plyrune/fixture/v1")?;
        hash.update(
            u64::try_from(resources.len())
                .map_err(Refusal::Length)?
                .to_be_bytes(),
        );
        for (path, content) in &resources {
            frame(&mut hash, path.as_bytes())?;
            frame(&mut hash, b"regular")?;
            frame(&mut hash, content.as_bytes())?;
        }
        Ok(Self {
            digest: SnapshotDigest(hash.finalize().into()),
            resources,
        })
    }

    /// Framed digest, independent of input enumeration order.
    pub fn digest(&self) -> SnapshotDigest {
        self.digest
    }

    /// All admitted paths and bytes, including resources containing no matches.
    pub fn resources(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.resources.iter().map(|(&p, &s)| (p, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry<'a>(path: &'a str, bytes: &'a [u8]) -> Entry<'a> {
        Entry {
            path,
            mode: "regular",
            bytes,
        }
    }

    #[test]
    fn framing_matches_independent_unicode_crlf_vector() {
        let f = Fixture::admit(&[entry("a", "é\r\n".as_bytes())]).unwrap();
        let got: String = f.digest().0.iter().map(|x| format!("{x:02x}")).collect();
        assert_eq!(
            got,
            "94a625e0b3067975d61377728cb97d36be9f8564856025ecf2c5a2bbbf4dbfa7"
        );
        assert_eq!(f.resources().collect::<Vec<_>>(), [("a", "é\r\n")]);
    }

    #[test]
    fn malformed_entries_never_become_a_partial_inventory() {
        for p in ["", "./a", "../x", "a/../x", "/a", "a//b", "a\\b"] {
            assert_eq!(
                Fixture::admit(&[entry(p, b"")]).err(),
                Some(Refusal::InvalidPath)
            );
        }
        assert_eq!(
            Fixture::admit(&[entry("a", b""), entry("a", b"")]).err(),
            Some(Refusal::Duplicate)
        );
        assert!(matches!(
            Fixture::admit(&[entry("a", &[0xff])]),
            Err(Refusal::Encoding(e)) if e.valid_up_to() == 0
        ));
    }
}
