//! Carries out `memex tree` (design §3.5e): the shape of a folder in one call.
//! It lists folders and files with their one-liners, and how much of each subtree has a header.
//! It resolves its one path as `audit` resolves a scope path, counts the files a header is expected on once, then lists the walked
//! folders and files depth first, in the order `show` lists a folder's children, and `render/tree.rs` prints them.
//! Cases: TREE-DEFAULT-01, TREE-DEPTH-01, TREE-DEPTH-02, TREE-SCOPE-01, TREE-SCOPE-02, TREE-SCOPE-03, TREE-SCOPE-04, TREE-NOHDR-01, TREE-NOHDR-02, TREE-COUNT-01, TREE-COUNT-02, TREE-GAP-01, TREE-LIMIT-01, TREE-JSON-01, TREE-JSON-02, TREE-SKIP-01, TREE-USAGE-04, TREE-OUTSIDE-01, TREE-OUTSIDE-02, TREE-OUTSIDE-03, TREE-READ-01

use std::collections::HashMap;
use std::path::Path;

use crate::cli::Verb;
use crate::corpus::{label_of, parent_of, Child, Corpus, Kind};
use crate::error::Error;
use crate::intent::coverage;
use crate::render;
use crate::render::tree::{Counts, Gap, Item, Tree};
use crate::scan::fail;
use crate::verbs::audit::scope::scope_path;
use crate::verbs::here;

/// The `--depth` when the flag is not given (design §3.5e Depth).
const DEFAULT_DEPTH: usize = 2;

/// The `--limit` when the flag is not given (design §3.5e Limit).
const DEFAULT_LIMIT: usize = 200;

/// The arguments of `memex tree` once the grammar has passed them.
pub struct Request {
    /// The folder or file to show; None shows the root.
    pub path: Option<String>,
    /// The value of `--depth`, if the flag was given.
    pub depth: Option<usize>,
    /// The value of `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// Lists the root, or the one path given, and returns the text or the JSON document. A path that names nothing is a fatal `not
/// found`, and one that names two entries a fatal `ambiguous` (design §3.5e Scope). An empty path is a usage error, as it is for
/// `audit`.
pub fn run(request: &Request) -> Result<String, Error> {
    if request.path.as_deref().is_some_and(str::is_empty) {
        return Err(fail(Verb::Tree, "empty path".to_string()));
    }
    let here::Here { cwd, store, corpus } = here::load_with_lines()?;
    tree_in(&corpus, &cwd, store.as_deref(), request)
}

/// The listing of `request` as typed from `cwd`, in the corpus of its root.
fn tree_in(
    corpus: &Corpus,
    cwd: &Path,
    store: Option<&Path>,
    request: &Request,
) -> Result<String, Error> {
    let (rel, kind) = scope_path(corpus, cwd, store, request.path.as_deref().unwrap_or("."))
        .map_err(Error::from)?;
    let coverage = Coverage::of(corpus);
    let depth = request.depth.unwrap_or(DEFAULT_DEPTH);
    let mut list = Listing::new(request.limit.unwrap_or(DEFAULT_LIMIT));
    let counts = match kind {
        Kind::Dir => {
            let walk = Walk {
                corpus,
                coverage: &coverage,
                depth,
            };
            walk.descend(&rel, 1, &mut list);
            coverage.folder(&rel)
        }
        Kind::File => {
            list.add(|| file_item(corpus, &coverage, &rel));
            coverage.file_counts(&rel)
        }
    };
    let tree = Tree {
        scope: label_of(&rel),
        depth,
        counts,
        total: list.total,
        items: list.items,
    };
    Ok(if request.json {
        render::tree::json(&tree)
    } else {
        render::tree::text(&tree)
    })
}

/// The entries printed so far, and how many there are in all: past the limit an entry is only counted, never built.
struct Listing {
    /// The most entries to print.
    limit: usize,
    /// Every entry within the depth, the printed ones and those cut.
    total: usize,
    /// The entries printed, in depth-first order.
    items: Vec<Item>,
}

impl Listing {
    /// An empty listing that prints at most `limit` entries.
    fn new(limit: usize) -> Listing {
        Listing {
            limit,
            total: 0,
            items: Vec::new(),
        }
    }

