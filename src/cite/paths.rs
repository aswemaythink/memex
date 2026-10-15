//! Resolves the tokens of a doc to the docs and walked entries that they name (design §2.3).
//! A path or link resolves from the citing folder, then the root, never above it; a label, a bare id or a prefix resolves against
//! the names of the scan, and a target's one-liner is read here too.
//! A scan for some targets (`Wanted`) skips every token that cannot name any of them, a necessary condition: a bare id or a prefix
//! must be an id of a doc that holds a target's label; a label must be a target's, or name such a doc by its id; a path must end in
//! a name that a target's paths end in, unless it ends in `..` or in nothing, when it is always resolved (`last_name`).
//! Cases: CITE-BARE-01, CITE-BARE-02, CITE-BARE-03, CITE-PREFIX-01, CITE-PREFIX-02, CITE-COLL-01, CITE-COLL-02, CITE-COLL-03, CITE-COLL-04, CITE-PATH-01, CITE-PATH-02, CITE-PATH-03, CITE-PATH-04, CITE-PATH-05, CITE-PATH-06, CITE-LINK-01, CITE-LINK-02, CITE-LINK-03, CITE-LINK-05, CITE-MAN-01, CITE-UNRES-01, CITE-FOLDER-01, CITE-FOLDER-02

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

use super::tokens::{self, Token};
use crate::corpus::{
    label_of, name_of, parent_of, readme_label, Corpus, Doc, Kind, CASES, HEADERS, READMES,
    RESERVED,
};
use crate::fence;
use crate::refs::{candidates, has_extension, is_case_id};

/// The collections of intent docs, which have a file and a folder. Their path tokens resolve from their own folder first.
const INTENT: [&str; 3] = [READMES, HEADERS, CASES];

/// Where a path names the walk.
pub enum Named {
    /// The root-relative path of a walked folder or file.
    Found(String),
    /// A candidate was made, and the walk holds no entry at its path.
    Missing,
    /// Every candidate climbs above the root, so the path is not a citation.
    Escape,
}

/// What a token points at.
pub enum Hit {
    /// A doc or a walked entry, by its label.
    Target(String),
    /// An explicit ref that names nothing, as it was written.
    Unresolved(String),
    /// A doc that a prefix names, with the prefix.
    Family { label: String, prefix: String },
}

/// The names that a scan resolves against: the bare-citable ids that docs declare, the collections that exist, and the manifest
/// docs by path.
pub struct Names<'a> {
    /// Each bare-citable id with the labels of the docs that hold it, in label order.
    declared: BTreeMap<&'a str, Vec<&'a str>>,
    /// The collection names that a `collection:id` label may use.
    collections: BTreeSet<&'a str>,
    /// The label of each manifest doc, by its path: a link to that path cites the doc, not the bare path.
    manifest: BTreeMap<&'a str, &'a str>,
}

/// The targets that a targeted scan looks for, with the tokens that can name them. A token that cannot name any of them is skipped
/// before it is resolved; every other token is resolved as a full scan resolves it, so the targets' hits are the same. Its tests
/// are necessary conditions: a bare id or a family prefix must be the id of a doc that holds a target's label; a label must be a
/// target's, or name a doc of a target by its id; a path must end in a name that a target's paths end in, unless it ends in `..`
/// or in nothing, when it is always resolved (`last_name`).
pub struct Wanted<'a> {
    /// The labels of the targets.
    labels: BTreeSet<&'a str>,
    /// The ids of the docs that hold a target's label.
    ids: BTreeSet<&'a str>,
    /// The last names that the targets' paths end in: their own, and those of the manifest paths that label them.
    names: BTreeSet<&'a str>,
}

