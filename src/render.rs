//! The sections and misses that `show` prints, with the lists they carry and the limits they keep (design §3.3 and §3.6). The text and JSON renderings are in render/.
//! Cases: SHOW-FILE-01, SHOW-DIR-01, SHOW-LOG-01, SHOW-LIMIT-01, SHOW-ERR-01, SHOW-ERR-02, SHOW-JSON-01

pub mod json;
pub mod text;

use crate::cite::Cite;
use crate::collections::frontmatter::Value;
use crate::corpus::Child;
use crate::error::Code;

/// One section: a file, a folder or a doc, with the parts that `show` prints for it.
pub struct Section {
    /// The label printed on the `===` line.
    pub label: String,
    /// The one-liner, when the target has one.
    pub title: Option<String>,
    /// The collection of a doc; None for a folder, or for a file with no doc.
    pub collection: Option<String>,
    /// The id of the doc or file.
    pub id: Option<String>,
    /// The root-relative path of an intent target; None for a log.
    pub path: Option<String>,
    /// The body: a header, a README, a Markdown file or a log body. None prints `missing` instead.
    pub text: Option<String>,
    /// What the body prints when there is no text: `(no header)` or `(no README)`.
    pub missing: &'static str,
    /// The frontmatter fields of a doc. None for a file or a folder, which print no fields block.
    pub fields: Option<Vec<(String, Value)>>,
    /// The children of a folder, with the limit applied. None for anything else.
    pub children: Option<Listing<Child>>,
    /// The citations the target makes.
    pub cites: Vec<Cite>,
    /// The refs that cite the target, with the limit applied.
    pub cited_by: Listing<Ref>,
    /// The READMEs above the target, nearest first. None when no chain is printed: under `--up 0`, or for a doc.
    pub chain: Option<Vec<Link>>,
}

/// One README in a chain, with its text in full.
pub struct Link {
    /// The root-relative path of the README.
    pub path: String,
    /// The whole README.
    pub text: String,
}

/// A ref in a cited-by list, with its one-liner when it resolves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    /// The label of the citing doc.
    pub label: String,
    /// The one-liner of the citing doc, when there is one.
    pub title: Option<String>,
}

/// The first items of a list, with the total so that the limits line can say what was left out (design §3.3).
pub struct Listing<T> {
    /// How many items the list has in all.
    pub total: usize,
    /// The first `limit` items.
    pub items: Vec<T>,
}

impl<T: Clone> Listing<T> {
    /// The listing of every item in `all`, keeping only the first `limit`.
    pub fn of(all: &[T], limit: usize) -> Listing<T> {
        Listing {
            total: all.len(),
            items: all.iter().take(limit).cloned().collect(),
        }
    }
}

/// A ref that `show` could not print: not found, or ambiguous with its candidates (design §3.1 and §3.6).
pub struct Miss {
    /// The ref as typed.
    pub arg: String,
    /// `not_found` or `ambiguous`.
    pub code: Code,
    /// The message after `memex: `, such as `not found: nope.rs`.
    pub message: String,
    /// The labels that an ambiguous ref could mean; empty otherwise.
    pub candidates: Vec<String>,
}

impl Miss {
    /// A ref that names nothing.
    pub fn not_found(arg: &str) -> Miss {
        Miss {
            arg: arg.to_string(),
            code: Code::NotFound,
            message: format!("not found: {arg}"),
            candidates: Vec::new(),
        }
    }

    /// A ref that names several targets, which are listed as candidates.
    pub fn ambiguous(arg: &str, candidates: Vec<String>) -> Miss {
        Miss {
            arg: arg.to_string(),
            code: Code::Ambiguous,
            message: format!("ambiguous: {arg}"),
            candidates,
        }
    }
}
