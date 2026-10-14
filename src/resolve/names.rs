//! Resolves a ref that is not a path: a label, a bare id or a basename (design §3.1, steps 3 to 6).
//! Cases: REF-COLL-01, REF-COLL-02, REF-AMBIG-02, REF-AMBIG-03, REF-BARE-01, REF-BARE-02, REF-BARE-03, REF-BASE-01, REF-BASE-02, REF-BASE-03, REF-CAND-01, REF-RESERVED-01, REF-RESERVED-02, REF-RESERVED-03, REF-RESERVED-04, REF-TRIP-01, REF-REL-05, CASE-DUP-01, CASE-BARE-01

use crate::corpus::{name_of, Corpus, Doc, Kind, HEADERS, READMES};

use super::{Outcome, Target};

/// The outcome for a ref that is not a path: the docs it names as a label (design §3.1), else as a bare id, else the one walked
/// file with that basename. A ref that matches nothing is not found.
pub fn by_name<'a>(corpus: &'a Corpus, arg: &str) -> Outcome<'a> {
    let named = corpus.named(arg);
    if !named.is_empty() {
        return pick(corpus, named);
    }
    let bare = bare_id(corpus, arg);
    if !bare.is_empty() {
        return pick(corpus, bare);
    }
    basename(corpus, arg)
}

/// The docs whose id is the ref: the exact matches, or when none match, the ones that differ from it only in case (design
/// §3.1, step 4). A README or a header is named by its path, which step 2 matches case-sensitively, so it is no bare id:
/// `readme.md` does not name `README.md`.
fn bare_id<'a>(corpus: &'a Corpus, arg: &str) -> Vec<&'a Doc> {
    let ids: Vec<&'a Doc> = corpus.all_docs().filter(|doc| !is_intent(doc)).collect();
    let exact: Vec<&'a Doc> = ids.iter().copied().filter(|doc| doc.id == arg).collect();
    if !exact.is_empty() {
        return exact;
    }
    let lower = arg.to_lowercase();
    ids.into_iter()
        .filter(|doc| doc.id.to_lowercase() == lower)
        .collect()
}

/// True for a README or a header: an intent doc, whose id is its path (design §3.1).
fn is_intent(doc: &Doc) -> bool {
    matches!(doc.collection.as_str(), READMES | HEADERS)
}

/// The outcome for the docs that one ref names: the target of the only one, or else every one as a candidate.
fn pick<'a>(corpus: &Corpus, docs: Vec<&'a Doc>) -> Outcome<'a> {
    match docs.as_slice() {
        [] => Outcome::NotFound,
        [only] => Outcome::Found(target_of(only)),
        _ => {
            let mut labels: Vec<String> = docs.iter().map(|doc| candidate(corpus, doc)).collect();
            labels.sort();
            Outcome::Ambiguous(labels)
        }
    }
}

/// The target of a doc: a README or a header is its file, which `show` prints as a path; any other doc is a doc.
fn target_of(doc: &Doc) -> Target<'_> {
    match doc.collection.as_str() {
        READMES | HEADERS => Target::Path {
            rel: doc.id.clone(),
            kind: Kind::File,
        },
        _ => Target::Doc(doc),
    }
}

/// The text that names a doc in a candidate list: its label, or for a label that several docs share, the file that declares
/// it. No form names one of two docs with the same label, so the defining file is the best that can be listed (design §4.2).
fn candidate(corpus: &Corpus, doc: &Doc) -> String {
    if corpus.labelled(&doc.label).len() > 1 {
        doc.path.clone().unwrap_or_else(|| doc.label.clone())
    } else {
        doc.label.clone()
    }
}

/// The one walked file whose basename is the ref (design §3.1, step 5). Folders are not matched, and nothing matches by
/// anything but the exact name.
fn basename<'a>(corpus: &Corpus, arg: &str) -> Outcome<'a> {
    let mut found: Vec<String> = corpus
        .files()
        .filter(|rel| name_of(rel) == arg)
        .map(str::to_string)
        .collect();
    found.sort();
    match found.len() {
        0 => Outcome::NotFound,
        1 => Outcome::Found(Target::Path {
            rel: found.swap_remove(0),
            kind: Kind::File,
        }),
        _ => Outcome::Ambiguous(found),
    }
}
