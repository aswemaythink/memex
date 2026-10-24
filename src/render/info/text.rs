//! Prints `info`'s atlas as text under the 3,000-byte cap (design §3.5).
//! It prints the header, one line per part and the use line; over the cap it cuts the roots block in one pass, then the longest list until the text fits.
//! Cases: INFO-ROOTS-01, INFO-ROOTS-02, INFO-ROOTS-03, INFO-ROOTS-04, INFO-ROOTS-07, INFO-ROOTS-08, INFO-HEAD-01, INFO-HEAD-02, INFO-INTENT-01, INFO-INTENT-02, INFO-INTENT-03, INFO-COLL-01, INFO-COLL-02, INFO-STREAM-01, INFO-STREAM-02, INFO-STREAM-03, INFO-STREAM-04, INFO-TO-01, INFO-TO-02, INFO-TO-03, INFO-TO-04, INFO-HERE-01, INFO-HERE-02, INFO-HERE-03, INFO-HERE-04, INFO-USE-01, INFO-CAP-01, INFO-NOROOT-01, INFO-NOROOT-02, INFO-WARN-01, INFO-ZERO-01, INFO-ZERO-02

use super::{Atlas, Collection, HereLine, Intent, Stream, ToCount};
use crate::render::text::clip;

/// The most bytes that the text of the atlas may take (design §3.5).
pub(super) const CAP: usize = 3_000;

/// The separator between the parts of a line (design §3.2 and §3.5).
const DOT: &str = " · ";

/// The use line, which names the three commands that the atlas points to (design §3.5).
const USE: &str = "memex show <path|ref> · memex search <words> · memex log <stream> <<'EOF' … EOF";

