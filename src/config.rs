//! Reads the two limits of `.memex/config.json` (design §4.1 item 4): keep_logs and max_logs_mb.
//! A missing file means no age limit and the default 10 MiB cap. A file that cannot be used prunes nothing at all, and one warning says why. A value outside its grammar disables only its own limit, and its warning names it (design §4.3 Bad config).
//! Cases: DISC-CONFIG-01, DISC-CONFIG-02, DISC-CONFIG-03, DISC-CONFIG-04, LOG-PRUNE-01, LOG-PRUNE-02, LOG-PRUNE-06, LOG-PRUNE-07, LOG-PRUNE-08, LOG-PRUNE-09, LOG-PRUNE-16, LOG-PRUNE-18, LOG-PRUNE-21, LOG-CAP-02, LOG-CAP-07, LOG-CAP-08, LOG-CAP-09, LOG-CAP-10, PORT-NAME-04

use std::io;
use std::path::Path;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::clock;
use crate::error;
use crate::root::{lists, slash};
use crate::text;

/// One mebibyte in bytes, the unit of max_logs_mb (design §4.3 Size cap).
const MIB: u64 = 1_048_576;
/// The max_logs_mb default of design §4.3: the store is capped at 10 MiB.
const DEFAULT_MAX_LOGS_MB: u64 = 10;

/// The two limits `log` reads, and the warnings to print once per run, in the order they were found.
pub struct Loaded {
    /// Entries older than this many seconds are pruned by age. None means no age limit: keep_logs is absent, `forever`, outside
    /// the grammar, or in a file that cannot be used.
    pub keep: Option<i64>,
    /// The store is pruned to at or under this many bytes. None means no size pruning: max_logs_mb is outside the grammar, or the
    /// file cannot be used.
    pub cap: Option<u64>,
    /// Full stderr lines, `memex: warning: <path>: <reason>` or `memex: warning: cannot read <path> (<ErrorKind>)`. A file that
    /// cannot be used has one; otherwise each invalid value has its own, keep_logs first.
    pub warnings: Vec<String>,
}

/// The keys of config.json, each read as its raw value so that `null` is a value and not an absent key. An unknown key is an
/// error, so a typo is reported and not ignored, and serde's message names both keys.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default, deserialize_with = "present")]
    keep_logs: Option<Value>,
    #[serde(default, deserialize_with = "present")]
    max_logs_mb: Option<Value>,
}

/// Reads a key that is there as its value, `null` included. `Option`'s own reading would make `null` the same as no key.
fn present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}

/// Reads `<owner>/.memex/config.json`. The name is matched against the listing of `.memex/`, so `Config.json` is not the file
/// (design §4.4 Name matching). A missing file means the defaults, with no warning. A file that cannot be read or parsed, that is
/// not a JSON object, or that has an unknown key, prunes nothing, with one warning naming the file and the reason.
pub fn load(owner: &Path) -> Loaded {
    let memex = owner.join(".memex");
    let path = memex.join("config.json");
    match lists(&memex, "config.json") {
        Ok(false) => defaults(),
        // A listed name that cannot be read, such as a dangling symlink, is a warning and not a missing file.
        Ok(true) => match text::read(&path) {
            Ok(contents) => parse(&path, &contents),
            Err(failure) => unreadable(&path, &failure),
        },
        Err(failure) => unreadable(&path, &failure),
    }
}

/// The limits of a folder with no config.json: no age limit, and the default size cap (design §4.3).
fn defaults() -> Loaded {
    Loaded {
        keep: None,
        cap: Some(DEFAULT_MAX_LOGS_MB * MIB),
        warnings: Vec::new(),
    }
}

/// Parses the text of a config file. It must be a JSON object, and its keys must be the two known ones. An array, string or number
/// is not read by position, so it warns and prunes nothing.
fn parse(path: &Path, contents: &str) -> Loaded {
    let value = match serde_json::from_str::<Value>(contents) {
        Ok(value) => value,
        Err(failure) => return unusable(warning(path, &failure.to_string())),
    };
    if !value.is_object() {
        return unusable(warning(path, "not a JSON object: not pruning"));
    }
    match serde_json::from_str::<File>(contents) {
        Ok(file) => judge(path, &file),
        Err(failure) => unusable(warning(path, &failure.to_string())),
    }
}

