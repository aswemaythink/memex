//! Carries out `memex audit` (design §3.5d): where the intent layer is missing or broken.
//! `audit/scope.rs` turns the arguments into the scope they name, `audit/kinds.rs` finds each kind's findings over it, and
//! `render/audit.rs` prints the report. A path that names nothing is reported, and the others are still audited.
//! Cases: AUD-SCOPE-01, AUD-SCOPE-02, AUD-SCOPE-03, AUD-SCOPE-04, AUD-SCOPE-05, AUD-SCOPE-06, AUD-SCOPE-07, AUD-GLOB-01, AUD-GLOB-02, AUD-GLOB-03, AUD-HEADERS-01, AUD-HEADERS-02, AUD-HEADERS-03, AUD-READMES-01, AUD-READMES-02, AUD-READMES-03, AUD-READMES-04, AUD-READMES-05, AUD-LENGTH-01, AUD-LENGTH-02, AUD-LENGTH-03, AUD-LENGTH-04, AUD-LENGTH-05, AUD-LENGTH-06, AUD-LENGTH-07, AUD-ONELINERS-01, AUD-ONELINERS-02, AUD-ONELINERS-03, AUD-ONELINERS-04, AUD-THIN-01, AUD-THIN-02, AUD-THIN-03, AUD-THIN-04, AUD-THIN-05, AUD-THIN-06, AUD-THIN-07, AUD-CITES-01, AUD-CITES-02, AUD-CITES-03, AUD-CITES-04, AUD-CITES-05, AUD-CITES-06, AUD-IDS-01, AUD-IDS-02, AUD-LIMIT-01, AUD-LIMIT-02, AUD-LIMIT-03, AUD-JSON-01, AUD-JSON-02, AUD-JSON-03, AUD-EMPTY-01, AUD-EXIT-01, AUD-EXIT-02, AUD-EXIT-03, AUD-REFINE-01, AUD-REFINE-02, AUD-REFINE-03, AUD-REFINE-04

mod counts;
mod kinds;
mod roots;
mod scope;

use std::path::Path;

use crate::cli::Verb;
use crate::corpus::Corpus;
use crate::error::Error;
use crate::render::audit::Summary;
use crate::render::{self, text};
use crate::scan::fail;
use crate::verbs::here;

/// The `--limit` of each kind when the flag is not given (design §3.5d).
const DEFAULT_LIMIT: usize = 20;

/// The arguments of `memex audit` once the grammar has passed them.
pub struct Request {
    /// The paths and globs, in the order given. None audits the root.
    pub paths: Vec<String>,
    /// The value of `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// What `audit` prints: stdout, stderr, and whether a path named nothing, which makes the exit status 1.
pub struct Report {
    /// The report as text, or the JSON document with `--json`.
    pub stdout: String,
    /// One line per miss in text mode. Empty with `--json`, where the misses are in the document.
    pub stderr: String,
    /// True when at least one path named nothing or several things.
    pub failed: bool,
}

/// Audits the root, or the paths and globs given. An empty path is a usage error, as it is for `show`. The log store is
/// loaded too, so that a reference to a log entry resolves; logs themselves are not audited (design §3.5d).
pub fn run(request: &Request) -> Result<Report, Error> {
    if request.paths.iter().any(String::is_empty) {
        return Err(fail(Verb::Audit, "empty path".to_string()));
    }
    let here::Here { cwd, store, corpus } = here::load_with_lines()?;
    Ok(audit_in(&corpus, &cwd, store.as_deref(), request))
}

/// Audits `request` as typed from `cwd`, in the corpus of its root. The walked entries are read once, from the corpus.
fn audit_in(corpus: &Corpus, cwd: &Path, store: Option<&Path>, request: &Request) -> Report {
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
        let stdout = render::audit::json(&summary);
        return Report {
            stdout,
            stderr: String::new(),
            failed,
        };
    }
    Report {
        stdout: render::audit::text(&summary),
        stderr: text::misses(&summary.errors),
        failed,
    }
}
