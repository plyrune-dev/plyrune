import Extracted.MatcherSpike

open Aeneas Aeneas.Std Result matcher_spike

namespace Spike

/-- The entire needle fits and every byte of the window agrees. -/
def Occurs (xs pat : List U8) (p : Nat) : Prop :=
  p + pat.length ≤ xs.length ∧ ∀ k, k < pat.length → xs[p + k]? = pat[k]?

/-- Characterize the *whole ordered answer*, not just its members. Only a
miss may step by one; every hit must be emitted. First stops at that hit;
All continues at its exclusive end. The only ordinary exit is exhaustion. -/
inductive Greedy (xs pat : List U8) (first : Bool) : Nat → List Nat → Prop
  | exhausted {p} : xs.length ≤ p → Greedy xs pat first p []
  | miss {p tail} : p < xs.length → ¬ Occurs xs pat p → Greedy xs pat first (p+1) tail →
      Greedy xs pat first p tail
  | firstHit {p} : first = true → Occurs xs pat p → Greedy xs pat first p [p]
  | allHit {p tail} : first = false → Occurs xs pat p →
      Greedy xs pat first (p + pat.length) tail → Greedy xs pat first p (p :: tail)

theorem matches_loop_spec (xs pat : Slice U8) (pos j : Usize)
    (hfit : pos.val + pat.val.length ≤ xs.val.length) (hj : j.val ≤ pat.val.length)
    (hpref : ∀ k, k < j.val → xs.val[pos.val + k]? = pat.val[k]?) :
    matches_at_loop xs pat pos j ⦃ r => r = true ↔ Occurs xs.val pat.val pos.val ⦄ := by
  unfold matches_at_loop
  apply loop.spec_decr_nat (fun j : Usize => pat.val.length - j.val)
    (fun j : Usize => j.val ≤ pat.val.length ∧
      ∀ k, k < j.val → xs.val[pos.val + k]? = pat.val[k]?)
  · intro j ⟨hj, hpref⟩
    unfold matches_at_loop.body
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

theorem matches_at_spec (xs pat : Slice U8) (pos : Usize) (hp : pos.val ≤ xs.val.length) :
    matches_at xs pat pos ⦃ r => r = true ↔ Occurs xs.val pat.val pos.val ⦄ := by
  unfold matches_at
  apply WP.spec_bind (Usize.sub_spec (by scalar_tac))
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

/-- Completing any legal suffix of the cursor after these hits gives a
legal traversal from the initial cursor. In particular no earlier hit was
skipped to reach this state. The length conjunct discharges Vec.push. -/
def Prefix (xs pat : List U8) (first : Bool) (initial : Nat)
    (hits : alloc.vec.Vec Usize) (p : Usize) : Prop :=
  hits.val.length ≤ p.val ∧ ∀ tail, Greedy xs pat first p.val tail →
    Greedy xs pat first initial (hits.val.map (·.val) ++ tail)

theorem scan_from_spec (xs pat : Slice U8) (first : Bool) (initial : Usize)
    (hne : 0 < pat.val.length) :
    scan_from xs pat first initial ⦃ r => Greedy xs.val pat.val first initial.val
      (r.val.map (·.val)) ⦄ := by
  unfold scan_from scan_from_loop
  apply loop.spec_decr_nat
    (fun s : alloc.vec.Vec Usize × Usize => xs.val.length - s.2.val)
    (fun s : alloc.vec.Vec Usize × Usize => Prefix xs.val pat.val first initial.val s.1 s.2)
  · rintro ⟨hits, p⟩ ⟨hlen, hpref⟩
    unfold scan_from_loop.body
    try dsimp only
    by_cases hlt : p < xs.len
    · rw [if_pos hlt]
      have hp : p.val < xs.val.length := by scalar_tac
      apply WP.spec_bind (matches_at_spec xs pat p (by omega))
      intro matchp hm
      by_cases hmatch : matchp = true
      · rw [if_pos hmatch]
        have hoc := hm.mp hmatch
        have hfit := hoc.1
        have hmax := xs.property
        apply WP.spec_bind (alloc.vec.Vec.push_spec hits p (by scalar_tac))
        intro pushed hpush
        split_ifs with hfirst
        · simp only [WP.spec_ok]
          simpa [hpush] using hpref [p.val] (.firstHit hfirst hoc)
        · have hf : first = false := by cases first <;> simp_all
          apply WP.spec_bind (Usize.add_spec (by scalar_tac))
          intro next hnext
          have hn : next.val = p.val + pat.val.length := by scalar_tac
          simp only [WP.spec_ok, Prefix]
          and_intros
          all_goals try (simp only [hpush, List.length_append, List.length_cons, List.length_nil]; scalar_tac)
          intro tail ht
          have rest : Greedy xs.val pat.val first (p.val + pat.val.length) tail := hn ▸ ht
          simpa [hpush, List.append_assoc] using hpref (p.val :: tail) (.allHit hf hoc rest)
          omega
      · rw [if_neg hmatch]
        have hno : ¬ Occurs xs.val pat.val p.val := fun h => hmatch (hm.mpr h)
        apply WP.spec_bind (Usize.add_spec (by scalar_tac))
        intro next hnext
        have hn : next.val = p.val + 1 := by scalar_tac
        simp only [WP.spec_ok, Prefix]
        and_intros
        all_goals try scalar_tac
        intro tail ht
        exact hpref tail (.miss hp hno (hn ▸ ht))
    · simp only [if_neg hlt, WP.spec_ok]
      simpa using hpref [] (.exhausted (by scalar_tac))
  · simp [Prefix, alloc.vec.Vec.new]

/-- Total entrypoint specification: exact refusal or exact greedy starts,
for arbitrary immutable bytes, needle and mode. No premise of completeness. -/
theorem scan_spec (xs pat : Slice U8) (first : Bool) :
    scan xs pat first ⦃ r => match r with
      | none => pat.val.length = 0
      | some hits => 0 < pat.val.length ∧ Greedy xs.val pat.val first 0 (hits.val.map (·.val)) ⦄ := by
  unfold scan
  dsimp only
  split_ifs with hempty
  · simp only [WP.spec_ok]
    scalar_tac
  · have hne : 0 < pat.val.length := by scalar_tac
    apply WP.spec_bind (scan_from_spec xs pat first _ hne)
    intro hits hh
    exact (by simpa [WP.spec_ok, hne] using hh)

end Spike
