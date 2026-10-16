//! Finds a file's header block past its skipped lines (design §2.1), and tells whether it runs on.
//! Cases: HDR-SLASH-01, HDR-SLASH-02, HDR-SLASH-03, HDR-SLASH-04, HDR-SLASH-05, HDR-HASH-01, HDR-DOC-01, HDR-DOC-02, HDR-DOC-03, HDR-DOC-04, HDR-DASH-01, HDR-DASH-02, HDR-DASH-03, HDR-SEMI-01, HDR-PERCENT-01, HDR-MARKUP-01, HDR-PAREN-01, HDR-SKIP-01, HDR-SKIP-02, HDR-SKIP-03, HDR-SKIP-04, HDR-SKIP-05, HDR-SKIP-06, HDR-SKIP-07, HDR-BLOCK-01, HDR-BLOCK-02, HDR-NONE-01, HDR-CAP-03, HDR-CAP-04, HDR-CAP-05

use std::ops::Range;

use super::families::{Family, PRAGMAS};

/// A header as the lines of a file give it: its text, and whether its block was still open when the lines ran out. An open block
/// goes on past them, so a file that is longer than the lines it was read from has a header cut at the read limit (design §2.1).
pub struct Found {
    /// The header text: the comment markers removed, and the blank lines at either end dropped.
    pub text: String,
    /// True when the block runs to the last line, with no line after it that ends it: a longer file would go on with it.
    pub open: bool,
}

/// The header of a file whose lines are `lines`, read with `family`. None when no header block leads the file.
pub fn header_text(family: &Family, lines: &[&str]) -> Option<Found> {
    scan(family, lines).0
}

/// What the scan finds in a file's lines (design §2.1): the header as `header_text` gives it, then the ranges of indices into `lines`
/// of the license block and of the header block. The header range runs from the header's first line to its last comment line, so the
/// blank lines after that comment are left out. A range is empty when its block is absent.
pub(super) fn scan(family: &Family, lines: &[&str]) -> (Option<Found>, Range<usize>, Range<usize>) {
    let (start, license) = skip_leading(family, lines);
    let (body, open, end) = header_body(family, &lines[start..]);
    let text = trim_blank_edges(body).join("\n");
    if text.is_empty() {
        return (None, license, 0..0);
    }
    (Some(Found { text, open }), license, start..start + end)
}

/// The body lines of the header that starts `lines`, whether it is open at their end, and the index after its last line of content:
/// a module docstring when the first line opens one, else the comment block.
fn header_body(family: &Family, lines: &[&str]) -> (Vec<String>, bool, usize) {
    if let Some((first, tail)) = lines.split_first() {
        if let Some(delimiter) = docstring_at(family, first.trim_start()) {
            return docstring(first, tail, delimiter);
        }
    }
    comment_block(family, lines)
}

/// The index of the first line after the blank, pragma and leading license lines (design §2.1), and the range of the license block
/// it skips, empty when it skips none. Only the first license block is skipped; a later one is part of the header.
fn skip_leading(family: &Family, lines: &[&str]) -> (usize, Range<usize>) {
    let mut at = 0;
    let mut license = None;
    while let Some(line) = lines.get(at) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || PRAGMAS.iter().any(|pragma| trimmed.starts_with(*pragma)) {
            at += 1;
            continue;
        }
        match license_end(family, lines, at) {
            Some(end) if license.is_none() => {
                license = Some(at..end);
                at = end;
            }
            _ => break,
        }
    }
    (at, license.unwrap_or(0..0))
}

/// The index just past the license block at `at`, when one opens there. The block is the comments from `at` up to the first
/// blank line, so a blank line inside a `/* */` block ends it too, unlike a header block (design §2.1). It is a license when it
/// holds `SPDX-License-Identifier` or its first text starts with `Copyright`.
fn license_end(family: &Family, lines: &[&str], at: usize) -> Option<usize> {
    let blank = lines[at..]
        .iter()
        .position(|line| line.trim().is_empty())
        .map_or(lines.len(), |offset| at + offset);
    let scope = &lines[..blank];
    let mut texts: Vec<String> = Vec::new();
    let mut next = at;
    while let Some((body, end)) = comment_at(family, scope, next) {
        texts.extend(body);
        next = end;
    }
    let spdx = texts
        .iter()
        .any(|text| text.contains("SPDX-License-Identifier"));
    let copyright = texts
        .iter()
        .find(|text| !text.trim().is_empty())
        .is_some_and(|text| text.trim_start().starts_with("Copyright"));
    (next > at && (spdx || copyright)).then_some(next)
}

