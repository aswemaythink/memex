//! Counts each in-scope file's lines for `length` and `thin` (design §3.5d Lines).
//! The corpus kept each file's head facts as it loaded, so a file that its head holds whole is counted without a read; a longer one
//! is read past its head only, on several threads (`par.rs`).
//! Cases: CHK-LENGTH-01, CHK-LENGTH-03, CHK-LENGTH-04, CHK-LENGTH-05, CHK-LENGTH-06, CHK-THIN-01, CHK-REFINE-03

use super::scope::Scope;
use crate::corpus::Corpus;
use crate::intent::header::lines::{self, Head};
use crate::par;

/// One in-scope file with a comment family, with its line counts (design §3.5d Lines).
pub struct Counted {
    /// The root-relative path of the file, with `/` separators.
    pub rel: String,
    /// The lines of the file: its whole text, and the header and license blocks that its head holds.
    pub counts: lines::Counts,
}

/// The line counts of each file in the scope that has a comment family, in path order. A file whose head could not be read has no
/// head facts and is left out, since its read already warned (design §4.2); so is one whose rest can no longer be read.
pub fn in_scope(corpus: &Corpus, scope: &Scope) -> Vec<Counted> {
    let files: Vec<(&String, &Head)> = corpus
        .heads
        .iter()
        .filter(|(rel, _)| scope.holds_file(rel))
        .collect();
    par::map(&files, |(rel, head)| {
        let counts = head.counts(&corpus.root.join(rel)).ok()?;
        Some(Counted {
            rel: (*rel).clone(),
            counts,
        })
    })
    .into_iter()
    .flatten()
    .collect()
}
