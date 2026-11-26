use plyrune_core::{
    fixture::{Entry, Fixture, IncarnationId, OwnerId, RangeRefusal, StoreBinding, ViewRefusal},
    selection::{SelectionRefusal, refs},
};

fn entry<'a>(path: &'a str, text: &'a str) -> Entry<'a> {
    Entry {
        path,
        mode: "regular",
        bytes: text.as_bytes(),
    }
}

#[test]
fn rows_cover_the_independent_scope_even_when_the_display_would_be_empty() {
    let entries = [
        entry("config/sub/b", "e\u{301} é😀é\r\n"),
        entry("config2/lookalike", "é"),
        entry("config/empty", ""),
        entry("config/sub/a", "e\u{301}"),
        entry("before", "é"),
    ];
    let fixture = Fixture::admit(&entries).unwrap();
    let selection = refs(&fixture, "config/", "é").unwrap();
    assert_eq!(selection.snapshot(), fixture.digest());
    assert!(std::ptr::eq(selection.fixture(), &fixture));
    assert_eq!(selection.prefix(), "config/");
    assert_eq!(selection.needle(), "é");
    assert_eq!(selection.match_count(), 2);
    let observed: Vec<_> = selection
        .rows()
        .iter()
        .map(|row| {
            let offsets: Vec<_> = row.ranges().iter().map(|r| (r.start(), r.end())).collect();
            (row.path(), offsets)
        })
        .collect();
    assert_eq!(
        observed,
        [
            ("config/empty", vec![]),
            ("config/sub/a", vec![]),
            ("config/sub/b", vec![(4, 6), (10, 12)]),
        ]
    );
    let no_hits = refs(&fixture, "config/", "none").unwrap();
    assert_eq!(no_hits.match_count(), 0);
    assert_eq!(no_hits.rows().len(), 3);
    assert!(no_hits.rows().iter().all(|row| row.ranges().is_empty()));
    let no_paths = refs(&fixture, "absent/", "é").unwrap();
    assert!(no_paths.rows().is_empty());
    assert_eq!(no_paths.match_count(), 0);
}

#[test]
fn captures_resolve_greedy_occurrences_only_on_the_bound_subject() {
    let entries = [
        entry("x/a", "aaaaa"),
        entry("x/b", "aaaaa"),
        entry("x/c", ""),
    ];
    let store = StoreBinding::new(OwnerId::from_host(13), IncarnationId::from_host(5));
    let bound = Fixture::admit(&entries).unwrap().bind(store);
    let identical = Fixture::admit(&entries).unwrap().bind(store);
    let selection = refs(bound.fixture(), "x/", "aa").unwrap();
    let coords: Vec<_> = selection.rows()[0]
        .ranges()
        .iter()
        .map(|r| (r.start(), r.end()))
        .collect();
    assert_eq!(coords, [(0, 2), (2, 4)]);
    let capture = selection.capture(&bound, "x/a", 1).unwrap();
    assert_eq!(bound.open("x/a").unwrap().resolve(&capture), Ok("aa"));
    assert_eq!(
        bound.open("x/b").unwrap().resolve(&capture),
        Err(ViewRefusal::Admission(RangeRefusal::WrongResource))
    );
    assert_eq!(
        selection.capture(&identical, "x/a", 0).err(),
        Some(SelectionRefusal::WrongFixture)
    );
    for (path, index) in [("x/a", 2), ("x/c", 0), ("y/a", 0)] {
        assert_eq!(
            selection.capture(&bound, path, index).err(),
            Some(SelectionRefusal::UnknownMatch)
        );
    }
}

#[test]
fn captures_are_not_display_slices_and_snapshot_binds_outside_scope() {
    let needle = format!("{}\r\n", "é".repeat(2048));
    let store = StoreBinding::new(OwnerId::from_host(1), IncarnationId::from_host(2));
    let bound = Fixture::admit(&[entry("x/hit", &needle), entry("other", "before")])
        .unwrap()
        .bind(store);
    let other = Fixture::admit(&[entry("x/hit", &needle), entry("other", "after")])
        .unwrap()
        .bind(store);
    let selection = refs(bound.fixture(), "x/", &needle).unwrap();
    let changed = refs(other.fixture(), "x/", &needle).unwrap();
    assert_ne!(selection.snapshot(), changed.snapshot());
    assert_eq!((selection.match_count(), changed.match_count()), (1, 1));
    let capture = selection.capture(&bound, "x/hit", 0).unwrap();
    assert_eq!(
        bound.open("x/hit").unwrap().resolve(&capture),
        Ok(needle.as_str())
    );
    assert_eq!(
        other.open("x/hit").unwrap().resolve(&capture),
        Err(ViewRefusal::Admission(RangeRefusal::WrongSnapshot))
    );
}

#[test]
fn queries_obey_the_frozen_prefix_and_literal_profile() {
    let fixture = Fixture::admit(&[]).unwrap();
    for prefix in ["", "/", "no-slash", "x//", "x/./", "../x/", "x\\y/", "é/"] {
        assert_eq!(
            refs(&fixture, prefix, "x").err(),
            Some(SelectionRefusal::InvalidPrefix)
        );
    }
    assert_eq!(
        refs(&fixture, "x/", "").err(),
        Some(SelectionRefusal::EmptyNeedle)
    );
    let mut prefix = "a".repeat(254);
    prefix.push('/');
    assert!(refs(&fixture, &prefix, "x").is_ok());
    prefix.insert(0, 'a');
    assert_eq!(
        refs(&fixture, &prefix, "x").err(),
        Some(SelectionRefusal::PrefixLimit)
    );
    let mut needle = "a".repeat(8 * 1024 * 1024);
    assert!(refs(&fixture, "x/", &needle).is_ok());
    needle.push('b');
    assert_eq!(
        refs(&fixture, "x/", &needle).err(),
        Some(SelectionRefusal::NeedleLimit)
    );
}

#[test]
fn hit_budgets_are_inclusive_and_excess_never_returns_a_partial_selection() {
    let full = "a".repeat(4096);
    let excess = "a".repeat(4097);
    let accepted = Fixture::admit(&[entry("x/a", &full), entry("x/b", &full)]).unwrap();
    assert_eq!(refs(&accepted, "x/", "a").unwrap().match_count(), 8192);
    let too_many_here = Fixture::admit(&[entry("x/a", &excess)]).unwrap();
    assert_eq!(
        refs(&too_many_here, "x/", "a").err(),
        Some(SelectionRefusal::ResourceMatchLimit)
    );
    let too_many_total =
        Fixture::admit(&[entry("x/a", &full), entry("x/b", &full), entry("x/c", "a")]).unwrap();
    assert_eq!(
        refs(&too_many_total, "x/", "a").err(),
        Some(SelectionRefusal::TotalMatchLimit)
    );
}

#[test]
fn comparison_budget_counts_all_possible_windows_across_the_whole_scope() {
    let needle = "a".repeat(8192);
    let haystack = "b".repeat(16383);
    let extra = "b".repeat(8192);
    // 8192 candidate starts times 8192 bytes exactly meets the limit.
    let admitted = Fixture::admit(&[entry("x/a", &haystack), entry("x2/b", &extra)]).unwrap();
    assert_eq!(refs(&admitted, "x/", &needle).unwrap().match_count(), 0);
    let excessive = Fixture::admit(&[entry("x/a", &haystack), entry("x/b", &extra)]).unwrap();
    assert_eq!(
        refs(&excessive, "x/", &needle).err(),
        Some(SelectionRefusal::ComparisonLimit)
    );
}
