//! Prints `info`'s atlas as text under the 3,000-byte cap (design §3.5).
//! It prints the header, one line per part and the use line, and cuts the longest list until the text fits.
//! Cases: INFO-HEAD-01, INFO-HEAD-02, INFO-INTENT-01, INFO-INTENT-02, INFO-INTENT-03, INFO-COLL-01, INFO-COLL-02, INFO-STREAM-01, INFO-STREAM-02, INFO-STREAM-03, INFO-STREAM-04, INFO-TO-01, INFO-TO-02, INFO-TO-03, INFO-TO-04, INFO-HERE-01, INFO-HERE-02, INFO-HERE-03, INFO-HERE-04, INFO-USE-01, INFO-CAP-01, INFO-NOROOT-01, INFO-NOROOT-02, INFO-WARN-01, INFO-ZERO-01, INFO-ZERO-02

use super::{Atlas, Collection, HereLine, Intent, Stream, ToCount};
use crate::render::text::clip;

/// The most bytes that the text of the atlas may take (design §3.5).
pub(super) const CAP: usize = 3_000;

/// The separator between the parts of a line (design §3.2 and §3.5).
const DOT: &str = " · ";

/// The use line, which names the three commands that the atlas points to (design §3.5).
const USE: &str = "memex show <path|ref> · memex search <words> · memex log <stream> <<'EOF' … EOF";

/// The text of the atlas (design §3.5): the header, then one line per part that has something to show. Text over the cap has
/// its longest list line cut to its first items and a `+N more` ending, until the whole text is under the cap.
pub fn text(atlas: &Atlas) -> String {
    let mut lists = [
        List::new("collections", collection_items(&atlas.collections)),
        List::new("streams", stream_items(&atlas.streams)),
    ];
    loop {
        let out = assemble(atlas, &lists);
        if out.len() <= CAP {
            return out;
        }
        let longest = lists
            .iter_mut()
            .filter(|list| list.shown > 0)
            .max_by_key(|list| list.line_len());
        match longest {
            Some(list) => list.shown -= 1,
            None => return out,
        }
    }
}

/// A list line of the text: its label, its items, and how many of them the cap lets it show.
struct List {
    /// The label printed before the value.
    label: &'static str,
    /// Every item of the list, in the order they print.
    items: Vec<String>,
    /// How many items print before the `+N more` ending; all of them when the list is not cut.
    shown: usize,
}

impl List {
    /// A list that shows every item.
    fn new(label: &'static str, items: Vec<String>) -> List {
        let shown = items.len();
        List {
            label,
            items,
            shown,
        }
    }

    /// The value of the line: the shown items joined with ` · `, then `+N more` when some items are cut.
    fn value(&self) -> String {
        let mut parts: Vec<String> = self.items[..self.shown].to_vec();
        if self.shown < self.items.len() {
            parts.push(format!("+{} more", self.items.len() - self.shown));
        }
        parts.join(DOT)
    }

    /// The length in bytes of the whole line, label included, which the cap compares.
    fn line_len(&self) -> usize {
        self.label.len() + self.value().len()
    }
}

/// The whole text with each list at its current length: the header, the intent line, each list that has items, the to and
/// here lines, then the use line. Every label is padded to the longest label that prints, and the value follows two spaces on.
fn assemble(atlas: &Atlas, lists: &[List]) -> String {
    let mut lines: Vec<(String, String)> =
        vec![("intent".to_string(), intent_value(&atlas.intent))];
    for list in lists.iter().filter(|list| !list.items.is_empty()) {
        lines.push((list.label.to_string(), list.value()));
    }
    if let Some(to) = &atlas.to {
        lines.push((format!("to {}", to.agent), to_value(to)));
    }
    if let Some(here) = &atlas.here {
        lines.push(("here".to_string(), here_value(here)));
    }
    lines.push(("use".to_string(), USE.to_string()));
    let width = lines
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let mut out = format!("memex  {} · log store {}\n", atlas.root, atlas.store_label);
    for (label, value) in lines {
        out.push_str(&format!("{label:<width$}  {value}\n"));
    }
    out
}

/// The intent line's value: the built-in counts in the order readmes, headers and cases.
fn intent_value(intent: &Intent) -> String {
    [
        counted(intent.readmes, "readme", "readmes"),
        counted(intent.headers, "header", "headers"),
        counted(intent.cases, "case", "cases"),
    ]
    .join(DOT)
}

/// A count with its noun, in the singular for one and the plural otherwise.
fn counted(count: usize, one: &str, many: &str) -> String {
    let noun = if count == 1 { one } else { many };
    format!("{count} {noun}")
}

/// The items of the collections line: each name and its doc count.
fn collection_items(collections: &[Collection]) -> Vec<String> {
    collections
        .iter()
        .map(|collection| format!("{} {}", collection.name, collection.count))
        .collect()
}

/// The items of the streams line: each name and its entry count, then how many are new. The first stream with new entries says
/// `in 24h`, and the later ones say only `new`; a stream with none says nothing (design §3.5).
fn stream_items(streams: &[Stream]) -> Vec<String> {
    let mut items = Vec::new();
    let mut first = true;
    for stream in streams {
        let note = match stream.new_24h {
            0 => String::new(),
            count if first => {
                first = false;
                format!(" ({count} new in 24h)")
            }
            count => format!(" ({count} new)"),
        };
        items.push(format!("{} {}{note}", stream.name, stream.count));
    }
    items
}

/// The value of the to line: the count of messages, then the search that finds them.
fn to_value(to: &ToCount) -> String {
    let messages = counted(to.count, "message", "messages");
    format!("{messages} → memex search to:{}", to.agent)
}

/// The value of the here line: the folder, then its one-liner cut at 120 characters; the dash and the one-liner are left out
/// when the README has none.
fn here_value(here: &HereLine) -> String {
    if here.title.is_empty() {
        here.path.clone()
    } else {
        format!("{} — {}", here.path, clip(&here.title))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_of_one_is_singular() {
        assert_eq!(counted(1, "readme", "readmes"), "1 readme");
        assert_eq!(counted(0, "readme", "readmes"), "0 readmes");
        assert_eq!(counted(2, "readme", "readmes"), "2 readmes");
    }

    #[test]
    fn a_cut_list_ends_with_its_count_of_the_rest() {
        let mut list = List::new("streams", vec!["a 1".to_string(), "b 2".to_string()]);
        assert_eq!(list.value(), "a 1 · b 2");
        list.shown = 1;
        assert_eq!(list.value(), "a 1 · +1 more");
        list.shown = 0;
        assert_eq!(list.value(), "+2 more");
    }

    #[test]
    fn a_to_line_at_zero_says_zero_messages() {
        let to = ToCount {
            agent: "x".to_string(),
            count: 0,
        };
        assert_eq!(to_value(&to), "0 messages → memex search to:x");
    }
}
