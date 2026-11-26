import Extracted.PlyruneKernel

open Aeneas Aeneas.Std Result plyrune_kernel

namespace Proofs.Literal

/-- The entire needle fits and every byte of its window agrees. -/
def Occurs (xs pat : List U8) (p : Nat) : Prop :=
  p + pat.length ≤ xs.length ∧ ∀ k, k < pat.length → xs[p + k]? = pat[k]?

/-- Prefix invariant for the byte comparison; the caller has admitted the full window. -/
theorem matches_loop_spec (xs pat : Slice U8) (pos j : Usize)
    (hfit : pos.val + pat.val.length ≤ xs.val.length) (hj : j.val ≤ pat.val.length)
    (hpref : ∀ k, k < j.val → xs.val[pos.val + k]? = pat.val[k]?) :
    literal.matches_at_loop xs pat pos j ⦃ r => r = true ↔ Occurs xs.val pat.val pos.val ⦄ := by
  unfold literal.matches_at_loop
  apply loop.spec_decr_nat (fun j : Usize => pat.val.length - j.val)
    (fun j : Usize => j.val ≤ pat.val.length ∧
      ∀ k, k < j.val → xs.val[pos.val + k]? = pat.val[k]?)
  · intro j ⟨hj, hpref⟩
    unfold literal.matches_at_loop.body
    try dsimp only
    split_ifs with hlt
    · have hjlt : j.val < pat.val.length := by scalar_tac
      have hmax := xs.property
      apply WP.spec_bind (Usize.add_spec (by scalar_tac))
      intro offset hoff
      step as ⟨x, hx⟩
      step as ⟨y, hy⟩
      have hbytes : xs.val[pos.val + j.val]? = some x := by
        have ho : pos.val + j.val = offset.val := by scalar_tac
        rw [ho, List.getElem?_eq_getElem (by scalar_tac), hx]
      have hneedle : pat.val[j.val]? = some y := by
        rw [List.getElem?_eq_getElem hjlt, hy]
      split_ifs with hne
      · simp only [WP.spec_ok, Bool.false_eq_true, false_iff]
        intro h
        have heq := h.2 j.val hjlt
        rw [hbytes, hneedle] at heq
        exact (bne_iff_ne.mp hne) (Option.some.inj heq)
      · rw [bne_iff_ne, not_not] at hne
        apply WP.spec_bind (Usize.add_spec (by scalar_tac))
        intro next hnext
        simp only [WP.spec_ok]
        and_intros
        all_goals try scalar_tac
        intro k hk
        by_cases hk' : k < j.val
        · exact hpref _ hk'
        · have : k = j.val := by scalar_tac
          simpa [this, hbytes, hneedle] using congrArg some hne
    · simp only [WP.spec_ok, true_iff]
      exact ⟨hfit, fun k hk => hpref k (by scalar_tac)⟩
  · exact ⟨hj, hpref⟩

/-- Exact comparison even for invalid positions and for the empty byte window. -/
theorem matches_at_spec (xs pat : Slice U8) (pos : Usize) :
    literal.matches_at xs pat pos ⦃ r => r = true ↔ Occurs xs.val pat.val pos.val ⦄ := by
  unfold literal.matches_at
  dsimp only
  split_ifs with hp
  · simp only [WP.spec_ok, Bool.false_eq_true, false_iff]
    intro h
    have := h.1
    scalar_tac
  · apply WP.spec_bind (Usize.sub_spec (by scalar_tac))
    intro avail havail
    split_ifs with hshort
    · simp only [WP.spec_ok, Bool.false_eq_true, false_iff]
      intro h
      have := h.1
      scalar_tac
    · apply matches_loop_spec
      · scalar_tac
      · simp
      · simp

/-- Independent least-position characterization. A miss excludes *every*
position; a hit must fit, agree bytewise and exclude *all earlier* hits. -/
def Earliest (xs pat : List U8) : Option Usize → Prop
  | none => pat.length = 0 ∨ ∀ k, ¬ Occurs xs pat k
  | some p => 0 < pat.length ∧ Occurs xs pat p.val ∧ ∀ k, k < p.val → ¬ Occurs xs pat k

