//! Tells whether two plain words differ by exactly one edit, the fuzzy match of PLAN §4.
//! Cases: SRCH-FUZZY-01, SRCH-FUZZY-02

/// True when `a` and `b` differ by exactly one insertion, deletion or substitution. Equal words differ by none, so they are not
/// one edit apart. The check is linear in the length of the words.
pub fn one_edit(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if long.len() - short.len() > 1 || a == b {
        return false;
    }
    let mut at = 0;
    while at < short.len() && short[at] == long[at] {
        at += 1;
    }
    if long.len() == short.len() {
        short[at + 1..] == long[at + 1..]
    } else {
        short[at..] == long[at + 1..]
    }
}

#[cfg(test)]
mod tests {
    use super::one_edit;

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
}
