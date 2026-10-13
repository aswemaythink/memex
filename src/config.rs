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

/// What a keep_logs value asks for (design §4.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keep {
    /// Entries older than this many seconds are pruned.
    For(i64),
    /// `forever`: nothing is pruned.
    Forever,
    /// A value outside the grammar, as written: nothing is pruned.
    Invalid(String),
}

/// The settings `log` reads, and the warning to print once per run when the file could not be used.
pub struct Loaded {
    /// The keep_logs value as written, which prune reads with `parse_keep`. It is empty when the file cannot be used. Empty is not a valid value, so nothing is pruned.
    pub keep_logs: String,
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

/// Parses a keep_logs value: `forever`, or `Nm`, `Nh`, `Nd` or `Nw` (design §4.3). Anything else is `Invalid`.
pub fn parse_keep(raw: &str) -> Keep {
    if raw == "forever" {
        return Keep::Forever;
    }
    match clock::parse_duration(raw) {
        Some(seconds) => Keep::For(seconds),
        None => Keep::Invalid(raw.to_string()),
    }
}

/// Reads `<owner>/.memex/config.json`. A missing file means the default, with no warning. A file that cannot be read or parsed, including one with an unknown key, means no pruning, with one warning naming the file and the reason.
pub fn load(owner: &Path) -> Loaded {
    let path = owner.join(".memex").join("config.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        // A dangling symlink is present but unreadable, so it is not treated as a missing file.
        Err(failure) if failure.kind() == io::ErrorKind::NotFound && !is_symlink(&path) => {
            return defaults();
        }
        Err(failure) => {
            let reason = format!("cannot read it ({:?})", failure.kind());
            return unusable(warning(&path, &reason));
        }
    };
    match serde_json::from_slice::<File>(&bytes) {
        Ok(file) => judge(&path, file.keep_logs),
        Err(failure) => unusable(warning(&path, &failure.to_string())),
    }
}

/// True when `path` is a symlink, even one whose target does not exist.
fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

/// The default `3d`, with no warning: the store owner has no config.json.
fn defaults() -> Loaded {
    Loaded {
        keep_logs: DEFAULT_KEEP_LOGS.to_string(),
        warning: None,
    }
}

/// A config file that cannot be used. Its keep_logs is empty, which is no valid value, so nothing is pruned.
fn unusable(warning: String) -> Loaded {
    Loaded {
        keep_logs: String::new(),
        warning: Some(warning),
    }
}

/// A usable file's keep_logs, with a warning when the value is outside the grammar.
fn judge(path: &Path, keep_logs: String) -> Loaded {
    let problem = match parse_keep(&keep_logs) {
        Keep::Invalid(value) => Some(warning(
            path,
            &format!("invalid keep_logs '{value}': not pruning"),
        )),
        Keep::For(_) | Keep::Forever => None,
    };
    Loaded {
        keep_logs,
        warning: problem,
    }
}

/// The stderr line for a config file that cannot be used, or holds a value that is not valid.
fn warning(path: &Path, reason: &str) -> String {
    format!("memex: warning: {}: {reason}", slash(path))
}
