## Problem

At `bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42`, `Stamped<P>` records a fact and a `u64` version, but no logical subject identity. `verify(self, subject)` compares only the supplied subject's version and then returns evidence borrowing that supplied subject.

Two different instances of the same type can legitimately have revision 0 while the proposition holds for only one. A stamp minted for the first can therefore be accepted for the second.

Evidence: [Versioned and Stamped](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten/src/origin.rs#L194-L258), [generated mint_stamped](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten-macros/src/lib.rs#L610-L623). The documented Versioned contract requires changes when relevant state changes; it does not require globally identifying subjects.

**Verification status: source-inspected, independently reviewed; no executed Rust reproduction in this authoring environment.**

## Minimal regression using the existing fixture

Add this beside the existing stamp test in [tests/behaviour.rs](https://github.com/tauten-dev/tauten/blob/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42/crates/tauten/tests/behaviour.rs#L140-L202):

```rust
#[test]
fn a_stamp_cannot_be_rebound_to_another_queue() {
    let first = Queue { jobs: vec![4], version: 0 };
    let other = Queue { jobs: vec![], version: 0 };
    let stamp = first.check_later(0).expect("present in first");
    assert!(stamp.verify(&other).is_err());
}
```

The current implementation's equality branch implies this assertion would fail: slot 0 does not exist in `other`, but its revision matches.

## Requested outcome

Bind the existing runtime carrier to a documented logical subject identity, incarnation where needed, and revision, or another representation providing equivalent guarantees. Decide the API through the normal RFC process. Do not prescribe pointer identity, a global UUID service, or lifetime brands.

A proposed downstream consumer, Plyrune, needs to carry captures and check reports between calls. Its interim design uses explicit owner/snapshot/candidate bindings rather than relying on a bare revision stamp.

## Acceptance

- Unchanged binding of the same subject succeeds.
- Different subject with the same revision refuses.
- Changed revision refuses.
- Recreated/restored subjects cannot accept old stamps merely by reusing a revision.
- Move/clone semantics and revision exhaustion, reset and reuse assumptions are documented. The existing fixture uses wrapping_add; replace that example or state and enforce its restricted contract.
- A negative control that removes identity/incarnation comparison is caught by the wrong-subject fixture; retain the positive control.
- Proof claims distinguish comparison correctness under binding assumptions from correctness of each binding producer.
- Update carrier documentation and examples; no claim about external filesystem concurrency follows from a Rust borrow or revision comparison.

Related: #9 introduced this carrier. This is a correctness/scope follow-up to that existing API, not a reopening of #30's rejected lifetime-brand mechanism. Filed in later for owner triage; an adopter can use explicit bindings in the meantime.
