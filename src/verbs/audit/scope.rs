//! Turns the arguments of `memex audit` into the scope they name (design §3.5d).
//! A file is in scope when it is named, when it matches a glob, or when a folder above it is; a folder likewise. A path is
//! resolved as `show` resolves one (design §3.1), so a path that `show` cannot find is not found by audit either.
//! `tree` resolves its one path through `scope_path`, so the two cannot drift apart. A path is folded lexically first: an
//! absolute or root-climbing path that stays inside the root names its walked entry (a link is not walked, so it names nothing),
//! and one that leaves it is checked by its canonical path once, so no spelling loads another root's corpus.
//! Cases: AUD-SCOPE-01, AUD-SCOPE-02, AUD-SCOPE-03, AUD-SCOPE-04, AUD-SCOPE-05, AUD-SCOPE-06, AUD-SCOPE-07, AUD-GLOB-01, AUD-GLOB-02, AUD-GLOB-03, AUD-OUTSIDE-01, AUD-OUTSIDE-02, AUD-OUTSIDE-03, AUD-OUTSIDE-04, AUD-OUTSIDE-05, TREE-OUTSIDE-01, TREE-OUTSIDE-02, TREE-OUTSIDE-03, TREE-OUTSIDE-04, TREE-SYMLINK-01

use std::collections::BTreeSet;
use std::iter::successors;
use std::path::{Component, Path, PathBuf, Prefix};

use crate::collections::globs;
use crate::corpus::{label_of, parent_of, Corpus, Kind};
use crate::render::Miss;
use crate::resolve::{resolve, Outcome, Target};
use crate::walk::{rel_string, Entry};

/// The files and folders that an audit covers: those named or matched, and everything under a named or matched folder.
#[derive(Default)]
pub struct Scope {
    /// The root-relative paths of the files that are named or matched.
    files: BTreeSet<String>,
    /// The root-relative paths of the folders that are named or matched; `""` is the root.
    folders: BTreeSet<String>,
}

impl Scope {
    /// True when a walked file lies in the scope: it is named, or a folder above it is.
    pub fn holds_file(&self, rel: &str) -> bool {
        self.files.contains(rel) || self.under_folder(rel)
    }

    /// True when a walked folder lies in the scope: it is named, or a folder above it is.
    pub fn holds_folder(&self, rel: &str) -> bool {
        self.folders.contains(rel) || self.under_folder(rel)
    }

    /// True when a folder above `rel` is in the scope, so everything under that folder is too.
    fn under_folder(&self, rel: &str) -> bool {
        ancestors(rel).any(|folder| self.folders.contains(folder))
    }

    /// Adds one walked entry, as a folder or as a file.
    fn add(&mut self, rel: &str, is_dir: bool) {
        if is_dir {
            self.folders.insert(rel.to_string());
        } else {
            self.files.insert(rel.to_string());
        }
    }
}

/// The folders above a root-relative path, nearest first, ending with the root `""`. The root has none.
fn ancestors(rel: &str) -> impl Iterator<Item = &str> {
    successors(parent_of(rel), |parent| parent_of(parent))
}

/// The scope that the arguments name, the labels that print it, and a miss for each argument that names nothing.
#[derive(Default)]
pub struct Resolved {
    /// The files and folders the arguments name, with the globs' matches.
    pub scope: Scope,
    /// The label of each argument that names something, once each, in the order given (design §3.5d scope line).
    pub labels: Vec<String>,
    /// The arguments that name nothing, or several things, with the errors `show` gives them (design §3.5c).
    pub misses: Vec<Miss>,
}

/// Resolves each argument in order: a glob (one holding `*`, `?` or `[`) is matched against the walked entries, and any other
/// argument is a path, as `show` resolves one.
pub fn resolve_args(
    corpus: &Corpus,
    cwd: &Path,
    store: Option<&Path>,
    args: &[String],
    tree: &[Entry],
) -> Resolved {
    let mut resolved = Resolved::default();
    for arg in args {
        if is_glob(arg) {
            glob(corpus, arg, tree, &mut resolved);
        } else {
            path(corpus, cwd, store, arg, &mut resolved);
        }
    }
    resolved
}

