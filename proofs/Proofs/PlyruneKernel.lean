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