/// A config file that cannot be read: nothing is pruned, and the warning names the path and the io::ErrorKind.
fn unreadable(path: &Path, failure: &io::Error) -> Loaded {
    let message = error::Error::io_at("cannot read", path, failure).message;
    unusable(error::warning(&message))
}

/// A config file that cannot be used: nothing is pruned, by age or by size, and the warning says why.
fn unusable(warning: String) -> Loaded {
    Loaded {
        keep: None,
        cap: None,
        warnings: vec![warning],
    }
}

/// Judges the two values of a file that parsed. Each invalid value warns and disables only its own limit (design §4.3 Bad config).
fn judge(path: &Path, file: &File) -> Loaded {
    let (keep, keep_warning) = keep_limit(path, file.keep_logs.as_ref());
    let (cap, cap_warning) = cap_limit(path, file.max_logs_mb.as_ref());
    Loaded {
        keep,
        cap,
        warnings: keep_warning.into_iter().chain(cap_warning).collect(),
    }
}

/// The age limit of keep_logs (design §4.3 Age limit): `forever` and no key mean none, and `Nm`, `Nh`, `Nd` or `Nw` is that many
/// seconds. Any other value warns, and gives no age limit.
fn keep_limit(path: &Path, value: Option<&Value>) -> (Option<i64>, Option<String>) {
    let Some(value) = value else {
        return (None, None);
    };
    if value.as_str() == Some("forever") {
        return (None, None);
    }
    match value.as_str().and_then(clock::parse_duration) {
        Some(seconds) => (Some(seconds), None),
        None => (
            None,
            Some(invalid(path, "keep_logs", value, "not pruning by age")),
        ),
    }
}

/// The size cap of max_logs_mb (design §4.3 Size cap): no key means the default, and a whole number of MiB from 1 is that many
/// bytes, if they fit in a u64. Any other value warns, and gives no size cap.
fn cap_limit(path: &Path, value: Option<&Value>) -> (Option<u64>, Option<String>) {
    let Some(value) = value else {
        return (Some(DEFAULT_MAX_LOGS_MB * MIB), None);
    };
    let bytes = value
        .as_u64()
        .filter(|&mebibytes| mebibytes >= 1)
        .and_then(|mebibytes| mebibytes.checked_mul(MIB));
    match bytes {
        Some(bytes) => (Some(bytes), None),
        None => (
            None,
            Some(invalid(path, "max_logs_mb", value, "not pruning by size")),
        ),
    }
}

/// The warning for a value outside its grammar: a string in single quotes, any other value as its JSON text.
fn invalid(path: &Path, key: &str, value: &Value, effect: &str) -> String {
    let shown = match value {
        Value::String(string) => format!("'{string}'"),
        other => other.to_string(),
    };
    warning(path, &format!("invalid {key} {shown}: {effect}"))
}

