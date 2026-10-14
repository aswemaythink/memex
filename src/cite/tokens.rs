//! Splits a line of a doc's text into the tokens that citations are made of (design §2.3).
//! Cases: CITE-BARE-01, CITE-BARE-02, CITE-PREFIX-01, CITE-COLL-01, CITE-COLL-03, CITE-PATH-04, CITE-LINK-01, CITE-LINK-02, CITE-LINK-04

use crate::refs::is_id_body;

/// One token of a line. Its kind fixes how it resolves, in the precedence of design §2.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A markdown link destination that is not a URL or an anchor, with any `#fragment` removed.
    Link(String),
    /// A `collection:id` whose collection is known, as its label.
    Label(String),
    /// A family prefix: the `NAME-` of a `NAME-*` token.
    Prefix(String),
    /// A run of path characters, with trailing dots removed. It may be a bare id or a path.
    Word(String),
}

/// The tokens of one line, in order. `known` says whether a word names a collection, which makes `word:id` a label.
pub fn tokens(line: &str, known: &dyn Fn(&str) -> bool) -> Vec<Token> {
    let bytes = line.as_bytes();
    let last_close = line.rfind(')');
    let mut found = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        at = if bytes[at] == b']' && bytes.get(at + 1) == Some(&b'(') {
            link(line, at, last_close, &mut found)
        } else if is_run(bytes[at]) {
            word(line, at, known, &mut found)
        } else {
            at + 1
        };
    }
    found
}

/// True for a family prefix such as `NAME-` or `X-`: an id body followed by one dash (design §2.3).
fn is_family_prefix(text: &str) -> bool {
    text.strip_suffix('-').is_some_and(is_id_body)
}

/// The markdown link whose `](` is at `at`: its destination runs to the next `)`. Returns where scanning resumes.
/// `last_close` is the line's last `)`. With none after the `](`, there is no link and nothing is searched for, so a
/// line of many `](` takes time linear in its length.
fn link(line: &str, at: usize, last_close: Option<usize>, found: &mut Vec<Token>) -> usize {
    let open = at + 2;
    let gap = match last_close {
        Some(close) if close >= open => line[open..].find(')'),
        _ => None,
    };
    let Some(gap) = gap else {
        return at + 1;
    };
    if let Some(token) = link_token(&line[open..open + gap]) {
        found.push(token);
    }
    open + gap + 1
}

/// The token of a link's inside, or None for a URL, an in-page anchor or an empty target. A destination in `<...>` is read
/// to its `>`, and a title after a space is dropped.
fn link_token(inside: &str) -> Option<Token> {
    let inside = inside.trim();
    let dest = match inside.strip_prefix('<') {
        Some(rest) => rest.split('>').next()?,
        None => inside.split_whitespace().next()?,
    };
    if dest.starts_with('#') || dest.starts_with("//") || has_scheme(dest) {
        return None;
    }
    let path = dest.split_once('#').map_or(dest, |(path, _)| path);
    (!path.is_empty()).then(|| Token::Link(path.to_string()))
}

/// True for a destination that opens with a URL scheme, such as `https:` or `mailto:`.
fn has_scheme(dest: &str) -> bool {
    dest.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte))
    })
}

/// The word, label, prefix or URL that starts at `start`. Returns where scanning resumes.
fn word(line: &str, start: usize, known: &dyn Fn(&str) -> bool, found: &mut Vec<Token>) -> usize {
    let end = start + run_len(&line.as_bytes()[start..]);
    if line[end..].starts_with("://") {
        return line[end..]
            .find(char::is_whitespace)
            .map_or(line.len(), |gap| end + gap);
    }
    let run = &line[start..end];
    if let Some((token, next)) = label_or_prefix(line, run, end, known) {
        found.push(token);
        return next;
    }
    let word = run.trim_end_matches('.');
    if !word.is_empty() {
        found.push(Token::Word(word.to_string()));
    }
    end
}

/// The `collection:id` label, when `run` is a known collection, or the `NAME-*` prefix, when `run` ends in a dash and a `*`
/// follows it. `end` is the byte after the run. Returns the token and where scanning resumes after it.
fn label_or_prefix(
    line: &str,
    run: &str,
    end: usize,
    known: &dyn Fn(&str) -> bool,
) -> Option<(Token, usize)> {
    let next = *line.as_bytes().get(end)?;
    match next {
        b':' if known(run) => {
            let id_start = end + 1;
            let id_len = run_len(&line.as_bytes()[id_start..]);
            let id = line[id_start..id_start + id_len].trim_end_matches('.');
            (!id.is_empty()).then(|| (Token::Label(format!("{run}:{id}")), id_start + id.len()))
        }
        b'*' if is_family_prefix(run) => Some((Token::Prefix(run.to_string()), end + 1)),
        _ => None,
    }
}

