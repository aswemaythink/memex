//! Splits text into the tokens that the index and the query share: words, ids and paths.
//! Cases: SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-TOKEN-03

/// True when a term is a whole id or path, such as `name-001` or `crates/x/hash.rs`: it holds a character that is not a letter or a
/// digit. Every other term is a plain word, and only plain words are stemmed or matched by prefix or by one edit.
pub fn is_whole(term: &str) -> bool {
    term.bytes().any(|byte| !byte.is_ascii_alphanumeric())
}

/// Hands each token of a doc's text to `emit`, in order: for each run of letters, digits, dashes, dots and slashes, the whole run
/// when it is an id or a path, then its lowercase words. `NAME-001` gives `name-001`, `name` and `001`. A token is valid only
/// during its call, since a capital is lowercased in a buffer that the next token reuses.
pub fn for_each_token(text: &str, mut emit: impl FnMut(&str)) {
    visit(text, true, &mut emit);
}

/// The tokens of a query term. A whole id or path stands alone, without its words, so that `NAME-001` matches that id and
/// no other. Any other run gives its lowercase words, as `for_each_token` does.
pub fn query_tokens(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    visit(text, false, &mut |token: &str| {
        found.push(token.to_string())
    });
    found
}

/// Hands each token of a text to `emit`, lowercased. `with_words` says whether a whole id or path also gives its words.
fn visit(text: &str, with_words: bool, emit: &mut dyn FnMut(&str)) {
    let mut lower = String::new();
    for run in text.split(|ch: char| !is_run_char(ch)) {
        let run = run.trim_matches(|ch: char| matches!(ch, '-' | '/' | '.'));
        if is_id_or_path(run) {
            emit_lowercase(run, &mut lower, emit);
            if !with_words {
                continue;
            }
        }
        for word in words(run) {
            emit_lowercase(word, &mut lower, emit);
        }
    }
}

/// Hands `token` to `emit` in lowercase: as it is when it has no capital, else as a lowercase copy made in `lower`.
fn emit_lowercase(token: &str, lower: &mut String, emit: &mut dyn FnMut(&str)) {
    if token.bytes().any(|byte| byte.is_ascii_uppercase()) {
        lower.clear();
        lower.extend(token.chars().map(|ch| ch.to_ascii_lowercase()));
        emit(lower.as_str());
    } else {
        emit(token);
    }
}

/// The characters that can make up a run: letters, digits, and the dash, dot and slash of ids and paths.
fn is_run_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '/' | '.')
}

/// True for a run that names an id or a path: it has a dash or a slash, or a dot followed by a letter, as in `hash.rs`.
fn is_id_or_path(run: &str) -> bool {
    run.contains(['-', '/'])
        || run
            .as_bytes()
            .windows(2)
            .any(|pair| pair[0] == b'.' && pair[1].is_ascii_alphabetic())
}

/// The words of a run, as written: its maximal runs of letters and digits.
fn words(run: &str) -> impl Iterator<Item = &str> + '_ {
    run.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tokens of a text, as strings.
    fn tokens(text: &str) -> Vec<String> {
        let mut found = Vec::new();
        for_each_token(text, |token| found.push(token.to_string()));
        found
    }

    #[test]
    fn an_id_is_a_whole_token_and_its_words_too() {
        assert_eq!(tokens("NAME-001"), ["name-001", "name", "001"]);
        assert_eq!(query_tokens("NAME-001"), ["name-001"]);
    }

    #[test]
    fn a_path_is_a_whole_token_and_a_sentence_dot_is_not_part_of_it() {
        assert_eq!(query_tokens("src/hash.rs."), ["src/hash.rs"]);
        assert_eq!(tokens("see hash.rs."), ["see", "hash.rs", "hash", "rs"]);
    }

    #[test]
    fn version_numbers_and_colons_split_into_plain_words() {
        assert_eq!(query_tokens("std::io"), ["std", "io"]);
        assert_eq!(query_tokens("v1.2"), ["v1", "2"]);
    }
}
