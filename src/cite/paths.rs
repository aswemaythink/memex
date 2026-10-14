//! Resolves a path or link from the citing folder, then the root, never above it (design §2.3).
//! It also gives a target's one-liner, which a folder takes from its README.
//! Cases: CITE-PATH-01, CITE-PATH-02, CITE-PATH-03, CITE-PATH-05, CITE-LINK-01, CITE-LINK-03, CITE-FOLDER-01

use crate::corpus::{readme_label, Corpus, Kind};
use crate::refs::{candidates, has_extension};

/// Where a path names the walk.
pub enum Named {
    /// The root-relative path of a walked folder or file.
    Found(String),
    /// A candidate was made, and the walk holds no entry at its path.
    Missing,
    /// Every candidate climbs above the root, so the path is not a citation.
    Escape,
}

/// Resolves `raw` from the folder `base`, which is empty for the root. A leading `/` names a root-relative path. Any other
/// path is tried from `base` first and then from the root (design §2.3).
pub fn locate(corpus: &Corpus, base: &str, raw: &str) -> Named {
    let candidates = candidates(base, raw);
    if candidates.is_empty() {
        return Named::Escape;
    }
    match candidates
        .into_iter()
        .find(|rel| corpus.kind(rel).is_some())
    {
        Some(rel) => Named::Found(rel),
        None => Named::Missing,
    }
}

/// True for a word that can name a path (design §2.3): it has a letter or a digit, and either a `/` or a `.ext`. A word of
/// punctuation alone, such as `/`, does not.
pub fn is_path_token(word: &str) -> bool {
    word.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && (word.contains('/') || has_extension(word))
}

/// The one-liner of the target with this label: a folder's README, or the doc with the label. Empty when there is none.
pub fn title_of(corpus: &Corpus, label: &str) -> String {
    let rel = if label == "." { "" } else { label };
    let key = match corpus.kind(rel) {
        Some(Kind::Dir) => readme_label(rel),
        _ => label.to_string(),
    };
    corpus
        .doc(&key)
        .map(|doc| doc.title.clone())
        .unwrap_or_default()
}
