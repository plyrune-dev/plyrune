use super::{Fixture, ViewRefusal};
use plyrune_kernel::binding::StoreBinding;
use plyrune_kernel::capture::{
    ByteOffset, CaptureKind, CaptureSchema, CaptureSubject, admit_capture,
};
use plyrune_kernel::range::RangeRefusal;

/// An admitted inventory in a host-issued store lifetime. Not a permission grant.
pub struct BoundFixture<'a> {
    fixture: Fixture<'a>,
    store: StoreBinding,
}

/// One authoritative resource. Its bytes and identity come from the same lookup.
pub struct FixtureResource<'a> {
    subject: CaptureSubject,
    text: &'a str,
}

/// A retained, in-process v1 text capture. No untrusted import constructor.
pub struct TextCapture {
    subject: CaptureSubject,
    start: ByteOffset,
    end: ByteOffset,
}

impl<'a> BoundFixture<'a> {
    pub(super) fn new(fixture: Fixture<'a>, store: StoreBinding) -> Self {
        Self { fixture, store }
    }

    /// Borrow this store's admitted inventory without detaching its identity.
    pub fn fixture(&self) -> &Fixture<'a> {
        &self.fixture
    }

    /// Open an exact path. The caller chooses the intended resource independently of a capture.
    ///
    /// # Errors
    /// Refuses a missing path or unrepresentable snapshot key.
    pub fn open(&self, path: &str) -> Result<FixtureResource<'a>, ViewRefusal> {
        let row = self
            .fixture
            .resources
            .get(path)
            .ok_or(ViewRefusal::UnknownPath)?;
        Ok(FixtureResource {
            subject: CaptureSubject::new(self.store, row.ordinal, self.fixture.digest.key()?),
            text: row.text,
        })
    }
}

impl<'a> FixtureResource<'a> {
    fn validate(&self, capture: &TextCapture) -> Result<(), RangeRefusal> {
        // Kind and schema are fixed by the sealed, in-process TextCapture type.
        admit_capture(
            CaptureSchema::V1,
            CaptureKind::TEXT,
            &capture.subject,
            &self.subject,
            self.text.as_bytes(),
            capture.start,
            capture.end,
        )
    }

    /// Retain half-open UTF-8 byte coordinates, including an empty boundary range.
    ///
    /// # Errors
    /// Rejects reversed, out-of-bounds and non-character-boundary offsets.
    pub fn capture(&self, start: ByteOffset, end: ByteOffset) -> Result<TextCapture, RangeRefusal> {
        let capture = TextCapture {
            subject: self.subject,
            start,
            end,
        };
        self.validate(&capture)?;
        Ok(capture)
    }

    /// Lower the sealed capture against this independently opened target.
    pub(crate) fn lower_cut(&self, capture: &TextCapture) -> Result<(usize, usize), RangeRefusal> {
        plyrune_kernel::replacement::lower_cut(
            &capture.subject,
            &self.subject,
            self.text.as_bytes(),
            capture.start,
            capture.end,
        )
    }

    /// Resolve against this resource, not a resource named by the capture's caller.
    /// Returns the exact original slice without allocating, copying, or applying a view budget.
    ///
    /// # Errors
    /// Rejects a foreign subject before coordinates. Never substitutes an empty result on error.
    pub fn resolve(&self, capture: &TextCapture) -> Result<&'a str, ViewRefusal> {
        self.validate(capture).map_err(ViewRefusal::Admission)?;
        self.text
            .get(capture.start.get()..capture.end.get())
            .ok_or(ViewRefusal::Representation)
    }
}
