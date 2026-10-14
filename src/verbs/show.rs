//! Carries out `memex show` (design §3.3 and §3.6).
//! It resolves each ref, takes its section from `show/section.rs`, and reports the sections, the misses and the exit status.
//! Cases: SHOW-DUP-01, SHOW-DUP-02, SHOW-ERR-01, SHOW-ERR-02, SHOW-ERR-03, SHOW-IO-01, SHOW-IO-02, SHOW-JSON-01, SHOW-JSON-04, SHOW-JSON-05, SHOW-SEP-01, REF-WT-01, REF-WT-02

mod section;

use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use self::section::{section_for, Options};
use crate::cli::Verb;
use crate::corpus::Corpus;
use crate::error::Error;
use crate::render::{self, Listing, Miss, Section};
use crate::resolve::{resolve, Elsewhere, Outcome, Target};
use crate::scan::fail;
use crate::verbs::here;

/// The `--limit` of a list when the flag is not given (design §3.3).
const DEFAULT_LIMIT: usize = 50;

/// The arguments of `memex show` once the grammar has passed them.
pub struct Request {
    /// The refs, in the order given.
    pub refs: Vec<String>,
    /// The value of `--up`, if the flag was given.
    pub up: Option<usize>,
    /// The value of `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// What `show` prints: stdout, stderr, and whether a ref failed, which makes the exit status 1.
pub struct Report {
    /// The sections as text, or the JSON document with `--json`.
    pub stdout: String,
    /// One line per miss in text mode. Empty with `--json`, where the misses are in the document.
    pub stderr: String,
    /// True when at least one ref was not shown.
    pub failed: bool,
}

/// Shows each ref in the order given. A ref that fails is reported and the others still print. An empty ref is a usage error,
/// and a ref given twice prints once, at its first position (design §3.3). A requested file or folder that cannot be read or
/// listed is that ref's `io` error, so the verb exits 1 like any failed ref. A fatal problem, such as an unreadable cwd, is an
/// error.
pub fn run(request: &Request) -> Result<Report, Error> {
    if request.refs.iter().any(String::is_empty) {
        return Err(fail(Verb::Show, "empty ref".to_string()));
    }
    let here::Here { cwd, store, corpus } = here::load()?;
    Ok(show_in(&corpus, &cwd, store.as_deref(), request))
}

/// Shows the refs of `request` as typed from `cwd`, in the corpus of their root. `run` passes the cwd's own corpus; a test can
/// pass one that it built.
fn show_in(corpus: &Corpus, cwd: &Path, store: Option<&Path>, request: &Request) -> Report {
    let options = Options {
        up: request.up,
        limit: request.limit.unwrap_or(DEFAULT_LIMIT),
    };
    let mut sections = Vec::new();
    let mut misses = Vec::new();
    let mut seen = BTreeSet::new();
    for arg in &request.refs {
        if !seen.insert(arg.as_str()) {
            continue;
        }
        let built = match resolve(corpus, cwd, store, arg) {
            Outcome::Found(target) => section_for(corpus, &target, &options),
            Outcome::Elsewhere(other) => elsewhere_section(&other, &options),
            Outcome::NotFound => {
                misses.push(Miss::not_found(arg));
                continue;
            }
            Outcome::Ambiguous(candidates) => {
                misses.push(Miss::ambiguous(arg, candidates));
                continue;
            }
        };
        match built {
            Ok(section) => sections.push(section),
            Err(failure) => misses.push(Miss::unreadable(arg, &failure)),
        }
    }
    let failed = !misses.is_empty();
    if request.json {
        let stdout = render::json::document(&sections, &misses);
        return Report {
            stdout,
            stderr: String::new(),
            failed,
        };
    }
    Report {
        stdout: render::text::sections(&sections),
        stderr: render::text::misses(&misses),
        failed,
    }
}

/// The section of an entry in another root: built from that root's own corpus, and labelled by its absolute path, which
/// round-trips from anywhere (design §3.1). Its cites and cited by name labels of that root, which do not round-trip from
/// the cwd, so they print as none until absolute refs are designed for them.
fn elsewhere_section(other: &Elsewhere, options: &Options) -> Result<Section, io::Error> {
    let target = Target::Path {
        rel: other.rel.clone(),
        kind: other.kind,
    };
    let mut section = section_for(&other.corpus, &target, options)?;
    section.label.clone_from(&other.label);
    section.cites.clear();
    section.cited_by = Listing {
        total: 0,
        items: Vec::new(),
    };
    Ok(section)
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    /// A requested file that cannot be read fails its ref with an `io` error, in text and in JSON, and the other refs still print
    /// (design §3.3). Setup cannot chmod, so the test does.
    #[test]
    fn a_requested_file_that_cannot_be_read_is_its_refs_io_error() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let root = dir.path();
        fs::write(root.join("README.md"), "# Demo\n\nThe demo.\n").expect("a file");
        fs::write(root.join("notes.md"), "# Notes\n\nNotes.\n").expect("a file");
        fs::set_permissions(root.join("README.md"), fs::Permissions::from_mode(0o000))
            .expect("a mode");
        if fs::read(root.join("README.md")).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let corpus = Corpus::load(root, None);
        let request = |json: bool| Request {
            refs: vec!["README.md".to_string(), "notes.md".to_string()],
            up: None,
            limit: None,
            json,
        };
        let text = show_in(&corpus, root, None, &request(false));
        let json = show_in(&corpus, root, None, &request(true));
        fs::set_permissions(root.join("README.md"), fs::Permissions::from_mode(0o644))
            .expect("restored");
        assert!(text.failed);
        assert_eq!(
            text.stderr,
            "memex: cannot read README.md (PermissionDenied)\n"
        );
        assert!(text.stdout.contains("=== notes.md ==="), "{}", text.stdout);
        assert!(json.stdout.contains(r#""code":"io""#), "{}", json.stdout);
    }

    /// A requested folder that cannot be listed fails its ref with an `io` error, in text and in JSON, and the other refs still
    /// print (design §3.3, SHOW-IO-02). Setup cannot chmod, so the test does.
    #[test]
    fn a_requested_folder_that_cannot_be_listed_is_its_refs_io_error() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let root = dir.path();
        fs::create_dir(root.join("locked")).expect("a folder");
        fs::write(root.join("locked").join("inside.rs"), "//! Inside.\n").expect("a file");
        fs::write(root.join("notes.md"), "# Notes\n\nNotes.\n").expect("a file");
        let open = fs::Permissions::from_mode(0o755);
        fs::set_permissions(root.join("locked"), fs::Permissions::from_mode(0o000))
            .expect("a mode");
        if fs::read_dir(root.join("locked")).is_ok() {
            // Running as root lists a folder whatever its mode, so there is nothing to check.
            fs::set_permissions(root.join("locked"), open).expect("restored");
            return;
        }
        let corpus = Corpus::load(root, None);
        let request = |json: bool| Request {
            refs: vec!["locked".to_string(), "notes.md".to_string()],
            up: None,
            limit: None,
            json,
        };
        let text = show_in(&corpus, root, None, &request(false));
        let json = show_in(&corpus, root, None, &request(true));
        fs::set_permissions(root.join("locked"), open).expect("restored");
        assert!(text.failed);
        assert_eq!(
            text.stderr,
            "memex: cannot read locked (PermissionDenied)\n"
        );
        assert!(text.stdout.contains("=== notes.md ==="), "{}", text.stdout);
        assert!(!text.stdout.contains("=== locked ==="), "{}", text.stdout);
        assert!(json.failed);
        assert!(json.stdout.contains(r#""code":"io""#), "{}", json.stdout);
        assert!(
            !json.stdout.contains(r#""ref":"locked""#),
            "{}",
            json.stdout
        );
    }
}
