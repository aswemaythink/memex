//! Finds the citations of a doc and the docs that cite a label (design §2.3). S3.1 fills it in; until then nothing is cited.
//! Cases: none (stub; CITE-* in S3.1 pins it)

use crate::corpus::Doc;

/// One citation found in a doc: its target, the target's one-liner, and whether the target exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cite {
    /// The label of the cited target.
    pub reference: String,
    /// The one-liner of the target; empty when it does not resolve.
    pub title: String,
    /// True when the target exists.
    pub resolved: bool,
}

/// The citations inside a doc. None are found yet.
pub fn cites(_doc: &Doc) -> Vec<Cite> {
    Vec::new()
}

/// The labels of the docs that cite `label`. None are found yet.
pub fn cited_by(_label: &str) -> Vec<String> {
    Vec::new()
}
