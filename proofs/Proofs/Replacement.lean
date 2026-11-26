import Proofs.Literal
import Proofs.PlyruneKernel

open Aeneas Aeneas.Std Result plyrune_kernel
open Proofs.Literal (Occurs)

namespace Proofs.Replacement

/-- The cut is on the independently supplied v1 resource, at precisely the
presented offsets. Splicery's Rust cut constructor is a separate host mapping. -/
theorem lower_cut_spec (a b : capture.CaptureSubject) (xs : Slice U8) (lo hi : Usize) :
    replacement.lower_cut a b xs lo hi ⦃ r => r =
      if a.store.owner = b.store.owner ∧ a.store.incarnation = b.store.incarnation then
        if a.resource = b.resource then
          if a.snapshot.high = b.snapshot.high ∧ a.snapshot.low = b.snapshot.low then
            if lo ≤ hi ∧ hi ≤ Slice.len xs then
              if Proofs.Text.boundary xs lo && Proofs.Text.boundary xs hi then
                core.result.Result.Ok (lo, hi)
              else core.result.Result.Err range.RangeRefusal.NotCharBoundary
            else core.result.Result.Err range.RangeRefusal.OutOfBounds
          else core.result.Result.Err range.RangeRefusal.WrongSnapshot
        else core.result.Result.Err range.RangeRefusal.WrongResource
      else core.result.Result.Err range.RangeRefusal.WrongStore ⦄ := by
  unfold replacement.lower_cut
  apply WP.spec_bind (Proofs.Capture.admit_capture_spec _ _ a b xs lo hi)
  intro r hr
  simp [capture.CaptureSchema.V1, capture.CaptureKind.TEXT] at hr
  split_ifs at hr <;> subst r <;> simp_all [capture.ByteOffset.get, WP.spec_ok]

/-- At a successful step *all* required bytes fit and agree at this position;
the next step starts immediately after them. None is exactly a non-fitting or
unequal term. No hashing, UTF-8, or nonempty-term premise. -/
theorem consume_spec (xs : Slice U8) (p : Usize) (term : Slice U8) :
    replacement.consume xs p term ⦃ r => match r with
      | none => ¬ Occurs xs.val term.val p.val
      | some q => Occurs xs.val term.val p.val ∧ q.val = p.val + term.val.length ⦄ := by
  unfold replacement.consume
  apply WP.spec_bind (Proofs.Literal.matches_at_spec xs term p)
  intro v hv
  cases v with
  | false => simpa [WP.spec_ok] using hv
  | true =>
    have h := hv.mp rfl
    have hfit := h.1
    have hmax := xs.property
    simp only [ite_true]
    apply WP.spec_bind (Usize.add_spec (by scalar_tac))
    intro q hq
    simp only [WP.spec_ok]
    exact ⟨h, by scalar_tac⟩

/-- Extra bytes at file end, and missing/extra rows at inventory end, are not success. -/
theorem finished_spec (consumed total : Usize) :
    replacement.finished consumed total ⦃ r => r = true ↔ consumed.val = total.val ⦄ := by
  simp [replacement.finished, WP.spec_ok] <;> scalar_tac

private theorem occurs_split {xs term : List U8} {p : Nat} (h : Occurs xs term p) :
    xs.drop p = term ++ xs.drop (p + term.length) := by
  have htake : (xs.drop p).take term.length = term := by
    apply List.ext_getElem?
    intro i
    by_cases hi : i < term.length
    · rw [List.getElem?_take_of_lt hi, List.getElem?_drop]
      exact h.2 i hi
    · rw [List.getElem?_take_eq_none (by omega), List.getElem?_eq_none (by omega)]
  simpa only [htake, List.drop_drop] using
    (List.take_append_drop term.length (xs.drop p)).symm

private theorem occurs_of_split {xs term tail : List U8} {p : Nat}
    (hp : p ≤ xs.length) (hs : xs.drop p = term ++ tail) : Occurs xs term p := by
  constructor
  · have hlen := congrArg List.length hs
    simp only [List.length_drop, List.length_append] at hlen
    omega
  · intro i hi
    rw [← List.getElem?_drop, hs, List.getElem?_append_left hi]

