//! Reads the store owner's `.memex/config.json` (design §4.1 item 4).
//! It parses the keep_logs value (design §4.3). A missing file means the default `3d`. A file that cannot be used, a file that is not a JSON object, or a value outside the grammar, means nothing is pruned, and one warning says why.
//! Cases: DISC-CONFIG-01, DISC-CONFIG-02, DISC-CONFIG-03, DISC-CONFIG-04, LOG-PRUNE-01, LOG-PRUNE-02, LOG-PRUNE-06, LOG-PRUNE-07, LOG-PRUNE-08, LOG-PRUNE-09, LOG-PRUNE-16, LOG-PRUNE-18, PORT-NAME-04

use std::io;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::clock;
use crate::root::{lists, slash};
use crate::text;

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

/// Reads `<owner>/.memex/config.json`. The name is matched against the listing of `.memex/`, so `Config.json` is not the file
/// (design §4.4 Name matching). A missing file means the default, with no warning. A file that cannot be read or parsed, that is
/// not a JSON object, or that has an unknown key, means no pruning, with one warning naming the file and the reason.
pub fn load(owner: &Path) -> Loaded {
    let memex = owner.join(".memex");
    let path = memex.join("config.json");
    match lists(&memex, "config.json") {
        Ok(false) => judge(&path, DEFAULT_KEEP_LOGS),
        // A listed name that cannot be read, such as a dangling symlink, is a warning and not a missing file.
        Ok(true) => match text::read(&path) {
            Ok(contents) => parse(&path, &contents),
            Err(failure) => unreadable(&path, &failure),
        },
        Err(failure) => unreadable(&path, &failure),
    }
}

/// Parses the text of a config file. It must be a JSON object, whose one key is keep_logs. An array, string or number is not
/// read by position, so it warns and prunes nothing.
fn parse(path: &Path, contents: &str) -> Loaded {
    let value = match serde_json::from_str::<Value>(contents) {
        Ok(value) => value,
        Err(failure) => return unusable(warning(path, &failure.to_string())),
    };
    if !value.is_object() {
        return unusable(warning(path, "not a JSON object: not pruning"));
    }
    match serde_json::from_str::<File>(contents) {
        Ok(file) => judge(path, &file.keep_logs),
        Err(failure) => unusable(warning(path, &failure.to_string())),
    }
}

/// A config file that cannot be read: nothing is pruned, and the warning names the io::ErrorKind.
fn unreadable(path: &Path, failure: &io::Error) -> Loaded {
    let reason = format!("cannot read it ({:?})", failure.kind());
    unusable(warning(path, &reason))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn a_value_that_is_not_an_object_is_unusable_and_prunes_nothing() {
        for contents in [r#"["1h"]"#, r#""3d""#, "5", "null"] {
            let loaded = parse(Path::new("config.json"), contents);
            assert_eq!(loaded.keep, None, "{contents}");
            let line = loaded.warning.unwrap_or_default();
            assert!(line.ends_with("not a JSON object: not pruning"), "{line}");
        }
    }

    #[test]
    fn an_object_gives_its_keep_logs_and_the_default_without_one() {
        let two_days = Some(2 * 86_400);
        assert_eq!(
            parse(Path::new("config.json"), r#"{"keep_logs": "2d"}"#).keep,
            two_days
        );
        assert_eq!(parse(Path::new("config.json"), "{}").keep, Some(3 * 86_400));
    }

    #[test]
    fn a_config_named_in_another_case_is_not_the_file() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let memex = dir.path().join(".memex");
        fs::create_dir(&memex).expect("a memex folder");
        fs::write(memex.join("Config.json"), br#"{"keep_logs": "forever"}"#).expect("a config");
        let loaded = load(dir.path());
        assert_eq!(loaded.keep, Some(3 * 86_400));
        assert_eq!(loaded.warning, None);
    }
}
