//! Checks a stream name before it names a folder (design §3.1 and §3.4): its grammar, the built-in and Windows device names that are reserved, and the manifests that already own a name.
//! Cases: LOG-WRITE-13, LOG-STREAM-01, LOG-STREAM-02, LOG-STREAM-03, LOG-STREAM-04, LOG-STREAM-05

use std::fs;
use std::io;
use std::path::Path;

use crate::cli::Verb;
use crate::error::Error;
use crate::root::{nearest_memex, slash};
use crate::scan::fail;

/// The built-in collection names, which no stream may take (design §3.1).
const RESERVED: [&str; 4] = ["readmes", "headers", "cases", "logs"];

/// The Windows device names, which no stream may take on any OS, so a store stays portable (design §3.4). Windows
/// refuses them as folder names; the grammar allows no dot, so only the bare names can arrive.
const DEVICE_NAMES: [&str; 24] = [
    "con", "prn", "aux", "nul", "com0", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
    "com8", "com9", "lpt0", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

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

/// Refuses a name no stream may take: a built-in name, which belongs to a built-in collection (design §3.1 and §3.4), or
/// a Windows device name, which Windows cannot hold as a folder on any OS (design §3.4).
pub fn check_reserved(stream: &str) -> Result<(), Error> {
    if RESERVED.contains(&stream) {
        return Err(fail(
            Verb::Log,
            format!("stream name '{stream}' is reserved"),
        ));
    }
    if DEVICE_NAMES.contains(&stream) {
        return Err(fail(
            Verb::Log,
            format!("stream name '{stream}' is a Windows device name"),
        ));
    }
    Ok(())
}

/// Refuses a name that a manifest owns: a file `.memex/collections/<stream>.json` under the nearest `.memex/` above `cwd`,
/// or under the store owner's `.memex/` (design §4.1 item 4 and §4.2). The file names are read from each folder's listing and
/// compared exactly, never probed (design §4.4 Name matching). Manifests are not loaded until a later slice.
pub fn check_collections(stream: &str, cwd: &Path, owner: &Path) -> Result<(), Error> {
    let file = format!("{stream}.json");
    let roots = nearest_memex(cwd).into_iter().chain([owner.to_path_buf()]);
    for root in roots {
        if lists_file(&root.join(".memex").join("collections"), &file)? {
            return Err(fail(
                Verb::Log,
                format!("stream name '{stream}' is a collection"),
            ));
        }
    }
    Ok(())
}

/// True when the folder `dir` lists a file named exactly `file`. A folder that does not exist lists nothing, so there are
/// no collections there; any other failure to list it is an I/O error that names the folder.
fn lists_file(dir: &Path, file: &str) -> Result<bool, Error> {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(failure) => return Err(io_error(dir, &failure)),
    };
    for item in listing {
        let item = item.map_err(|failure| io_error(dir, &failure))?;
        if item.file_name().to_str() == Some(file) && item.path().is_file() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// An I/O error that names the folder and the io::ErrorKind, never the OS text (design §4.4 Errors).
fn io_error(dir: &Path, failure: &io::Error) -> Error {
    Error::io(&format!("cannot read {}", slash(dir)), failure)
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

    #[test]
    fn device_names_are_refused_and_their_neighbours_are_not() {
        for name in ["con", "prn", "aux", "nul", "com0", "com9", "lpt0", "lpt9"] {
            assert!(check_reserved(name).is_err(), "{name}");
        }
        for name in ["com10", "con-1", "nul2", "lpt"] {
            assert!(check_reserved(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn a_manifest_name_is_matched_by_its_exact_file_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".memex/collections")).unwrap();
        std::fs::write(root.join(".memex/collections/Notes.json"), b"{}").unwrap();
        assert!(check_collections("notes", root, root).is_ok());
    }

    #[test]
    fn a_missing_collections_folder_lists_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(check_collections("notes", dir.path(), dir.path()).is_ok());
    }
}
