//! Says which files a header is expected on, and which have one (design §3.5d kind 1, §3.5e Counts).
//! `audit`'s `headers` kind and `tree`'s counts both read it, so the two never disagree.
//! Cases: AUD-HEADERS-01, AUD-HEADERS-02, AUD-HEADERS-03, TREE-COUNT-01

use crate::corpus::{Corpus, Doc};

/// True when the graph holds a header doc for the file at this root-relative path.
pub fn has_header(corpus: &Corpus, rel: &str) -> bool {
    corpus.labelled(rel).iter().any(Doc::is_header)
}

/// The files a header is expected on, in path order, each with whether it has one: the walked files with a comment family that are
/// neither empty nor binary (design §2.1). The corpus keeps their head facts only when it was loaded with the line facts
/// (`here::load_with_lines`); a corpus loaded without them has none. A file that could not be read has no facts and is not judged.
pub fn expected(corpus: &Corpus) -> impl Iterator<Item = (&str, bool)> {
    corpus
        .heads
        .iter()
        .filter(|(_, head)| !head.is_empty())
        .map(|(rel, _)| (rel.as_str(), has_header(corpus, rel)))
}