/// True for an argument that is a glob: it holds `*`, `?` or `[` (design §3.5d scope).
fn is_glob(arg: &str) -> bool {
    arg.contains(['*', '?', '['])
}

/// Adds the walked entries that a glob matches, with the manifest include rules (design §4.2). A glob that matches nothing is
/// not an error, and its label still prints.
fn glob(corpus: &Corpus, arg: &str, tree: &[Entry], resolved: &mut Resolved) {
    let candidates: Vec<&Entry> = tree.iter().filter(|entry| !entry.rel.is_empty()).collect();
    for entry in globs::hits(&corpus.root, &[arg.to_string()], &candidates) {
        resolved.scope.add(&entry.rel, entry.is_dir);
    }
    label(resolved, arg.to_string());
}

/// Adds a path argument as `show` resolves it (design §3.1): the file or folder it names, or a miss when it names nothing or
/// several things. A doc (a label or a bare id) is not a path, so it names nothing here, and so does a path outside the root,
/// which is judged lexically, so audit reads only the root it runs in.
fn path(corpus: &Corpus, cwd: &Path, store: Option<&Path>, arg: &str, resolved: &mut Resolved) {
    match scope_path(corpus, cwd, store, arg) {
        Ok((rel, kind)) => {
            resolved.scope.add(&rel, kind == Kind::Dir);
            label(resolved, label_of(&rel));
        }
        Err(miss) => resolved.misses.push(miss),
    }
}

/// The root-relative path and kind that `arg` names as `show` resolves a path (design §3.1 steps 1, 2 and 5), or a miss when it
/// names nothing or several things. A doc (a label or a bare id) is not a path, so it names nothing here, and so does a path
/// outside the root, which is judged before anything is resolved: a path that stays inside the root once folded and is absolute
/// or climbs above the root names its walked entry and goes no further, so no other root is ever walked or warned about, whatever
/// the spelling (design §3.5d scope).
pub(crate) fn scope_path(
    corpus: &Corpus,
    cwd: &Path,
    store: Option<&Path>,
    arg: &str,
) -> Result<(String, Kind), Miss> {
    let folded = fold(cwd, arg);
    if folded.starts_with(&corpus.root) {
        if Path::new(arg).is_absolute() || climbs(&corpus.root, cwd, arg) {
            return named(corpus, &folded, arg);
        }
    } else if !lands_in_root(&corpus.root, cwd, arg) {
        return Err(Miss::outside(arg));
    }
    match resolve(corpus, cwd, store, arg) {
        Outcome::Found(Target::Path { rel, kind }) => Ok((rel, kind)),
        Outcome::Ambiguous(candidates) => Err(Miss::ambiguous(arg, candidates)),
        Outcome::Found(Target::Doc(_)) | Outcome::Elsewhere(_) | Outcome::NotFound => {
            Err(Miss::not_found(arg))
        }
    }
}

/// The path that `arg` names when joined to the cwd (an absolute `arg` stands alone), with `.` and `..` folded and no file system
/// access. A drive letter is upper-cased and a verbatim (`\\?\`) disk prefix is written plainly, so the folded path compares with
/// the canonical root whatever the spelling (design §3.5d scope).
fn fold(cwd: &Path, arg: &str) -> PathBuf {
    let mut folded = PathBuf::new();
    for part in cwd.join(arg).components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                folded.pop();
            }
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                    folded.push(format!("{}:", char::from(letter).to_ascii_uppercase()));
                }
                _ => folded.push(part.as_os_str()),
            },
            Component::RootDir | Component::Normal(_) => folded.push(part.as_os_str()),
        }
    }
    dunce::simplified(&folded).to_path_buf()
}

