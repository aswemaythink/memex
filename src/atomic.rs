//! Makes the temp file that every atomic write goes through (design §4.4 Atomic writes and File modes).
//! It is dot-prefixed, in the folder of the file it becomes, and made with an ordinary file's mode, so a persisted file is one.
//! Cases: PORT-MODE-01, INIT-FRESH-01

use std::fs;
use std::io;
use std::path::Path;

use tempfile::NamedTempFile;

/// A dot-prefixed temp file in `dir`, made with `permissions`, else with those of any new file: 0o644 before the umask on unix,
/// not tempfile's private 0o600. It is removed when dropped, unless it is persisted.
pub fn temp_in(dir: &Path, permissions: Option<fs::Permissions>) -> io::Result<NamedTempFile> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".");
    if let Some(permissions) = permissions.or_else(new_file_mode) {
        builder.permissions(permissions);
    }
    builder.tempfile_in(dir)
}

/// The mode a new file gets before the umask: 0o644 on unix (design §4.4 File modes).
#[cfg(unix)]
fn new_file_mode() -> Option<fs::Permissions> {
    use std::os::unix::fs::PermissionsExt;
    Some(fs::Permissions::from_mode(0o644))
}

/// Windows has no mode to set: a temp file is an ordinary writable file already.
#[cfg(not(unix))]
fn new_file_mode() -> Option<fs::Permissions> {
    None
}
