//! Writes the AGENTS.md block and the CLAUDE.md import (design §3.5b item 2).
//! Cases: INIT-AGENTS-01, INIT-AGENTS-02, INIT-AGENTS-03, INIT-AGENTS-04, INIT-AGENTS-05, INIT-CLAUDE-01, INIT-CLAUDE-02, INIT-CLAUDE-03, INIT-DAMAGE-01, INIT-DAMAGE-02, INIT-DAMAGE-03

use std::path::Path;

use super::files::{create_new, label, lists, read_text, replace};
use super::{Action, Report};
use crate::error::Error;

/// The marker line that opens the block.
const BEGIN: &str = "<!-- memex:begin -->";
/// The marker line that closes the block.
const END: &str = "<!-- memex:end -->";
/// The snippet between the markers: design §6, with one source (assets/README.md).
const SNIPPET: &str = include_str!("../../../assets/agents-snippet.md");
/// The line that CLAUDE.md ends with, to import AGENTS.md (design §3.5b item 2).
const IMPORT: &str = "@AGENTS.md";
/// The warning for AGENTS.md whose markers are damaged, printed after `memex: warning: ` (design §3.5b item 2).
const DAMAGED: &str = "AGENTS.md: memex markers are damaged; fix them by hand";

/// What the text of AGENTS.md calls for.
#[derive(Debug, PartialEq, Eq)]
enum Merge {
    /// The block already is the snippet, so nothing is written.
    Same,
    /// The whole new text of the file.
    Changed(String),
    /// The markers are not one pair in order, so nothing is changed.
    Damaged,
}

/// Puts the block into AGENTS.md (design §3.5b item 2): created when the file is missing, appended when it has no markers, and
/// replaced between the markers when it has them. A file whose markers are damaged is left as it is, with a warning.
pub fn agents_md(target: &Path, report: &mut Report) -> Result<(), Error> {
    let path = target.join("AGENTS.md");
    let name = label(target, &path);
    if !lists(target, "AGENTS.md")? {
        create_new(target, "AGENTS.md", marked("\n").as_bytes())?;
        report.action(Action::Created, &name);
        return Ok(());
    }
    let text = read_text(&path, target)?;
    match merge(&text) {
        Merge::Same => report.action(Action::Kept, &name),
        Merge::Damaged => {
            report.warn(DAMAGED);
            report.action(Action::Kept, &name);
        }
        Merge::Changed(new) => {
            replace(&path, new.as_bytes())?;
            report.action(Action::Updated, &name);
        }
    }
    Ok(())
}

/// Adds `@AGENTS.md` as the final line of CLAUDE.md when no line of it is exactly that (design §3.5b item 2). A missing CLAUDE.md
/// stays missing. A CLAUDE.md that is the same file as AGENTS.md is kept, so the import never lands in AGENTS.md itself.
pub fn claude_md(target: &Path, report: &mut Report) -> Result<(), Error> {
    if !lists(target, "CLAUDE.md")? {
        return Ok(());
    }
    let path = target.join("CLAUDE.md");
    let name = label(target, &path);
    let text = read_text(&path, target)?;
    if imports(&text) || same_file(&path, &target.join("AGENTS.md")) {
        report.action(Action::Kept, &name);
        return Ok(());
    }
    replace(&path, with_import(&text).as_bytes())?;
    report.action(Action::Updated, &name);
    Ok(())
}

/// The new state of AGENTS.md, from its text. A leading byte order mark stays where it is.
fn merge(text: &str) -> Merge {
    let (bom, body) = split_bom(text);
    let eol = eol_of(body);
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    let begins = marker_lines(&lines, BEGIN);
    let ends = marker_lines(&lines, END);
    match (begins.as_slice(), ends.as_slice()) {
        ([], []) => Merge::Changed(format!("{bom}{}", append_block(body, eol))),
        ([begin], [end]) if begin < end => replace_block(bom, &lines, (*begin, *end), eol),
        _ => Merge::Damaged,
    }
}