/// True when the relative `arg`, typed from `cwd` inside `root`, climbs above the root at some step, as `resolve` sees it (the
/// case where it cannot name an entry of the root as typed).
fn climbs(root: &Path, cwd: &Path, arg: &str) -> bool {
    let mut depth = cwd
        .strip_prefix(root)
        .map_or(0, |below| below.components().count());
    for part in Path::new(arg).components() {
        match part {
            Component::ParentDir if depth == 0 => return true,
            Component::ParentDir => depth -= 1,
            Component::Normal(_) => depth += 1,
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    false
}

/// The entry of the root that a folded path inside it names, as the walk holds it: a symlink is not walked, so a path through
/// one names nothing here, and no other root is read (design §3.5d scope).
fn named(corpus: &Corpus, folded: &Path, arg: &str) -> Result<(String, Kind), Miss> {
    let rel = folded.strip_prefix(&corpus.root).ok().and_then(rel_string);
    match rel.and_then(|rel| corpus.kind(&rel).map(|kind| (rel, kind))) {
        Some(found) => Ok(found),
        None => Err(Miss::not_found(arg)),
    }
}

/// True when the canonical path of `arg` joined to the cwd lies inside `root`: a symlinked spelling of a path inside the root.
/// This is one call and never a walk; a path that cannot be canonicalised names nothing (design §3.5d scope).
fn lands_in_root(root: &Path, cwd: &Path, arg: &str) -> bool {
    dunce::canonicalize(cwd.join(arg)).is_ok_and(|full| full.starts_with(root))
}

/// Adds a label to the scope line, unless it is already there.
fn label(resolved: &mut Resolved, label: String) {
    if !resolved.labels.contains(&label) {
        resolved.labels.push(label);
    }
}

/// Every walked entry under the root, the root itself included, in label order, as the corpus kept them: no file is read. Paths
/// are root-relative with `/` separators (design §4.4).
pub fn walked(corpus: &Corpus) -> Vec<Entry> {
    corpus
        .entries()
        .map(|(rel, kind)| Entry {
            rel: rel.to_string(),
            path: corpus.root.join(rel),
            is_dir: kind == Kind::Dir,
        })
        .collect()
}

/// The walked files and folders that the scope holds: the counts of the scope line (design §3.5d). The root is a folder, so it
/// counts when the scope holds it.
pub fn counts(scope: &Scope, tree: &[Entry]) -> (usize, usize) {
    let files = tree
        .iter()
        .filter(|entry| !entry.is_dir && scope.holds_file(&entry.rel))
        .count();
    let folders = tree
        .iter()
        .filter(|entry| entry.is_dir && scope.holds_folder(&entry.rel))
        .count();
    (files, folders)
}

#[cfg(test)]
mod tests {
    use super::scope_path;
    use crate::corpus::Corpus;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// A canonical root holding `src/lib.rs` with a header.
    fn root() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let root = dunce::canonicalize(dir.path()).expect("a canonical root");
        fs::create_dir_all(root.join(".memex")).expect("the store folder");
        fs::create_dir_all(root.join("src")).expect("the src folder");
        fs::write(root.join("src").join("lib.rs"), "//! The library.\n").expect("the file");
        (dir, root)
    }

    fn found(root: &Path, arg: &str) -> Option<String> {
        let corpus = Corpus::load(root, None);
        scope_path(&corpus, root, None, arg).ok().map(|(rel, _)| rel)
    }

    #[cfg(unix)]
    #[test]
    fn an_absolute_path_through_a_link_inside_the_root_names_nothing() {
        let (_dir, root) = root();
        let other = tempfile::tempdir().expect("another folder");
        std::os::unix::fs::symlink(other.path(), root.join("out")).expect("the link");
        let arg = root.join("out").to_string_lossy().into_owned();
        assert_eq!(found(&root, &arg), None);
        assert_eq!(found(&root, &format!("{arg}/..")).as_deref(), Some(""));
    }

    #[test]
    fn an_absolute_path_inside_the_root_names_its_entry() {
        let (_dir, root) = root();
        let arg = root.join("src").to_string_lossy().into_owned();
        assert_eq!(found(&root, &arg).as_deref(), Some("src"));
    }

    #[cfg(windows)]
    #[test]
    fn a_drive_letter_or_verbatim_spelling_of_the_root_is_found() {
        let (_dir, root) = root();
        let plain = root.join("src").to_string_lossy().into_owned();
        let lower = format!("{}{}", plain[..1].to_ascii_lowercase(), &plain[1..]);
        assert_eq!(found(&root, &lower).as_deref(), Some("src"));
        let verbatim = format!(r"\\?\{plain}");
        assert_eq!(found(&root, &verbatim).as_deref(), Some("src"));
        assert_eq!(found(&root, r"C:\"), None);
    }
}
