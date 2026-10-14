//! Resolves a path that leaves the current root against the root that holds it (design §3.1).
//! Cases: REF-ABS-01, REF-REL-03, REF-REL-04, REF-REL-06, REF-WT-01, REF-WT-02, REF-WT-03, REF-WT-04

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

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
    match exact(Path::new(""), Path::new(arg)) {
        Some(full) => place(corpus, store, &full),
        None => Ok(Outcome::NotFound),
    }
}

/// A relative path that climbs above the current root (design §3.1, step 2). None when the path names no entry there as typed,
/// so that the ref is tried as a name.
pub fn relative<'a>(
    corpus: &'a Corpus,
    cwd: &Path,
    store: Option<&Path>,
    arg: &str,
) -> Result<Option<Outcome<'a>>, Error> {
    match exact(cwd, Path::new(arg)) {
        Some(full) => place(corpus, store, &full).map(Some),
        None => Ok(None),
    }
}

/// The canonical path that `arg` names from `base`, when each part of `arg` is spelled the way its folder lists it, so that
/// names match case-sensitively on every OS (design §3.1, step 2). `base` is a canonical folder, or empty for an absolute `arg`.
/// None when a part is not listed exactly, or when the path names nothing.
fn exact(base: &Path, arg: &Path) -> Option<PathBuf> {
    let mut found = base.to_path_buf();
    for part in arg.components() {
        match part {
            Component::Prefix(_) | Component::RootDir => found.push(part.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(parent) = found.parent() {
                    found = parent.to_path_buf();
                }
            }
            Component::Normal(name) => {
                if !listed(&found, name) {
                    return None;
                }
                found = dunce::canonicalize(found.join(name)).ok()?;
            }
        }
    }
    Some(found)
}

/// True when the folder `dir` lists an entry named exactly `name`. A name is matched against the listing and never probed, so a
/// case-insensitive file system does not match another spelling (design §4.4, name matching).
fn listed(dir: &Path, name: &OsStr) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| entries.flatten().any(|entry| entry.file_name() == name))
}

/// The outcome for a canonical path: an entry of the current root when it lies inside it, else an entry of its own root.
fn place<'a>(corpus: &'a Corpus, store: Option<&Path>, full: &Path) -> Result<Outcome<'a>, Error> {
    match full.strip_prefix(&corpus.root).ok().and_then(rel_string) {
        Some(rel) => Ok(found_path(corpus, rel)),
        None => elsewhere(full, store),
    }
}

/// The entry at `full` in its own nearest root (design §3.1, other worktrees). The root is found from the path, as design
/// §4.1 finds it for `show`, and it is read with the same log store. With no `.memex/` or git above the path, that root is the
/// folder itself, or the file's parent, so the walk never climbs to the enclosing tree (REF-WT-04). A path that root's walk
/// does not hold is not found.
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
        Some(kind) => Outcome::Elsewhere(Box::new(Elsewhere {
            corpus,
            rel,
            kind,
            label: slash(full),
        })),
        None => Outcome::NotFound,
    })
}
