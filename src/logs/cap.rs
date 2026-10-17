//! Prunes the store to its size cap after the age pass (design §4.3 Size cap).
//! It stats each entry once, in parallel, and counts the regular files by length. Then it deletes the oldest, by entry name and then by stream name in byte order, until the total is at or under the cap. The entry just written is never deleted, though its bytes count. A refused delete warns and its bytes still count; a path that is gone takes its bytes off the total, silently.
//! Cases: LOG-CAP-01, LOG-CAP-02, LOG-CAP-03, LOG-CAP-04, LOG-CAP-05, LOG-CAP-06, LOG-CAP-11, LOG-CAP-12, LOG-CAP-13, LOG-CAP-14

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::logs::prune::{remove_if_empty, warn};
use crate::par;

/// A regular file that the size pass counts: its path and its length in bytes.
struct Counted {
    path: PathBuf,
    bytes: u64,
}

impl Counted {
    /// The deletion order: the entry name, then the name of its stream folder, both in byte order. The stamp leads an entry name,
    /// so the order is oldest first (design §4.3 Size cap).
    fn order(&self) -> (OsString, OsString) {
        let name = self.path.file_name().map(OsStr::to_os_string);
        let stream = self
            .stream()
            .and_then(Path::file_name)
            .map(OsStr::to_os_string);
        (name.unwrap_or_default(), stream.unwrap_or_default())
    }

    /// The stream folder that holds the entry.
    fn stream(&self) -> Option<&Path> {
        self.path.parent()
    }
}

/// Deletes the oldest of `entries` until their total is at or under `cap` bytes, and never `written`. `entries` are the paths that
/// the age pass left. A path that is not a regular file counts for nothing and is never touched (design §4.3 What counts).
pub fn prune_to(entries: Vec<PathBuf>, cap: u64, written: &Path) {
    let mut counted = measure(entries);
    let total = counted
        .iter()
        .fold(0_u64, |sum, entry| sum.saturating_add(entry.bytes));
    if total > cap {
        sweep(&mut counted, total, cap, written);
    }
}

/// The regular files among `entries`, with their lengths, in the order given. Each path is stat'd once, in parallel. A path that is
/// gone counts for nothing, silently; any other failure to read one warns and counts for nothing (design §4.3 Vanished entries).
fn measure(entries: Vec<PathBuf>) -> Vec<Counted> {
    let lengths = par::map(&entries, |path| regular_len(path));
    entries
        .into_iter()
        .zip(lengths)
        .filter_map(|(path, length)| match length {
            Ok(Some(bytes)) => Some(Counted { path, bytes }),
            Ok(None) => None,
            Err(failure) => {
                warn("cannot read the entry", &path, &failure);
                None
            }
        })
        .collect()
}

/// The length of `path` when it is a regular file. It is read with `symlink_metadata`, so a symlink is described and never
/// followed. None for any other kind of path.
fn regular_len(path: &Path) -> io::Result<Option<u64>> {
    let meta = fs::symlink_metadata(path)?;
    Ok(meta.is_file().then_some(meta.len()))
}

