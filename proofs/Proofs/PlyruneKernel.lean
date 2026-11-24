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
