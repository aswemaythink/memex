//! Reads a CASES.md into case docs (design §2.2): each column-0 item that opens with a case id.
//! Cases: CASE-ITEM-01, CASE-ITEM-02, CASE-ITEM-03, CASE-ITEM-04, CASE-CLAIM-01, CASE-CLAIM-02, CASE-DETAIL-01, CASE-DETAIL-02, CASE-DETAIL-03, CASE-GROUP-01, CASE-GROUP-02, CASE-FENCE-01, CASE-FENCE-02, CASE-GRAMMAR-01, CASE-GRAMMAR-02, CASE-GRAMMAR-03, CASE-CRLF-01, CASE-CRLF-02, CASE-PLAN-01, CASE-EVID-01, CASE-EVID-02, CASE-EVID-03, CASE-EVID-04, CASE-EVID-05, CASE-EVID-06, CASE-EVID-07, CASE-EVID-08

use crate::collections::frontmatter::Value;
use crate::corpus::{parent_of, Doc};

/// The collection of case docs. Their labels are `cases:ID`, and `show` takes them that way (design §3.1).
pub const COLLECTION: &str = "cases";

/// The case docs of the CASES.md at `rel`, whose text is `text`, in the order they are written (design §2.2). A leading
/// byte order mark is dropped (design §4.4), and `\r\n` line ends are read like `\n`.
pub fn parse(rel: &str, text: &str) -> Vec<Doc> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut reader = Reader::new(rel);
    for line in text.lines() {
        reader.line(line);
    }
    reader.finish()
}

/// The group of a case doc: the heading above its item, which the doc keeps as its `group` field (design §2.2).
pub fn group_of(doc: &Doc) -> Option<String> {
    doc.fields.iter().find_map(|(key, value)| match value {
        Value::Str(text) if key == "group" => Some(text.clone()),
        _ => None,
    })
}

/// The evidence of a case (design §2.2): each backticked path in its claim or detail that names a file of the walk. A path
/// resolves from the folder of its CASES.md first and then from the root, and each file is listed once, in the order it is
/// first written. `is_file` says whether a root-relative path names a file of the walk.
pub fn evidence(doc: &Doc, is_file: impl Fn(&str) -> bool) -> Vec<String> {
    let folder = doc.path.as_deref().and_then(parent_of).unwrap_or("");
    let mut found: Vec<String> = Vec::new();
    for span in code_spans(&doc.text) {
        let Some(token) = path_token(span) else {
            continue;
        };
        let Some(rel) = resolve(folder, token, &is_file) else {
            continue;
        };
        if !found.contains(&rel) {
            found.push(rel);
        }
    }
    found
}

/// One pass over a CASES.md: the item being read, the heading above it, and whether a code fence is open.
struct Reader<'a> {
    /// The root-relative path of the CASES.md, which each case doc records as its path.
    rel: &'a str,
    /// The text of the last heading at column 0, which groups the items after it.
    group: Option<String>,
    /// The item whose detail lines may still follow.
    open: Option<Item>,
    /// True between a column-0 code fence and the next one. Fenced lines are code, not intent (design §2.3).
    fenced: bool,
    /// The case docs read so far, in order.
    docs: Vec<Doc>,
}

impl<'a> Reader<'a> {
    /// A reader for the CASES.md at `rel`, before its first line.
    fn new(rel: &'a str) -> Self {
        Reader {
            rel,
            group: None,
            open: None,
            fenced: false,
            docs: Vec::new(),
        }
    }

    /// Reads one line. A column-0 fence opens or closes a code block and ends the open item; any other line is read unless a
    /// fence is open.
    fn line(&mut self, line: &str) {
        if is_fence(line) {
            self.close();
            self.fenced = !self.fenced;
        } else if !self.fenced {
            self.plain(line);
        }
    }

    /// Reads a line outside any fence. A blank or indented line may belong to the open item; any other line ends it, and a
    /// column-0 line may open a heading or an item.
    fn plain(&mut self, line: &str) {
        if line.trim().is_empty() {
            if let Some(item) = self.open.as_mut() {
                item.blank();
            }
        } else if line.starts_with([' ', '\t']) {
            if let Some(item) = self.open.as_mut() {
                item.push(line);
            }
        } else {
            self.close();
            self.column_zero(line);
        }
    }

    /// Reads a column-0 line that is not a fence: a heading sets the group, and an item opens a case.
    fn column_zero(&mut self, line: &str) {
        match heading_text(line) {
            Some(heading) => self.group = (!heading.is_empty()).then(|| heading.to_string()),
            None => self.open = Item::start(line, self.group.as_deref()),
        }
    }

    /// Ends the open item, if there is one, and keeps its doc.
    fn close(&mut self) {
        if let Some(item) = self.open.take() {
            self.docs.push(item.into_doc(self.rel));
        }
    }

    /// The docs of the whole file, once every line has been read.
    fn finish(mut self) -> Vec<Doc> {
        self.close();
        self.docs
    }
}

/// A case whose item is being read: its id, its claim, its detail lines and the group above it.
struct Item {
    /// The case id, which the item opens with.
    id: String,
    /// The text after the id on the item's line, trimmed. Empty when the line has nothing after the id.
    claim: String,
    /// The indented lines under the claim, with their indent removed. Blank lines between them are kept.
    detail: Vec<String>,
    /// The heading above the item, when there is one.
    group: Option<String>,
}