/// The text with the lines between the markers replaced by the snippet. When they already are the snippet, the answer is Same.
fn replace_block(bom: &str, lines: &[&str], (begin, end): (usize, usize), eol: &str) -> Merge {
    let current: Vec<&str> = lines[begin + 1..end]
        .iter()
        .map(|&line| trim_eol(line))
        .collect();
    let snippet: Vec<&str> = SNIPPET.lines().collect();
    if current == snippet {
        return Merge::Same;
    }
    let mut out = String::from(bom);
    out.push_str(&lines[..=begin].concat());
    out.push_str(&snippet_text(eol));
    out.push_str(&lines[end..].concat());
    Merge::Changed(out)
}

/// The text with a blank line and the marked block at its end. A newline comes first when the text lacks one.
fn append_block(body: &str, eol: &str) -> String {
    let mut out = body.to_string();
    if !body.is_empty() && !body.ends_with('\n') {
        out.push_str(eol);
    }
    out.push_str(eol);
    out.push_str(&marked(eol));
    out
}

/// The text with `@AGENTS.md` as its final line, after a newline if the text lacks one, in the file's own line ending.
fn with_import(text: &str) -> String {
    let (bom, body) = split_bom(text);
    let eol = eol_of(body);
    let mut out = String::from(bom);
    out.push_str(body);
    if !body.is_empty() && !body.ends_with('\n') {
        out.push_str(eol);
    }
    out.push_str(IMPORT);
    out.push_str(eol);
    out
}

/// The marked block with `eol` line endings: the begin marker, the snippet, then the end marker.
fn marked(eol: &str) -> String {
    format!("{BEGIN}{eol}{}{END}{eol}", snippet_text(eol))
}

/// The snippet with `eol` at the end of every line.
fn snippet_text(eol: &str) -> String {
    SNIPPET.lines().map(|line| format!("{line}{eol}")).collect()
}

/// The indexes of the lines that are exactly `marker`, ignoring the blanks and the line ending around it.
fn marker_lines(lines: &[&str], marker: &str) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == marker)
        .map(|(index, _)| index)
        .collect()
}

/// True when some line of the text is exactly the import. A line ending is not part of a line.
fn imports(text: &str) -> bool {
    split_bom(text).1.lines().any(|line| line == IMPORT)
}

/// True when both paths name one file once symlinks are followed. A path that does not resolve is never the same file.
fn same_file(left: &Path, right: &Path) -> bool {
    match (dunce::canonicalize(left), dunce::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

/// The text split into its leading byte order mark, which is kept as it is, and the rest.
fn split_bom(text: &str) -> (&str, &str) {
    match text.strip_prefix('\u{feff}') {
        Some(rest) => ("\u{feff}", rest),
        None => ("", text),
    }
}

/// The line ending the text uses: CRLF when it has one, else LF, so that a block written into it matches the file.
fn eol_of(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// The line without its line ending.
fn trim_eol(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_markers_append_a_blank_line_and_the_block() {
        let expected = format!("# Notes\n\n{}", marked("\n"));
        assert_eq!(merge("# Notes\n"), Merge::Changed(expected));
    }

    #[test]
    fn a_block_that_is_the_snippet_is_the_same() {
        let text = format!("# Notes\n{}", marked("\n"));
        assert_eq!(merge(&text), Merge::Same);
    }

    #[test]
    fn markers_that_do_not_pair_once_in_order_are_damaged() {
        let begin_only = format!("{BEGIN}\nx\n");
        let end_first = format!("{END}\n{BEGIN}\n");
        let two_pairs = format!("{}{}", marked("\n"), marked("\n"));
        assert_eq!(merge(&begin_only), Merge::Damaged);
        assert_eq!(merge(&end_first), Merge::Damaged);
        assert_eq!(merge(&two_pairs), Merge::Damaged);
    }

    #[test]
    fn a_crlf_file_gets_a_crlf_block() {
        let Merge::Changed(text) = merge("# Notes\r\n") else {
            panic!("a file without markers is changed");
        };
        assert!(text.ends_with("<!-- memex:end -->\r\n"));
        assert!(!text.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn the_import_goes_on_a_new_final_line_once() {
        assert_eq!(with_import("# A"), "# A\n@AGENTS.md\n");
        assert_eq!(with_import("# A\n"), "# A\n@AGENTS.md\n");
        assert!(imports("x\n@AGENTS.md\r\n"));
        assert!(!imports("x\n @AGENTS.md\n"));
    }
}
