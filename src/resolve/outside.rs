//! Resolves a path that leaves the current root against the root that holds it (design §3.1).
//! Cases: REF-ABS-01, REF-REL-03, REF-REL-04, REF-WT-01, REF-WT-02, REF-WT-03

use std::path::Path;

use crate::corpus::{self, Corpus, Kind};
use crate::error::Error;
use crate::root::slash;
use crate::walk::rel_string;

use super::{found_path, Outcome};

/// An entry of another root: that root's corpus, the entry's path inside it, and the label it prints under.
pub struct Elsewhere {
    /// The corpus of the root that holds the entry, read with the same log store.
    pub corpus: Corpus,
    /// The root-relative path of the entry in that root; empty for the root itself.
    pub rel: String,
    /// Whether the entry is a folder or a file.
    pub kind: Kind,
    /// The label to print: the absolute path with `/` separators. A root-relative label would name an entry of the current
    /// root, so it would not round-trip.
    pub label: String,
}

/// An absolute path (design §3.1, step 1). A path that names no entry on the file system is not found.
pub fn absolute<'a>(
    corpus: &'a Corpus,
    store: Option<&Path>,
    arg: &str,
) -> Result<Outcome<'a>, Error> {
    match dunce::canonicalize(arg) {
        Ok(full) => place(corpus, store, &full),
        Err(_) => Ok(Outcome::NotFound),
    }
}

/// A relative path that climbs above the current root (design §3.1, step 2). None when the file system holds no entry there,
/// so that the ref is tried as a name.
pub fn relative<'a>(
    corpus: &'a Corpus,
    cwd: &Path,
    store: Option<&Path>,
    arg: &str,
) -> Result<Option<Outcome<'a>>, Error> {
    match dunce::canonicalize(cwd.join(arg)) {
        Ok(full) => place(corpus, store, &full).map(Some),
        Err(_) => Ok(None),
    }
}

/// The outcome for a canonical path: an entry of the current root when it lies inside it, else an entry of its own root.
fn place<'a>(corpus: &'a Corpus, store: Option<&Path>, full: &Path) -> Result<Outcome<'a>, Error> {
    match full.strip_prefix(&corpus.root).ok().and_then(rel_string) {
        Some(rel) => Ok(found_path(corpus, rel)),
        None => elsewhere(full, store),
    }
}

/// The entry at `full` in its own nearest root (design §3.1, other worktrees). The root is found from the path, as design
/// §4.1 finds it for `show`, and it is read with the same log store. A path that root's walk does not hold is not found.
fn elsewhere<'a>(full: &Path, store: Option<&Path>) -> Result<Outcome<'a>, Error> {
    let start = if full.is_dir() {
        full
    } else {
        full.parent().unwrap_or(full)
    };
    let root = corpus::root_of(start);
    let Some(rel) = full.strip_prefix(&root).ok().and_then(rel_string) else {
        return Ok(Outcome::NotFound);
    };
    let corpus = Corpus::load(&root, store)?;
    Ok(match corpus.kind(&rel) {
        Some(kind) => Outcome::Elsewhere(Elsewhere {
            corpus,
            rel,
            kind,
            label: slash(full),
        }),
        None => Outcome::NotFound,
    })
}