/// Deletes `counted` in order, oldest first, until `total` is at or under `cap`. The written entry is passed over. A delete that
/// succeeds, or finds its path gone, takes its bytes off the total. Any other failure warns, and its bytes stay in the total
/// (design §4.3 Size cap). Then each stream folder that a delete emptied is removed (design §4.3 Folders).
fn sweep(counted: &mut [Counted], mut total: u64, cap: u64, written: &Path) {
    counted.sort_by_cached_key(Counted::order);
    let mut emptied = Vec::new();
    for entry in counted.iter() {
        if total <= cap {
            break;
        }
        if entry.path.as_path() == written {
            continue;
        }
        match fs::remove_file(&entry.path) {
            Ok(()) => {
                total = total.saturating_sub(entry.bytes);
                emptied.extend(entry.stream().map(Path::to_path_buf));
            }
            Err(failure) if failure.kind() == io::ErrorKind::NotFound => {
                total = total.saturating_sub(entry.bytes);
            }
            Err(failure) => warn("cannot delete the entry", &entry.path, &failure),
        }
    }
    emptied.sort();
    emptied.dedup();
    for dir in &emptied {
        remove_if_empty(dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::prune::PRINTED;

    /// Writes a regular file of `bytes` zero bytes at `path`, making its folder first.
    fn file(path: &Path, bytes: usize) {
        fs::create_dir_all(path.parent().expect("a folder")).expect("a stream folder");
        fs::write(path, vec![0_u8; bytes]).expect("a file");
    }

    /// Empties `PRINTED` on this thread, so that a test sees only what its own run printed.
    fn clear() {
        PRINTED.with(|printed| printed.borrow_mut().clear());
    }

    /// The warning lines that `PRINTED` holds now.
    fn lines() -> Vec<String> {
        PRINTED.with(|printed| printed.borrow().clone())
    }

    #[test]
    fn the_oldest_name_goes_first_and_the_written_entry_stays() {
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let written = store.join("impl/20261013-120000-new.md");
        let entries = [
            store.join("b/20261009-120000-aa.md"),
            store.join("a/20261009-120000-zz.md"),
            store.join("impl/20261010-120000-mid.md"),
            written.clone(),
        ];
        for entry in &entries {
            file(entry, 100);
        }
        // Four entries of 100 bytes, cap 250: the two oldest by name go, "aa" from b before "zz" from a.
        prune_to(entries.to_vec(), 250, &written);
        assert!(!entries[0].exists(), "b/...-aa.md is the oldest by name");
        assert!(!entries[1].exists(), "a/...-zz.md goes second");
        assert!(entries[2].is_file(), "the next-oldest is under the cap");
        assert!(written.is_file(), "the written entry stays");
        assert!(lines().is_empty(), "{:?}", lines());
    }

    #[test]
    fn equal_names_go_by_stream_name_in_byte_order() {
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let written = store.join("impl/20261013-120000-new.md");
        let same_a = store.join("a/20261009-120000-same.md");
        let same_b = store.join("b/20261009-120000-same.md");
        let zz_a = store.join("a/20261009-120000-zz.md");
        for entry in [&same_a, &same_b, &zz_a, &written] {
            file(entry, 100);
        }
        // Cap 350 with 400 bytes: one delete is enough, and it is a/...-same.md (name first, then stream a before b).
        let entries = vec![
            same_a.clone(),
            same_b.clone(),
            zz_a.clone(),
            written.clone(),
        ];
        prune_to(entries, 350, &written);
        assert!(!same_a.exists(), "a/...-same.md goes");
        assert!(same_b.is_file(), "b/...-same.md stays");
        assert!(zz_a.is_file(), "a/...-zz.md stays");
        assert!(written.is_file(), "the written entry stays");
    }

    #[test]
    fn the_written_entry_is_never_deleted_even_when_oldest() {
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let written = store.join("impl/20261001-120000-first.md");
        let later = store.join("a/20261009-120000-later.md");
        file(&written, 100);
        file(&later, 100);
        // The written entry counts (200 > 150), is passed over as the oldest, and the next one goes.
        prune_to(vec![written.clone(), later.clone()], 150, &written);
        assert!(written.is_file(), "the written entry is never deleted");
        assert!(!later.exists(), "the next-oldest goes instead");
    }

    #[cfg(unix)]
    #[test]
    fn a_refused_delete_warns_keeps_its_bytes_and_the_next_one_goes() {
        use std::os::unix::fs::PermissionsExt;
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let refused = store.join("old/20261001-120000-refused.md");
        let next = store.join("b/20261002-120000-next.md");
        let written = store.join("impl/20261013-120000-new.md");
        for entry in [&refused, &next, &written] {
            file(entry, 100);
        }
        let old = store.join("old");
        fs::set_permissions(&old, fs::Permissions::from_mode(0o555)).expect("read-only");
        // A user who may still write to the read-only folder, such as root, deletes the entry: there is nothing to test then.
        if fs::write(old.join("probe"), b"").is_ok() {
            fs::set_permissions(&old, fs::Permissions::from_mode(0o755)).expect("restored");
            return;
        }
        // 300 bytes over a cap of 250: the refused entry keeps its 100 bytes, so the next one goes too.
        prune_to(
            vec![refused.clone(), next.clone(), written.clone()],
            250,
            &written,
        );
        fs::set_permissions(&old, fs::Permissions::from_mode(0o755)).expect("restored");
        assert!(refused.is_file(), "a refused entry stays");
        assert!(!next.exists(), "the next-oldest is deleted instead");
        let printed = lines();
        assert_eq!(printed.len(), 1, "{printed:?}");
        assert!(
            printed[0].contains("prune: cannot delete the entry (PermissionDenied)"),
            "{printed:?}"
        );
    }

    #[test]
    fn a_path_gone_at_delete_subtracts_its_bytes_silently() {
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let gone = store.join("a/20261001-120000-gone.md");
        let real = store.join("b/20261002-120000-real.md");
        let written = store.join("impl/20261013-120000-new.md");
        file(&real, 2_000);
        file(&written, 100);
        // The total is 2,600 with a cap of 1,000: the vanished 500 bytes are gone, so the 2,000 real bytes must go too.
        let mut counted = vec![
            Counted {
                path: gone,
                bytes: 500,
            },
            Counted {
                path: real.clone(),
                bytes: 2_000,
            },
            Counted {
                path: written.clone(),
                bytes: 100,
            },
        ];
        sweep(&mut counted, 2_600, 1_000, &written);
        assert!(
            !real.exists(),
            "the real entry goes once the vanished one is counted gone"
        );
        assert!(written.is_file(), "the written entry stays");
        assert!(
            !store.join("b").exists(),
            "the emptied stream folder is removed"
        );
        assert!(lines().is_empty(), "{:?}", lines());
    }

    #[test]
    fn only_regular_files_are_counted_and_a_gone_path_counts_for_nothing() {
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let real = store.join("impl/20261001-120000-real.md");
        file(&real, 300);
        let folder = store.join("impl/20261002-120000-folder.md");
        fs::create_dir_all(&folder).expect("a folder named like an entry");
        let missing = store.join("impl/20261003-120000-missing.md");
        let counted = measure(vec![real.clone(), folder, missing]);
        assert_eq!(
            counted.len(),
            1,
            "a folder and a missing path count for nothing"
        );
        assert_eq!(counted[0].path, real);
        assert_eq!(counted[0].bytes, 300);
        assert!(lines().is_empty(), "{:?}", lines());
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_counts_for_nothing_and_is_never_followed() {
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path();
        let target = store.join("outside/20261001-120000-target.md");
        file(&target, 5_000);
        let alias = store.join("impl/20261001-120000-alias.md");
        fs::create_dir_all(store.join("impl")).expect("a stream folder");
        std::os::unix::fs::symlink(&target, &alias).expect("a symlink");
        let counted = measure(vec![alias.clone()]);
        assert!(counted.is_empty(), "a symlink is not a regular file");
        assert!(alias.is_symlink(), "the symlink stays");
        assert!(target.is_file(), "what it points at stays");
    }

    #[cfg(unix)]
    #[test]
    fn a_stat_failure_other_than_not_found_warns_and_counts_nothing() {
        use std::os::unix::fs::PermissionsExt;
        clear();
        let root = tempfile::tempdir().expect("a temporary folder");
        let stream = root.path().join("impl");
        let entry = stream.join("20261001-120000-hidden.md");
        file(&entry, 400);
        // Without search permission the folder can be listed but not stat'd: the lstat of the entry is refused.
        fs::set_permissions(&stream, fs::Permissions::from_mode(0o444)).expect("no search");
        if fs::symlink_metadata(&entry).is_ok() {
            fs::set_permissions(&stream, fs::Permissions::from_mode(0o755)).expect("restored");
            return;
        }
        let counted = measure(vec![entry.clone()]);
        fs::set_permissions(&stream, fs::Permissions::from_mode(0o755)).expect("restored");
        assert!(
            counted.is_empty(),
            "an entry that cannot be read counts for nothing"
        );
        let printed = lines();
        assert_eq!(printed.len(), 1, "{printed:?}");
        assert!(
            printed[0].contains("prune: cannot read the entry (PermissionDenied)"),
            "{printed:?}"
        );
    }
}
