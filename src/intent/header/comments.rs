//! Lists a file's comment blocks past its header, for `show --comments` (design §3.3 Comments).
//! A block is read as the header is (`scan.rs`): a run of comment lines that a blank line does not end, and that ends at the first
//! line that is neither comment nor blank. A docstring is not a comment; a module docstring is one only as the first statement.
//! Cases: SHOW-COMMENTS-01, SHOW-COMMENTS-03, SHOW-COMMENTS-04, SHOW-COMMENTS-05, SHOW-COMMENTS-06, SHOW-COMMENTS-07, SHOW-COMMENTS-08

use super::families::Family;
use super::scan;

/// One comment block: the line it starts on, and its text as the header prints it, with the markers removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentBlock {
    /// The line number of the block's first line, counted from 1.
    pub line: usize,
    /// The block's lines joined by newlines, with the blank lines at either end dropped.
    pub text: String,
}

/// The comment blocks of a file's `lines` after its header, in file order (design §3.3 Comments). With a header, these are the blocks
/// after it. With none, they are the blocks after the lines §2.1 skips (the shebang, the pragmas and the license block), so the first
/// of them is listed even where a scan of the whole file would call it a header.
pub fn after_header(family: &Family, lines: &[&str]) -> Vec<CommentBlock> {
    let (start, _) = scan::skip_leading(family, lines);
    let (found, _, header) = scan::scan(family, lines);
    let from = if found.is_some() { header.end } else { start };
    walk(family, lines, start, from)
}

/// The blocks that start on or after line `from`, walking `lines` from `start`. The walk reads the header too, so that it knows
/// which lines are code and which are a docstring's.
fn walk(family: &Family, lines: &[&str], start: usize, from: usize) -> Vec<CommentBlock> {
    let mut blocks = Vec::new();
    let mut at = start;
    let mut statement = false;
    while at < lines.len() {
        let step = step(family, lines, at, statement);
        if at >= from {
            blocks.extend(step.block);
        }
        at = step.end;
        statement = step.statement;
    }
    blocks
}

/// What the line at `at` starts: the index after it, the comment block it is when it is one, and whether it is a statement.
struct Step {
    /// The index of the first line after this step.
    end: usize,
    /// The comment block that starts here, if one does.
    block: Option<CommentBlock>,
    /// True for code and for a string, which both come before a module docstring can stand.
    statement: bool,
}

/// The step at line `at`, given whether a statement came before it: a blank line, a comment block, or a line of code or a string.
fn step(family: &Family, lines: &[&str], at: usize, statement: bool) -> Step {
    if lines[at].trim().is_empty() {
        return Step {
            end: at + 1,
            block: None,
            statement,
        };
    }
    if scan::comment_at(family, lines, at).is_some() {
        return comment_step(family, lines, at, statement);
    }
    string_or_code_step(family, lines, at, statement)
}

/// The step at a comment: the run of comment lines from `at`, which a blank line inside does not end, is one block.
fn comment_step(family: &Family, lines: &[&str], at: usize, statement: bool) -> Step {
    let (body, _, content) = scan::comment_block(family, &lines[at..]);
    Step {
        end: at + content,
        block: Some(CommentBlock {
            line: at + 1,
            text: scan::trim_blank_edges(body).join("\n"),
        }),
        statement,
    }
}

/// The step at a line that is neither blank nor a comment. A module docstring, opened before any statement, is a block. Any other
/// string runs to its closing line and is not one. A line of code is a statement of its own.
fn string_or_code_step(family: &Family, lines: &[&str], at: usize, statement: bool) -> Step {
    let line = lines[at];
    if !statement {
        if let Some(delimiter) = scan::docstring_at(family, line.trim_start()) {
            let (body, _, end) = scan::docstring(line, &lines[at + 1..], delimiter);
            return Step {
                end: at + end,
                block: Some(CommentBlock {
                    line: at + 1,
                    text: scan::trim_blank_edges(body).join("\n"),
                }),
                statement: true,
            };
        }
    }
    Step {
        end: string_end(family, lines, at).unwrap_or(at + 1),
        block: None,
        statement: true,
    }
}

/// The index after the triple-quoted string that line `at` opens and does not close: the line that closes it, or the end of
/// `lines`. None when the line opens no such string, closes it on the same line, or has a comment before the delimiter.
fn string_end(family: &Family, lines: &[&str], at: usize) -> Option<usize> {
    let line = lines[at];
    let (position, delimiter) = family
        .docstrings
        .iter()
        .filter_map(|delimiter| line.find(*delimiter).map(|found| (found, *delimiter)))
        .min()?;
    if line[..position].contains('#') || line[position + delimiter.len()..].contains(delimiter) {
        return None;
    }
    let closing = lines[at + 1..]
        .iter()
        .position(|next| next.contains(delimiter));
    Some(closing.map_or(lines.len(), |offset| at + offset + 2))
}

#[cfg(test)]
mod tests {
    use super::super::families::family_of;
    use super::*;
    use std::path::Path;

    /// The blocks of `text` as a file named `name`, with their lines and texts as `(line, text)` pairs.
    fn blocks(name: &str, text: &str) -> Vec<(usize, String)> {
        let family = family_of(Path::new(name)).expect("a comment family");
        let lines: Vec<&str> = text.lines().collect();
        after_header(family, &lines)
            .into_iter()
            .map(|block| (block.line, block.text))
            .collect()
    }

    #[test]
    fn a_blank_line_inside_a_run_keeps_it_one_block() {
        let got = blocks(
            "a.rs",
            "//! Head.\nfn a() {}\n\n// one\n\n// two\nfn x() {}\n",
        );
        assert_eq!(got, vec![(4, "one\n\ntwo".to_string())]);
    }

    #[test]
    fn a_docstring_inside_a_function_is_not_a_block() {
        let text = "# Head.\n\nimport os\n\ndef f():\n    \"\"\"Doc.\n    # not a comment\n    \"\"\"\n    return 1\n";
        assert_eq!(blocks("tool.py", text), Vec::new());
    }

    #[test]
    fn a_module_docstring_after_a_hash_header_is_a_block() {
        let got = blocks("tool.py", "# Head.\n\n\"\"\"Module doc.\"\"\"\nimport os\n");
        assert_eq!(got, vec![(3, "Module doc.".to_string())]);
    }

    #[test]
    fn a_multi_line_module_docstring_is_one_block_from_its_first_line() {
        let got = blocks(
            "tool.py",
            "# Head.\n\n\"\"\"Module doc\n  over two lines.\n\"\"\"\nimport os\n",
        );
        assert_eq!(got, vec![(3, "Module doc\n  over two lines.".to_string())]);
    }

    #[test]
    fn a_string_in_code_that_spans_lines_hides_its_hash_lines() {
        let text = "# Head.\n\nx = \"\"\"\n# inside\n\"\"\"\n# real\ny = 1\n";
        assert_eq!(blocks("tool.py", text), vec![(6, "real".to_string())]);
    }

    #[test]
    fn a_comment_after_code_on_its_line_is_not_read() {
        assert_eq!(
            blocks("a.rs", "//! Head.\nfn x() {} // trailing\n"),
            Vec::new()
        );
    }
}
