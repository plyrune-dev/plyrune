use super::*;
use std::fs as disk;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;

const REVIEW: [u8; 32] = [7; 32];
const ROWS: [(&str, &str); 3] = [
    ("scope/a", "new-a"),
    ("scope/b", "new-b"),
    ("scope/c", "keep"),
];
const BASE: [(&str, &str); 3] = [
    ("scope/a", "old-a"),
    ("scope/b", "old-b"),
    ("scope/c", "keep"),
];

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("plyrune-publication-{}-{tag}", std::process::id()));
        disk::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }

    fn source(&self, name: &str) -> Directory {
        let root = self.0.join(name);
        disk::create_dir_all(root.join("scope/empty")).unwrap();
        for (path, text) in BASE {
            disk::write(root.join(path), text).unwrap();
        }
        disk::set_permissions(root.join("scope/a"), disk::Permissions::from_mode(0o751)).unwrap();
        Directory::read(&root, "scope/").unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        disk::remove_dir_all(&self.0).unwrap();
    }
}

fn apply(input: &Directory, dest: Destination<'_>) -> Publication {
    publish(input, &ROWS, &REVIEW, &REVIEW, dest)
}

#[test]
fn in_place_changes_only_different_files_with_their_modes() {
    let dir = Scratch::new("in-place");
    let input = dir.source("src");
    let untouched = disk::metadata(input.root.join("scope/c")).unwrap();
    let result = apply(&input, Destination::InPlace);
    assert_eq!(result.status, Status::Complete, "{result:?}");
    assert_eq!(result.installed_files, ["scope/a", "scope/b"]);
    assert!(result.staging_directories_remaining.is_empty());
    for (path, text) in ROWS {
        assert_eq!(disk::read(input.root.join(path)).unwrap(), text.as_bytes());
    }
    assert_eq!(
        disk::metadata(input.root.join("scope/a")).unwrap().mode() & 0o777,
        0o751
    );
    let kept = disk::metadata(input.root.join("scope/c")).unwrap();
    assert_eq!(
        (kept.ino(), kept.ctime(), kept.ctime_nsec()),
        (untouched.ino(), untouched.ctime(), untouched.ctime_nsec())
    );
    assert_eq!(disk::read_dir(input.root.join("scope")).unwrap().count(), 4);
}

#[test]
fn into_materializes_new_or_existing_empty_but_never_clobbers() {
    let dir = Scratch::new("into");
    let input = dir.source("src");
    for existing in [false, true] {
        let dest = dir.0.join(if existing { "empty" } else { "new" });
        if existing {
            disk::create_dir(&dest).unwrap();
        }
        let result = apply(&input, Destination::Into(&dest));
        assert_eq!(result.status, Status::Complete, "{result:?}");
        assert_eq!(
            result.created_directories.iter().any(|p| p == "."),
            !existing
        );
        assert!(dest.join("scope/empty").is_dir());
        for (path, text) in ROWS {
            assert_eq!(disk::read(dest.join(path)).unwrap(), text.as_bytes());
        }
        assert_eq!(
            disk::metadata(dest.join("scope/a")).unwrap().mode() & 0o777,
            0o751
        );
        assert_eq!(
            apply(&input, Destination::Into(&dest)).status,
            Status::Refused
        );
    }
    for (path, text) in BASE {
        assert_eq!(disk::read(input.root.join(path)).unwrap(), text.as_bytes());
    }
}

#[test]
fn no_change_never_creates_even_an_empty_destination_or_stage() {
    let dir = Scratch::new("no-change");
    let input = dir.source("src");
    let output = dir.0.join("absent");
    let parent = input.root.join("scope");
    let before = disk::metadata(&parent).unwrap();
    for dest in [Destination::InPlace, Destination::Into(&output)] {
        let result = publish(&input, &BASE, &REVIEW, &REVIEW, dest);
        assert_eq!(result.status, Status::NoChange, "{result:?}");
        assert!(result.installed_files.is_empty());
        assert!(!output.exists());
    }
    let after = disk::metadata(parent).unwrap();
    assert_eq!(
        (before.ctime(), before.ctime_nsec()),
        (after.ctime(), after.ctime_nsec())
    );
}

#[test]
fn refuses_destination_links_overlap_and_unrelated_entries() {
    let dir = Scratch::new("refusals");
    let input = dir.source("src");
    let private = dir.0.join("private");
    disk::create_dir(&private).unwrap();
    let link = dir.0.join("link");
    symlink(&private, &link).unwrap();
    let unrelated = private.join("other");
    disk::write(&unrelated, b"unrelated").unwrap();
    for path in [
        &link,
        &link.join("new"),
        &private,
        &input.root,
        &dir.0,
        &input.root.join("scope/empty"),
        &input.root.join("new"),
    ] {
        let result = apply(&input, Destination::Into(path));
        assert_eq!(result.status, Status::Refused, "{path:?}: {result:?}");
        assert!(result.created_directories.is_empty());
    }
    assert_eq!(disk::read(unrelated).unwrap(), b"unrelated");
    assert!(!private.join("new").exists());
}