/// The stderr line for a config file that cannot be used, or holds a value that is not valid.
fn warning(path: &Path, reason: &str) -> String {
    error::warning(&format!("{}: {reason}", slash(path)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// The limits that `text` gives as the config.json of a folder.
    fn read(text: &str) -> Loaded {
        parse(Path::new("config.json"), text)
    }

    /// The stderr line for `reason` about config.json, as `read` prints it.
    fn line(reason: &str) -> String {
        format!("memex: warning: config.json: {reason}")
    }

    #[test]
    fn a_value_that_is_not_an_object_is_unusable_and_prunes_nothing() {
        for contents in [r#"["1h"]"#, r#""3d""#, "5", "null"] {
            let loaded = read(contents);
            assert_eq!((loaded.keep, loaded.cap), (None, None), "{contents}");
            assert_eq!(loaded.warnings.len(), 1, "{contents}");
            assert!(
                loaded.warnings[0].ends_with("not a JSON object: not pruning"),
                "{contents}"
            );
        }
    }

    #[test]
    fn an_unknown_key_is_unusable_and_its_warning_names_both_keys() {
        let loaded = read(r#"{"keep_log": "3d"}"#);
        assert_eq!((loaded.keep, loaded.cap), (None, None));
        assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
        assert!(
            loaded.warnings[0]
                .contains("unknown field `keep_log`, expected `keep_logs` or `max_logs_mb`"),
            "{:?}",
            loaded.warnings
        );
    }

    #[test]
    fn absent_keys_give_no_age_limit_and_the_default_cap() {
        let loaded = read("{}");
        assert_eq!((loaded.keep, loaded.cap), (None, Some(10 * MIB)));
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn keep_logs_takes_a_duration_or_forever() {
        assert_eq!(read(r#"{"keep_logs": "2d"}"#).keep, Some(2 * 86_400));
        assert_eq!(read(r#"{"keep_logs": "90m"}"#).keep, Some(90 * 60));
        let forever = read(r#"{"keep_logs": "forever"}"#);
        assert_eq!(forever.keep, None);
        assert!(forever.warnings.is_empty());
    }

    #[test]
    fn an_invalid_keep_logs_warns_and_gives_no_age_limit() {
        let soon = read(r#"{"keep_logs": "soon"}"#);
        assert_eq!((soon.keep, soon.cap), (None, Some(10 * MIB)));
        assert_eq!(
            soon.warnings,
            vec![line("invalid keep_logs 'soon': not pruning by age")]
        );
        // A non-string is a warning too, null included: it is a value, not an absent key.
        for (value, shown) in [("3", "3"), ("null", "null"), (r#"["1d"]"#, r#"["1d"]"#)] {
            let loaded = read(&format!(r#"{{"keep_logs": {value}}}"#));
            assert_eq!(loaded.keep, None, "{value}");
            assert_eq!(
                loaded.warnings,
                vec![line(&format!(
                    "invalid keep_logs {shown}: not pruning by age"
                ))],
                "{value}"
            );
        }
    }

    #[test]
    fn max_logs_mb_is_a_whole_number_of_mebibytes() {
        assert_eq!(read(r#"{"max_logs_mb": 1}"#).cap, Some(MIB));
        assert_eq!(read(r#"{"max_logs_mb": 10}"#).cap, Some(10 * MIB));
        // The largest count whose bytes still fit in a u64.
        let largest = read(r#"{"max_logs_mb": 17592186044415}"#);
        assert_eq!(largest.cap, Some(17_592_186_044_415 * MIB));
        assert!(largest.warnings.is_empty());
    }

    #[test]
    fn an_invalid_max_logs_mb_warns_and_gives_no_size_cap() {
        for (value, shown) in [
            ("0", "0"),
            ("-1", "-1"),
            ("1.5", "1.5"),
            ("1.0", "1.0"),
            (r#""1""#, "'1'"),
            ("null", "null"),
            ("true", "true"),
            ("[1]", "[1]"),
            // 2^44 mebibytes is 2^64 bytes, one more than a u64 holds.
            ("17592186044416", "17592186044416"),
        ] {
            let loaded = read(&format!(r#"{{"max_logs_mb": {value}}}"#));
            assert_eq!((loaded.keep, loaded.cap), (None, None), "{value}");
            assert_eq!(
                loaded.warnings,
                vec![line(&format!(
                    "invalid max_logs_mb {shown}: not pruning by size"
                ))],
                "{value}"
            );
        }
    }

    #[test]
    fn an_invalid_value_disables_only_its_own_limit() {
        let both = read(r#"{"keep_logs": "soon", "max_logs_mb": 0}"#);
        assert_eq!((both.keep, both.cap), (None, None));
        assert_eq!(
            both.warnings,
            vec![
                line("invalid keep_logs 'soon': not pruning by age"),
                line("invalid max_logs_mb 0: not pruning by size"),
            ]
        );
        let age_only = read(r#"{"keep_logs": "1d", "max_logs_mb": 0}"#);
        assert_eq!((age_only.keep, age_only.cap), (Some(86_400), None));
        assert_eq!(
            age_only.warnings,
            vec![line("invalid max_logs_mb 0: not pruning by size")]
        );
        let size_only = read(r#"{"keep_logs": "soon", "max_logs_mb": 1}"#);
        assert_eq!((size_only.keep, size_only.cap), (None, Some(MIB)));
    }

    #[test]
    fn a_config_named_in_another_case_is_not_the_file() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let memex = dir.path().join(".memex");
        fs::create_dir(&memex).expect("a memex folder");
        fs::write(memex.join("Config.json"), br#"{"keep_logs": "1d"}"#).expect("a config");
        let loaded = load(dir.path());
        assert_eq!((loaded.keep, loaded.cap), (None, Some(10 * MIB)));
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn a_folder_with_no_memex_folder_has_the_defaults() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let loaded = load(dir.path());
        assert_eq!((loaded.keep, loaded.cap), (None, Some(10 * MIB)));
        assert!(loaded.warnings.is_empty());
    }
}