    /// Counts one entry, and builds and keeps it only while the limit allows.
    fn add(&mut self, make: impl FnOnce() -> Item) {
        self.total += 1;
        if self.items.len() < self.limit {
            self.items.push(make());
        }
    }
}

/// What the walk of a folder reads: the corpus, the counts of every folder, and the depth to list to.
struct Walk<'a> {
    corpus: &'a Corpus,
    coverage: &'a Coverage<'a>,
    depth: usize,
}

impl Walk<'_> {
    /// Lists the entries of the folder `rel` at `level` levels below the scope, each followed by its own entries while the depth
    /// allows (design §3.5e What is listed). A folder's own README.md is not listed, since its one-liner is on the folder's line.
    fn descend(&self, rel: &str, level: usize, list: &mut Listing) {
        for child in self.corpus.children(rel) {
            if child.kind == Kind::File && child.name == "README.md" {
                continue;
            }
            list.add(|| item(&child, level, self.coverage));
            if child.kind == Kind::Dir && level < self.depth {
                self.descend(&child.rel, level + 1, list);
            }
        }
    }
}

/// The entry of a child at `level`: a folder with its one-liner or `(no README)`, and its subtree's counts; a file with its header's
/// one-liner, `(no header)`, a Markdown file's title, or nothing (design §3.5e Lines).
fn item(child: &Child, level: usize, coverage: &Coverage<'_>) -> Item {
    let title = child.title.clone().unwrap_or_default();
    match child.kind {
        Kind::Dir => {
            let counts = coverage.folder(&child.rel);
            let no_readme = child.title.is_none() && counts.files > 0;
            Item {
                path: child.rel.clone(),
                dir: true,
                depth: level,
                title,
                gap: no_readme.then_some(Gap::Readme),
                counts: Some(counts),
            }
        }
        Kind::File => Item {
            path: child.rel.clone(),
            dir: false,
            depth: level,
            gap: (coverage.file(&child.rel) == Some(false)).then_some(Gap::Header),
            title,
            counts: None,
        },
    }
}

/// The entry of the file a path argument names, at the first level. It is a child of its folder like any other, so it reads as one.
fn file_item(corpus: &Corpus, coverage: &Coverage<'_>, rel: &str) -> Item {
    let parent = parent_of(rel).unwrap_or_default();
    let found = corpus.children(parent).into_iter().find(|c| c.rel == rel);
    match found {
        Some(child) => item(&child, 1, coverage),
        None => Item {
            path: rel.to_string(),
            dir: false,
            depth: 1,
            title: String::new(),
            gap: None,
            counts: None,
        },
    }
}

/// The files a header is expected on, counted for every folder above them at once (design §3.5e Counts).
struct Coverage<'a> {
    /// Each folder's counts, by root-relative path; the root is the empty path. A folder with no such file has none.
    folders: HashMap<&'a str, Counts>,
    /// Each such file, with whether it has a header.
    files: HashMap<&'a str, bool>,
}

impl<'a> Coverage<'a> {
    /// Counts the files of the corpus that a header is expected on (`intent/coverage.rs`), up through their folders.
    fn of(corpus: &'a Corpus) -> Coverage<'a> {
        let mut folders: HashMap<&str, Counts> = HashMap::new();
        let mut files = HashMap::new();
        for (rel, has_header) in coverage::expected(corpus) {
            files.insert(rel, has_header);
            let mut dir = parent_of(rel);
            while let Some(folder) = dir {
                let counts = folders.entry(folder).or_default();
                counts.files += 1;
                counts.no_header += usize::from(!has_header);
                dir = parent_of(folder);
            }
        }
        Coverage { folders, files }
    }

    /// The counts of a folder's whole subtree.
    fn folder(&self, rel: &str) -> Counts {
        self.folders.get(rel).copied().unwrap_or_default()
    }

    /// Whether the file has a header, when a header is expected on it; None for any other file.
    fn file(&self, rel: &str) -> Option<bool> {
        self.files.get(rel).copied()
    }

    /// The counts of a file as the scope: one file, with no header if it has none, or none when no header is expected on it.
    fn file_counts(&self, rel: &str) -> Counts {
        match self.file(rel) {
            Some(has_header) => Counts {
                files: 1,
                no_header: usize::from(!has_header),
            },
            None => Counts::default(),
        }
    }
}
