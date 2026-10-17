//! Finds the shared names and duplicate ids of the collections (design §2.2 and §4.2 Names).
//! A collection named like the graph or a log stream still loads, so each such name gets one warning that names its first file, and
//! `--in` keeps meaning the graph or the stream. Two docs of a collection with one id are a duplicate, which `info` reports.
//! Cases: COLL-SHARED-01, COLL-SHARED-02, COLL-SHARED-03, COLL-SHARED-04, COLL-DUP-01, COLL-DUP-02, COLL-DUP-03

use std::collections::BTreeMap;

use crate::corpus::{Corpus, Doc, HEADERS, LOGS, READMES};
use crate::error;

/// The file that the manifest of the collection `name` is read from, root-relative with `/` separators (design §4.2). A doc that has
/// no path, which a json or folder manifest gives, is named by this file.
pub fn manifest_file(name: &str) -> String {
    format!(".memex/collections/{name}.json")
}

/// The file a doc comes from, for a warning (design §2.2 and §4.2): its path, or for a doc with no path, its manifest's file.
pub fn source(doc: &Doc) -> String {
    doc.path
        .clone()
        .unwrap_or_else(|| manifest_file(&doc.collection))
}

/// The warnings of the collections that a graph or log name also names (design §4.2 Names), one line each, in name order. Each names
/// the file the collection starts in: the first of its docs' files in byte order, or its manifest's file when it has no doc. Only
/// the docs of such collections are looked at, so this costs no allocation per doc on every verb.
pub fn name_warnings(corpus: &Corpus) -> Vec<String> {
    let mut first: BTreeMap<&str, String> = BTreeMap::new();
    for doc in corpus
        .all_docs()
        .filter(|doc| !doc.is_graph() && !corpus.is_log(doc) && is_shared(corpus, &doc.collection))
    {
        let file = source(doc);
        match first.get_mut(doc.collection.as_str()) {
            Some(kept) if file < *kept => *kept = file,
            Some(_) => {}
            None => {
                first.insert(doc.collection.as_str(), file);
            }
        }
    }
    for name in corpus
        .manifests
        .iter()
        .map(|loaded| loaded.manifest.name.as_str())
    {
        if is_shared(corpus, name) {
            first.entry(name).or_insert_with(|| manifest_file(name));
        }
    }
    first
        .into_iter()
        .filter_map(|(name, file)| shared_line(corpus, name, &file))
        .collect()
}

/// True for a collection name that the graph, `logs` or a log stream also takes (design §4.2 Names).
fn is_shared(corpus: &Corpus, name: &str) -> bool {
    matches!(name, READMES | HEADERS | LOGS) || corpus.is_stream(name)
}

/// The warning for a collection named like the graph or a log stream, which names its file (design §4.2 Names). None for any other
/// name. `logs` takes the stream form, since `--in logs` means every stream.
fn shared_line(corpus: &Corpus, name: &str, file: &str) -> Option<String> {
    if name == READMES || name == HEADERS {
        Some(error::warning(&format!(
            "collection '{name}' (in {file}) shares a name with the graph; --in {name} means the graph — rename it to reach it with --in"
        )))
    } else if name == LOGS || corpus.is_stream(name) {
        Some(error::warning(&format!(
            "collection '{name}' (in {file}) shares a name with a log stream; --in {name} means the stream"
        )))
    } else {
        None
    }
}

/// The labels that more than one collection doc holds, each with those docs, in label order (design §2.2 Duplicate ids). The graph's
/// docs and the log entries are left out: their labels are unique by construction.
pub fn duplicates(corpus: &Corpus) -> Vec<(String, Vec<&Doc>)> {
    let mut by_label: BTreeMap<&str, Vec<&Doc>> = BTreeMap::new();
    for doc in corpus
        .all_docs()
        .filter(|doc| !doc.is_graph() && !corpus.is_log(doc))
    {
        by_label.entry(doc.label.as_str()).or_default().push(doc);
    }
    by_label
        .into_iter()
        .filter(|(_, docs)| docs.len() > 1)
        .map(|(label, docs)| (label.to_string(), docs))
        .collect()
}
