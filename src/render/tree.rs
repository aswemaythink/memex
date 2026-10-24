//! Prints `tree`'s listing as text and as its `--json` document (design §3.5e and §3.6).
//! The text is a heading with the scope's counts, one indented line per entry, and the gap legend `show` prints.
//! Cases: TREE-DEFAULT-01, TREE-DEPTH-01, TREE-DEPTH-02, TREE-SCOPE-01, TREE-SCOPE-02, TREE-NOHDR-01, TREE-NOHDR-02, TREE-COUNT-01, TREE-COUNT-02, TREE-GAP-01, TREE-LIMIT-01, TREE-JSON-01, TREE-JSON-02, TREE-SKIP-01

use serde_json::{json, Value as Json};

use super::text::{clip, GAP_LEGEND, NO_HEADER, NO_README};

/// The placeholder a line prints in place of its one-liner (design §3.5e Lines), which is also its JSON `gap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    /// A folder with no README whose subtree holds files a header is expected on.
    Readme,
    /// A file a header is expected on that has none.
    Header,
}

impl Gap {
    /// The placeholder text of the gap, which `show` prints too (design §3.3).
    fn placeholder(self) -> &'static str {
        match self {
            Gap::Readme => NO_README,
            Gap::Header => NO_HEADER,
        }
    }

    /// The word JSON gives the gap (design §3.6).
    fn word(self) -> &'static str {
        match self {
            Gap::Readme => "readme",
            Gap::Header => "header",
        }
    }
}

/// The counts of a folder's subtree: the files a header is expected on, and how many of them have none (design §3.5e Counts).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// The files a header is expected on.
    pub files: usize,
    /// Those with no header.
    pub no_header: usize,
}

/// One printed entry: a folder or a file below the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// The root-relative path, with no trailing `/`.
    pub path: String,
    /// True for a folder.
    pub dir: bool,
    /// How many levels below the scope: 1 for the scope's children.
    pub depth: usize,
    /// The one-liner in full; empty when there is none.
    pub title: String,
    /// The placeholder the line prints, if any.
    pub gap: Option<Gap>,
    /// A folder's subtree counts; None for a file.
    pub counts: Option<Counts>,
}

/// What `tree` prints: the scope with its counts, and the entries within the depth, cut to the limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    /// The scope's label: `.` for the root.
    pub scope: String,
    /// The depth the listing went to.
    pub depth: usize,
    /// The scope's counts.
    pub counts: Counts,
    /// The number of entries within the depth, the printed ones and those cut.
    pub total: usize,
    /// The entries printed, in depth-first order.
    pub items: Vec<Item>,
}

/// The text of the listing (design §3.5e): the first line, one line per entry, then each gap line once.
pub fn text(tree: &Tree) -> String {
    let mut out = heading(tree);
    for item in &tree.items {
        out.push_str(&line(item));
    }
    for (placeholder, fix) in GAP_LEGEND {
        if tree
            .items
            .iter()
            .any(|item| item.gap.is_some_and(|gap| gap.placeholder() == placeholder))
        {
            out.push_str(fix);
            out.push('\n');
        }
    }
    out
}

/// The first line: the scope, its counts, and how many entries are shown of how many; a cut list names the flag that shows all.
fn heading(tree: &Tree) -> String {
    let (shown, total) = (tree.items.len(), tree.total);
    let rest = if shown < total {
        format!("; --limit {total} for all")
    } else {
        String::new()
    };
    format!(
        "tree  {} · {}  (showing {shown} of {total}{rest})\n",
        tree.scope,
        counted(tree.counts)
    )
}

/// The counts as the first line and a folder line write them: the files, then ` · <H> no header` unless H is 0.
fn counted(counts: Counts) -> String {
    let noun = if counts.files == 1 { "file" } else { "files" };
    let mut text = format!("{} {noun}", counts.files);
    if counts.no_header > 0 {
        text.push_str(&format!(" · {} no header", counts.no_header));
    }
    text
}

/// The line of one entry: two spaces per level, its label (a folder's ends with `/`), then its one-liner or placeholder and, for a
/// folder with files, its counts in parentheses, each after two spaces. An entry with none of these is its label alone.
fn line(item: &Item) -> String {
    let label = if item.dir {
        format!("{}/", item.path)
    } else {
        item.path.clone()
    };
    let mut parts: Vec<String> = Vec::new();
    match item.gap {
        Some(gap) => parts.push(gap.placeholder().to_string()),
        None if !item.title.is_empty() => parts.push(clip(&item.title)),
        None => {}
    }
    if let Some(counts) = item.counts.filter(|counts| counts.files > 0) {
        parts.push(format!("({})", counted(counts)));
    }
    let indent = "  ".repeat(item.depth);
    if parts.is_empty() {
        format!("{indent}{label}\n")
    } else {
        format!("{indent}{label}  {}\n", parts.join("  "))
    }
}

/// The `--json` document (design §3.6): the scope, the depth, the scope's counts, the total, and the printed entries, with one line
/// break at the end. A folder's entry also has its counts.
pub fn json(tree: &Tree) -> String {
    let out = json!({
        "scope": tree.scope,
        "depth": tree.depth,
        "files": tree.counts.files,
        "no_header": tree.counts.no_header,
        "total": tree.total,
        "items": tree.items.iter().map(item_json).collect::<Vec<Json>>(),
    });
    format!("{out}\n")
}

/// One entry as JSON: its path, kind, depth, full one-liner and gap, and a folder's counts.
fn item_json(item: &Item) -> Json {
    let mut out = json!({
        "path": item.path,
        "kind": if item.dir { "dir" } else { "file" },
        "depth": item.depth,
        "title": item.title,
        "gap": item.gap.map(Gap::word),
    });
    if let Some(counts) = item.counts {
        out["files"] = json!(counts.files);
        out["no_header"] = json!(counts.no_header);
    }
    out
}
