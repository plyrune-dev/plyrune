//! Prepare literal replacements without effects, then check the entire raw candidate.
//! A content review identifies the caller-stated task, not its owner or authorization.
use crate::fixture::{BoundFixture, Entry, Fixture, RangeRefusal, SnapshotDigest, ViewRefusal};
use crate::selection::{self, SelectedResource, Selection, SelectionRefusal};
use plyrune_kernel::replacement::{consume, finished, same_bytes};
use sha2::{Digest, Sha256};
use splicery_splice::{Cut, Edit, MAX_CANDIDATE_BYTES, Piece};
use std::borrow::Cow;

/// Interpretation bound into each content review; not a workflow or approval profile.
pub const CHECKER_PROFILE: &str = "plyrune/check/fixture-literal-exact/v1";
const TOTAL_OUTPUT: usize = 32 * 1024 * 1024;

/// No refusal carries a partial candidate.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Invalid query or incomplete selection.
    Selection(SelectionRefusal),
    /// Replacement, projected per-resource size or aggregate output exceeded a limit.
    Budget,
    /// Failed to resolve an authoritative capture subject.
    Source(ViewRefusal),
    /// Lowering rejected the capture or its coordinates.
    Cut(RangeRefusal),
    /// The pinned splice implementation refused the batch.
    Splice(splicery_splice::Refusal),
    /// A coordinate, digest frame or encoding could not be represented.
    Representation,
    /// Missing, extra, reordered, renamed or nonregular candidate resource.
    Inventory,
    /// Actual bytes or selected ranges disagreed with the original source and task.
    Mismatch,
}

/// A matched occurrence need not cause a byte change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// At least one resource differs from the original.
    Changed,
    /// Every resource is byte-identical to the original.
    NoChange,
}

struct Checked {
    requirement: [u8; 32],
    candidate: [u8; 32],
    review: [u8; 32],
    changed: usize,
}

/// Immutable checked rows in raw path order. Every row has fixture mode `regular`.
/// Unchanged rows in a prepared candidate borrow their original bytes.
pub struct ReviewedCandidate<'a> {
    selection: Selection<'a, 'a>,
    rows: Vec<(&'a str, Cow<'a, str>)>,
    checked: Checked,
}

impl<'a> ReviewedCandidate<'a> {
    /// All paths, including out-of-scope and zero-match resources.
    pub fn rows(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.rows.iter().map(|(path, text)| (*path, text.as_ref()))
    }

    /// Complete retained discovery result, not a truncated display of changed rows.
    pub fn selection(&self) -> &Selection<'a, 'a> {
        &self.selection
    }

    /// Digest of the authoritative base inventory.
    pub fn base_digest(&self) -> SnapshotDigest {
        self.selection.snapshot()
    }

    /// Frozen literal/v1 task identity, independent of snapshot and matches.
    pub fn requirement_digest(&self) -> &[u8; 32] {
        &self.checked.requirement
    }

    /// Digest of all actual candidate paths, modes and bytes in canonical order.
    pub fn candidate_digest(&self) -> &[u8; 32] {
        &self.checked.candidate
    }

    /// Content-only identity. A filesystem review must also bind origin and metadata.
    pub fn review_id(&self) -> &[u8; 32] {
        &self.checked.review
    }

    /// Number of nonoverlapping occurrences, including identical replacements.
    pub fn matched(&self) -> usize {
        self.selection.match_count()
    }

    /// Number of resources whose actual bytes changed.
    pub fn changed(&self) -> usize {
        self.checked.changed
    }

    /// No-change is not a completed modification.
    pub fn disposition(&self) -> Disposition {
        if self.changed() == 0 {
            Disposition::NoChange
        } else {
            Disposition::Changed
        }
    }
}

/// Compare the independently expected and recomputed review identities, in digest order.
#[must_use]
pub fn same_review(expected: &[u8; 32], presented: &[u8; 32]) -> bool {
    same_bytes(expected, presented)
}

#[derive(Clone, Copy)]
struct Task<'a> {
    prefix: &'a str,
    needle: &'a str,
    replacement: &'a str,
}

fn discover<'a>(
    fixture: &'a Fixture<'_>,
    prefix: &'a str,
    needle: &'a str,
    replacement: &str,
) -> Result<Selection<'a, 'a>, Refusal> {
    if replacement.len() > MAX_CANDIDATE_BYTES {
        return Err(Refusal::Budget);
    }
    selection::refs(fixture, prefix, needle).map_err(Refusal::Selection)
}

// One witness in source order for each scoped path, not merely each edited path.
fn witness<'r, 'a>(
    selected: &mut std::slice::Iter<'r, SelectedResource<'a>>,
    path: &str,
    prefix: &str,
) -> Result<Option<&'r SelectedResource<'a>>, Refusal> {
    if !path.starts_with(prefix) {
        return Ok(None);
    }
    let row = selected.next().ok_or(Refusal::Inventory)?;
    if !same_bytes(row.path().as_bytes(), path.as_bytes()) {
        return Err(Refusal::Inventory);
    }
    Ok(Some(row))
}

