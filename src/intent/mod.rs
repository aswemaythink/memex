//! The readers of the intent layer and the one-liner rule they share (design §2).
//! Paragraphs, the first sentence, and the one-liner of a block.
//! Cases: SHOW-DIR-01, SHOW-FILE-01, SHOW-LINE-01

pub mod cases;
pub mod header;
pub mod readme;

/// The paragraphs of a block: runs of non-blank lines, each joined with newlines. Blank lines separate them.
pub fn paragraphs(text: &str) -> Vec<String> {
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
