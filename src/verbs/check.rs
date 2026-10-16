//! Carries out `memex check` (design §3.5d): where the intent layer is missing or broken.
//! `check/scope.rs` turns the arguments into the scope they name, `check/kinds.rs` finds each kind's findings over it, and
//! `render/check.rs` prints the report. A path that names nothing is reported, and the others are still checked.
//! Cases: CHK-SCOPE-01, CHK-SCOPE-02, CHK-SCOPE-03, CHK-SCOPE-04, CHK-SCOPE-05, CHK-SCOPE-06, CHK-SCOPE-07, CHK-GLOB-01, CHK-GLOB-02, CHK-GLOB-03, CHK-HEADERS-01, CHK-HEADERS-02, CHK-HEADERS-03, CHK-READMES-01, CHK-READMES-02, CHK-READMES-03, CHK-ONELINERS-01, CHK-ONELINERS-02, CHK-ONELINERS-03, CHK-ONELINERS-04, CHK-CITES-01, CHK-CITES-02, CHK-CITES-03, CHK-CITES-04, CHK-CITES-05, CHK-CITES-06, CHK-IDS-01, CHK-IDS-02, CHK-LIMIT-01, CHK-LIMIT-02, CHK-LIMIT-03, CHK-JSON-01, CHK-JSON-02, CHK-JSON-03, CHK-EMPTY-01, CHK-EXIT-01, CHK-EXIT-02, CHK-EXIT-03

mod kinds;
mod scope;

use std::path::Path;

use crate::cli::Verb;
use crate::corpus::Corpus;
use crate::error::Error;
use crate::render::check::Summary;
use crate::render::{self, text};
use crate::scan::fail;
use crate::verbs::here;

/// The `--limit` of each kind when the flag is not given (design §3.5d).
const DEFAULT_LIMIT: usize = 20;

/// The arguments of `memex check` once the grammar has passed them.
pub struct Request {
    /// The paths and globs, in the order given. None checks the root.
    pub paths: Vec<String>,
    /// The value of `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// What `check` prints: stdout, stderr, and whether a path named nothing, which makes the exit status 1.
pub struct Report {
    /// The report as text, or the JSON document with `--json`.
    pub stdout: String,
    /// One line per miss in text mode. Empty with `--json`, where the misses are in the document.
    pub stderr: String,
    /// True when at least one path named nothing or several things.
    pub failed: bool,
}

/// Checks the root, or the paths and globs given. An empty path is a usage error, as it is for `show`. The log store is
/// loaded too, so that a reference to a log entry resolves; logs themselves are not checked (design §3.5d).
pub fn run(request: &Request) -> Result<Report, Error> {
    if request.paths.iter().any(String::is_empty) {
        return Err(fail(Verb::Check, "empty path".to_string()));
    }
    let here::Here { cwd, store, corpus } = here::load()?;
    Ok(check_in(&corpus, &cwd, store.as_deref(), request))
}

/// Checks `request` as typed from `cwd`, in the corpus of its root. The walked entries are read once, from the corpus.
fn check_in(corpus: &Corpus, cwd: &Path, store: Option<&Path>, request: &Request) -> Report {
    let root = [".".to_string()];
    let args = if request.paths.is_empty() {
        &root[..]
    } else {
        &request.paths[..]
    };
    let tree = scope::walked(corpus);
    let resolved = scope::resolve_args(corpus, cwd, store, args, &tree);
    let (files, folders) = scope::counts(&resolved.scope, &tree);
    let ctx = kinds::Ctx {
        corpus,
        scope: &resolved.scope,
        tree: &tree,
        limit: request.limit.unwrap_or(DEFAULT_LIMIT),
    };
    let summary = Summary {
        scope: resolved.labels,
        files,
        folders,
        groups: kinds::all(&ctx),
        errors: resolved.misses,
    };
    let failed = !summary.errors.is_empty();
    if request.json {
        let stdout = render::check::json(&summary);
        return Report {
            stdout,
            stderr: String::new(),
            failed,
        };
    }
    Report {
        stdout: render::check::text(&summary),
        stderr: text::misses(&summary.errors),
        failed,
    }
}
