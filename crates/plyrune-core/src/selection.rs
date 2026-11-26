//! Complete literal/v1 references into an explicitly admitted, immutable fixture.
use crate::fixture::{
    BoundFixture, ByteOffset, Fixture, SnapshotDigest, TextCapture, ViewRefusal, valid_path,
};
use plyrune_kernel::{literal, range::admit_text};

const PREFIX_LIMIT: usize = 255;
const NEEDLE_LIMIT: usize = 8 * 1024 * 1024;
const RESOURCE_MATCH_LIMIT: usize = 4096;
const TOTAL_MATCH_LIMIT: usize = 8192;
const COMPARISON_LIMIT: usize = 64 * 1024 * 1024;

/// No partial selection or incomplete empty result accompanies a refusal.
#[derive(Debug, Eq, PartialEq)]
pub enum SelectionRefusal {
    /// The directory prefix exceeds 255 bytes including its final slash.
    PrefixLimit,
    /// The literal exceeds the 8 MiB query budget.
    NeedleLimit,
    /// The prefix is not a fixture path followed by one slash.
    InvalidPrefix,
    /// An empty literal cannot advance a greedy search.
    EmptyNeedle,
    /// The conservative bound on byte comparisons exceeds 64 Mi across the scope.
    ComparisonLimit,
    /// More than 4096 occurrences in one resource.
    ResourceMatchLimit,
    /// More than 8192 occurrences across the scope.
    TotalMatchLimit,
    /// Returned resource ordinals differ from the authoritative scope.
    WrongInventory,
    /// Cannot materialize a scanner coordinate on its source image.
    Representation,
    /// The caller supplied another fixture instance, even if its bytes are equal.
    WrongFixture,
    /// No retained match at this path and zero-based index.
    UnknownMatch,
    /// The source resource or capture failed exact admission.
    Source(ViewRefusal),
}

/// An admitted half-open byte range in its containing row, not a portable handle.
pub struct ReferenceRange {
    start: ByteOffset,
    end: ByteOffset,
}

impl ReferenceRange {
    /// Inclusive byte offset.
    pub fn start(&self) -> usize {
        self.start.get()
    }

    /// Exclusive byte offset.
    pub fn end(&self) -> usize {
        self.end.get()
    }
}

/// One in-scope source row, including when the search found no occurrences.
pub struct SelectedResource<'a> {
    ordinal: usize,
    path: &'a str,
    ranges: Vec<ReferenceRange>,
}

impl SelectedResource<'_> {
    /// Canonical admitted path, borrowed from the immutable fixture.
    pub fn path(&self) -> &str {
        self.path
    }

    /// Greedy, left-to-right, nonoverlapping ranges in the original bytes.
    pub fn ranges(&self) -> &[ReferenceRange] {
        &self.ranges
    }
}

/// Complete evidence for one exact prefix and needle on one live fixture borrow.
/// There is no constructor from caller-supplied rows, coordinates or snapshot IDs.
pub struct Selection<'f, 'src> {
    fixture: &'f Fixture<'src>,
    prefix: &'f str,
    needle: &'f str,
    rows: Vec<SelectedResource<'f>>,
    match_count: usize,
}

impl<'f, 'src> Selection<'f, 'src> {
    /// The entire source inventory, not just positive matches or the searched subset.
    pub fn fixture(&self) -> &'f Fixture<'src> {
        self.fixture
    }

    /// Content identity of the entire admitted manifest, including negative dependencies.
    pub fn snapshot(&self) -> SnapshotDigest {
        self.fixture.digest()
    }

    /// Exact directory prefix of the search.
    pub fn prefix(&self) -> &str {
        self.prefix
    }

    /// Exact nonempty UTF-8 literal; no Unicode normalization or case folding.
    pub fn needle(&self) -> &str {
        self.needle
    }

    /// Complete searched inventory in raw path order, retaining zero-hit rows.
    pub fn rows(&self) -> &[SelectedResource<'f>] {
        &self.rows
    }

    /// Number of retained occurrences across all rows.
    pub fn match_count(&self) -> usize {
        self.match_count
    }

