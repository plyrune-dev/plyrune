import Extracted.PlyruneKernel

open Aeneas Aeneas.Std Result plyrune_kernel

namespace Proofs.Range

/-- Both successful admission and the exact refusal, including empty ranges. -/
theorem admit_spec (len lo hi : Std.Usize) :
    range.admit len lo hi ⦃ r => r = if lo ≤ hi ∧ hi ≤ len then
      core.result.Result.Ok () else core.result.Result.Err range.RangeRefusal.OutOfBounds ⦄ := by
  by_cases a : lo > hi <;> by_cases b : hi > len <;>
    simp [range.admit, a, b, WP.spec_ok] <;> scalar_tac

end Proofs.Range

namespace Proofs.Binding

theorem same_store_spec (a b : binding.StoreBinding) :
    binding.same_store a b ⦃ r => (r = true ↔
      a.owner = b.owner ∧ a.incarnation = b.incarnation) ⦄ := by
  by_cases h : a.owner = b.owner <;> simp [binding.same_store, h, WP.spec_ok]

end Proofs.Binding

namespace Proofs.Text

/-- The end is a boundary, a point beyond it is not. Inside the image only
the UTF-8 continuation-byte interval is excluded. Image admission must check
well-formed UTF-8 before interpreting this as a character boundary. -/
def boundary (xs : Slice Std.U8) (i : Std.Usize) : Bool :=
  match xs.val[i.val]? with
  | none => decide (i.val = xs.val.length)
  | some x => !(decide (128 ≤ x.val ∧ x.val ≤ 191))

theorem boundary_spec (xs : Slice Std.U8) (i : Std.Usize) :
    range.char_boundary xs i ⦃ r => r = boundary xs i ⦄ := by
  unfold range.char_boundary boundary
  simp only [core.slice.Slice.get, core.slice.index.Usize.get, Slice.getElem?_Usize_eq, bind_ok]
  cases h : xs.val[i.val]? with
  | none => simp [WP.spec_ok] <;> scalar_tac
  | some x =>
    by_cases hc : x ≥ 128#u8 <;> simp [range.is_continuation, hc, WP.spec_ok] <;> scalar_tac

theorem admit_text_spec (xs : Slice Std.U8) (lo hi : Std.Usize) :
    range.admit_text xs lo hi ⦃ r => r = if lo ≤ hi ∧ hi ≤ Slice.len xs then
      if boundary xs lo && boundary xs hi then core.result.Result.Ok ()
      else core.result.Result.Err range.RangeRefusal.NotCharBoundary
    else core.result.Result.Err range.RangeRefusal.OutOfBounds ⦄ := by
  unfold range.admit_text
  apply WP.spec_bind (Proofs.Range.admit_spec _ _ _)
  intro r hr
  by_cases h : lo ≤ hi ∧ hi ≤ Slice.len xs
  · simp only [h, if_true] at hr ⊢
    subst r
    apply WP.spec_bind (boundary_spec xs lo)
    intro l hl
    subst l
    cases hx : boundary xs lo
    · simp [WP.spec_ok]
    · simp only [ite_true, Bool.true_and]
      apply WP.spec_bind (boundary_spec xs hi)
      intro v hv
      subst v
      cases boundary xs hi <;> simp [WP.spec_ok]
  · simp only [h, if_false] at hr ⊢
    subst r
    simp [WP.spec_ok]

end Proofs.Text

namespace Proofs.Capture

/-- A foreign store is refused even for valid coordinates; within the
same store both kinds of range refusal and nonempty success are preserved. -/
theorem store_range_spec (a b : binding.StoreBinding) (xs : Slice Std.U8) (lo hi : Std.Usize) :
    capture.admit_store_range a b xs lo hi ⦃ r => r =
      if a.owner = b.owner ∧ a.incarnation = b.incarnation then
        if lo ≤ hi ∧ hi ≤ Slice.len xs then
          if Proofs.Text.boundary xs lo && Proofs.Text.boundary xs hi then core.result.Result.Ok ()
          else core.result.Result.Err range.RangeRefusal.NotCharBoundary
        else core.result.Result.Err range.RangeRefusal.OutOfBounds
      else core.result.Result.Err range.RangeRefusal.WrongStore ⦄ := by
  unfold capture.admit_store_range
  apply WP.spec_bind (Proofs.Binding.same_store_spec a b)
  intro v hv
  cases v with
  | false => simp_all [WP.spec_ok]
  | true =>
    simp_all only [true_iff, if_true]
    exact Proofs.Text.admit_text_spec xs lo hi

