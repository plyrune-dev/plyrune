use super::*;
use std::ffi::OsStr;
use std::fs as host;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt, symlink};
use std::sync::atomic::{AtomicU64, Ordering};

struct Tree(PathBuf);

impl Tree {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("plyrune-dir-{}-{n}", std::process::id()));
            match host::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => {
                    host::create_dir(path.join("s")).unwrap();
                    return Self(path);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("create test tree: {e}"),
            }
        }
    }

    fn read(&self) -> Result<Directory, String> {
        Directory::read(&self.0, "s/")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        host::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn captures_hidden_files_empty_directories_and_exact_text_without_root_writes() {
    let tree = Tree::new();
    let scope = tree.0.join("s");
    let file = scope.join(".env");
    host::write(&file, "á\r\n").unwrap();
    host::set_permissions(&file, host::Permissions::from_mode(0o541)).unwrap();
    host::create_dir(scope.join("empty")).unwrap();
    symlink("/absent", tree.0.join("broken-outside")).unwrap();
    host::set_permissions(&tree.0, host::Permissions::from_mode(0o555)).unwrap();
    let result = tree.read();
    host::set_permissions(&tree.0, host::Permissions::from_mode(0o700)).unwrap();
    let captured = result.unwrap();
    let fixture = captured.fixture().unwrap();
    assert_eq!(
        fixture.resources().collect::<Vec<_>>(),
        [("s/.env", "á\r\n")]
    );
    assert!(captured.dirs.contains_key("s/empty"));
    let meta = &captured.files["s/.env"].meta;
    assert_eq!(*meta, FileMeta::from_stat(&file.metadata().unwrap()));
    assert_eq!(meta.mode, 0o541);
}

#[test]
fn identity_and_recheck_cover_topology_inode_and_permissions_not_directory_timestamps() {
    let tree = Tree::new();
    let target = tree.0.join("s/content");
    host::write(&target, b"x").unwrap();
    let src = tree.read().unwrap();
    let id = src.identity();
    let expected = src.fixture().unwrap().digest();
    let parent = src.parent_for("s/content").unwrap();
    src.recheck_file(&parent, "s/content").unwrap();
    host::write(tree.0.join("outside"), b"unrelated").unwrap();
    assert_eq!(id, tree.read().unwrap().identity());
    src.recheck().unwrap();
    let empty = tree.0.join("s/empty");
    host::create_dir(&empty).unwrap();
    assert!(src.recheck().is_err());
    let changed = tree.read().unwrap();
    assert_ne!(id, changed.identity());
    assert_eq!(expected, changed.fixture().unwrap().digest());
    host::remove_dir(&empty).unwrap();
    assert!(changed.recheck().is_err());
    src.recheck().unwrap();
    host::write(tree.0.join("other"), b"x").unwrap();
    host::rename(tree.0.join("other"), &target).unwrap();
    assert!(src.recheck_file(&parent, "s/content").is_err());
    assert!(src.recheck().is_err());
    let renamed = tree.read().unwrap();
    assert_ne!(id, renamed.identity());
    assert_eq!(expected, renamed.fixture().unwrap().digest());
    host::set_permissions(&target, host::Permissions::from_mode(0o741)).unwrap();
    assert!(renamed.recheck().is_err());
    assert_ne!(renamed.identity(), tree.read().unwrap().identity());
}

#[test]
fn pinned_parent_cannot_authorize_a_detached_directory() {
    let tree = Tree::new();
    host::write(tree.0.join("s/f"), b"x").unwrap();
    let src = tree.read().unwrap();
    let parent = src.parent_for("s/f").unwrap();
    host::rename(tree.0.join("s"), tree.0.join("old")).unwrap();
    host::create_dir(tree.0.join("s")).unwrap();
    assert!(src.recheck_file(&parent, "s/f").is_err());
}

#[test]
fn refuses_root_prefix_and_descendant_symlinks_even_for_a_directory() {
    let tree = Tree::new();
    let inner = tree.0.join("s");
    host::create_dir(inner.join("sub")).unwrap();
    host::create_dir(inner.join("sub/nested")).unwrap();
    symlink("s", tree.0.join("alias")).unwrap();
    assert!(Directory::read(&tree.0.join("alias"), "sub/").is_err());
    assert!(Directory::read(&tree.0.join("alias/sub"), "nested/").is_err());
    assert!(Directory::read(&tree.0, "alias/sub/").is_err());
    symlink("sub", inner.join("link")).unwrap();
    assert!(tree.read().is_err());
    host::remove_file(inner.join("link")).unwrap();
    tree.read().unwrap();
}

#[test]
fn rejects_fifo_socket_multiple_links_special_bits_and_extended_attributes() {
    let tree = Tree::new();
    let entry = tree.0.join("s/item");
    fs::mkfifoat(fs::CWD, &entry, Mode::RWXU).unwrap();
    assert!(
        tree.read()
            .unwrap_err()
            .contains("unsupported source entry")
    );
    host::remove_file(&entry).unwrap();
    let socket = std::os::unix::net::UnixListener::bind(&entry).unwrap();
    assert!(
        tree.read()
            .unwrap_err()
            .contains("unsupported source entry")
    );
    drop(socket);
    host::remove_file(&entry).unwrap();
    host::write(&entry, b"ok").unwrap();
    host::hard_link(&entry, tree.0.join("outside")).unwrap();
    assert!(tree.read().unwrap_err().contains("single-link"));
    host::remove_file(tree.0.join("outside")).unwrap();
    host::set_permissions(&entry, host::Permissions::from_mode(0o4755)).unwrap();
    assert!(tree.read().unwrap_err().contains("special permission bits"));
    host::set_permissions(&entry, host::Permissions::from_mode(0o644)).unwrap();
    tree.read().unwrap();
    fs::setxattr(&entry, "user.plyrune", b"1", fs::XattrFlags::CREATE).unwrap();
    assert!(tree.read().unwrap_err().contains("extended attributes"));
}

#[test]
fn refuses_unrepresentable_names_text_and_prefixes_without_skipping_them() {
    let tree = Tree::new();
    for prefix in ["", "s", "/s/", "s//", "s/./", "s/../", "s/space name/"] {
        assert!(Directory::read(&tree.0, prefix).is_err());
    }
    let bad_name = tree.0.join("s").join(OsStr::from_bytes(b"bad\xff"));
    host::write(&bad_name, b"ok").unwrap();
    assert!(tree.read().unwrap_err().contains("non-UTF-8 name"));
    host::remove_file(bad_name).unwrap();
    host::write(tree.0.join("s/ok"), b"ok").unwrap();
    host::write(tree.0.join("s/bad"), [0xff]).unwrap();
    assert!(tree.read().unwrap_err().contains("not UTF-8"));
}

#[test]
fn bounds_empty_directory_count_and_depth_and_statted_lengths() {
    let tree = Tree::new();
    let dir = tree.0.join("s");
    for i in 0..MAX_DIRECTORIES {
        host::create_dir(dir.join(i.to_string())).unwrap();
    }
    assert!(tree.read().unwrap_err().contains("directory budget"));
    host::remove_dir_all(&dir).unwrap();
    host::create_dir(&dir).unwrap();
    let mut deepest = dir.clone();
    for _ in 0..MAX_DEPTH {
        deepest.push("d");
    }
    host::create_dir_all(deepest).unwrap();
    assert!(tree.read().unwrap_err().contains("depth budget"));
    host::remove_dir_all(dir.join("d")).unwrap();
    let file = File::create(dir.join("large")).unwrap();
    file.set_len(MAX_FILE + 1).unwrap();
    assert!(tree.read().unwrap_err().contains("8 MiB"));
    file.set_len(MAX_FILE).unwrap();
    for i in 0..4 {
        File::create(dir.join(i.to_string()))
            .unwrap()
            .set_len(MAX_FILE)
            .unwrap();
    }
    assert!(tree.read().unwrap_err().contains("32 MiB"));
}
