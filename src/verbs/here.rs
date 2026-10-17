//! Loads what a reading verb works on: the canonical cwd, its log store, and the corpus of its root.
//! `load_in` takes a folder in place of the cwd, for init; `load_with_lines` keeps the line facts that `audit` reads.
//! The store is found from the cwd, and the root is the one that `root_of` names (design §4.1 item 1).
//! The corpus is never freed: `Here` holds it in a `ManuallyDrop`, since the process exits after the verb.
//! Cases: none (shared loading; show and search cite their own cases)

use std::env;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

use crate::corpus::{self, Corpus};
use crate::error::{Code, Error};
use crate::root;

/// The place a reading verb runs in: the cwd, the log store, and the corpus of the cwd's root.
pub struct Here {
    /// The cwd, canonical, so that it compares with the canonical root.
    pub cwd: PathBuf,
    /// The log store, or None when there is no `.memex/`. Reads work without one.
    pub store: Option<PathBuf>,
    /// The intent layer of the cwd's root and the logs of the store. Never dropped: the process exits after the verb.
    pub corpus: ManuallyDrop<Corpus>,
}

/// Loads the corpus of the cwd's root. The store is found (which runs git) while the intent layer is read.
pub fn load() -> Result<Here, Error> {
    load_in(current_dir()?)
}

/// Loads as `load` does, with the line facts that `audit` reads kept in the corpus's `heads` (design §3.5d Lines).
pub fn load_with_lines() -> Result<Here, Error> {
    load_from(current_dir()?, true)
}

/// Loads as `load` does, as if memex ran in `cwd`, a canonical folder. init reads its target this way, so the process cwd never
/// changes.
pub fn load_in(cwd: PathBuf) -> Result<Here, Error> {
    load_from(cwd, false)
}

/// Loads from `cwd`, a canonical folder, keeping the line facts of the files when `lines` is set.
fn load_from(cwd: PathBuf, lines: bool) -> Result<Here, Error> {
    let root = corpus::root_of(&cwd);
    let (corpus, store) = Corpus::load_finding(&root, lines, || store_of(&cwd))?;
    Ok(Here {
        cwd,
        store,
        corpus: ManuallyDrop::new(corpus),
    })
}

/// The cwd, canonical.
pub fn current_dir() -> Result<PathBuf, Error> {
    let cwd = env::current_dir()
        .map_err(|failure| Error::io("cannot read the current folder", &failure))?;
    dunce::canonicalize(&cwd)
        .map_err(|failure| Error::io("cannot resolve the current folder", &failure))
}

/// The log store that `log` would write to, or None when there is no `.memex/` (design §4.1).
fn store_of(cwd: &Path) -> Result<Option<PathBuf>, Error> {
    match root::locate(cwd) {
        Ok(place) => Ok(Some(place.store)),
        Err(error) if error.code == Code::NoRoot => Ok(None),
        Err(error) => Err(error),
    }
}