/-- This applies equally to raw paths and review digests. Canonical subject
framing, SHA-256 and the issuance/authority of a review are not proved here. -/
theorem same_bytes_spec (expected presented : Slice U8) :
    replacement.same_bytes expected presented ⦃ r => r = true ↔ expected.val = presented.val ⦄ := by
  unfold replacement.same_bytes
  dsimp only
  split_ifs with heq
  · have hlen : expected.val.length = presented.val.length := by scalar_tac
    apply WP.spec_mono (Proofs.Literal.matches_at_spec presented expected 0#usize)
    intro r hr
    rw [hr]
    constructor
    · intro ho
      have hs := occurs_split ho
      simpa [hlen] using hs.symm
    · intro h
      apply occurs_of_split (tail := []) <;> simp [h]
  · have hlen : expected.val.length ≠ presented.val.length := by
      intro h
      exact heq (by scalar_tac)
    simp only [WP.spec_ok, Bool.false_eq_true, false_iff]
    exact fun h => hlen (congrArg List.length h)

/-- Model of the core loop *calling the extracted decisions*, including the
final exhaustion test. The loop structure, initializing/threading the cursor
and supplying the right raw slices in Rust are tested bridge obligations. -/
def checkTerms (xs : Slice U8) : List (Slice U8) → Usize → Result Bool
  | [], p => replacement.finished p xs.len
  | term :: terms, p => do
    let next ← replacement.consume xs p term
    match next with
    | none => ok false
    | some q => checkTerms xs terms q

/-- Starting at zero this is equality of the entire raw candidate and the
concatenation of every required term, including empty terms and suffixes. -/
theorem checkTerms_spec (xs : Slice U8) (terms : List (Slice U8)) (p : Usize) :
    checkTerms xs terms p ⦃ r => r = true ↔ p.val ≤ xs.val.length ∧
      xs.val.drop p.val = terms.flatMap Slice.val ⦄ := by
  induction terms generalizing p with
  | nil =>
    apply WP.spec_mono (finished_spec p xs.len)
    intro r hr
    rw [hr]
    simp only [List.flatMap_nil, List.drop_eq_nil_iff]
    scalar_tac
  | cons term terms ih =>
    unfold checkTerms
    apply WP.spec_bind (consume_spec xs p term)
    intro next hn
    cases next with
    | none =>
      simp only [WP.spec_ok, Bool.false_eq_true, false_iff, List.flatMap_cons]
      exact fun ⟨hp, hs⟩ => hn (occurs_of_split hp hs)
    | some q =>
      obtain ⟨h, hq⟩ := hn
      have hfit := h.1
      have hpBound : p.val ≤ xs.val.length := by omega
      apply WP.spec_mono (ih q)
      intro r hr
      rw [hr]
      simp only [hfit, hpBound, true_and, List.flatMap_cons, hq,
        occurs_split h, List.append_cancel_left_eq]

/-- Gaps are the exact original byte slices preceding each admitted cut, in
order. There is one final suffix, including on an unchanged or empty file.
The caller must obtain this decomposition from the *independent* complete
selection, its authoritative base images, and ordered nonoverlapping cuts. -/
structure RequiredRow where
  path : Slice U8
  gaps : List (Slice U8)
  suffix : Slice U8

def RequiredRow.terms (row : RequiredRow) (fixed : Slice U8) : List (Slice U8) :=
  row.gaps.flatMap (fun gap => [gap, fixed]) ++ [row.suffix]

def RequiredRow.image (row : RequiredRow) (fixed : Slice U8) : List U8 × List U8 :=
  (row.path.val, (row.terms fixed).flatMap Slice.val)

/-- Lockstep iteration over the whole independently required inventory and the
actual ordered candidate inventory. Neither row list is synthesized from the
other. The two unequal-length cases are exactly a failed cardinality check. -/
def checkManifest (fixed : Slice U8) : List RequiredRow → List (Slice U8 × Slice U8) → Result Bool
  | [], [] => ok true
  | [], _ :: _ | _ :: _, [] => ok false
  | row :: required, (path, data) :: actual => do
    let pathOK ← replacement.same_bytes row.path path
    if pathOK then
      let dataOK ← checkTerms data (row.terms fixed) 0#usize
      if dataOK then checkManifest fixed required actual else ok false
    else ok false

/-- Both directions: under the decomposition premise above, acceptance is
exactly the required list of paths and fixed replacements. All gap, suffix,
and no-hit bytes are included as raw lists. No digest assumption here. -/
theorem checkManifest_spec (fixed : Slice U8) (required : List RequiredRow)
    (actual : List (Slice U8 × Slice U8)) :
    checkManifest fixed required actual ⦃ r => r = true ↔
      required.map (·.image fixed) = actual.map (fun (p, d) => (p.val, d.val)) ⦄ := by
  induction required generalizing actual with
  | nil => cases actual <;> simp [checkManifest, WP.spec_ok]
  | cons row required ih =>
    cases actual with
    | nil => simp [checkManifest, WP.spec_ok]
    | cons pair actual =>
      obtain ⟨path, data⟩ := pair
      unfold checkManifest
      apply WP.spec_bind (same_bytes_spec row.path path)
      intro pathOK hp
      cases pathOK with
      | false =>
        have hdiff : row.path.val ≠ path.val := fun h => Bool.noConfusion (hp.mpr h)
        simp [RequiredRow.image, WP.spec_ok, hdiff]
      | true =>
        have hep := hp.mp rfl
        simp only [ite_true]
        apply WP.spec_bind (checkTerms_spec data (row.terms fixed) 0#usize)
        intro dataOK hd
        cases dataOK with
        | false =>
          have hdiff : (row.terms fixed).flatMap Slice.val ≠ data.val := by
            intro he
            exact Bool.noConfusion (hd.mpr ⟨by simp, by simpa using he.symm⟩)
          simp [RequiredRow.image, WP.spec_ok, hdiff]
        | true =>
          apply WP.spec_mono (ih actual)
          intro r hr
          have hed : (row.terms fixed).flatMap Slice.val = data.val := by
            simpa using (hd.mp rfl).2.symm
          simpa [RequiredRow.image, hep, hed] using hr

private def emptyBytes : Slice U8 := Slice.from [] (by simp)

/-- One whole-resource cut (no surrounding gap bytes) accepts any fixed
image, including a nonempty change and deletion. -/
theorem one_cut_accepts (path fixed : Slice U8) :
    checkManifest fixed [⟨path, [emptyBytes], emptyBytes⟩] [(path, fixed)]
      ⦃ r => r = true ⦄ := by
  apply WP.spec_mono (checkManifest_spec _ _ _)
  intro r hr
  exact hr.mpr (by simp [RequiredRow.image, RequiredRow.terms, emptyBytes])

/-- A zero-match row is positive evidence for its original path and every
original byte. No-change acceptance here is not authority to publish. -/
theorem no_hit_accepts (path source fixed : Slice U8) :
    checkManifest fixed [⟨path, [], source⟩] [(path, source)] ⦃ r => r = true ⦄ := by
  apply WP.spec_mono (checkManifest_spec _ _ _)
  intro r hr
  exact hr.mpr (by simp [RequiredRow.image, RequiredRow.terms])

theorem one_cut_refuses_altered (path fixed other : Slice U8) (h : fixed.val ≠ other.val) :
    checkManifest fixed [⟨path, [emptyBytes], emptyBytes⟩] [(path, other)]
      ⦃ r => r = false ⦄ := by
  apply WP.spec_mono (checkManifest_spec _ _ _)
  intro r hr
  cases r <;> simp_all [RequiredRow.image, RequiredRow.terms, emptyBytes]

end Proofs.Replacement
