//! Resolves a ref to what it names (design §3.1, steps 1 to 3): an absolute path, a path that the cwd or the root holds, or an exact `collection:id` label.
//! Cases: SHOW-FILE-01, SHOW-FILE-03, SHOW-DIR-01, SHOW-ROOT-01, SHOW-ROOT-03, SHOW-LOG-01, SHOW-LOG-02, SHOW-ERR-01, SHOW-ERR-02, SHOW-JSON-05

use std::path::{Component, Path};

use crate::corpus::{label_of, Corpus, Doc, Kind};
use crate::walk::rel_string;

/// What a ref names: a folder or file of the walked tree, or a doc whose label is not a path.
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
    /// The ref names exactly one target.
    Found(Target<'a>),
    /// Nothing matches the ref.
    NotFound,
    /// Several targets match: their labels, sorted.
    Ambiguous(Vec<String>),
}

/// Resolves `arg` as typed from the cwd, which is `cwd_rel` relative to the root (empty at the root). A relative path that
/// the cwd or the root holds wins when it is the only one; when both hold different entries it is ambiguous. `.` is the
/// root's own label, so it always names the root, whatever the cwd (design §3.1, step 2).
pub fn resolve<'a>(corpus: &'a Corpus, cwd_rel: &str, arg: &str) -> Outcome<'a> {
    if arg.is_empty() {
        return Outcome::NotFound;
    }
    if arg == "." {
        return found_path(corpus, String::new());
    }
    if Path::new(arg).is_absolute() {
        return absolute(corpus, arg);
    }
    let mut hits: Vec<String> = [normalize(cwd_rel, arg), normalize("", arg)]
        .into_iter()
        .flatten()
        .filter(|rel| corpus.kind(rel).is_some())
        .collect();
    hits.sort();
    hits.dedup();
    match hits.as_slice() {
        [] => by_label(corpus, arg),
        [only] => found_path(corpus, only.clone()),
        _ => Outcome::Ambiguous(hits.iter().map(|rel| label_of(rel)).collect()),
    }
}

/// An absolute path: the entry it names, when that entry is inside the root and the walk holds it.
fn absolute<'a>(corpus: &Corpus, arg: &str) -> Outcome<'a> {
    let Ok(full) = dunce::canonicalize(arg) else {
        return Outcome::NotFound;
    };
    match full.strip_prefix(&corpus.root).ok().and_then(rel_string) {
        Some(rel) => found_path(corpus, rel),
        None => Outcome::NotFound,
    }
}

/// The outcome for a root-relative path: found when the walk holds an entry there, not found otherwise.
fn found_path<'a>(corpus: &Corpus, rel: String) -> Outcome<'a> {
    match corpus.kind(&rel) {
        Some(kind) => Outcome::Found(Target::Path { rel, kind }),
        None => Outcome::NotFound,
    }
}

/// The exact `collection:id` label, when a doc has it.
fn by_label<'a>(corpus: &'a Corpus, label: &str) -> Outcome<'a> {
    match corpus.doc(label) {
        Some(doc) => Outcome::Found(Target::Doc(doc)),
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
    use super::normalize;

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
}
