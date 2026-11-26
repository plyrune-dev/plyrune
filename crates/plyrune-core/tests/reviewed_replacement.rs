use plyrune_core::fixture::{BoundFixture, Entry, Fixture, IncarnationId, OwnerId, StoreBinding};
use plyrune_core::replacement::{Disposition, Refusal, check_candidate, preview, same_review};

fn regular<'a>(path: &'a str, text: &'a str) -> Entry<'a> {
    Entry {
        path,
        mode: "regular",
        bytes: text.as_bytes(),
    }
}

fn bound<'a>(entries: &[Entry<'a>]) -> BoundFixture<'a> {
    Fixture::admit(entries).unwrap().bind(StoreBinding::new(
        OwnerId::from_host(183),
        IncarnationId::from_host(1),
    ))
}

fn digest(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
}

#[test]
fn fixture_oracle_checks_complete_outputs_and_framing_including_negative_witnesses() {
    // examples/{input,expected-candidate}.json, not values from the renderer.
    let input = bound(&[
        regular(
            "config/server/api.env",
            "API_URL=legacy_endpoint\nFALLBACK=legacy_endpoint\n",
        ),
        regular(
            "config/client/app.env",
            "PRIMARY=legacy_endpoint\nSECONDARY=legacy_endpoint\n",
        ),
        regular(
            "config/jobs/worker.env",
            "UPSTREAM=legacy_endpoint\nBACKUP=legacy_endpoint\n",
        ),
        regular("config/unchanged.env", "API_URL=modern_endpoint\n"),
        regular(
            "docs/notes.txt",
            "The old setting was called legacy_endpoint.\n",
        ),
    ]);
    let expected = [
        regular(
            "config/client/app.env",
            "PRIMARY=service_endpoint\nSECONDARY=service_endpoint\n",
        ),
        regular(
            "config/jobs/worker.env",
            "UPSTREAM=service_endpoint\nBACKUP=service_endpoint\n",
        ),
        regular(
            "config/server/api.env",
            "API_URL=service_endpoint\nFALLBACK=service_endpoint\n",
        ),
        regular("config/unchanged.env", "API_URL=modern_endpoint\n"),
        regular(
            "docs/notes.txt",
            "The old setting was called legacy_endpoint.\n",
        ),
    ];
    let reviewed = preview(&input, "config/", "legacy_endpoint", "service_endpoint").unwrap();
    assert_eq!(reviewed.selection().rows().len(), 4);
    assert_eq!(reviewed.matched(), 6);
    assert_eq!(reviewed.changed(), 3);
    assert_eq!(reviewed.disposition(), Disposition::Changed);
    assert_eq!(reviewed.rows().len(), expected.len());
    for ((path, text), reference) in reviewed.rows().zip(&expected) {
        assert_eq!((path, text.as_bytes()), (reference.path, reference.bytes));
    }
    // qualification/requirement-vectors.json + independent SHA-256 framing
    // of the complete expected-candidate.json, not the prepared output.
    assert_eq!(
        *reviewed.requirement_digest(),
        digest("181a225e55b80995712132d99dd6bbbf04b131ec8a98c97a85c552e0198a2dfe")
    );
    assert_eq!(
        *reviewed.candidate_digest(),
        digest("3586c185513ca100fb4be1193393269aea99bc580a114b34a04762547f0535af")
    );
    assert_eq!(
        *reviewed.review_id(),
        digest("89de426ad6c05cc4d01a93848fbf7bf59654d400b95cb4da35f16058ab345036")
    );
    let external = check_candidate(
        input.fixture(),
        "config/",
        "legacy_endpoint",
        "service_endpoint",
        &expected,
    )
    .unwrap();
    assert!(same_review(external.review_id(), reviewed.review_id()));
}