impl Item {
    /// The item that a column-0 line opens: a `-` or `*` item whose first element is a backticked case id (design §2.2).
    fn start(line: &str, group: Option<&str>) -> Option<Item> {
        let rest = line
            .strip_prefix("- ")
            .or_else(|| line.strip_prefix("* "))?
            .trim_start();
        let body = rest.strip_prefix('`')?;
        let end = body.find('`')?;
        let id = &body[..end];
        is_case_id(id).then(|| Item {
            id: id.to_string(),
            claim: body[end + 1..].trim().to_string(),
            detail: Vec::new(),
            group: group.map(str::to_string),
        })
    }

    /// Adds an indented line to the detail, without its indent.
    fn push(&mut self, line: &str) {
        self.detail.push(line.trim().to_string());
    }

    /// Adds a blank line to the detail. A blank line before the first detail line, or after another blank one, is not kept.
    fn blank(&mut self) {
        if self.detail.last().is_some_and(|last| !last.is_empty()) {
            self.detail.push(String::new());
        }
    }

    /// The case doc of this item, labelled `cases:ID`: its claim is the title, and its claim and detail are the text.
    fn into_doc(mut self, rel: &str) -> Doc {
        if self.detail.last().is_some_and(String::is_empty) {
            self.detail.pop();
        }
        let text = {
            let mut lines: Vec<&str> = Vec::new();
            if !self.claim.is_empty() {
                lines.push(&self.claim);
            }
            lines.extend(self.detail.iter().map(String::as_str));
            lines.join("\n")
        };
        Doc {
            label: format!("{COLLECTION}:{}", self.id),
            collection: COLLECTION.to_string(),
            id: self.id,
            path: Some(rel.to_string()),
            title: self.claim,
            text,
            fields: self
                .group
                .map(|group| ("group".to_string(), Value::Str(group)))
                .into_iter()
                .collect(),
        }
    }
}

/// True for a line that opens or closes a code fence: three backticks or tildes at column 0.
fn is_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

/// The text of an ATX heading, which is one or more `#` followed by a space or the end of the line, with the hashes removed.
/// None for any other line.
fn heading_text(line: &str) -> Option<&str> {
    let hashes = line.bytes().take_while(|byte| *byte == b'#').count();
    let rest = line.get(hashes..)?;
    (hashes > 0 && (rest.is_empty() || rest.starts_with(' '))).then(|| rest.trim())
}

/// True for a case id (design §2.2): an upper-case letter first, then capitals and digits in `-`-separated parts, and at
/// least two parts in all. A lower-case letter anywhere makes it not an id.
fn is_case_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    let [head, tail @ ..] = parts.as_slice() else {
        return false;
    };
    !tail.is_empty()
        && head.starts_with(|c: char| c.is_ascii_uppercase())
        && parts.iter().all(|part| is_capitals_and_digits(part))
}

/// True for a non-empty run of upper-case ASCII letters and digits.
fn is_capitals_and_digits(part: &str) -> bool {
    !part.is_empty()
        && part
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

/// The text between each pair of backticks on each line. Each line is split on its own, so a stray backtick cannot shift
/// the pairs of the lines after it.
fn code_spans(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .flat_map(|line| line.split('`').skip(1).step_by(2))
}

/// The span as a path token (design §2.3): only path characters, with trailing `.,:;)` dropped, and either a `/` or an
/// extension. None for any other span.
fn path_token(span: &str) -> Option<&str> {
    let token = span.trim_end_matches(['.', ',', ':', ';', ')']);
    let plain = !token.is_empty()
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._/@+-".contains(&byte));
    let named = token.contains('/') || has_extension(token);
    (plain && named).then_some(token)
}

/// True when the last dot of a token is followed by one or more letters or digits.
fn has_extension(token: &str) -> bool {
    token.rsplit_once('.').is_some_and(|(_, ext)| {
        !ext.is_empty() && ext.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

/// The root-relative file that a path token names (design §2.2): from `folder` and then from the root. A token that begins
/// with `/` names a file from the root only (design §2.3).
fn resolve(folder: &str, token: &str, is_file: &impl Fn(&str) -> bool) -> Option<String> {
    let candidates: Vec<Option<String>> = match token.strip_prefix('/') {
        Some(from_root) => vec![join("", from_root)],
        None => vec![join(folder, token), join("", token)],
    };
    candidates.into_iter().flatten().find(|rel| is_file(rel))
}

/// The root-relative path that `token` names from the folder `base`: `.` parts are dropped and each `..` climbs one folder.
/// None when a `..` would climb above the root.
fn join(base: &str, token: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for part in token.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_case_id_has_capitals_and_digits_in_at_least_two_parts() {
        assert!(is_case_id("NAME-001"));
        assert!(is_case_id("A-1"));
        assert!(is_case_id("LIB-MEASURE-001"));
        assert!(!is_case_id("name-001"));
        assert!(!is_case_id("NODASH"));
        assert!(!is_case_id("TRAILING-"));
        assert!(!is_case_id("DOUBLE--1"));
    }

    #[test]
    fn a_path_token_needs_a_slash_or_an_extension() {
        assert_eq!(path_token("src/cli.rs."), Some("src/cli.rs"));
        assert_eq!(path_token("README"), None);
        assert_eq!(path_token("UTF-8"), None);
        assert_eq!(path_token("memex show x"), None);
    }

    #[test]
    fn a_join_stays_inside_the_root() {
        assert_eq!(
            join("tests", "../src/cli.rs").as_deref(),
            Some("src/cli.rs")
        );
        assert_eq!(join("", "./a/./b").as_deref(), Some("a/b"));
        assert_eq!(join("", "../x"), None);
    }
}
