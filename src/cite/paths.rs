//! Resolves a path or link from the citing folder, then the root, never above it (design §2.3).
//! It also gives a target's one-liner, which a folder takes from its README.
//! Cases: CITE-PATH-01, CITE-PATH-02, CITE-PATH-03, CITE-PATH-05, CITE-LINK-01, CITE-LINK-03, CITE-FOLDER-01

use crate::corpus::{readme_label, Corpus, Kind};

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

/// The root-relative paths that `raw` can name, in the order they are tried. Empty when every one climbs above the root.
fn candidates(base: &str, raw: &str) -> Vec<String> {
    match raw.strip_prefix('/') {
        Some(from_root) => join("", from_root).into_iter().collect(),
        None => [join(base, raw), join("", raw)]
            .into_iter()
            .flatten()
            .collect(),
    }
}

/// The root-relative path that `token` names from the folder `base`: empty and `.` parts drop out, and each `..` climbs one
/// folder. None when a `..` would climb above the root.
fn join(base: &str, token: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for part in token.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.join("/"))
}

/// True when the last dot of a word is followed by one or more letters or digits.
fn has_extension(word: &str) -> bool {
    word.rsplit_once('.').is_some_and(|(_, ext)| {
        !ext.is_empty() && ext.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::join;

    #[test]
    fn a_join_stays_inside_the_root() {
        assert_eq!(join("src/deep", "../cli.rs").as_deref(), Some("src/cli.rs"));
        assert_eq!(join("", "./a/./b/").as_deref(), Some("a/b"));
        assert_eq!(join("", "../x"), None);
        assert_eq!(join("a", "../../x"), None);
    }
}
