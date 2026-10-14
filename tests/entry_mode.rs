//! Pins the mode of a new log entry, which goldens cannot show (PORT-MODE-01).
//! A new entry is 0644 before the umask (design §4.4 File modes), so it is 0644 less the umask. The binary runs under the
//! umask this run inherits, and under each umask in UMASKS, which `sh` sets for the child. The inherited umask is read back
//! from a file made by File::create, which gets 0o666 before the umask, so the test needs no libc.
//! Cases: PORT-MODE-01

#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The umasks the binary runs under: the usual one, a group-writable one, a private one and none.
const UMASKS: [u32; 4] = [0o022, 0o002, 0o077, 0o000];

/// The umask this run inherits, read back from a file made by File::create (0o666 before the umask).
fn inherited_umask(dir: &Path) -> u32 {
    let probe = dir.join("umask-probe");
    fs::File::create(&probe).expect("a probe file");
    let made = fs::metadata(&probe)
        .expect("the probe's metadata")
        .permissions()
        .mode()
        & 0o777;
    0o666 & !made
}

/// The permission bits of a path.
fn mode_of(path: &Path) -> u32 {
    fs::metadata(path)
        .expect("the entry's metadata")
        .permissions()
        .mode()
        & 0o777
}

/// Runs `memex log impl` in `dir` with a body on stdin, and returns the one entry it wrote. With a umask, `sh` sets it and
/// then runs memex, so memex inherits it.
fn log_entry(dir: &Path, umask: Option<u32>) -> PathBuf {
    let bin = env!("CARGO_BIN_EXE_memex");
    let mut command = match umask {
        Some(mask) => {
            let mut sh = Command::new("sh");
            sh.arg("-c")
                .arg(format!("umask {mask:03o} && exec \"$0\" \"$@\""))
                .arg(bin);
            sh
        }
        None => Command::new(bin),
    };
    let mut child = command
        .args(["log", "impl"])
        .current_dir(dir)
        .env("MEMEX_NOW", "2026-10-13T12:00:00Z")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("memex starts");
    child
        .stdin
        .take()
        .expect("a stdin pipe")
        .write_all(b"# Mode check\n\nbody\n")
        .expect("the body is written");
    let out = child.wait_with_output().expect("memex finishes");
    assert_eq!(
        out.status.code(),
        Some(0),
        "log failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let entries: Vec<PathBuf> = fs::read_dir(dir.join(".memex/logs/impl"))
        .expect("the stream folder")
        .map(|entry| entry.expect("a folder entry").path())
        .collect();
    assert_eq!(entries.len(), 1, "one entry is written: {entries:?}");
    entries.into_iter().next().expect("the entry")
}

/// A fresh store folder in a fresh temp folder, which a run of `log` can write to.
fn store() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    fs::create_dir(dir.path().join(".memex")).expect("a store folder");
    dir
}

#[test]
fn a_new_log_entry_is_0644_under_the_runners_umask() {
    let dir = store();
    let umask = inherited_umask(dir.path());
    let entry = log_entry(dir.path(), None);
    assert_eq!(
        mode_of(&entry),
        0o644 & !umask,
        "a new entry should be 0644 under umask {umask:03o}"
    );
}

#[test]
fn a_new_log_entry_is_0644_before_the_umask_whatever_the_umask() {
    for umask in UMASKS {
        let dir = store();
        let entry = log_entry(dir.path(), Some(umask));
        assert_eq!(
            mode_of(&entry),
            0o644 & !umask,
            "a new entry should be 0644 before umask {umask:03o}, and it is {:04o}",
            mode_of(&entry)
        );
    }
}
