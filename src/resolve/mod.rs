//! Resolves a ref to what it names, in the order of design §3.1: path, label, bare id, basename.
//! Cases: REF-ABS-01, REF-REL-01, REF-REL-02, REF-REL-03, REF-REL-04, REF-AMBIG-01, REF-COLL-01, REF-COLL-02, REF-BARE-01, REF-CAND-01

mod names;
mod outside;

use std::path::{Component, Path};

use crate::corpus::{label_of, Corpus, Doc, Kind};
use crate::walk::rel_string;

pub use outside::Elsewhere;

/// What a ref names: a folder or file of the current root, or a doc whose label is not a path.
pub enum Target<'a> {
    /// A folder or file, by its root-relative path. The root is the empty path.
    Path {
        /// The root-relative path.
        rel: String,
        /// Whether it is a folder or a file.
        kind: Kind,
    },
    /// A doc with a `collection:id` label.
    Doc(&'a Doc),
}

/// The result of resolving one ref.
pub enum Outcome<'a> {
    /// The ref names exactly one target of the current root.
    Found(Target<'a>),
    /// The ref names an entry of another root, which resolves against that root's own corpus (design §3.1). Boxed, as its corpus
    /// is large next to the other outcomes.
    Elsewhere(Box<Elsewhere>),
    /// Nothing matches the ref.
    NotFound,
    /// Several targets match: their labels, sorted. Memex never guesses between them.
    Ambiguous(Vec<String>),
}

/// Resolves `arg` as typed from `cwd`, which is an absolute path inside the root of `corpus`. `store` is the log store, which
/// a path in another root is read with (design §3.1).
pub fn resolve<'a>(corpus: &'a Corpus, cwd: &Path, store: Option<&Path>, arg: &str) -> Outcome<'a> {
    if arg.is_empty() {
        return Outcome::NotFound;
    }
    if arg == "." {
        return found_path(corpus, String::new());
    }
    if Path::new(arg).is_absolute() {
        return outside::absolute(corpus, store, arg);
    }
    relative(corpus, cwd, store, arg).unwrap_or_else(|| names::by_name(corpus, arg))
}

/// A relative path (design §3.1, step 2): the entry that the cwd or the root holds, when exactly one does, and an ambiguity
/// when both hold different entries. None when no path matches, so that the ref is tried as a name.
fn relative<'a>(
    corpus: &'a Corpus,
    cwd: &Path,
    store: Option<&Path>,
    arg: &str,
) -> Option<Outcome<'a>> {
    let cwd_rel = cwd
        .strip_prefix(&corpus.root)
        .ok()
        .and_then(rel_string)
        .unwrap_or_default();
    let Some(from_cwd) = normalize(&cwd_rel, arg) else {
        // The cwd-relative path climbs above the root, so it may name an entry of another root.
        return outside::relative(corpus, cwd, store, arg);
    };
    let mut hits: Vec<String> = [Some(from_cwd), normalize("", arg)]
        .into_iter()
        .flatten()
        .filter(|rel| corpus.kind(rel).is_some())
        .collect();
    hits.sort();
    hits.dedup();
    match hits.as_slice() {
        [] => None,
        [only] => Some(found_path(corpus, only.clone())),
        _ => Some(Outcome::Ambiguous(
            hits.iter().map(|rel| label_of(rel)).collect(),
        )),
    }
}

/// The outcome for a root-relative path: found when the walk holds an entry there, not found otherwise.
pub(super) fn found_path<'a>(corpus: &Corpus, rel: String) -> Outcome<'a> {
    match corpus.kind(&rel) {
        Some(kind) => Outcome::Found(Target::Path { rel, kind }),
        None => Outcome::NotFound,
    }
}

/// The root-relative path that `arg` names when joined to `base`, or None when it is not a relative path or a `..` climbs
/// above the root.
fn normalize(base: &str, arg: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for component in Path::new(arg).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Normal(part) => parts.push(part.to_str()?),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::{normalize, resolve, Outcome, Target};
    use crate::corpus::Corpus;
    use std::fs;

    #[test]
    fn dots_and_parents_stay_inside_the_root() {
        assert_eq!(
            normalize("src/deep", "../cli.rs").as_deref(),
            Some("src/cli.rs")
        );
        assert_eq!(normalize("", "./a/./b/"), Some("a/b".to_string()));
        assert_eq!(normalize("", "../x"), None);
        assert_eq!(normalize("a", "../../x"), None);
    }

    /// A unit test for an absolute path inside the root: no case can name the sandbox's absolute path, so the path is built here.
    #[test]
    fn an_absolute_path_inside_the_root_names_its_root_relative_entry() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let root = dunce::canonicalize(dir.path()).expect("a canonical root");
        fs::create_dir_all(root.join(".memex")).expect("the store folder");
        fs::create_dir_all(root.join("src")).expect("the src folder");
        fs::write(
            root.join("src").join("cli.rs"),
            "//! Parses the arguments.\n",
        )
        .expect("the file");
        let corpus = Corpus::load(&root, None);
        let arg = root
            .join("src")
            .join("cli.rs")
            .to_string_lossy()
            .into_owned();
        let Outcome::Found(Target::Path { rel, .. }) = resolve(&corpus, &root, None, &arg) else {
            panic!("an absolute path to a walked file is found");
        };
        assert_eq!(rel, "src/cli.rs");
    }
}
