//! Splits a document's flat-YAML frontmatter from its body (design §4.2 and §4.3).
//! It reads the fields: `key: scalar`, `key: [a, b]`, `- item` lists, and nested text kept as written.
//! `after_frontmatter` cuts a closed block from a text, the same way, for the one-liners and titles that read the body.
//! Cases: SHOW-LOG-01, SHOW-JSON-03, SHOW-MDCHILD-03

use crate::text::line_of;

/// One frontmatter value: a scalar, or nested text kept as written, or a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A scalar, or nested text kept as written.
    Str(String),
    /// A flow list `[a, b]`, or a block of `- item` lines.
    List(Vec<String>),
}

/// A document split at its frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// The frontmatter fields, in the order the file gives them. A repeated key keeps its first value.
    pub fields: Vec<(String, Value)>,
    /// The text after the closing fence, verbatim. The whole text when there is no frontmatter.
    pub body: String,
}

/// The line that opens and closes a frontmatter block.
const FENCE: &str = "---";

/// Splits `text` at its frontmatter. A text with no leading fence, or whose fence never closes, has no fields and keeps
/// its whole text as the body.
pub fn split(text: &str) -> Document {
    let pieces: Vec<&str> = text.split_inclusive('\n').collect();
    if pieces.first().is_none_or(|first| line_of(first) != FENCE) {
        return whole(text);
    }
    for (index, piece) in pieces.iter().enumerate().skip(1) {
        if line_of(piece) == FENCE {
            let consumed: usize = pieces[..=index].iter().map(|piece| piece.len()).sum();
            let lines: Vec<&str> = pieces[1..index]
                .iter()
                .map(|piece| line_of(piece))
                .collect();
            return Document {
                fields: fields_of(&lines),
                body: text[consumed..].to_string(),
            };
        }
    }
    whole(text)
}

/// The text after a closed frontmatter block at the top of `text`, which `text::decode` has read, by the rule `split` uses: a first
/// line `---` and the next `---` line. A text with no such block, or whose block never closes, is whole. A README's one-liner and
/// search body, a Markdown file's title, and `show`'s text body (design §3.3) are read from it; `show --json` keeps the whole file.
pub fn after_frontmatter(text: &str) -> &str {
    let mut pieces = text.split_inclusive('\n');
    let Some(first) = pieces.next().filter(|first| line_of(first) == FENCE) else {
        return text;
    };
    let mut consumed = first.len();
    for piece in pieces {
        consumed += piece.len();
        if line_of(piece) == FENCE {
            return &text[consumed..];
        }
    }
    text
}

/// A document with no fields, whose body is the whole text.
fn whole(text: &str) -> Document {
    Document {
        fields: Vec::new(),
        body: text.to_string(),
    }
}

/// The fields of the frontmatter lines. Blank lines are skipped, a line that opens a key starts a field, and any other line
/// belongs to the field above it. Each key keeps its first value.
fn fields_of(lines: &[&str]) -> Vec<(String, Value)> {
    let mut blocks: Vec<(&str, &str, Vec<&str>)> = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((key, inline)) = key_line(line) {
            blocks.push((key, inline, Vec::new()));
        } else if let Some(block) = blocks.last_mut() {
            block.2.push(line.trim_end());
        }
    }
    let mut fields: Vec<(String, Value)> = Vec::new();
    for (key, inline, nested) in blocks {
        if fields.iter().all(|(name, _)| name != key) {
            fields.push((key.to_string(), value_of(inline, &nested)));
        }
    }
    fields
}

/// The key and the inline value of a line that opens a field: an identifier at column 0, then a colon.
fn key_line(line: &str) -> Option<(&str, &str)> {
    let (name, rest) = line.split_once(':')?;
    let mut chars = name.chars();
    let opens = matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic() || ch == '_');
    let rest_ok = chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    (opens && rest_ok).then_some((name, rest.trim()))
}

/// The value of a field: its inline text when it has no nested lines. A nested block of `- item` lines is a list, and any
/// other nested text is kept as written, after the inline text.
fn value_of(inline: &str, nested: &[&str]) -> Value {
    if nested.is_empty() {
        return scalar(inline);
    }
    if inline.is_empty()
        && nested
            .iter()
            .all(|line| line.trim_start().starts_with("- "))
    {
        let items = nested
            .iter()
            .filter_map(|line| line.trim_start().strip_prefix("- "))
            .map(|item| unquote(item.trim()).to_string())
            .collect();
        return Value::List(items);
    }
    let mut parts: Vec<&str> = Vec::new();
    if !inline.is_empty() {
        parts.push(inline);
    }
    parts.extend(nested.iter().copied());
    Value::Str(parts.join("\n"))
}

/// A flow list `[a, b]` or a scalar, from an inline value. Empty items of a flow list are dropped.
fn scalar(text: &str) -> Value {
    match text
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
    {
        Some(inner) => Value::List(
            inner
                .split(',')
                .map(|item| unquote(item.trim()).to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        ),
        None => Value::Str(unquote(text).to_string()),
    }
}

/// The text inside one pair of matching quotes, or the text itself.
fn unquote(text: &str) -> &str {
    match text.as_bytes() {
        [first, .., last] if first == last && (*first == b'"' || *first == b'\'') => {
            &text[1..text.len() - 1]
        }
        _ => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_flow_lists_and_block_lists_become_fields() {
        let doc = split(
            "---\r\nstream: impl\r\ntags: [a, 'b']\r\nto:\r\n- x\r\n- \"y\"\r\n---\r\n# Body\r\n",
        );
        assert_eq!(
            doc.fields,
            vec![
                ("stream".to_string(), Value::Str("impl".to_string())),
                (
                    "tags".to_string(),
                    Value::List(vec!["a".to_string(), "b".to_string()])
                ),
                (
                    "to".to_string(),
                    Value::List(vec!["x".to_string(), "y".to_string()])
                ),
            ]
        );
        assert_eq!(doc.body, "# Body\r\n");
    }

    #[test]
    fn nested_text_is_kept_and_a_text_without_a_block_is_all_body() {
        let doc = split("---\nnotes:\n  first: one\n---\nbody\n");
        assert_eq!(
            doc.fields,
            vec![("notes".to_string(), Value::Str("  first: one".to_string()))]
        );
        assert_eq!(split("# Title\n---\nx").fields, Vec::new());
        assert_eq!(
            split("---\nopen: never closed\n").body,
            "---\nopen: never closed\n"
        );
    }

    #[test]
    fn only_a_closed_block_at_the_top_is_cut_from_the_text() {
        assert_eq!(after_frontmatter("---\ntitle: D\n---\n# D\n"), "# D\n");
        assert_eq!(after_frontmatter("---\ntitle: D\n"), "---\ntitle: D\n");
        assert_eq!(
            after_frontmatter("# D\n---\nx\n---\n"),
            "# D\n---\nx\n---\n"
        );
    }
}
