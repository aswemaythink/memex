//! Reads the lines a doc's citations come from, its frontmatter fields first (design §2.3).
//! A frontmatter collection doc, and a folder doc whose marker is Markdown, cite their field values before their text, each value and
//! each list item on its own, and never their `memex` key. Every other doc cites its text alone.
//! Cases: CITE-FIELD-01, CITE-FIELD-02, CITE-FIELD-04, CITE-FIELD-05, CITE-FIELD-06, CITE-FIELD-07, CITE-FIELD-08, CITE-FIELD-09

use crate::collections::frontmatter::Value;
use crate::corpus::Doc;
use crate::fence;

/// The frontmatter key that names a collection. Its value is the collection's name, which is never a citation, so it is not read
/// (design §2.2 and §2.3).
const NAME_KEY: &str = "memex";

/// The lines that a doc's citations are read from, in order (design §2.3): the values of its frontmatter fields when they are cited,
/// each scalar and each list item on its own, then the lines of its text outside fenced code. Each value is read for fences on its
/// own, so a fence in one value hides nothing in another.
pub fn cited_lines(doc: &Doc) -> Vec<&str> {
    let text = fence::live_lines(&doc.text);
    if !doc.cites_fields() {
        return text;
    }
    let mut lines = Vec::new();
    for (_, value) in doc.fields.iter().filter(|(key, _)| key != NAME_KEY) {
        match value {
            Value::Str(scalar) => lines.extend(fence::live_lines(scalar)),
            Value::List(items) => {
                for item in items {
                    lines.extend(fence::live_lines(item));
                }
            }
        }
    }
    lines.extend(text);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Origin;

    /// A doc of `origin` with these fields and this text.
    fn doc_of(origin: Origin, fields: Vec<(String, Value)>, text: &str) -> Doc {
        Doc {
            label: "decisions:x".to_string(),
            collection: "decisions".to_string(),
            id: "x".to_string(),
            path: Some("decisions/x.md".to_string()),
            title: String::new(),
            text: text.to_string(),
            fields,
            origin,
        }
    }

    #[test]
    fn a_collection_doc_reads_its_fields_first_each_item_alone_and_never_its_memex_key() {
        let fields = vec![
            ("memex".to_string(), Value::Str("decisions".to_string())),
            ("case".to_string(), Value::Str("MONEY-001".to_string())),
            (
                "tags".to_string(),
                Value::List(vec!["```".to_string(), "MONEY-002".to_string()]),
            ),
        ];
        let want = ["MONEY-001", "MONEY-002", "See MONEY-003"];
        let whole = doc_of(Origin::Whole, fields.clone(), "See MONEY-003");
        assert_eq!(cited_lines(&whole), want);
        let folder = Origin::Other {
            folder: String::new(),
            frontmatter: true,
        };
        assert_eq!(cited_lines(&doc_of(folder, fields, "See MONEY-003")), want);
    }

    #[test]
    fn a_log_and_a_json_doc_read_their_text_alone() {
        let fields = vec![("case".to_string(), Value::Str("MONEY-001".to_string()))];
        let json = Origin::Other {
            folder: String::new(),
            frontmatter: false,
        };
        let text = "See MONEY-003";
        assert_eq!(
            cited_lines(&doc_of(Origin::Log, fields.clone(), text)),
            [text]
        );
        assert_eq!(cited_lines(&doc_of(json, fields, text)), [text]);
    }
}
