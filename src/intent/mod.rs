//! The readers of the intent layer and the one-liner rule they share (design §2).
//! Paragraphs, the first sentence, and the one-liner of a block; `Origin` tells the graph, entries, whole files, manifest docs and
//! log entries apart, `cites_fields` says which docs' frontmatter fields are cited (design §2.3), and `by_label` groups the docs by
//! label (design §3.1).
//! Cases: SHOW-DIR-01, SHOW-FILE-01, SHOW-LINE-01, CITE-FIELD-01, CITE-FIELD-06, CITE-FIELD-07, CITE-FIELD-09

pub mod entries;
pub mod files;
pub mod front;
pub mod header;
pub mod readme;
pub mod shared;

use std::collections::BTreeMap;

use crate::corpus::{Doc, HEADERS, READMES};

/// Where a doc was read from (design §2.2, §4.2 and §4.3): the graph's own docs, an entry of an entry index, a whole Markdown file,
/// a manifest's doc, or a log entry. Each reader sets it, and the citation scan, the search groups and `show` read it. A log entry
/// is its own origin, not a name, so a collection that shares a stream's name is never a log (design §4.2 Names).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// A README or a header: the graph's doc, labelled by its path.
    Graph,
    /// An entry of an entry index (design §2.2), labelled `collection:ID`, with the index's path.
    Entry,
    /// A Markdown file that is one doc of its collection (design §2.2), labelled `collection:id`, with its path.
    Whole,
    /// A manifest's doc (design §4.2), labelled `collection:id`, with no path. `folder` is the root-relative folder its relative
    /// citations resolve from first (design §2.3): the folder that holds a folder doc's marker, or the folder of a json doc's file.
    /// It is empty when that folder is the root. `frontmatter` is true for a folder doc whose marker is a Markdown file, so its fields
    /// are that file's frontmatter, which design §2.3 cites.
    Other { folder: String, frontmatter: bool },
    /// A log entry (design §4.3), labelled `stream:id`, with no path.
    Log,
}

impl Doc {
    /// True for an entry of an entry index, whose claim is its title and whose path is that index (design §2.2).
    pub fn is_entry(&self) -> bool {
        self.origin == Origin::Entry
    }

    /// True for a doc of the graph, a README or a header, which is labelled by its path (design §3.1).
    pub fn is_graph(&self) -> bool {
        self.origin == Origin::Graph
    }

    /// True when the doc's frontmatter fields are cited like its text (design §2.3): a frontmatter collection doc, and a folder doc
    /// whose marker is a Markdown file. A log's frontmatter is memex's own record of the run, so a log is not one.
    pub fn cites_fields(&self) -> bool {
        match &self.origin {
            Origin::Whole => true,
            Origin::Other { frontmatter, .. } => *frontmatter,
            Origin::Graph | Origin::Entry | Origin::Log => false,
        }
    }

    /// True for a README of the graph (design §3.1). A collection named `readmes` is not one, whatever its doc's name.
    pub fn is_readme(&self) -> bool {
        self.is_graph() && self.collection == READMES
    }

    /// True for a header of the graph (design §3.1). A collection named `headers` is not one.
    pub fn is_header(&self) -> bool {
        self.is_graph() && self.collection == HEADERS
    }
}

/// The intent docs grouped by label (design §3.1). The sort is stable, so each label's docs keep the order they were read in, and
/// the groups go to the map in one pass.
pub fn by_label(mut docs: Vec<Doc>) -> BTreeMap<String, Vec<Doc>> {
    docs.sort_by(|left, right| left.label.cmp(&right.label));
    let mut groups: Vec<(String, Vec<Doc>)> = Vec::new();
    for doc in docs {
        match groups.last_mut() {
            Some((label, group)) if *label == doc.label => group.push(doc),
            _ => groups.push((doc.label.clone(), vec![doc])),
        }
    }
    groups.into_iter().collect()
}

/// The paragraphs of a block: runs of non-blank lines, each joined with newlines. Blank lines separate them.
fn paragraphs(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut run: Vec<&str> = Vec::new();
    for line in text.lines() {
        if !line.trim().is_empty() {
            run.push(line.trim_end());
        } else if !run.is_empty() {
            found.push(run.join("\n"));
            run.clear();
        }
    }
    if !run.is_empty() {
        found.push(run.join("\n"));
    }
    found
}

/// The one-liner of a block: the first sentence of its first paragraph, or empty when it has no paragraph.
pub fn lead(text: &str) -> String {
    paragraphs(text)
        .first()
        .map_or_else(String::new, |paragraph| sentence(paragraph))
}

/// The first sentence of a paragraph, with its line breaks and runs of spaces made into single spaces. A sentence ends at
/// `.`, `!` or `?` followed by whitespace or the end of the paragraph. A paragraph with no such mark is one sentence.
pub fn sentence(paragraph: &str) -> String {
    let flat = paragraph
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    let mut marks = flat.char_indices().peekable();
    while let Some((index, ch)) = marks.next() {
        let ends = matches!(ch, '.' | '!' | '?')
            && marks.peek().is_none_or(|(_, next)| next.is_whitespace());
        if ends {
            return flat[..index + ch.len_utf8()].to_string();
        }
    }
    flat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sentence_ends_at_a_mark_before_whitespace_or_the_end() {
        assert_eq!(
            sentence("Reads §4.1 of the design. Then more."),
            "Reads §4.1 of the design."
        );
        assert_eq!(sentence("Ends with no mark"), "Ends with no mark");
        assert_eq!(sentence("Line one\nline two!"), "Line one line two!");
        assert_eq!(sentence("Version v1.2 is fine"), "Version v1.2 is fine");
    }

    #[test]
    fn paragraphs_split_on_blank_lines() {
        assert_eq!(
            paragraphs("a\nb\n\n\nc\n"),
            vec!["a\nb".to_string(), "c".to_string()]
        );
    }
}
