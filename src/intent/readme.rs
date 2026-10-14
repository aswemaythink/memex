//! Reads a README's one-liner (design §2): the first sentence of the first paragraph after its H1, or the H1 text when no paragraph follows it.
//! Cases: SHOW-DIR-01, SHOW-ROOT-01, SHOW-LINE-01

use super::{lead, paragraphs, sentence};

/// The one-liner of a README. A README whose first line is not an H1 takes the first sentence of its first paragraph.
pub fn one_liner(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(first) = lines.iter().position(|line| !line.trim().is_empty()) else {
        return String::new();
    };
    let Some(heading) = lines[first].strip_prefix("# ") else {
        return lead(text);
    };
    let rest = lines[first + 1..].join("\n");
    let prose = paragraphs(&rest)
        .into_iter()
        .find(|paragraph| !paragraph.starts_with('#'));
    match prose {
        Some(paragraph) => sentence(&paragraph),
        None => heading.trim().to_string(),
    }
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
}