#[test]
fn stale_subject_review_or_negative_witness_refuses_before_effects() {
    let dir = Scratch::new("stale");
    let input = dir.source("src");
    let out = dir.0.join("absent");
    let destination = Destination::Into(&out);
    assert_eq!(
        publish(&input, &ROWS, &REVIEW, &[0; 32], destination).status,
        Status::Refused
    );
    assert_eq!(
        publish(&input, &ROWS[..2], &REVIEW, &REVIEW, destination).status,
        Status::Refused
    );
    disk::set_permissions(
        input.root.join("scope/c"),
        disk::Permissions::from_mode(0o701),
    )
    .unwrap();
    assert_eq!(apply(&input, destination).status, Status::Refused);
    let fresh = Directory::read(&input.root, "scope/").unwrap();
    disk::write(input.root.join("scope/extra"), "new outside evidence").unwrap();
    assert_eq!(apply(&fresh, destination).status, Status::Refused);
    assert!(!out.exists());
}

#[test]
fn refuses_swapped_symlink_and_hardlink_without_touching_victim() {
    let dir = Scratch::new("links");
    let victim = dir.0.join("victim");
    disk::write(&victim, "secret").unwrap();
    for (name, symbolic) in [("one", true), ("two", false)] {
        let input = dir.source(name);
        let path = input.root.join("scope/b");
        disk::remove_file(&path).unwrap();
        if symbolic {
            symlink(&victim, &path).unwrap();
        } else {
            disk::hard_link(&victim, &path).unwrap();
        }
        assert_eq!(apply(&input, Destination::InPlace).status, Status::Refused);
        assert_eq!(disk::read(input.root.join("scope/a")).unwrap(), b"old-a");
        assert_eq!(disk::read(&victim).unwrap(), b"secret");
    }
}

#[test]
fn injected_failure_after_actual_first_rename_reports_precise_prefix() {
    let dir = Scratch::new("partial");
    let input = dir.source("src");
    let mut effects = Publication::new();
    let error = run(
        &input,
        &ROWS,
        &REVIEW,
        &REVIEW,
        Destination::InPlace,
        &mut effects,
        |path| {
            assert_eq!(path, "scope/a");
            Err("injected after actual rename".into())
        },
    )
    .unwrap_err();
    effects.failed(error);
    assert_eq!(effects.status, Status::Partial);
    assert_eq!(effects.installed_files, ["scope/a"]);
    assert_eq!(disk::read(input.root.join("scope/a")).unwrap(), b"new-a");
    assert_eq!(disk::read(input.root.join("scope/b")).unwrap(), b"old-b");
    let stage = input
        .root
        .join(effects.staging_directories_remaining.first().unwrap());
    assert_eq!(disk::metadata(&stage).unwrap().mode() & 0o777, 0o700);
    assert_eq!(disk::read(stage.join("b")).unwrap(), b"new-b");
    assert!(
        effects
            .recovery
            .unwrap()
            .contains("Do not roll back or replay")
    );
    assert_eq!(apply(&input, Destination::InPlace).status, Status::Refused);
}

#[test]
fn abort_on_outsider_target_substitution_does_not_clobber_it() {
    let dir = Scratch::new("substitution");
    let input = dir.source("src");
    let output = dir.0.join("dest");
    let mut effects = Publication::new();
    let error = run(
        &input,
        &ROWS,
        &REVIEW,
        &REVIEW,
        Destination::Into(&output),
        &mut effects,
        |_| disk::write(output.join("scope/b"), "outsider").map_err(err),
    )
    .unwrap_err();
    effects.failed(error);
    assert_eq!(effects.status, Status::Partial);
    assert_eq!(effects.installed_files, ["scope/a"]);
    assert_eq!(effects.created_directories, [".", "scope", "scope/empty"]);
    assert_eq!(disk::read(output.join("scope/b")).unwrap(), b"outsider");
}

#[test]
fn failure_after_destination_creation_is_not_a_refusal() {
    let dir = Scratch::new("mkdir-partial");
    let input = dir.source("src");
    let dest = dir.0.join("output");
    let mut authority = authority::acquire(&input, Destination::Into(&dest)).unwrap();
    let mut effects = Publication::new();
    let (path, root) = authority
        .output
        .take()
        .unwrap()
        .enter(&mut effects)
        .unwrap();
    disk::write(dest.join("scope"), "intervening entry").unwrap();
    let failure = Materialized::create(path, root, &input, &mut effects)
        .err()
        .unwrap();
    effects.failed(failure);
    assert_eq!(effects.status, Status::Partial);
    assert!(effects.installed_files.is_empty());
    assert_eq!(effects.created_directories, ["."]);
    assert_eq!(
        disk::read(dest.join("scope")).unwrap(),
        b"intervening entry"
    );
}

