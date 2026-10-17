//! Sets, restores and probes the `chmod` modes of a case.
//! It parses the octal modes, and tells whether the runner is root. Modes exist only on unix: the windows arms refuse a change or do nothing.
//! Cases: all (runner)

#[cfg(unix)]
use std::fs;
use std::io;
use std::path::Path;

/// The mode of a `chmod` entry: one to four octal digits, such as `000` or `555`. None for any other text.
pub fn parse_mode(text: &str) -> Option<u32> {
    let digits_ok =
        (1..=4).contains(&text.len()) && text.bytes().all(|b| (b'0'..=b'7').contains(&b));
    if digits_ok {
        u32::from_str_radix(text, 8).ok()
    } else {
        None
    }
}

/// Sets the permission bits of `path` to exactly `mode`.
#[cfg(unix)]
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

/// Windows has no modes to set, so a chmod entry is refused there.
#[cfg(not(unix))]
pub fn set_mode(_path: &Path, _mode: u32) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "chmod is unix-only",
    ))
}

/// Gives the owner back the rights it needs to delete what a chmod entry touched: rwx on each folder from `root` down to
/// `path`, and rw on a file. Folders go shallowest first, since a folder with no search right hides what lies below it.
#[cfg(unix)]
pub fn restore_owner(root: &Path, path: &Path) {
    let mut chain: Vec<&Path> = path
        .ancestors()
        .take_while(|dir| dir.starts_with(root) && *dir != root)
        .collect();
    chain.reverse();
    for dir in chain {
        add_owner_bits(dir);
    }
}

/// Windows keeps no modes that a sandbox could hold back from its own deletion, so there is nothing to restore.
#[cfg(not(unix))]
pub fn restore_owner(_root: &Path, _path: &Path) {}

/// Adds rwx (a folder) or rw (a file) to the owner bits of one path. A symlink is never followed, and a failure is left
/// for the temp folder's own removal to report.
#[cfg(unix)]
fn add_owner_bits(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    let owner = if meta.is_dir() { 0o700 } else { 0o600 };
    let mode = (meta.permissions().mode() | owner) & 0o7777;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

/// True when this process can read a file that has no mode bits, which only root can do. A chmod case cannot hold its
/// modes then, so it is an ignored trial. A probe that cannot be set up counts as not root.
pub fn running_as_root() -> bool {
    probe_reads_a_locked_file().unwrap_or(false)
}

/// Makes a file with mode 000 and reads it. Some(true) when the read works, Some(false) when it is refused, and None when
/// the probe cannot be set up.
#[cfg(unix)]
fn probe_reads_a_locked_file() -> Option<bool> {
    let dir = tempfile::tempdir().ok()?;
    let probe = dir.path().join("probe");
    fs::write(&probe, b"probe").ok()?;
    set_mode(&probe, 0o000).ok()?;
    Some(fs::read(&probe).is_ok())
}

/// Windows has no mode bits to lock a file with, so the probe has no answer and the runner is not treated as root.
#[cfg(not(unix))]
fn probe_reads_a_locked_file() -> Option<bool> {
    None
}