/// True for the bytes that a run is made of: ASCII letters and digits, and `._/@+-` (design §2.3).
fn is_run(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"._/@+-".contains(&byte)
}

/// The number of leading bytes of `bytes` that are run bytes.
fn run_len(bytes: &[u8]) -> usize {
    bytes.iter().take_while(|byte| is_run(**byte)).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn words(line: &str) -> Vec<Token> {
        tokens(line, &|name: &str| name == "cases")
    }

    #[test]
    fn a_trailing_dot_is_not_part_of_a_word() {
        assert_eq!(
            words("see NAME-001."),
            vec![
                Token::Word("see".to_string()),
                Token::Word("NAME-001".to_string())
            ]
        );
    }

    #[test]
    fn a_known_collection_makes_a_label_and_a_star_makes_a_prefix() {
        assert_eq!(
            words("cases:NAME-001 and NAME-*"),
            vec![
                Token::Label("cases:NAME-001".to_string()),
                Token::Word("and".to_string()),
                Token::Prefix("NAME-".to_string())
            ]
        );
    }

    #[test]
    fn a_url_and_an_anchor_are_not_links() {
        let links = |line: &str| {
            words(line)
                .into_iter()
                .filter(|token| matches!(token, Token::Link(_)))
                .count()
        };
        assert_eq!(links("[a](#top)"), 0);
        assert_eq!(links("[a](https://x/y)"), 0);
        assert_eq!(links("[a](src/cli.rs#top)"), 1);
        assert_eq!(words("https://src/cli.rs"), Vec::new());
    }

    #[test]
    fn a_bracket_paren_run_links_only_to_the_next_paren() {
        assert_eq!(
            words("a](b](c)"),
            vec![
                Token::Word("a".to_string()),
                Token::Link("b](c".to_string())
            ]
        );
        assert_eq!(
            words("[x](y)"),
            vec![Token::Word("x".to_string()), Token::Link("y".to_string())]
        );
        assert_eq!(words("](](]("), Vec::new());
        assert_eq!(words("x) ](](]("), vec![Token::Word("x".to_string())]);
        assert_eq!(words("](](]()"), vec![Token::Link("](](".to_string())]);
    }

    /// The tokens of the scan before `link` took `last_close`: each `](` searches for a `)` to the end of the line.
    fn tokens_searching_to_end(line: &str, known: &dyn Fn(&str) -> bool) -> Vec<Token> {
        let bytes = line.as_bytes();
        let mut found = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            at = if bytes[at] == b']' && bytes.get(at + 1) == Some(&b'(') {
                let open = at + 2;
                match line[open..].find(')') {
                    Some(gap) => {
                        found.extend(link_token(&line[open..open + gap]));
                        open + gap + 1
                    }
                    None => at + 1,
                }
            } else if is_run(bytes[at]) {
                word(line, at, known, &mut found)
            } else {
                at + 1
            };
        }
        found
    }

    /// Every line of the files under tests/cases/cite, where the CITE-* fixtures live.
    fn fixture_lines() -> Vec<String> {
        fn walk(dir: &Path, lines: &mut Vec<String>) {
            for entry in fs::read_dir(dir).expect("the cite fixtures") {
                let path = entry.expect("a fixture entry").path();
                if path.is_dir() {
                    walk(&path, lines);
                } else if let Ok(text) = fs::read_to_string(&path) {
                    lines.extend(text.lines().map(str::to_string));
                }
            }
        }
        let mut lines = Vec::new();
        walk(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/cite"),
            &mut lines,
        );
        lines
    }

    #[test]
    fn the_tokens_are_those_of_the_scan_that_searched_to_the_line_end() {
        let known = |name: &str| name == "cases";
        let mut lines = fixture_lines();
        lines.extend(
            [
                "a](b](c)",
                "[x](y)",
                "](](](",
                "x) ](](](",
                "](](](]()",
                "](a)",
            ]
            .map(str::to_string),
        );
        assert!(lines.len() > 100, "the cite fixtures were not found");
        for line in &lines {
            assert_eq!(
                tokens(line, &known),
                tokens_searching_to_end(line, &known),
                "{line:?}"
            );
        }
    }
}
