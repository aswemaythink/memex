//! Builds the citation index (design §2.3): each doc's cites and the backlinks they give.
//! Cases: CITE-BARE-01, CITE-BARE-02, CITE-BARE-03, CITE-PREFIX-01, CITE-PREFIX-02, CITE-COLL-01, CITE-COLL-02, CITE-COLL-03, CITE-COLL-04, CITE-PATH-01, CITE-PATH-02, CITE-PATH-03, CITE-PATH-04, CITE-PATH-05, CITE-PATH-06, CITE-LINK-01, CITE-LINK-02, CITE-LINK-03, CITE-LINK-05, CITE-MAN-01, CITE-UNRES-01, CITE-FOLDER-01, CITE-FOLDER-02, CITE-FENCE-01, CITE-SELF-01, CITE-BACK-01, CITE-BACK-02, CITE-BACK-03, CITE-BACK-04, CITE-BACK-05, SRCH-ORPHAN-01, SRCH-ORPHAN-02

use std::collections::{BTreeMap, BTreeSet};

use super::paths::{self, Named};
use super::tokens::{self, Token};
use super::{Cite, Citer};
use crate::corpus::{label_of, parent_of, Corpus, Doc};
use crate::fence;
use crate::par;

/// The collections of intent docs, which have a file and a folder. Their path tokens resolve from their own folder first.
const INTENT: [&str; 3] = ["readmes", "headers", "cases"];

/// The reserved collection names (design §3.1). A `collection:id` label may always use them, whether or not a doc holds them.
const RESERVED: [&str; 4] = ["readmes", "headers", "cases", "logs"];

/// The citations of every doc, and the backlinks of every target.
#[derive(Debug, Default)]
pub struct Index {
    /// The citations of each citing doc, by its label, in first-appearance order.
    cites: BTreeMap<String, Vec<Cite>>,
    /// The docs that cite each target, by the target's label, sorted by the citing doc's label.
    citers: BTreeMap<String, Vec<Citer>>,
    /// The label of each manifest doc, by its path: a link to that path cites the doc, not the bare path.
    manifest: BTreeMap<String, String>,
}

/// The names that a scan resolves against: the bare-citable ids that docs declare, and the collections that exist.
struct Names {
    /// Each bare-citable id with the labels of the docs that hold it, in label order.
    declared: BTreeMap<String, Vec<String>>,
    /// The collection names that a `collection:id` label may use.
    collections: BTreeSet<String>,
    /// The label of each manifest doc, by its path, as `Index::manifest` holds it.
    manifest: BTreeMap<String, String>,
}

/// What a token points at.
enum Hit {
    /// A doc or a walked entry, by its label.
    Target(String),
    /// An explicit ref that names nothing, as it was written.
    Unresolved(String),
    /// A doc that a prefix names, with the prefix.
    Family { label: String, prefix: String },
}

/// The hits of one doc, reduced to the citations it makes and the targets it cites by prefix.
#[derive(Default)]
struct Gathered {
    /// The citations, in first-appearance order.
    cites: Vec<Cite>,
    /// The targets it cites directly.
    direct: BTreeSet<String>,
    /// The targets it cites by prefix, each with the prefix that names it first.
    families: BTreeMap<String, String>,
}

impl Index {
    /// The index of every doc in `corpus`: each doc's citations, and the backlinks of every target. The docs are scanned on several
    /// threads, then recorded in the order of `all_docs`, so the backlinks come out as a sequential scan gives them.
    pub fn build(corpus: &Corpus) -> Index {
        let docs: Vec<&Doc> = corpus.all_docs().collect();
        let names = Names::of(&docs);
        let hits = par::map(&docs, |doc| hits_of(corpus, doc, &names));
        let mut index = Index {
            manifest: names.manifest.clone(),
            ..Index::default()
        };
        for (doc, hits) in docs.into_iter().zip(hits) {
            index.record(corpus, doc, hits);
        }
        for citers in index.citers.values_mut() {
            citers.sort_by(|left, right| left.label.cmp(&right.label));
        }
        index
    }

    /// The citations a doc makes, in first-appearance order. Empty for a doc that cites nothing, and for a label that is no doc.
    pub fn cites(&self, label: &str) -> &[Cite] {
        self.cites.get(label).map_or(&[], Vec::as_slice)
    }