fn add_output(total: usize, len: usize) -> Result<usize, Refusal> {
    if len > MAX_CANDIDATE_BYTES {
        return Err(Refusal::Budget);
    }
    total
        .checked_add(len)
        .filter(|&sum| sum <= TOTAL_OUTPUT)
        .ok_or(Refusal::Budget)
}

// This pass precedes *any* allocation of splice literals or rendered bytes.
// Each literal piece is owned by upstream. Their combined size per batch is
// bounded by its projected output (untouched bytes have nonnegative size).
fn project(selection: &Selection<'_, '_>, replacement: &str) -> Result<(), Refusal> {
    let mut selected = selection.rows().iter();
    let mut total = 0;
    for (path, text) in selection.fixture().resources() {
        let hits =
            witness(&mut selected, path, selection.prefix())?.map_or(0, |r| r.ranges().len());
        let removed = hits
            .checked_mul(selection.needle().len())
            .ok_or(Refusal::Budget)?;
        let added = hits.checked_mul(replacement.len()).ok_or(Refusal::Budget)?;
        let output = text
            .len()
            .checked_sub(removed)
            .and_then(|gap| gap.checked_add(added));
        total = add_output(total, output.ok_or(Refusal::Budget)?)?;
    }
    if selected.next().is_some() {
        return Err(Refusal::Inventory);
    }
    Ok(())
}

fn render(
    bound: &BoundFixture<'_>,
    selection: &Selection<'_, '_>,
    path: &str,
    text: &str,
    row: &SelectedResource<'_>,
    replacement: &str,
) -> Result<String, Refusal> {
    let source = text.as_bytes();
    let target = bound.open(path).map_err(Refusal::Source)?;
    // Splicery's token identifies the original byte image. It does *not*
    // distinguish identical files: the independent target below must admit
    // the path/store/snapshot subject before these numbers can become a cut.
    let image: [u8; 32] = Sha256::digest(source).into();
    let mut edits = Vec::with_capacity(row.ranges().len());
    for index in 0..row.ranges().len() {
        let capture = selection
            .capture(bound, row.path(), index)
            .map_err(Refusal::Selection)?;
        let (lo, hi) = target.lower_cut(&capture).map_err(Refusal::Cut)?;
        let original = source.get(lo..hi).ok_or(Refusal::Representation)?;
        if !same_bytes(original, selection.needle().as_bytes()) {
            return Err(Refusal::Mismatch);
        }
        let parts = if replacement.is_empty() {
            Vec::new()
        } else {
            vec![Piece::Literal(replacement.as_bytes().to_vec())]
        };
        edits.push(Edit {
            target: Cut {
                capture: image,
                lo,
                hi,
            },
            parts,
        });
    }
    let order = splicery_splice::prepare(source, true, &image, &edits).map_err(Refusal::Splice)?;
    let output = splicery_splice::render(source, &edits, &order).map_err(Refusal::Splice)?;
    String::from_utf8(output.bytes).map_err(|_| Refusal::Representation)
}

fn frame(hash: &mut Sha256, bytes: &[u8]) -> Result<(), Refusal> {
    let length = u64::try_from(bytes.len()).map_err(|_| Refusal::Representation)?;
    hash.update(length.to_be_bytes());
    hash.update(bytes);
    Ok(())
}

fn framed_hash(parts: &[&[u8]]) -> Result<[u8; 32], Refusal> {
    let mut hash = Sha256::new();
    for part in parts {
        frame(&mut hash, part)?;
    }
    Ok(hash.finalize().into())
}

// No splice operations, rendered replacements, or expected candidate buffer
// enter this verifier. Re-enumerate the task from the admitted source, compare
// both coordinates and original/replacement chunks, then require exhaustion.
fn check_resource(
    source: &str,
    actual: &[u8],
    row: Option<&SelectedResource<'_>>,
    task: Task<'_>,
) -> Result<(), Refusal> {
    let mut cursor = 0;
    let mut src = 0;
    if let Some(row) = row {
        let mut ranges = row.ranges().iter();
        for (start, literal) in source.match_indices(task.needle) {
            let end = start
                .checked_add(literal.len())
                .ok_or(Refusal::Representation)?;
            let range = ranges.next().ok_or(Refusal::Mismatch)?;
            if range.start() != start || range.end() != end {
                return Err(Refusal::Mismatch);
            }
            let gap = source
                .as_bytes()
                .get(src..start)
                .ok_or(Refusal::Representation)?;
            cursor = consume(actual, cursor, gap).ok_or(Refusal::Mismatch)?;
            cursor =
                consume(actual, cursor, task.replacement.as_bytes()).ok_or(Refusal::Mismatch)?;
            src = end;
        }
        if ranges.next().is_some() {
            return Err(Refusal::Mismatch);
        }
    }
    let suffix = source
        .as_bytes()
        .get(src..)
        .ok_or(Refusal::Representation)?;
    cursor = consume(actual, cursor, suffix).ok_or(Refusal::Mismatch)?;
    if !finished(cursor, actual.len()) {
        return Err(Refusal::Mismatch);
    }
    Ok(())
}

