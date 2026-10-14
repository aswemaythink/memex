//! Splits text into the tokens that the index and the query share: words, ids and paths.
//! Cases: SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-TOKEN-03

/// True when a term is a whole id or path, such as `name-001` or `crates/x/hash.rs`: it holds a character that is not a letter or a
/// digit. Every other term is a plain word, and only plain words are stemmed or matched by prefix or by one edit.
pub fn is_whole(term: &str) -> bool {
    term.bytes().any(|byte| !byte.is_ascii_alphanumeric())
}

/// The tokens of a doc's text: for each run of letters, digits, dashes, dots and slashes, the whole run when it is an id or a
/// path, then its lowercase words. `NAME-001` gives `name-001`, `name` and `001`.
pub fn tokens(text: &str) -> Vec<String> {
    runs(text, true)
}

/// The tokens of a query term. A whole id or path stands alone, without its words, so that `NAME-001` matches that id and
/// no other. Any other run gives its lowercase words, as `tokens` does.
pub fn query_tokens(text: &str) -> Vec<String> {
    runs(text, false)
}

/// The tokens of each run of a text. `with_words` says whether a whole id or path also gives its words.
fn runs(text: &str, with_words: bool) -> Vec<String> {
    let mut found = Vec::new();
    for run in text.split(|ch: char| !is_run_char(ch)) {
        let run = run.trim_matches(|ch: char| matches!(ch, '-' | '/' | '.'));
        if is_id_or_path(run) {
            found.push(run.to_ascii_lowercase());
            if !with_words {
                continue;
            }
        }
        found.extend(words(run));
    }
    found
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

/// The lowercase words of a run: its maximal runs of letters and digits.
fn words(run: &str) -> impl Iterator<Item = String> + '_ {
    run.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;

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