/// The text of the atlas (design §3.5): the header, then one line per part that has something to show. Text over the cap has
/// its roots block cut first, from its end, to a `+N more → memex tree` line, and then its longest list line cut to its first
/// items and a `+N more` ending, until the whole text is under the cap.
pub fn text(atlas: &Atlas) -> String {
    let mut lists = [
        List::new("collections", collection_items(&atlas.collections)),
        List::new("streams", stream_items(&atlas.streams)),
    ];
    let values: Vec<String> = atlas
        .roots
        .iter()
        .map(|root| dashed(&root.path, &root.title))
        .collect();
    let roots = fitting_roots(atlas, &lists, &values);
    loop {
        let out = join(atlas, rows(atlas, &lists, &values, roots));
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

/// How many root lines the cap lets print: the most that fit, cutting from the end, or 0 when none does. Each line's length is
/// known once it is rendered, and the label column does not change with the count, so the cut is found by arithmetic on one
/// assembled text rather than by assembling it once per root (a tree with 10,000 roots made that quadratic).
fn fitting_roots(atlas: &Atlas, lists: &[List], values: &[String]) -> usize {
    let all = rows(atlas, lists, values, values.len());
    let width = all.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    let mut len = join_len(width, &all, atlas);
    if len <= CAP || values.is_empty() {
        return values.len();
    }
    let line = |value_len: usize| width + 2 + value_len + 1;
    for shown in (0..values.len()).rev() {
        len -= line(values[shown].len());
        let more = line(more_line(values.len() - shown).len());
        if len + more <= CAP {
            return shown;
        }
    }
    0
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

/// The rows of the text, each a label and its value, with each list at its current length and `roots` root lines shown: the graph
/// line, each list that has items, the roots block, the to and here lines, then the use line.
fn rows(atlas: &Atlas, lists: &[List], values: &[String], roots: usize) -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = vec![("graph".to_string(), intent_value(&atlas.intent))];
    for list in lists.iter().filter(|list| !list.items.is_empty()) {
        lines.push((list.label.to_string(), list.value()));
    }
    lines.extend(roots_lines(values, roots));
    if let Some(to) = &atlas.to {
        lines.push((format!("to {}", to.agent), to_value(to)));
    }
    if let Some(here) = &atlas.here {
        lines.push(("here".to_string(), here_value(here)));
    }
    lines.push(("use".to_string(), USE.to_string()));
    lines
}

/// The header line, then every row with its label padded to the longest label and its value two spaces on.
fn join(atlas: &Atlas, lines: Vec<(String, String)>) -> String {
    let width = lines
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let mut out = header(atlas);
    for (label, value) in lines {
        out.push_str(&format!("{label:<width$}  {value}\n"));
    }
    out
}

/// The length in bytes that `join` gives these rows, labels padded to `width` (labels are ASCII, so bytes and characters agree).
fn join_len(width: usize, lines: &[(String, String)], atlas: &Atlas) -> usize {
    header(atlas).len()
        + lines
            .iter()
            .map(|(label, value)| width.max(label.len()) + 2 + value.len() + 1)
            .sum::<usize>()
}

/// The first line of the atlas: the root, then the log store.
fn header(atlas: &Atlas) -> String {
    format!("memex  {} · log store {}\n", atlas.root, atlas.store_label)
}

/// The lines of the roots block (design §3.5): the first is labelled `roots`, the rest have no label and so are indented to its
/// column. Each shows a root folder; when some are cut, a last line says how many and where to see them.
fn roots_lines(values: &[String], shown: usize) -> Vec<(String, String)> {
    let mut lines: Vec<String> = values[..shown].to_vec();
    if shown < values.len() {
        lines.push(more_line(values.len() - shown));
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(index, value)| (if index == 0 { "roots" } else { "" }.to_string(), value))
        .collect()
}

/// The last line of a cut roots block: how many roots are left out and where to see them.
fn more_line(cut: usize) -> String {
    format!("+{cut} more → memex tree")
}

/// The graph line's value: the counts of READMEs and headers, in that order (design §3.5).
fn intent_value(intent: &Intent) -> String {
    [
        counted(intent.readmes, "readme", "readmes"),
        counted(intent.headers, "header", "headers"),
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
    dashed(&here.path, &here.title)
}

/// A folder's name, then ` — ` and its one-liner cut at 120 characters; the name alone when there is no one-liner.
fn dashed(path: &str, title: &str) -> String {
    if title.is_empty() {
        path.to_string()
    } else {
        format!("{path} — {}", clip(title))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::info::Root;

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

    /// The cut the one-at-a-time loop made before `fitting_roots`: the most roots, from the end down, whose text fits.
    fn cut_one_at_a_time(atlas: &Atlas, lists: &[List], values: &[String]) -> usize {
        (0..=values.len())
            .rev()
            .find(|&shown| join(atlas, rows(atlas, lists, values, shown)).len() <= CAP)
            .unwrap_or(0)
    }

    #[test]
    fn the_roots_cut_in_one_pass_matches_the_cut_one_at_a_time() {
        for (count, title_len, here) in [
            (0, 0, false),
            (3, 10, false),
            (40, 90, false),
            (45, 70, true),
            (300, 0, true),
            (2_000, 30, false),
        ] {
            let atlas = Atlas {
                root: "/repo".to_string(),
                store: None,
                store_label: "none".to_string(),
                intent: Intent::default(),
                collections: Vec::new(),
                streams: Vec::new(),
                roots: (0..count)
                    .map(|index| Root {
                        path: format!("dir{index:04}"),
                        title: "word ".repeat(title_len / 5 + index % 3),
                    })
                    .collect(),
                to: None,
                here: here.then(|| HereLine {
                    path: "dir0001".to_string(),
                    title: "Here.".to_string(),
                }),
                warnings: Vec::new(),
            };
            let lists = [List::new("streams", Vec::new())];
            let values: Vec<String> = atlas
                .roots
                .iter()
                .map(|root| dashed(&root.path, &root.title))
                .collect();
            assert_eq!(
                fitting_roots(&atlas, &lists, &values),
                cut_one_at_a_time(&atlas, &lists, &values),
                "{count} roots"
            );
        }
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
