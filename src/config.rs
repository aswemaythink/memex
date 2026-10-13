//! Reads the store owner's `.memex/config.json` (design §4.1 item 4): its keep_logs setting, the default `3d`, and one warning line when the file cannot be used.
//! Cases: DISC-CONFIG-01, DISC-CONFIG-02

use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use crate::root::slash;

/// The keep_logs default of design §4.3: entries older than three days are pruned.
const DEFAULT_KEEP_LOGS: &str = "3d";

/// The settings `log` reads, and the warning to print once per run when the file could not be used.
pub struct Loaded {
    /// The keep_logs value, as written; S1.3 parses it.
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

/// Reads `<owner>/.memex/config.json`. A missing file means the defaults with no warning. A file that
/// is not valid, or cannot be read, means the defaults and one warning naming the file and the reason.
pub fn load(owner: &Path) -> Loaded {
    let path = owner.join(".memex").join("config.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return defaults(None),
        Err(failure) => {
            let reason = format!("cannot read it ({:?})", failure.kind());
            return defaults(Some(warning(&path, &reason)));
        }
    };
    match serde_json::from_slice::<File>(&bytes) {
        Ok(file) => Loaded {
            keep_logs: file.keep_logs,
            warning: None,
        },
        Err(failure) => defaults(Some(warning(&path, &failure.to_string()))),
    }
}

/// The defaults, with the warning that explains why they are used, if there is one.
fn defaults(warning: Option<String>) -> Loaded {
    Loaded {
        keep_logs: DEFAULT_KEEP_LOGS.to_string(),
        warning,
    }
}

/// The stderr line for a config file that cannot be used.
fn warning(path: &Path, reason: &str) -> String {
    format!("memex: warning: {}: {reason}", slash(path))
}
