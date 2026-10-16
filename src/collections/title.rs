//! Gives a collection doc its one-liner, by one rule for every kind (design §4.2 `title`).
//! It is the `title` field when that field has text, else the default line of the doc's text: its first ATX heading outside a
//! code fence, else its first line with text. No sentence is cut from either.
//! Cases: COLL-TITLE-01, MANK-FOLDER-07, MANK-JSON-05, MANK-TITLE-01, MANK-TITLE-02, MANK-TITLE-03

use super::frontmatter::Value;
use crate::fence;

/// The one-liner of a doc: the text of the field `title` names when it has text, else the default line of `text`.
pub fn one_liner(title: Option<&str>, fields: &[(String, Value)], text: &str) -> String {
    title
        .and_then(|key| title_field(fields, key))
        .unwrap_or_else(|| default_title(text))
}

/// The text of the field `key` when it is a string with some text. An empty field gives none, so the default applies.
fn title_field(fields: &[(String, Value)], key: &str) -> Option<String> {
    fields.iter().find_map(|(name, value)| match value {
        Value::Str(text) if name == key && !text.is_empty() => Some(text.clone()),
        _ => None,
    })
}

/// The default one-liner of a text: its first heading outside a code fence, else its first line with text, else nothing.
pub fn default_title(text: &str) -> String {
    heading(text)
        .or_else(|| first_line(text))
        .unwrap_or_default()
}

/// The text of the first ATX heading that is not inside a code fence, under the one fence definition (design §2.3).
fn heading(text: &str) -> Option<String> {
    fence::live_lines(text)
        .into_iter()
        .find_map(|line| heading_text(line.trim_start()))
}

/// The text of an ATX heading line: one to six `#` marks, then a space, a tab or the end of the line, and the text after them.
fn heading_text(line: &str) -> Option<String> {
    let marks = line.bytes().take_while(|byte| *byte == b'#').count();
    let rest = &line[marks..];
    let is_heading = (1..=6).contains(&marks) && (rest.is_empty() || rest.starts_with([' ', '\t']));
    let text = rest.trim();
    (is_heading && !text.is_empty()).then(|| text.to_string())
}

/// The first line of the text that has any text, trimmed.
fn first_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// COLL-TITLE-01 and MANK-TITLE-01: a heading inside a code fence is not the default title, a fence indented four spaces is text,
    /// and a `#` with no space after it is text.
    #[test]
    fn headings_skip_fences_and_need_a_space_after_the_marks() {
        assert_eq!(default_title("```\n# no\n```\n# Yes\n"), "Yes");
        assert_eq!(default_title("    ```\n# Yes\n"), "Yes");
        assert_eq!(default_title("#tag line\nSecond\n"), "#tag line");
        assert_eq!(default_title("##  Two  \n"), "Two");
        assert_eq!(default_title("\n\n  Plain  \n"), "Plain");
        assert_eq!(default_title(""), "");
    }

    /// MANK-TITLE-02: seven marks are not a heading, and six are.
    #[test]
    fn six_marks_make_a_heading_and_seven_do_not() {
        assert_eq!(default_title("####### Seven\n"), "####### Seven");
        assert_eq!(default_title("Lead\n###### Six\n"), "Six");
    }

    /// MANK-TITLE-03 and MANK-JSON-05: a title field with text is the one-liner whole, an empty one falls back, and no
    /// manifest title gives the default.
    #[test]
    fn a_title_field_is_whole_and_an_empty_one_falls_back() {
        let fields = [
            ("claim".to_string(), Value::Str("One. Two.".to_string())),
            ("empty".to_string(), Value::Str(String::new())),
        ];
        assert_eq!(one_liner(Some("claim"), &fields, "Body\n"), "One. Two.");
        assert_eq!(one_liner(Some("empty"), &fields, "Body\n"), "Body");
        assert_eq!(one_liner(Some("missing"), &fields, "# Head\n"), "Head");
        assert_eq!(one_liner(None, &fields, "# Head\n"), "Head");
    }
}