#[test]
fn racing_cooperating_apply_cannot_enter_source_or_destination() {
    let dir = Scratch::new("race");
    let first = dir.source("one");
    let second = dir.source("two");
    let out = dir.0.join("empty");
    disk::create_dir(&out).unwrap();
    let mut effects = Publication::new();
    let result = run(
        &first,
        &ROWS,
        &REVIEW,
        &REVIEW,
        Destination::Into(&out),
        &mut effects,
        |path| {
            if path == "scope/a" {
                std::thread::scope(|threads| {
                    let source = threads.spawn(|| apply(&first, Destination::InPlace));
                    let target = threads.spawn(|| apply(&second, Destination::Into(&out)));
                    for worker in [source, target] {
                        let result = worker.join().unwrap();
                        assert_eq!(result.status, Status::Refused, "{result:?}");
                        assert!(result.diagnostic.unwrap().contains("lock unavailable"));
                    }
                });
            }
            Ok(())
        },
    );
    assert_eq!(result.unwrap(), Status::Complete);
}

#[test]
fn target_symlink_inserted_between_replacements_is_not_replaced_or_followed() {
    let dir = Scratch::new("late-link");
    let input = dir.source("src");
    let victim = dir.0.join("victim");
    disk::write(&victim, "outside").unwrap();
    let second = input.root.join("scope/b");
    let mut effects = Publication::new();
    let error = run(
        &input,
        &ROWS,
        &REVIEW,
        &REVIEW,
        Destination::InPlace,
        &mut effects,
        |_| {
            disk::remove_file(&second)
                .and_then(|()| symlink(&victim, &second))
                .map_err(err)
        },
    )
    .unwrap_err();
    effects.failed(error);
    assert_eq!(effects.status, Status::Partial);
    assert_eq!(effects.installed_files, ["scope/a"]);
    assert_eq!(disk::read_link(second).unwrap(), victim);
    assert_eq!(disk::read(&victim).unwrap(), b"outside");
}

#[test]
fn modified_staged_inode_is_not_the_reviewed_candidate() {
    let dir = Scratch::new("stage-conflict");
    let input = dir.source("src");
    let mut effects = Publication::new();
    let stage = input
        .root
        .join(format!("scope/.plyrune-stage~{}/b", std::process::id()));
    let error = run(
        &input,
        &ROWS,
        &REVIEW,
        &REVIEW,
        Destination::InPlace,
        &mut effects,
        |_| disk::write(&stage, "other").map_err(err),
    )
    .unwrap_err();
    effects.failed(error);
    assert_eq!(effects.status, Status::Partial);
    assert_eq!(effects.installed_files, ["scope/a"]);
    assert_eq!(disk::read(input.root.join("scope/b")).unwrap(), b"old-b");
}

#[test]
fn sigkill_after_first_rename_leaves_a_reconcilable_mixed_tree() {
    const CHILD: &str = "PLYRUNE_PUBLICATION_KILL_TEST_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let pid = rustix::process::getppid().unwrap().as_raw_nonzero();
        let root = std::env::temp_dir().join(format!("plyrune-publication-{pid}-sigkill/src"));
        assert_eq!(
            std::env::current_dir().unwrap(),
            root.canonicalize().unwrap()
        );
        let input = Directory::read(&root, "scope/").unwrap();
        let mut effects = Publication::new();
        let result = run(
            &input,
            &ROWS,
            &REVIEW,
            &REVIEW,
            Destination::InPlace,
            &mut effects,
            |path| {
                assert_eq!(path, "scope/a");
                rustix::process::kill_process(
                    rustix::process::getpid(),
                    rustix::process::Signal::KILL,
                )
                .map_err(err)
            },
        );
        panic!("child survived: {result:?}");
    }
    let dir = Scratch::new("sigkill");
    let input = dir.source("src");
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "publish::tests::sigkill_after_first_rename_leaves_a_reconcilable_mixed_tree",
        ])
        .env(CHILD, "1")
        .current_dir(&input.root)
        .status()
        .unwrap();
    assert_eq!(status.signal(), Some(9));
    assert_eq!(disk::read(input.root.join("scope/a")).unwrap(), b"new-a");
    assert_eq!(disk::read(input.root.join("scope/b")).unwrap(), b"old-b");
    assert_eq!(disk::read(input.root.join("scope/c")).unwrap(), b"keep");
    let result = apply(&input, Destination::InPlace);
    assert_eq!(result.status, Status::Refused, "{result:?}");
    assert!(!result.diagnostic.unwrap().contains("lock unavailable"));
}
