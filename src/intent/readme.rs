//! Reads a README (design §2 and §4.2): its doc, and the one-liner that its folder lists.
//! Cases: INT-LINE-01, INT-LINE-02, INT-LINE-03, INT-LINE-04, INT-LINE-05, INT-NOHEAD-01, INT-NOHEAD-02, SHOW-DIR-01, SHOW-ROOT-01, SHOW-LINE-01

use super::{sentence, Origin};
use crate::corpus::{Doc, READMES};

/// The doc of the README `rel` whose whole text is `contents`, labelled by its path (design §3.1). The file is read by the
/// caller, which warns about one that cannot be read (design §4.2).
pub fn doc(rel: &str, contents: String) -> Doc {
    Doc {
        label: rel.to_string(),
        collection: READMES.to_string(),
        id: rel.to_string(),
        path: Some(rel.to_string()),
        title: one_liner(&contents),
        text: contents,
        fields: Vec::new(),
        origin: Origin::Graph,
    }
}

/// The one-liner of a README: the first sentence of its first prose paragraph after the H1, or the H1 text when no
/// paragraph follows it. A README whose first line is not an H1 is read from its top in the same way.
fn one_liner(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(first) = lines.iter().position(|line| !line.trim().is_empty()) else {
        return String::new();
    };
    let (heading, body) = match lines[first].strip_prefix("# ") {
        Some(heading) => (heading.trim(), &lines[first + 1..]),
        None => ("", &lines[..]),
    };
    match prose(body) {
        Some(paragraph) => sentence(&paragraph),
        None => heading.to_string(),
    }
}

/// The first prose paragraph of some lines: the first run of lines that are neither blank nor headings. A blank line or
/// a heading ends a run, and a heading is never prose.
fn prose(lines: &[&str]) -> Option<String> {
    let mut run: Vec<&str> = Vec::new();
    for &line in lines {
        if line.trim().is_empty() || is_heading(line) {
            if !run.is_empty() {
                break;
            }
        } else {
            run.push(line);
        }
    }
    (!run.is_empty()).then(|| run.join("\n"))
}

/// True for an ATX heading line: one to six `#` marks, then a space or the end of the line. `#tag` is prose.
fn is_heading(line: &str) -> bool {
    let text = line.trim_start();
    let rest = text.trim_start_matches('#');
    let marks = text.len() - rest.len();
    (1..=6).contains(&marks) && (rest.is_empty() || rest.starts_with(' '))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_prose_paragraph_after_the_h1_gives_the_line() {
        assert_eq!(one_liner("# Demo\n\nThe demo. More.\n"), "The demo.");
        assert_eq!(
            one_liner("# Demo\n\n## Part\n\nAfter the part.\n"),
            "After the part."
        );
        assert_eq!(one_liner("# Demo\n"), "Demo");
        assert_eq!(one_liner("No heading here.\n"), "No heading here.");
    }

    #[test]
    fn a_heading_ends_a_run_and_is_never_the_line() {
        assert_eq!(one_liner("# Demo\n## Part\nUnder it.\n"), "Under it.");
        assert_eq!(one_liner("## Part\n\nAfter it.\n"), "After it.");
        assert_eq!(one_liner("## Part\n"), "");
        assert_eq!(one_liner("#tag stays prose.\n"), "#tag stays prose.");
    }
}