impl<'a> Names<'a> {
    /// The names of a scan over `docs`: the ids that they declare with the case-id grammar, the collections they belong to, and
    /// the manifest docs by path. A manifest doc's path is one that no intent doc holds.
    pub fn of(docs: &[&'a Doc]) -> Names<'a> {
        let mut declared: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        let mut collections: BTreeSet<&str> = RESERVED.into_iter().collect();
        let mut manifest: BTreeMap<&str, &str> = BTreeMap::new();
        for doc in docs {
            collections.insert(doc.collection.as_str());
            if is_case_id(&doc.id) {
                declared
                    .entry(doc.id.as_str())
                    .or_default()
                    .push(doc.label.as_str());
            }
            if let Some(path) = doc.path.as_deref().filter(|_| !is_intent(doc)) {
                manifest.entry(path).or_insert(doc.label.as_str());
            }
        }
        if !manifest.is_empty() {
            for doc in docs.iter().filter(|doc| is_intent(doc)) {
                if let Some(path) = doc.path.as_deref() {
                    manifest.remove(path);
                }
            }
        }
        Names {
            declared,
            collections,
            manifest,
        }
    }

    /// The label that a walked entry at `rel` is cited by: the `collection:id` of the manifest doc at that path, else the path
    /// itself (design §2.3, §3.1).
    fn label_at(&self, rel: &str) -> String {
        self.manifest
            .get(rel)
            .map_or_else(|| label_of(rel), |label| (*label).to_string())
    }

    /// The labels of every declared id that starts with `prefix`, in id order. The ids with a prefix are one run of the sorted map,
    /// which starts at the prefix itself.
    fn family(&self, prefix: &str) -> Vec<String> {
        self.declared
            .range::<str, _>((Bound::Included(prefix), Bound::Unbounded))
            .take_while(|(id, _)| id.starts_with(prefix))
            .flat_map(|(_, labels)| labels.iter().map(|label| (*label).to_string()))
            .collect()
    }
}

impl<'a> Wanted<'a> {
    /// The filter for the targets `labels`: the ids of their docs, and the last names of their paths and of the manifest paths that
    /// label them (design §2.3).
    pub fn of(corpus: &'a Corpus, names: &Names<'a>, labels: &BTreeSet<&'a str>) -> Wanted<'a> {
        let mut wanted = Wanted {
            labels: labels.clone(),
            ids: BTreeSet::new(),
            names: BTreeSet::new(),
        };
        for &label in labels {
            wanted.names.insert(name_of(label));
            wanted
                .ids
                .extend(corpus.labelled(label).iter().map(|doc| doc.id.as_str()));
        }
        for (&path, held) in &names.manifest {
            if labels.contains(held) {
                wanted.names.insert(name_of(path));
            }
        }
        wanted
    }

    /// True for a token that can name a target: every token that the scan must resolve.
    fn admits(&self, token: &Token<'_>) -> bool {
        match *token {
            Token::Word(word) => {
                self.ids.contains(word) || (is_path_token(word) && self.may_name(word))
            }
            Token::Link(raw) => self.may_name(raw),
            Token::Label(label) => {
                self.labels.contains(label)
                    || label
                        .split_once(':')
                        .is_some_and(|(_, id)| self.ids.contains(id))
            }
            Token::Prefix(prefix) => self.has_id_from(prefix),
        }
    }

    /// True when some id of a target's doc starts with `prefix`. The ids are sorted, so the first id from `prefix` on decides it.
    fn has_id_from(&self, prefix: &str) -> bool {
        self.ids
            .range::<str, _>((Bound::Included(prefix), Bound::Unbounded))
            .next()
            .is_some_and(|id| id.starts_with(prefix))
    }

    /// True when a path token's last name is one of the targets', or the token climbs out of its folder and is always resolved.
    fn may_name(&self, raw: &str) -> bool {
        last_name(raw).is_none_or(|name| self.names.contains(name))
    }
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

/// The last name that the path of `raw` ends in: its last part that is neither empty nor `.`. Every path that `raw` resolves to
/// ends in this name, since the join pushes it last. None when that part is `..`, or when there is none: then the folder that
/// `raw` climbs to may be the citing doc's own folder, so such a token is always resolved (design §2.3).
pub fn last_name(raw: &str) -> Option<&str> {
    let part = raw.rsplit('/').find(|part| !matches!(*part, "" | "."))?;
    (part != "..").then_some(part)
}

/// True for a doc of an intent collection, whose path is a file that the walk holds (design §2.3).
pub fn is_intent(doc: &Doc) -> bool {
    INTENT.contains(&doc.collection.as_str())
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

/// The hits of every token in a doc's live lines, in order. Its path tokens resolve from its own folder when it is an intent doc
/// (design §2.3). With a `wanted` target, a token that cannot name it is skipped before it is resolved.
pub fn hits_of(corpus: &Corpus, doc: &Doc, names: &Names, wanted: Option<&Wanted>) -> Vec<Hit> {
    let base = base_folder(doc);
    let known = |name: &str| names.collections.contains(name);
    let mut hits = Vec::new();
    let mut found = Vec::new();
    for line in fence::live_lines(&doc.text) {
        tokens::tokens(line, &known, &mut found);
        for token in found.drain(..) {
            if wanted.is_none_or(|wanted| wanted.admits(&token)) {
                token_hits(corpus, base, names, token, &mut hits);
            }
        }
    }
    hits
}

/// The folder that a doc's path tokens resolve from first: the folder of its file for an intent doc, and the root for any
/// other doc, which has no folder (design §2.3).
fn base_folder(doc: &Doc) -> &str {
    match doc.path.as_deref() {
        Some(path) if is_intent(doc) => parent_of(path).unwrap_or(""),
        _ => "",
    }
}

/// Adds the hits of one token, in the precedence of design §2.3: a link, then a label, then a prefix, then a bare id, then a
/// path.
fn token_hits(corpus: &Corpus, base: &str, names: &Names, token: Token<'_>, hits: &mut Vec<Hit>) {
    match token {
        Token::Link(raw) => match locate(corpus, base, raw) {
            Named::Found(rel) => hits.push(Hit::Target(names.label_at(&rel))),
            Named::Missing => hits.push(Hit::Unresolved(raw.to_string())),
            Named::Escape => {}
        },
        Token::Label(label) => label_hits(corpus, label, hits),
        Token::Prefix(prefix) => {
            hits.extend(names.family(prefix).into_iter().map(|label| Hit::Family {
                label,
                prefix: prefix.to_string(),
            }))
        }
        Token::Word(word) => word_hits(corpus, base, names, word, hits),
    }
}

/// Adds the hits of a `collection:id` label: each doc it names, or an unresolved ref when it names none.
fn label_hits(corpus: &Corpus, label: &str, hits: &mut Vec<Hit>) {
    let named = corpus.named(label);
    if named.is_empty() {
        hits.push(Hit::Unresolved(label.to_string()));
    } else {
        hits.extend(named.into_iter().map(|doc| Hit::Target(doc.label.clone())));
    }
}

/// Adds the hits of a word: the docs that declare it when it is a bare id, and otherwise the walked entry that it names as a
/// path. A word that names nothing adds nothing, since only explicit refs can be unresolved.
fn word_hits(corpus: &Corpus, base: &str, names: &Names, word: &str, hits: &mut Vec<Hit>) {
    if let Some(labels) = names.declared.get(word) {
        hits.extend(labels.iter().map(|label| Hit::Target((*label).to_string())));
    } else if is_path_token(word) {
        if let Named::Found(rel) = locate(corpus, base, word) {
            hits.push(Hit::Target(names.label_at(&rel)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A doc with a path, as the names of a scan read it.
    fn doc(label: &str, collection: &str, path: &str) -> Doc {
        Doc {
            label: label.to_string(),
            collection: collection.to_string(),
            id: label.rsplit(':').next().unwrap_or(label).to_string(),
            path: Some(path.to_string()),
            title: String::new(),
            text: String::new(),
            fields: Vec::new(),
        }
    }

    #[test]
    fn a_manifest_doc_is_labelled_at_its_path_unless_an_intent_doc_holds_that_path() {
        let readme = doc("README.md", "readmes", "README.md");
        let copy = doc("docs:README", "docs", "README.md");
        let guide = doc("docs:guide", "docs", "guide.md");
        let names = Names::of(&[&readme, &copy, &guide]);
        assert_eq!(names.label_at("guide.md"), "docs:guide");
        assert_eq!(names.label_at("README.md"), "README.md");
    }
}