end Proofs.Capture

namespace Proofs.Snapshot

theorem inspect_spec (a b : snapshot.SnapshotKey) (xs : Slice Std.U8) (lo hi : Std.Usize) :
    snapshot.inspect_range a b xs lo hi ⦃ r => r =
      if a.high = b.high ∧ a.low = b.low then
        if lo ≤ hi ∧ hi ≤ Slice.len xs then
          if Proofs.Text.boundary xs lo && Proofs.Text.boundary xs hi then core.result.Result.Ok ()
          else core.result.Result.Err range.RangeRefusal.NotCharBoundary
        else core.result.Result.Err range.RangeRefusal.OutOfBounds
      else core.result.Result.Err range.RangeRefusal.WrongSnapshot ⦄ := by
  by_cases h : a.high = b.high <;> by_cases l : a.low = b.low <;>
    simp [snapshot.inspect_range, h, l, WP.spec_ok] <;>
    simpa using Proofs.Text.admit_text_spec xs lo hi

end Proofs.Snapshot

namespace Proofs.Capture

/-- The v1 text profile is fixed, not selected by the presenter. Every
identity component and both half-open byte boundaries must match before
success. The caller supplies an authoritative subject/image pair; this
theorem neither constructs a fixture nor connects a digest to its bytes. -/
theorem admit_capture_spec (schema : capture.CaptureSchema) (kind : capture.CaptureKind)
    (a b : capture.CaptureSubject) (xs : Slice Std.U8) (lo hi : capture.ByteOffset) :
    capture.admit_capture schema kind a b xs lo hi ⦃ r => r =
      if schema ≠ 1#u16 then core.result.Result.Err range.RangeRefusal.UnsupportedSchema
      else if kind ≠ 1#u16 then core.result.Result.Err range.RangeRefusal.WrongKind
      else if a.store.owner = b.store.owner ∧ a.store.incarnation = b.store.incarnation then
        if a.resource = b.resource then
          if a.snapshot.high = b.snapshot.high ∧ a.snapshot.low = b.snapshot.low then
            if lo ≤ hi ∧ hi ≤ Slice.len xs then
              if Proofs.Text.boundary xs lo && Proofs.Text.boundary xs hi then core.result.Result.Ok ()
              else core.result.Result.Err range.RangeRefusal.NotCharBoundary
            else core.result.Result.Err range.RangeRefusal.OutOfBounds
          else core.result.Result.Err range.RangeRefusal.WrongSnapshot
        else core.result.Result.Err range.RangeRefusal.WrongResource
      else core.result.Result.Err range.RangeRefusal.WrongStore ⦄ := by
  by_cases hs : schema = 1#u16
  · by_cases hk : kind = 1#u16
    · simp [capture.admit_capture, capture.CaptureSchema.V1, capture.CaptureKind.TEXT, hs, hk]
      apply WP.spec_bind (Proofs.Binding.same_store_spec a.store b.store)
      intro v hv
      cases v with
      | false => simp_all [WP.spec_ok]
      | true =>
        obtain ⟨ho, hi'⟩ := hv.mp rfl
        by_cases hr : a.resource = b.resource
        · simpa [ho, hi', hr] using Proofs.Snapshot.inspect_spec a.snapshot b.snapshot xs lo hi
        · simp [ho, hi', hr, WP.spec_ok]
    · simp [capture.admit_capture, capture.CaptureSchema.V1, capture.CaptureKind.TEXT, hs, hk]
  · simp [capture.admit_capture, capture.CaptureSchema.V1, hs]

end Proofs.Capture
