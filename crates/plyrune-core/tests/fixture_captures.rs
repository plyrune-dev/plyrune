use plyrune_core::fixture::{
    BoundFixture, ByteOffset, Entry, Fixture, IncarnationId, OwnerId, RangeRefusal, StoreBinding,
    ViewRefusal,
};

fn bound(owner: u64, incarnation: u64, note: &str, reversed: bool) -> BoundFixture<'_> {
    let row = |path, text: &'static str| Entry {
        path,
        mode: "regular",
        bytes: text.as_bytes(),
    };
    let mut rows = [
        row("a", "préfixe\r\n"),
        row("b", "préfixe\r\n"),
        Entry {
            path: "notes",
            mode: "regular",
            bytes: note.as_bytes(),
        },
    ];
    if reversed {
        rows.reverse();
    }
    Fixture::admit(&rows).unwrap().bind(StoreBinding::new(
        OwnerId::from_host(owner),
        IncarnationId::from_host(incarnation),
    ))
}

#[test]
fn capture_follows_immutable_subject_not_equal_bytes_or_input_order() {
    let b = ByteOffset::new;
    let first = bound(12, 4, "original", false);
    let capture = first.open("a").unwrap().capture(b(2), b(4)).unwrap();
    let reordered = bound(12, 4, "original", true);
    assert_eq!(reordered.open("a").unwrap().resolve(&capture), Ok("é"));
    for (owner, incarnation, note, path, refusal) in [
        (12, 4, "original", "b", RangeRefusal::WrongResource),
        (12, 4, "changed", "a", RangeRefusal::WrongSnapshot),
        (13, 4, "original", "a", RangeRefusal::WrongStore),
        (12, 5, "original", "a", RangeRefusal::WrongStore),
    ] {
        let target = bound(owner, incarnation, note, false);
        assert_eq!(
            target.open(path).unwrap().resolve(&capture),
            Err(ViewRefusal::Admission(refusal))
        );
    }
}

#[test]
fn invalid_coordinates_never_mint_a_capture_and_empty_ranges_resolve() {
    let b = ByteOffset::new;
    let fixture = bound(1, 2, "", false);
    let resource = fixture.open("a").unwrap();
    assert_eq!(
        resource.capture(b(3), b(4)).err(),
        Some(RangeRefusal::NotCharBoundary)
    );
    assert_eq!(
        resource.capture(b(20), b(1)).err(),
        Some(RangeRefusal::OutOfBounds)
    );
    let end = b("préfixe\r\n".len());
    assert_eq!(
        resource.resolve(&resource.capture(end, end).unwrap()),
        Ok("")
    );
    assert_eq!(
        fixture.open("missing").err(),
        Some(ViewRefusal::UnknownPath)
    );
}