    /// The docs that cite a target, sorted by label. A doc that cites it only by prefix is marked with the prefix. Roster
    /// `<-N` counts the entries with no prefix, since prefix citations are not counted there (design §2.3).
    pub fn cited_by(&self, label: &str) -> &[Citer] {
        self.citers.get(label).map_or(&[], Vec::as_slice)
    }

    /// The label of the manifest doc at this root-relative path, when one holds it. Its citations and backlinks are keyed by
    /// that label, so `show <path>` reads them under it (design §2.3).
    pub fn manifest_label(&self, rel: &str) -> Option<&str> {
        self.manifest.get(rel).map(String::as_str)
    }

    /// The number of docs that cite a target directly: `cited_by` without its prefix citations, which is the `<-N` of a roster
    /// (design §2.3).
    pub fn direct_count(&self, label: &str) -> usize {
        self.cited_by(label)
            .iter()
            .filter(|citer| citer.via.is_none())
            .count()
    }

    /// Records one doc's citations and the backlinks they give. Docs that share a label are next to each other in the scan,
    /// so a label's first doc keeps its citations and a repeat is not listed twice.
    fn record(&mut self, corpus: &Corpus, doc: &Doc, hits: Vec<Hit>) {
        let gathered = gather(corpus, doc, hits);
        for label in &gathered.direct {
            add_citer(self.citers.entry(label.clone()).or_default(), doc, None);
        }
        for (label, prefix) in gathered.families {
            if !gathered.direct.contains(&label) {
                let via = Some(format!("{prefix}*"));
                add_citer(self.citers.entry(label).or_default(), doc, via);
            }
        }
        if !gathered.cites.is_empty() {
            self.cites
                .entry(doc.label.clone())
                .or_insert(gathered.cites);
        }
    }
}

impl Names {
    /// The names of a scan: the ids that the docs declare with the case-id grammar, the collections they belong to, and the
    /// manifest docs by path.
    fn of(docs: &[&Doc]) -> Names {
        let mut declared: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut collections: BTreeSet<String> = RESERVED.into_iter().map(str::to_string).collect();
        let intent: BTreeSet<String> = docs
            .iter()
            .filter(|doc| INTENT.contains(&doc.collection.as_str()))
            .filter_map(|doc| doc.path.clone())
            .collect();
        let mut manifest: BTreeMap<String, String> = BTreeMap::new();
        for doc in docs {
            collections.insert(doc.collection.clone());
            if tokens::is_case_id(&doc.id) {
                declared
                    .entry(doc.id.clone())
                    .or_default()
                    .push(doc.label.clone());
            }
            if let Some(path) = manifest_path(doc, &intent) {
                manifest.entry(path).or_insert_with(|| doc.label.clone());
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
            .cloned()
            .unwrap_or_else(|| label_of(rel))
    }

    /// The labels of every declared id that starts with `prefix`, in id order.
    fn family(&self, prefix: &str) -> Vec<String> {
        self.declared
            .iter()
            .filter(|(id, _)| id.starts_with(prefix))
            .flat_map(|(_, labels)| labels.iter().cloned())
            .collect()
    }
}

/// The hits of every token in a doc's live lines, in order. Its path tokens resolve from its own folder when it is an
/// intent doc (design §2.3).
fn hits_of(corpus: &Corpus, doc: &Doc, names: &Names) -> Vec<Hit> {
    let base = base_folder(doc);
    let known = |name: &str| names.collections.contains(name);
    let mut hits = Vec::new();
    for line in fence::live_lines(&doc.text) {
        for token in tokens::tokens(line, &known) {
            token_hits(corpus, &base, names, token, &mut hits);
        }
    }
    hits
}

/// Adds the hits of one token, in the precedence of design §2.3: a link, then a label, then a prefix, then a bare id, then a
/// path.
fn token_hits(corpus: &Corpus, base: &str, names: &Names, token: Token, hits: &mut Vec<Hit>) {
    match token {
        Token::Link(raw) => match paths::locate(corpus, base, &raw) {
            Named::Found(rel) => hits.push(Hit::Target(names.label_at(&rel))),
            Named::Missing => hits.push(Hit::Unresolved(raw)),
            Named::Escape => {}
        },
        Token::Label(label) => label_hits(corpus, label, hits),
        Token::Prefix(prefix) => {
            hits.extend(names.family(&prefix).into_iter().map(|label| Hit::Family {
                label,
                prefix: prefix.clone(),
            }))
        }
        Token::Word(word) => word_hits(corpus, base, names, &word, hits),
    }
}

/// Adds the hits of a `collection:id` label: each doc it names, or an unresolved ref when it names none.
fn label_hits(corpus: &Corpus, label: String, hits: &mut Vec<Hit>) {
    let named = named_docs(corpus, &label);
    if named.is_empty() {
        hits.push(Hit::Unresolved(label));
    } else {
        hits.extend(named.into_iter().map(|doc| Hit::Target(doc.label.clone())));
    }
}

/// The docs that a `collection:id` label names, read as `show` reads a label (design §3.1): the docs that hold the label, or
/// else the docs that a reserved spelling names. `readmes:` and `headers:` name a doc by its path, and `logs:` names the entry
/// of every stream that holds the id, so one id can name several entries.
fn named_docs<'a>(corpus: &'a Corpus, label: &str) -> Vec<&'a Doc> {
    let held = corpus.labelled(label);
    if !held.is_empty() {
        return held.iter().collect();
    }
    let Some((collection, id)) = label.split_once(':') else {
        return Vec::new();
    };
    corpus
        .all_docs()
        .filter(|doc| match collection {
            "logs" => corpus.is_log(doc) && doc.id == id,
            "readmes" | "headers" => doc.collection == collection && doc.id == id,
            _ => false,
        })
        .collect()
}

