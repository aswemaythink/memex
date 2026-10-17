//! Finds the `$NAME` placeholders a golden may not hold (rule 5).
//! Bless uses it too, so it never writes such a golden.
//! Cases: all (runner)

/// The only placeholders a golden may hold.
const ALLOWED: [&str; 2] = ["$SANDBOX", "$VERSION"];

/// Every `$NAME` token in a golden other than `$SANDBOX` and `$VERSION`, with its 1-based line.
///
/// A token is `$` followed by `[A-Z_]`, then the run of `[A-Za-z0-9_]`. The `$ ` prompt of a
/// command line is not a token.
pub fn placeholder_problems(bytes: &[u8]) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for (index, line) in bytes.split(|b| *b == b'\n').enumerate() {
        for token in tokens(line) {
            if !ALLOWED.contains(&token.as_str()) {
                found.push((token, index + 1));
            }
        }
    }
    found
}

/// The `$NAME` tokens of one line, in order.
fn tokens(line: &[u8]) -> Vec<String> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < line.len() {
        let starts_token = line[at] == b'$'
            && line
                .get(at + 1)
                .is_some_and(|b| b.is_ascii_uppercase() || *b == b'_');
        if !starts_token {
            at += 1;
            continue;
        }
        let mut end = at + 1;
        while end < line.len() && (line[end].is_ascii_alphanumeric() || line[end] == b'_') {
            end += 1;
        }
        found.push(String::from_utf8_lossy(&line[at..end]).into_owned());
        at = end;
    }
    found
}
