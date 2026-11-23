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