    /// Mint a store-local capture from a retained occurrence. Resolve it with
    /// `bound.open(path)?.resolve(&capture)`, independently selecting the target path.
    ///
    /// # Errors
    /// Refuses an unrelated store's fixture, missing occurrence, or failed capture admission.
    pub fn capture(
        &self,
        bound: &BoundFixture<'_>,
        path: &str,
        match_index: usize,
    ) -> Result<TextCapture, SelectionRefusal> {
        if !std::ptr::eq(self.fixture, bound.fixture()) {
            return Err(SelectionRefusal::WrongFixture);
        }
        // Err carries an insertion position, not a matching resource.
        let Ok(index) = self.rows.binary_search_by(|row| row.path.cmp(path)) else {
            return Err(SelectionRefusal::UnknownMatch);
        };
        let range = self
            .rows
            .get(index)
            .and_then(|row| row.ranges.get(match_index))
            .ok_or(SelectionRefusal::UnknownMatch)?;
        bound
            .open(path)
            .map_err(SelectionRefusal::Source)?
            .capture(range.start, range.end)
            .map_err(|error| SelectionRefusal::Source(ViewRefusal::Admission(error)))
    }
}

// Search calls operate on successively shorter suffixes. No candidate start is
// examined twice, even when the previous hit ended at a UTF-8 multibyte boundary.
fn comparison_bound(text_len: usize, needle_len: usize) -> Result<usize, SelectionRefusal> {
    let Some(last_start) = text_len.checked_sub(needle_len) else {
        return Ok(0);
    };
    last_start
        .checked_add(1)
        .and_then(|windows| windows.checked_mul(needle_len))
        .ok_or(SelectionRefusal::ComparisonLimit)
}

fn scan_resource(
    text: &str,
    needle: &str,
    total: &mut usize,
) -> Result<Vec<ReferenceRange>, SelectionRefusal> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    let mut remaining = text.as_bytes();
    while let Some(relative) = literal::first_match(remaining, needle.as_bytes()) {
        if ranges.len() >= RESOURCE_MATCH_LIMIT {
            return Err(SelectionRefusal::ResourceMatchLimit);
        }
        if *total >= TOTAL_MATCH_LIMIT {
            return Err(SelectionRefusal::TotalMatchLimit);
        }
        let start = usize::checked_add(cursor, relative).ok_or(SelectionRefusal::Representation)?;
        let end = start
            .checked_add(needle.len())
            .ok_or(SelectionRefusal::Representation)?;
        admit_text(text.as_bytes(), start, end)
            .map_err(|error| SelectionRefusal::Source(ViewRefusal::Admission(error)))?;
        remaining = text
            .as_bytes()
            .get(end..)
            .ok_or(SelectionRefusal::Representation)?;
        cursor = end;
        *total = total
            .checked_add(1)
            .ok_or(SelectionRefusal::TotalMatchLimit)?;
        ranges.push(ReferenceRange {
            start: ByteOffset::new(start),
            end: ByteOffset::new(end),
        });
    }
    Ok(ranges)
}

/// Enumerate complete exact references under a fixture-v1 directory prefix.
/// The admitted fixture supplies both the inventory and every byte searched.
/// Budgets cover the entire query, not a displayed page or batch of edits.
///
/// # Errors
/// Refuses malformed or oversized queries, exceeded work or match budgets,
/// unrepresentable ranges, or a missing, extra or reordered inventory witness.
pub fn refs<'f, 'src>(
    fixture: &'f Fixture<'src>,
    prefix: &'f str,
    needle: &'f str,
) -> Result<Selection<'f, 'src>, SelectionRefusal> {
    if prefix.len() > PREFIX_LIMIT {
        return Err(SelectionRefusal::PrefixLimit);
    }
    if needle.len() > NEEDLE_LIMIT {
        return Err(SelectionRefusal::NeedleLimit);
    }
    if !prefix.strip_suffix('/').is_some_and(valid_path) {
        return Err(SelectionRefusal::InvalidPrefix);
    }
    if needle.is_empty() {
        return Err(SelectionRefusal::EmptyNeedle);
    }
    // Establish the expected domain from the manifest, never from emitted rows.
    let expected: Vec<_> = fixture
        .resources()
        .enumerate()
        .filter_map(|(ordinal, (path, _))| path.starts_with(prefix).then_some(ordinal))
        .collect();
    let mut rows = Vec::with_capacity(expected.len());
    let mut work = COMPARISON_LIMIT;
    let mut match_count = 0;
    for (ordinal, (path, text)) in fixture.resources().enumerate() {
        if path.starts_with(prefix) {
            work = work
                .checked_sub(comparison_bound(text.len(), needle.len())?)
                .ok_or(SelectionRefusal::ComparisonLimit)?;
            rows.push(SelectedResource {
                ordinal,
                path,
                ranges: scan_resource(text, needle, &mut match_count)?,
            });
        }
    }
    let returned: Vec<_> = rows.iter().map(|row| row.ordinal).collect();
    if !literal::inventory_matches(&expected, &returned) {
        return Err(SelectionRefusal::WrongInventory);
    }
    Ok(Selection {
        fixture,
        prefix,
        needle,
        rows,
        match_count,
    })
}
