//! Splits text into the tokens that the index and the query share: words, ids and paths.
//! Cases: SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-TOKEN-03, SRCH-UNICODE-01, SRCH-UNICODE-02, SRCH-UNICODE-03, SRCH-UNICODE-04

/// True when a term is a whole id or path, such as `name-001` or `crates/x/hash.rs`: it holds a character that is not a letter or a
/// digit. Every other term is a plain word, and only plain words are stemmed or matched by prefix or by one edit. A whole word
/// matches only whole terms: itself, and the longer ones it begins at `-`, `/` or `.` (design §3.2).
pub fn is_whole(term: &str) -> bool {
    term.chars().any(|ch| !is_word_char(ch))
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

/// Hands `token` to `emit` lowercased, as `str::to_lowercase` lowercases it. A token that no character changes is handed over as
/// it is. Any other is lowercased into `lower`, which is reused, one character at a time; a token with a capital sigma goes
/// through `str::to_lowercase` instead, since the final sigma depends on the rest of the word.
fn emit_lowercase(token: &str, lower: &mut String, emit: &mut dyn FnMut(&str)) {
    if token.chars().all(|ch| ch.to_lowercase().eq([ch])) {
        emit(token);
    } else {
        lower.clear();
        if token.contains('Σ') {
            lower.push_str(&token.to_lowercase());
        } else {
            lower.extend(token.chars().flat_map(char::to_lowercase));
        }
        emit(lower.as_str());
    }
}

/// The characters that can make up a run: the characters of words, and the dash, dot and slash of ids and paths.
fn is_run_char(ch: char) -> bool {
    is_word_char(ch) || matches!(ch, '-' | '/' | '.')
}

/// A character of a word: a letter or a digit of any script (design §3.2). It is `char::is_alphanumeric`, so the other numbers,
/// such as `½`, count as well.
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric()
}

/// True for a run that names an id or a path: it has a dash or a slash, or a dot followed by a letter, as in `hash.rs`.
fn is_id_or_path(run: &str) -> bool {
    run.contains(['-', '/'])
        || run
            .split('.')
            .skip(1)
            .any(|after| after.starts_with(char::is_alphabetic))
}

/// The words of a run, as written: its maximal runs of letters and digits.
fn words(run: &str) -> impl Iterator<Item = &str> + '_ {
    run.split(|ch: char| !is_word_char(ch))
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

    #[test]
    fn a_word_keeps_its_unicode_letters_and_a_cjk_run_is_one_word() {
        assert_eq!(tokens("naïve"), ["naïve"]);
        assert_eq!(tokens("東京タワー"), ["東京タワー"]);
        assert!(!is_whole("naïve"));
        assert!(is_whole("naïve-001"));
    }

    #[test]
    fn capitals_lowercase_as_str_to_lowercase_gives_them() {
        assert_eq!(tokens("CAFÉ STRAẞE"), ["café", "straße"]);
        assert_eq!(tokens("STRASSE"), ["strasse"]);
        assert_eq!(tokens("ΟΔΟΣ"), ["ΟΔΟΣ".to_lowercase()]);
    }
}
