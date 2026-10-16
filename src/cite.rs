//! Holds a doc's citations and backlinks (design §2.3): the Cite and Citer types and their index.
//! Cases: CITE-BARE-01, CITE-BARE-02, CITE-PREFIX-01, CITE-PREFIX-02, CITE-COLL-01, CITE-COLL-02, CITE-COLL-03, CITE-COLL-04, CITE-PATH-01, CITE-PATH-02, CITE-PATH-03, CITE-PATH-04, CITE-PATH-05, CITE-LINK-01, CITE-LINK-02, CITE-LINK-03, CITE-UNRES-01, CITE-FOLDER-01, CITE-FOLDER-02, CITE-FENCE-01, CITE-SELF-01, CITE-BACK-01, CITE-BACK-02, CITE-BACK-03, CITE-BACK-04, CITE-JSON-01, CITE-JSON-02, CITE-FIELD-01, CITE-FIELD-02, CITE-FIELD-03, CITE-FIELD-04, CITE-FIELD-05, CITE-FIELD-06, CITE-FIELD-07, CITE-FIELD-08, CITE-FIELD-09

mod index;
mod lines;
mod paths;
mod tokens;

pub use index::Index;

/// One citation a doc makes: its target, the target's one-liner, and whether the target exists (design §3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cite {
    /// The label of the cited target. For an unresolved explicit ref, the ref as it was written.
    pub reference: String,
    /// The one-liner of the target; empty when it has none, and when it does not resolve.
    pub title: String,
    /// True when the target exists.
    pub resolved: bool,
}

/// One doc that cites a target, as `cited by` lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citer {
    /// The label of the citing doc.
    pub label: String,
    /// The one-liner of the citing doc; empty when it has none.
    pub title: String,
    /// The prefix that the doc cites the target through, such as `NAME-*`. None when it cites the target directly.
    pub via: Option<String>,
}
