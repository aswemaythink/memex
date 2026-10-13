//! Checks a stream name before it names a folder (design §3.1 and §3.4): its grammar, the built-in names that are reserved, and the manifests that already own a name.
//! Cases: LOG-WRITE-13, LOG-STREAM-01, LOG-STREAM-02, LOG-STREAM-03

use std::path::Path;

use crate::cli::Verb;
use crate::error::Error;
use crate::root::nearest_memex;
use crate::scan::fail;

/// The built-in collection names, which no stream may take (design §3.1).
const RESERVED: [&str; 4] = ["readmes", "headers", "cases", "logs"];

/// A stream name is one to forty lowercase letters, digits or dashes, starting with a letter or digit (design §3.4).
/// Anything else would make the entry's folder land outside the store, so it is a usage error.
pub fn check_grammar(stream: &str) -> Result<(), Error> {
    let mut chars = stream.chars();
    let first_ok =
        matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase() || ch.is_ascii_digit());
    let rest_ok = chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
    if first_ok && rest_ok && stream.len() <= 40 {
        return Ok(());
    }
    let message = format!(
        "invalid stream name '{stream}': use up to 40 lowercase letters, digits and dashes, starting with a letter or digit"
    );
    Err(fail(Verb::Log, message))
}

/// Refuses a built-in name, which belongs to a built-in collection (design §3.1 and §3.4).
pub fn check_reserved(stream: &str) -> Result<(), Error> {
    if RESERVED.contains(&stream) {
        return Err(fail(
            Verb::Log,
            format!("stream name '{stream}' is reserved"),
        ));
    }
    Ok(())
}

/// Refuses a name that a manifest owns: a file `.memex/collections/<stream>.json` under the nearest `.memex/` above `cwd`,
/// or under the store owner's `.memex/` (design §4.1 item 4 and §4.2). Manifests are not loaded until a later slice,
/// so the file name is the whole check.
pub fn check_collections(stream: &str, cwd: &Path, owner: &Path) -> Result<(), Error> {
    let file = format!("{stream}.json");
    let roots = nearest_memex(cwd).into_iter().chain([owner.to_path_buf()]);
    let owned = roots.into_iter().any(|root| {
        root.join(".memex")
            .join("collections")
            .join(&file)
            .is_file()
    });
    if owned {
        return Err(fail(
            Verb::Log,
            format!("stream name '{stream}' is a collection"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_takes_the_design_shape() {
        assert!(check_grammar("impl-2").is_ok());
        assert!(check_grammar(&"a".repeat(40)).is_ok());
        assert!(check_grammar(&"a".repeat(41)).is_err());
        assert!(check_grammar("-impl").is_err());
        assert!(check_grammar("Impl").is_err());
    }

    #[test]
    fn a_manifest_above_the_cwd_owns_its_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".memex/collections")).unwrap();
        std::fs::write(root.join(".memex/collections/notes.json"), b"{}").unwrap();
        let cwd = root.join("deeper");
        std::fs::create_dir_all(&cwd).unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        assert!(check_collections("notes", &cwd, elsewhere.path()).is_err());
        assert!(check_collections("other", &cwd, elsewhere.path()).is_ok());
    }
}