#[test]
fn raw_checker_refuses_wrong_files_missing_gaps_surplus_bytes_and_nonregular_modes() {
    let input = bound(&[
        regular("a/x", "old/old!"),
        regular("a/y", "never"),
        regular("b/x", "old/old!"),
    ]);
    let check =
        |rows: &[Entry<'_>]| check_candidate(input.fixture(), "a/", "old", "N", rows).map(|_| ());
    let correct = || {
        [
            regular("a/x", "N/N!"),
            regular("a/y", "never"),
            regular("b/x", "old/old!"),
        ]
    };
    assert_eq!(check(&correct()), Ok(()));
    for (which, bad) in [
        (0, "old/N!"),
        (0, "N/old!"),
        (0, "NN!"),
        (0, "N/N"),
        (0, "N/N!!"),
        (1, "neveR"),
        (2, "N/N!"),
    ] {
        let mut rows = correct();
        rows[which].bytes = bad.as_bytes();
        assert_eq!(check(&rows), Err(Refusal::Mismatch));
    }
    let mut rows = correct();
    rows[1].mode = "symlink";
    assert_eq!(check(&rows), Err(Refusal::Inventory));
    let mut rows = correct();
    rows[1].path = "a/x";
    assert_eq!(check(&rows), Err(Refusal::Inventory));
    let [positive, _, outside] = correct();
    assert_eq!(check(&[positive, outside]), Err(Refusal::Inventory));
    let mut extra = Vec::from(correct());
    extra.push(regular("c/x", ""));
    assert_eq!(check(&extra), Err(Refusal::Inventory));
    // Equal original bytes at two different paths do not authorize a swap.
    assert_eq!(
        check(&[
            regular("a/x", "old/old!"),
            regular("a/y", "never"),
            regular("b/x", "N/N!")
        ]),
        Err(Refusal::Mismatch)
    );
    let mut rows = correct();
    rows[0].bytes = &[0xff];
    assert_eq!(check(&rows), Err(Refusal::Mismatch));
}

#[test]
fn independently_specified_scope_cannot_be_narrowed_by_the_candidate_author() {
    let input = bound(&[
        regular("config/a", "old"),
        regular("config/server/a", "old"),
    ]);
    let subset = [
        regular("config/a", "old"),
        regular("config/server/a", "new"),
    ];
    assert!(check_candidate(input.fixture(), "config/server/", "old", "new", &subset).is_ok());
    assert_eq!(
        check_candidate(input.fixture(), "config/", "old", "new", &subset).err(),
        Some(Refusal::Mismatch)
    );
}

#[test]
fn unchanged_borrows_source_and_review_distinguishes_no_match_no_bytes_and_task() {
    let bytes = "xx".to_string();
    let input = bound(&[regular("a/x", &bytes)]);
    let identical = preview(&input, "a/", "x", "x").unwrap();
    let absent = preview(&input, "a/", "missing", "x").unwrap();
    for reviewed in [&identical, &absent] {
        assert_eq!(reviewed.disposition(), Disposition::NoChange);
        assert_eq!(reviewed.changed(), 0);
        assert_eq!(reviewed.rows().next().unwrap().1.as_ptr(), bytes.as_ptr());
    }
    assert_eq!(identical.matched(), 2);
    assert_eq!(absent.matched(), 0);
    assert_eq!(identical.candidate_digest(), absent.candidate_digest());
    assert!(!same_review(identical.review_id(), absent.review_id()));
    let empty = bound(&[]);
    let no_files = preview(&empty, "a/", "x", "x").unwrap();
    assert_eq!(no_files.matched(), 0);
    assert_eq!(no_files.rows().len(), 0);
    assert_eq!(no_files.disposition(), Disposition::NoChange);
    assert!(!same_review(no_files.review_id(), identical.review_id()));
}

#[test]
fn review_binds_original_even_when_the_same_task_converges_to_identical_output() {
    let first = bound(&[regular("p/f", "aaa")]);
    let second = bound(&[regular("p/f", "a")]);
    let a = preview(&first, "p/", "aa", "").unwrap();
    let b = preview(&second, "p/", "aa", "").unwrap();
    assert_eq!(a.rows().next(), Some(("p/f", "a")));
    assert_eq!(a.requirement_digest(), b.requirement_digest());
    assert_eq!(a.candidate_digest(), b.candidate_digest());
    assert_ne!(a.base_digest(), b.base_digest());
    assert!(!same_review(a.review_id(), b.review_id()));
}

#[test]
fn unicode_deletion_preserves_crlf_and_unmatched_suffix_bytes() {
    let input = bound(&[regular("p/a", "ééß\r\nééé"), regular("p/b", "e\u{301}")]);
    let deleted = preview(&input, "p/", "éé", "").unwrap();
    assert_eq!(
        deleted.rows().collect::<Vec<_>>(),
        [("p/a", "ß\r\né"), ("p/b", "e\u{301}")]
    );
    assert_eq!(deleted.matched(), 2);
    let expanded = preview(&input, "p/", "éé", "ééé").unwrap();
    assert_eq!(expanded.rows().next(), Some(("p/a", "éééß\r\néééé")));
    assert_eq!(expanded.matched(), 2); // never rescan the replacement
}

#[test]
fn projected_bytes_are_bounded_for_the_whole_batch_before_materialization() {
    const LIMIT: usize = 16 * 1024 * 1024;
    let replacement = "z".repeat(LIMIT);
    let one = bound(&[regular("p/a", "x")]);
    // An output larger than the 8 MiB base-admission cap is legal up to 16 MiB.
    let full = preview(&one, "p/", "x", &replacement).unwrap();
    assert_eq!(full.rows().next().unwrap().1.len(), LIMIT);
    let external = [regular("p/a", &replacement)];
    let checked = check_candidate(one.fixture(), "p/", "x", &replacement, &external).unwrap();
    assert!(same_review(full.review_id(), checked.review_id()));
    let repeated = bound(&[regular("p/a", "xx")]);
    assert_eq!(
        preview(&repeated, "p/", "x", &replacement).err(),
        Some(Refusal::Budget)
    );
    let aggregate = bound(&[
        regular("p/a", "x"),
        regular("p/b", "x"),
        regular("outside/a", "!"),
    ]);
    assert_eq!(
        preview(&aggregate, "p/", "x", &replacement).err(),
        Some(Refusal::Budget)
    );
}