/// Adds the hits of a word: the docs that declare it when it is a bare id, and otherwise the walked entry that it names as a
/// path. A word that names nothing adds nothing, since only explicit refs can be unresolved.
fn word_hits(corpus: &Corpus, base: &str, names: &Names, word: &str, hits: &mut Vec<Hit>) {
    if let Some(labels) = names.declared.get(word) {
        hits.extend(labels.iter().cloned().map(Hit::Target));
    } else if paths::is_path_token(word) {
        if let Named::Found(rel) = paths::locate(corpus, base, word) {
            hits.push(Hit::Target(names.label_at(&rel)));
        }
    }
}

/// The path of a manifest doc: a doc of a collection that is not an intent collection, with a file that no intent doc holds. Its
/// path is the file that a link or a path token names, so the link cites the doc (design §2.3).
fn manifest_path(doc: &Doc, intent: &BTreeSet<String>) -> Option<String> {
    let path = doc.path.as_ref()?;
    let manifest = !INTENT.contains(&doc.collection.as_str());
    (manifest && !intent.contains(path)).then(|| path.clone())
}

/// The folder that a doc's path tokens resolve from first: the folder of its file for an intent doc, and the root for any
/// other doc, which has no folder (design §2.3).
fn base_folder(doc: &Doc) -> String {
    match doc.path.as_deref() {
        Some(path) if INTENT.contains(&doc.collection.as_str()) => {
            parent_of(path).unwrap_or("").to_string()
        }
        _ => String::new(),
    }
}

/// The label of the folder that a README is the doc of. A README never cites its own folder (design §2.3, self-citation).
fn own_folder(doc: &Doc) -> Option<String> {
    let path = doc.path.as_deref()?;
    (doc.collection == "readmes").then(|| label_of(parent_of(path).unwrap_or("")))
}

/// Reduces one doc's hits: each target once, in first-appearance order, with the doc's own label and folder left out.
fn gather(corpus: &Corpus, doc: &Doc, hits: Vec<Hit>) -> Gathered {
    let own = own_folder(doc);
    let is_self = |label: &str| label == doc.label || own.as_deref() == Some(label);
    let mut gathered = Gathered::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for hit in hits {
        match hit {
            Hit::Target(label) if is_self(&label) => {}
            Hit::Target(label) => {
                if seen.insert(label.clone()) {
                    gathered.cites.push(Cite {
                        title: paths::title_of(corpus, &label),
                        reference: label.clone(),
                        resolved: true,
                    });
                }
                gathered.direct.insert(label);
            }
            Hit::Unresolved(raw) => {
                if seen.insert(raw.clone()) {
                    gathered.cites.push(Cite {
                        reference: raw,
                        title: String::new(),
                        resolved: false,
                    });
                }
            }
            Hit::Family { label, prefix } => {
                if !is_self(&label) {
                    gathered.families.entry(label).or_insert(prefix);
                }
            }
        }
    }
    gathered
}

/// Adds the backlink that `doc` makes to a target's list, unless the previous entry is the same doc.
fn add_citer(list: &mut Vec<Citer>, doc: &Doc, via: Option<String>) {
    if list.last().is_none_or(|last| last.label != doc.label) {
        list.push(Citer {
            label: doc.label.clone(),
            title: doc.title.clone(),
            via,
        });
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