/-- Unconditional totality and exact first hit, relative to the supplied suffix. -/
theorem first_match_spec (xs pat : Slice U8) :
    literal.first_match xs pat ⦃ r => Earliest xs.val pat.val r ⦄ := by
  simp only [literal.first_match, core.slice.Slice.is_empty, Slice.length, bind_ok, decide_eq_true_eq]
  split_ifs with hempty
  · exact (by simpa only [WP.spec_ok, Earliest] using Or.inl hempty)
  · have hne : 0 < pat.val.length := by omega
    unfold literal.first_match_loop
    apply loop.spec_decr_nat (fun p : Usize => xs.val.length - p.val)
      (fun p : Usize => ∀ k, k < p.val → ¬ Occurs xs.val pat.val k)
    · intro p skipped
      unfold literal.first_match_loop.body
      dsimp only
      split_ifs with hlt
      · have hmax := xs.property
        apply WP.spec_bind (matches_at_spec xs pat p)
        intro m hm
        split_ifs with hit
        · exact (by simpa only [WP.spec_ok, Earliest] using And.intro hne ⟨hm.mp hit, skipped⟩)
        · apply WP.spec_bind (Usize.add_spec (by scalar_tac))
          intro next hnext
          simp only [WP.spec_ok]
          constructor
          · intro k hk
            by_cases before : k < p.val
            · exact skipped k before
            · have eq : k = p.val := by scalar_tac
              exact eq ▸ fun occ => hit (hm.mpr occ)
          · scalar_tac
      · simp only [WP.spec_ok, Earliest]
        right
        intro k occ
        by_cases before : k < p.val
        · exact skipped k before occ
        · have := occ.1
          scalar_tac
    · simp

/-- Exact sequence equality: the witness cannot omit a member, duplicate,
substitute or reorder an ordinal. There is no premise that the rows match. -/
theorem inventory_matches_spec (xs ys : Slice Usize) :
    literal.inventory_matches xs ys ⦃ r => r = true ↔ xs.val = ys.val ⦄ := by
  unfold literal.inventory_matches
  dsimp only
  split_ifs with hlen
  · simp only [WP.spec_ok, Bool.false_eq_true, false_iff]
    intro eq
    have := congrArg List.length eq
    exact (bne_iff_ne.mp hlen) (by scalar_tac)
  · rw [bne_iff_ne, not_not] at hlen
    have sameLen : xs.val.length = ys.val.length := by scalar_tac
    unfold literal.inventory_matches_loop
    apply loop.spec_decr_nat (fun i : Usize => ys.val.length - i.val)
      (fun i : Usize => ∀ k, k < i.val → xs.val[k]? = ys.val[k]?)
    · intro i prefixEq
      unfold literal.inventory_matches_loop.body
      dsimp only
      split_ifs with hlt
      · have inY : i.val < ys.val.length := by scalar_tac
        have inX : i.val < xs.val.length := by omega
        step as ⟨x, hx⟩
        step as ⟨y, hy⟩
        have getX : xs.val[i.val]? = some x := by rw [List.getElem?_eq_getElem inX, hx]
        have getY : ys.val[i.val]? = some y := by rw [List.getElem?_eq_getElem inY, hy]
        split_ifs with different
        · simp only [WP.spec_ok, Bool.false_eq_true, false_iff]
          intro eq
          have same : xs.val[i.val]? = ys.val[i.val]? := by rw [eq]
          rw [getX, getY] at same
          exact (bne_iff_ne.mp different) (Option.some.inj same)
        · rw [bne_iff_ne, not_not] at different
          apply WP.spec_bind (Usize.add_spec (by scalar_tac))
          intro next hnext
          simp only [WP.spec_ok]
          constructor
          · intro k hk
            by_cases before : k < i.val
            · exact prefixEq k before
            · have eq : k = i.val := by scalar_tac
              rw [eq, getX, getY, different]
          · scalar_tac
      · simp only [WP.spec_ok, true_iff]
        apply List.ext_getElem?
        intro k
        by_cases before : k < i.val
        · exact prefixEq k before
        · rw [List.getElem?_eq_none (by scalar_tac : xs.val.length ≤ k),
              List.getElem?_eq_none (by scalar_tac : ys.val.length ≤ k)]
    · simp

end Proofs.Literal
