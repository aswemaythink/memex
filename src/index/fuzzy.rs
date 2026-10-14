//! Tells whether two plain words differ by exactly one edit, the fuzzy match of PLAN §4.
//! Cases: SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-UNICODE-01, SRCH-UNICODE-02

/// True when `a` and `b` differ by exactly one insertion, deletion or substitution of a character. Equal words differ by none, so
/// they are not one edit apart. Characters count, not bytes: `naïve` is one substitution from `naive`. The check is linear in the
/// length of the words, and it copies neither of them.
pub fn one_edit(a: &str, b: &str) -> bool {
    let (a, b) = without_common_ends(a, b);
    matches!((capped_len(a), capped_len(b)), (1, 1) | (0, 1) | (1, 0))
}

/// `a` and `b` without the characters they share at their start, and then at their end. That keeps the edit distance. What is left
/// starts with different characters and ends with different characters, unless one side is empty, so one edit leaves at most one
/// character on each side.
fn without_common_ends<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    let head = common_bytes(a.chars().zip(b.chars()));
    let (a, b) = (&a[head..], &b[head..]);
    let tail = common_bytes(a.chars().rev().zip(b.chars().rev()));
    (&a[..a.len() - tail], &b[..b.len() - tail])
}

/// The bytes of the pairs of characters that are equal, counted from the first pair until the first pair that differs.
fn common_bytes(pairs: impl Iterator<Item = (char, char)>) -> usize {
    pairs
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum()
}

/// The number of characters in `word`, but no more than 2: the one-edit cases of `one_edit` leave at most one character on a side.
fn capped_len(word: &str) -> usize {
    word.chars().take(2).count()
}

#[cfg(test)]
mod tests {
    use super::one_edit;

    /// The Levenshtein distance of two words in characters: the reference that `one_edit` must agree with.
    fn distance(a: &str, b: &str) -> usize {
        let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
        let mut row: Vec<usize> = (0..=b.len()).collect();
        for (i, x) in a.iter().enumerate() {
            let mut diagonal = row[0];
            row[0] = i + 1;
            for (j, y) in b.iter().enumerate() {
                let above = row[j + 1];
                row[j + 1] = (diagonal + usize::from(x != y))
                    .min(above + 1)
                    .min(row[j] + 1);
                diagonal = above;
            }
        }
        row[b.len()]
    }

    #[test]
    fn one_edit_agrees_with_the_edit_distance_on_every_short_word() {
        let alphabet = ['a', 'b', 'ï'];
        let mut words = vec![String::new()];
        let mut frontier = vec![String::new()];
        for _ in 0..3 {
            frontier = frontier
                .iter()
                .flat_map(|word| alphabet.iter().map(move |ch| format!("{word}{ch}")))
                .collect();
            words.extend(frontier.iter().cloned());
        }
        for a in &words {
            for b in &words {
                assert_eq!(one_edit(a, b), distance(a, b) == 1, "{a:?} against {b:?}");
            }
        }
    }

    #[test]
    fn one_substitution_insertion_or_deletion_is_one_edit() {
        assert!(one_edit("hashs", "hashx"));
        assert!(one_edit("hash", "hashe"));
        assert!(one_edit("hashe", "hash"));
        assert!(one_edit("hasch", "hash"));
        assert!(one_edit("", "a"));
    }

    #[test]
    fn equal_words_and_two_edits_are_not_one_edit() {
        assert!(!one_edit("hash", "hash"));
        assert!(!one_edit("hash", "hahs"));
        assert!(!one_edit("hash", "hashes"));
        assert!(!one_edit("hashs", "hasx"));
    }

    #[test]
    fn characters_are_counted_not_bytes() {
        assert!(one_edit("naïve", "naive"));
        assert!(one_edit("café", "cafe"));
        assert!(one_edit("naïve", "nave"));
        assert!(!one_edit("straße", "strasse"));
    }
}