fn check_rows<'b>(
    fixture: &Fixture<'_>,
    task: Task<'_>,
    selection: &Selection<'_, '_>,
    mut actual: impl ExactSizeIterator<Item = (&'b str, &'b str, &'b [u8])>,
) -> Result<Checked, Refusal> {
    let base = fixture.resources();
    if !finished(base.len(), actual.len()) {
        return Err(Refusal::Inventory);
    }
    let mut candidate = Sha256::new();
    frame(&mut candidate, b"plyrune/candidate/fixture/v1")?;
    candidate.update(
        u64::try_from(base.len())
            .map_err(|_| Refusal::Representation)?
            .to_be_bytes(),
    );
    let mut selected = selection.rows().iter();
    let mut total = 0;
    let mut changed = 0usize;
    for (path, source) in base {
        let (name, mode, bytes) = actual.next().ok_or(Refusal::Inventory)?;
        if !same_bytes(name.as_bytes(), path.as_bytes()) || mode != "regular" {
            return Err(Refusal::Inventory);
        }
        total = add_output(total, bytes.len())?;
        let row = witness(&mut selected, path, task.prefix)?;
        check_resource(source, bytes, row, task)?;
        if bytes != source.as_bytes() {
            changed = changed.checked_add(1).ok_or(Refusal::Representation)?;
        }
        frame(&mut candidate, name.as_bytes())?;
        frame(&mut candidate, mode.as_bytes())?;
        frame(&mut candidate, bytes)?;
    }
    if selected.next().is_some() || actual.next().is_some() {
        return Err(Refusal::Inventory);
    }
    let candidate: [u8; 32] = candidate.finalize().into();
    let requirement = framed_hash(&[
        b"plyrune/requirement/literal/v1",
        task.prefix.as_bytes(),
        task.needle.as_bytes(),
        task.replacement.as_bytes(),
    ])?;
    let review = framed_hash(&[
        b"plyrune/review/fixture-literal/v1",
        fixture.digest().bytes(),
        &requirement,
        &candidate,
        CHECKER_PROFILE.as_bytes(),
    ])?;
    Ok(Checked {
        requirement,
        candidate,
        review,
        changed,
    })
}

/// Prepare every scoped edit against its original resource; independently check
/// the resulting whole manifest against the caller's task before returning it.
///
/// # Errors
/// Refuses invalid queries, work/output limits, bad cuts, splice failures or
/// disagreement between the raw product and the required complete candidate.
pub fn preview<'a>(
    bound: &'a BoundFixture<'_>,
    prefix: &'a str,
    needle: &'a str,
    replacement: &str,
) -> Result<ReviewedCandidate<'a>, Refusal> {
    let fixture = bound.fixture();
    let selection = discover(fixture, prefix, needle, replacement)?;
    project(&selection, replacement)?;
    let mut selected = selection.rows().iter();
    let mut rows = Vec::with_capacity(fixture.resources().len());
    for (path, text) in fixture.resources() {
        let row = witness(&mut selected, path, prefix)?;
        let output = match row {
            Some(row) if !row.ranges().is_empty() && needle != replacement => {
                Cow::Owned(render(bound, &selection, path, text, row, replacement)?)
            }
            Some(_) | None => Cow::Borrowed(text),
        };
        rows.push((path, output));
    }
    let task = Task {
        prefix,
        needle,
        replacement,
    };
    let checked = check_rows(
        fixture,
        task,
        &selection,
        rows.iter()
            .map(|(p, text)| (*p, "regular", text.as_bytes())),
    )?;
    Ok(ReviewedCandidate {
        selection,
        rows,
        checked,
    })
}

/// Check externally supplied raw rows in canonical path order without preparing
/// an expected output buffer or trusting an author's scope, hit list or digest.
/// Base and task are supplied independently of `candidate`. Success does not
/// authenticate their origin, assert human approval, or authorize publication.
///
/// # Errors
/// Refuses invalid queries, budget excesses and any discrepancy in the complete
/// path/mode/byte sequence, including unchanged and out-of-scope resources.
pub fn check_candidate<'a>(
    fixture: &'a Fixture<'_>,
    prefix: &'a str,
    needle: &'a str,
    replacement: &str,
    candidate: &[Entry<'a>],
) -> Result<ReviewedCandidate<'a>, Refusal> {
    let selection = discover(fixture, prefix, needle, replacement)?;
    project(&selection, replacement)?;
    let task = Task {
        prefix,
        needle,
        replacement,
    };
    let checked = check_rows(
        fixture,
        task,
        &selection,
        candidate
            .iter()
            .map(|entry| (entry.path, entry.mode, entry.bytes)),
    )?;
    let rows = candidate
        .iter()
        .map(|entry| {
            let text = std::str::from_utf8(entry.bytes).map_err(|_| Refusal::Representation)?;
            Ok((entry.path, Cow::Borrowed(text)))
        })
        .collect::<Result<_, _>>()?;
    Ok(ReviewedCandidate {
        selection,
        rows,
        checked,
    })
}
