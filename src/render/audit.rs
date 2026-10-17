//! Prints `audit`'s report as text and as its `--json` document (design §3.5d and §3.6).
//! The report is the scope line, then each kind with findings under its heading, its items and its fix line, and the misses of
//! the paths that named nothing. Paths in labels and JSON are root-relative, with `/` separators.
//! Cases: AUD-SCOPE-01, AUD-SCOPE-02, AUD-SCOPE-04, AUD-HEADERS-01, AUD-READMES-01, AUD-ONELINERS-01, AUD-CITES-01, AUD-IDS-01, AUD-LIMIT-01, AUD-LIMIT-02, AUD-LIMIT-03, AUD-EMPTY-01, AUD-LENGTH-01, AUD-THIN-01, AUD-THIN-06, AUD-THIN-07, AUD-JSON-01, AUD-JSON-02, AUD-JSON-03, AUD-EXIT-01

use serde_json::{json, Value as Json};

use super::{Listing, Miss};

/// The separator of the scope line and of the detail of `ids` (design §3.5d).
const DOT: &str = " · ";

/// One finding: its label and the detail after it, when the text has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// The label of the finding: a root-relative path, or `collection:id`.
    pub reference: String,
    /// The text after the label, such as a count or `→ <reference>`; None when the text has none.
    pub detail: Option<String>,
}

/// One kind's findings: the kind, what its list holds, its fix line, and the findings with the limit applied.
pub struct Group {
    /// The kind's name, such as `headers`.
    pub kind: &'static str,
    /// What the list holds, for its heading, such as `files with no header`.
    pub what: &'static str,
    /// The line that follows the list with the fix, or None for a kind with no fix line.
    pub fix: Option<&'static str>,
    /// True for a finding, which the count of findings includes. False for a pointer such as `thin`: it is listed, but not counted
    /// (design §3.5d).
    pub finding: bool,
    /// Every finding of the kind, in label order, with the first `--limit` of them kept.
    pub listing: Listing<Item>,
}

/// The whole report: its scope, its counts, its groups and the misses.
pub struct Summary {
    /// The labels of the scope: `.` for the root, else each argument's root-relative path or its glob as given.
    pub scope: Vec<String>,
    /// The walked files in the scope.
    pub files: usize,
    /// The walked folders in the scope, the root included when it is in scope.
    pub folders: usize,
    /// The kinds that have findings, and the pointers such as `thin` that have entries, in §3.5d order.
    pub groups: Vec<Group>,
    /// The arguments that named nothing, or several things.
    pub errors: Vec<Miss>,
}

impl Summary {
    /// The findings of every kind, which the scope line counts, whether or not the lists show them all. A pointer such as `thin` is
    /// not a finding, so it is left out (design §3.5d).
    pub fn findings(&self) -> usize {
        self.groups
            .iter()
            .filter(|group| group.finding)
            .map(|group| group.listing.total)
            .sum()
    }
}

/// The report as text (design §3.5d): the scope line, then `nothing to fix` when no kind has a finding, then each group, which
/// lists a pointer such as `thin` as well.
pub fn text(summary: &Summary) -> String {
    let mut out = format!("{:<12}{}\n", "audit", scope_line(summary));
    if summary.findings() == 0 {
        out.push_str("nothing to fix\n");
    }
    for group in &summary.groups {
        group_text(&mut out, group);
    }
    out
}

/// The scope line: the labels joined by ` · `, then the counts of files, folders and findings.
fn scope_line(summary: &Summary) -> String {
    let mut parts = Vec::new();
    if !summary.scope.is_empty() {
        parts.push(summary.scope.join(DOT));
    }
    parts.push(count(summary.files, "file", "files"));
    parts.push(count(summary.folders, "folder", "folders"));
    parts.push(count(summary.findings(), "finding", "findings"));
    parts.join(DOT)
}

/// `1 file` for one, else `N files`: the singular is used for a count of one.
fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}

/// A group's heading, its items and its fix line. The heading names the items shown and the total, and the flag that lists all of
/// them is named only when the list was cut: `(showing N of M; --limit M for all; <what>)` (design §3.5d, R33).
fn group_text(out: &mut String, group: &Group) {
    let listing = &group.listing;
    let shown = listing.items.len();
    let total = listing.total;
    let cut = if shown < total {
        format!("; --limit {total} for all")
    } else {
        String::new()
    };
    out.push_str(&format!(
        "{:<12}(showing {shown} of {total}{cut}; {})\n",
        group.kind, group.what
    ));
    for item in &listing.items {
        out.push_str(&item_line(item));
    }
    if let Some(fix) = group.fix {
        out.push_str(fix);
        out.push('\n');
    }
}

/// One item's line: two spaces and its label, then two more spaces and the detail when there is one.
fn item_line(item: &Item) -> String {
    match &item.detail {
        Some(detail) => format!("  {}  {detail}\n", item.reference),
        None => format!("  {}\n", item.reference),
    }
}

/// The `--json` document of design §3.6, one line with its newline: the scope, the counts, each kind with findings and the errors.
pub fn json(summary: &Summary) -> String {
    let kinds: Vec<Json> = summary.groups.iter().map(kind_json).collect();
    let errors: Vec<Json> = summary.errors.iter().map(miss_json).collect();
    let document = json!({
        "scope": summary.scope,
        "files": summary.files,
        "folders": summary.folders,
        "findings": summary.findings(),
        "kinds": kinds,
        "errors": errors,
    });
    format!("{document}\n")
}

/// One kind: its name, its total and the findings that its list shows, each with its ref and detail.
fn kind_json(group: &Group) -> Json {
    let items: Vec<Json> = group
        .listing
        .items
        .iter()
        .map(|item| json!({"ref": item.reference, "detail": item.detail}))
        .collect();
    json!({"kind": group.kind, "total": group.listing.total, "items": items})
}

/// One error: its argument, its code and its message, with the candidates of an ambiguous argument when it has any.
fn miss_json(miss: &Miss) -> Json {
    let mut value = json!({"arg": miss.arg, "code": miss.code.as_str(), "message": miss.message});
    if !miss.candidates.is_empty() {
        value["candidates"] = json!(miss.candidates);
    }
    value
}