/// The body lines of the header block from the start of `lines`, whether the block runs to their end, and the index after its last
/// comment line. A blank line is kept, and does not end the block; the block ends at the first line that is neither a comment nor
/// blank (design §2.1).
fn comment_block(family: &Family, lines: &[&str]) -> (Vec<String>, bool, usize) {
    let mut body = Vec::new();
    let mut at = 0;
    let mut content = 0;
    while let Some(line) = lines.get(at) {
        if line.trim().is_empty() {
            body.push(String::new());
            at += 1;
        } else if let Some((text, end)) = comment_at(family, lines, at) {
            body.extend(text);
            at = end;
            content = end;
        } else {
            return (body, false, content);
        }
    }
    (body, true, content)
}

/// The body lines of the comment that opens on line `at`, and the index after it. A block runs to its close line, and a line
/// comment is one line. None when the line opens no comment of the family.
fn comment_at(family: &Family, lines: &[&str], at: usize) -> Option<(Vec<String>, usize)> {
    let trimmed = lines.get(at)?.trim_start();
    if let Some((open, close)) = family
        .blocks
        .iter()
        .find(|(open, _)| trimmed.starts_with(*open))
    {
        return Some(block_at(lines, at, open, close));
    }
    let marker = family
        .lines
        .iter()
        .find(|marker| trimmed.starts_with(**marker))?;
    Some((vec![line_text(&trimmed[marker.len()..])], at + 1))
}

/// The body lines of the block comment that opens on line `at`, and the index after its close line. A block with no close runs
/// to the end of `lines`.
fn block_at(lines: &[&str], at: usize, open: &str, close: &str) -> (Vec<String>, usize) {
    let first = lines[at].trim_start();
    let rest = &first[open.len()..];
    if let Some(end) = rest.find(close) {
        return (vec![block_text(&rest[..end])], at + 1);
    }
    let mut body = vec![block_text(rest)];
    for (index, line) in lines.iter().enumerate().skip(at + 1) {
        if let Some(end) = line.find(close) {
            body.push(block_text(&line[..end]));
            return (body, index + 1);
        }
        body.push(block_text(line));
    }
    (body, lines.len())
}

/// The lines of a module docstring that opens with `delimiter` on `first`: the text between its quotes, which runs over `tail` until
/// the closing delimiter. Also whether it is open at the end of `tail` (no delimiter closes it), and the index after its last line.
fn docstring(first: &str, tail: &[&str], delimiter: &str) -> (Vec<String>, bool, usize) {
    let opening = first.trim_start();
    let rest = &opening[delimiter.len()..];
    if let Some(end) = rest.find(delimiter) {
        return (vec![rest[..end].trim().to_string()], false, 1);
    }
    let mut body = vec![rest.trim().to_string()];
    for (index, line) in tail.iter().enumerate() {
        if let Some(end) = line.find(delimiter) {
            body.push(line[..end].trim_end().to_string());
            return (body, false, index + 2);
        }
        body.push(line.trim_end().to_string());
    }
    (body, true, tail.len() + 1)
}

/// The docstring delimiter that `trimmed` opens with, when the family has docstrings.
fn docstring_at(family: &Family, trimmed: &str) -> Option<&'static str> {
    family
        .docstrings
        .iter()
        .copied()
        .find(|delimiter| trimmed.starts_with(*delimiter))
}

/// A line comment's text: the marker is gone, and so is the one space after it.
fn line_text(rest: &str) -> String {
    rest.strip_prefix(' ')
        .unwrap_or(rest)
        .trim_end()
        .to_string()
}

/// One line of a block comment's body. A leading `*` with its space is dropped, as Javadoc writes it, then one space after
/// the marker goes, and trailing blanks go too.
fn block_text(text: &str) -> String {
    let trimmed = text.trim_start();
    let body = match trimmed.strip_prefix('*') {
        Some(rest) if rest.is_empty() || rest.starts_with(' ') => rest,
        _ => text,
    };
    body.strip_prefix(' ')
        .unwrap_or(body)
        .trim_end()
        .to_string()
}

/// The lines without the blank ones at either end.
fn trim_blank_edges(mut body: Vec<String>) -> Vec<String> {
    while body.last().is_some_and(|line| line.trim().is_empty()) {
        body.pop();
    }
    let lead = body
        .iter()
        .take_while(|line| line.trim().is_empty())
        .count();
    body.drain(..lead);
    body
}
