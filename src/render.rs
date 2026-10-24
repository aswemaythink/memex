//! The sections and misses that `show` prints (design §3.3 and §3.6).
//! Each carries its lists and limits. The text and JSON renderings are in render/.
//! Cases: SHOW-FILE-01, SHOW-DIR-01, SHOW-LOG-01, SHOW-LIMIT-01, SHOW-ERR-01, SHOW-ERR-02, SHOW-JSON-01, CASE-SECT-01, CASE-JSON-01, SHOW-OTHER-01, CLI-HINT-01, CLI-HINT-02, SHOW-COMMENTS-01, SHOW-COMMENTS-02, SHOW-COMMENTS-03, SHOW-COMMENTS-05, SHOW-COMMENTS-09

pub mod audit;
pub mod clip;
pub mod info;
pub mod json;
pub mod search;
pub mod text;
pub mod tree;

use std::io;

use crate::cite::Cite;
use crate::collections::frontmatter::Value;
use crate::corpus::Child;
use crate::error::{Code, Error};
use crate::intent::header::CommentBlock;

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
    /// The root-relative path of an intent target, or of the file that defines a collection doc; None for a log.
    pub path: Option<String>,
    /// The body: a header, a README, a Markdown file or a log body. None prints `missing` instead.
    pub text: Option<String>,
    /// What the body prints when there is no text: `(no header)` or `(no README)`.
    pub missing: &'static str,
    /// The fields of a doc. None for a file or a folder, which print no fields block.
    pub fields: Option<Fields>,
    /// The children of a folder, with the limit applied. None for anything else.
    pub children: Option<Listing<Child>>,
    /// The comment blocks of a file, when its section lists them (design §3.3 Comments). None for anything else.
    pub comments: Option<Comments>,
    /// The citations of the target, or, for an entry of another root, the root that holds them (design §3.1).
    pub citations: Citations,
    /// The READMEs above the target, nearest first. None when no chain is printed: under `--up 0`, or for a doc.
    pub chain: Option<Vec<Link>>,
}

/// The fields of a doc (design §3.3 and §3.6): a log's or collection doc's frontmatter, or an entry's own parts.
pub enum Fields {
    /// Frontmatter or manifest fields, printed as `key: value` lines.
    Pairs(Vec<(String, Value)>),
    /// An entry's group and evidence, printed as blocks of their own in the entry's section.
    Case(CaseParts),
}

/// The parts of an entry that `show` prints beside its claim and detail (design §3.3 Entry).
pub struct CaseParts {
    /// The heading above the entry in its file; None when no heading comes above it.
    pub group: Option<String>,
    /// The root-relative paths of the files that evidence the case, in the order its text gives them.
    pub evidence: Vec<String>,
}

/// The comment blocks of a file, as `show` lists them (design §3.3 Comments): the first blocks, with their total, and whether the
/// file was cut at the read limit.
pub struct Comments {
    /// The blocks that the list shows, with the total of the blocks the read found.
    pub list: Listing<CommentBlock>,
    /// True when the file is longer than the bytes read, so the list ends with the cut row.
    pub cut: bool,
    /// True when `--comments` asked for the list. False for the list of a file with no header, which shows without the flag.
    pub asked: bool,
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
    /// The prefix that the doc cites the target by, such as `NAME-*`; None when it cites the target directly.
    pub via: Option<String>,
}

/// The citations of a section (design §3.1 and §3.3). An entry of another root has none computed: its citations name labels of
/// that root, which do not resolve from the cwd, so its root is printed in their place.
pub enum Citations {
    /// The citations the target makes, and the refs that cite it.
    Listed {
        /// The citations the target makes, in the order its text gives them.
        cites: Vec<Cite>,
        /// The refs that cite the target, with the limit applied.
        cited_by: Listing<Ref>,
    },
    /// The entry is in another root, which holds its citations.
    OtherRoot {
        /// The absolute path of that root, with `/` separators.
        root: String,
    },
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

/// A ref that `show` could not print: not found, ambiguous with its candidates, or a requested file that cannot be read
/// (design §3.1, §3.3 and §3.6).
pub struct Miss {
    /// The ref as typed.
    pub arg: String,
    /// `not_found`, `ambiguous` or `io`.
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
            message: format!("not found: {arg} (find refs with memex search <words>)"),
            candidates: Vec::new(),
        }
    }

    /// A path that leads outside the current root, which `audit` and `tree` never read (design §3.5d scope).
    pub fn outside(arg: &str) -> Miss {
        Miss {
            arg: arg.to_string(),
            code: Code::NotFound,
            message: format!("not found: {arg} (outside the root; run memex from there)"),
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

    /// A requested file that cannot be read: an `io` error for its ref, which names the ref as typed (design §3.3).
    pub fn unreadable(arg: &str, failure: &io::Error) -> Miss {
        Miss {
            arg: arg.to_string(),
            code: Code::Io,
            message: format!("cannot read {arg} ({:?})", failure.kind()),
            candidates: Vec::new(),
        }
    }
}

impl From<Miss> for Error {
    /// The fatal error for a miss: its code and message, with each candidate indented as a note.
    fn from(miss: Miss) -> Error {
        Error {
            code: miss.code,
            message: miss.message,
            notes: miss
                .candidates
                .iter()
                .map(|candidate| format!("  {candidate}"))
                .collect(),
        }
    }
}
