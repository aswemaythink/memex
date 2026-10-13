//! Reads the store owner's `.memex/config.json` (design §4.1 item 4) and parses its keep_logs value (design §4.3). A missing file means the default `3d`. A file that cannot be used, or a value outside the grammar, means nothing is pruned, and one warning says why.
//! Cases: DISC-CONFIG-01, DISC-CONFIG-02, LOG-PRUNE-01, LOG-PRUNE-02, LOG-PRUNE-06, LOG-PRUNE-07, LOG-PRUNE-08, LOG-PRUNE-09, LOG-PRUNE-16

use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use crate::clock;
use crate::root::slash;

/// The keep_logs default of design §4.3: entries older than three days are pruned.
const DEFAULT_KEEP_LOGS: &str = "3d";

/// The settings `log` reads, and the warning to print once per run when the file could not be used.
pub struct Loaded {
    /// Entries older than this many seconds are pruned. None prunes nothing: keep_logs is `forever`, outside the grammar, or in a file that cannot be used.
    pub keep: Option<i64>,
    /// The full stderr line, `memex: warning: <path>: <reason>`, or None.
    pub warning: Option<String>,
}

/// The keys of config.json. An unknown key is an error, so a typo is reported and not ignored.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default = "default_keep_logs")]
    keep_logs: String,
}

fn default_keep_logs() -> String {
    DEFAULT_KEEP_LOGS.to_string()
}

/// Reads `<owner>/.memex/config.json`. A missing file means the default, with no warning. A file that cannot be read or parsed, including one with an unknown key, means no pruning, with one warning naming the file and the reason.
pub fn load(owner: &Path) -> Loaded {
    let path = owner.join(".memex").join("config.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        // A dangling symlink is present but unreadable, so it is not treated as a missing file.
        Err(failure) if failure.kind() == io::ErrorKind::NotFound && !is_symlink(&path) => {
            return judge(&path, DEFAULT_KEEP_LOGS);
        }
        Err(failure) => {
            let reason = format!("cannot read it ({:?})", failure.kind());
            return unusable(warning(&path, &reason));
        }
    };
    match serde_json::from_slice::<File>(&bytes) {
        Ok(file) => judge(&path, &file.keep_logs),
        Err(failure) => unusable(warning(&path, &failure.to_string())),
    }
}

/// True when `path` is a symlink, even one whose target does not exist.
fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

/// A config file that cannot be used: nothing is pruned, and the warning says why.
fn unusable(warning: String) -> Loaded {
    Loaded {
        keep: None,
        warning: Some(warning),
    }
}

/// Parses a keep_logs value: `forever` keeps everything, `Nm`, `Nh`, `Nd` or `Nw` keeps that long (design §4.3), and anything else warns and prunes nothing.
fn judge(path: &Path, keep_logs: &str) -> Loaded {
    if keep_logs == "forever" {
        return Loaded {
            keep: None,
            warning: None,
        };
    }
    match clock::parse_duration(keep_logs) {
        Some(seconds) => Loaded {
            keep: Some(seconds),
            warning: None,
        },
        None => unusable(warning(
            path,
            &format!("invalid keep_logs '{keep_logs}': not pruning"),
        )),
    }
}

/// The stderr line for a config file that cannot be used, or holds a value that is not valid.
fn warning(path: &Path, reason: &str) -> String {
    format!("memex: warning: {}: {reason}", slash(path))
}
