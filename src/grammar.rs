//! The tables of design §3's grammar.
//! Which flags each verb takes, what each flag takes after it, and how many words each verb takes.
//! Cases: CLI-*, CLI-HINTB-01, AUD-EXIT-02, AUD-EXIT-03

use crate::cli::Verb;

/// What a memex flag takes after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Takes {
    Nothing,
    Count,
    List,
    Text,
}

/// Every flag memex knows. A verb accepts the ones its grammar line names, and `--help` after any verb.
pub(crate) const FLAGS: [(&str, Takes); 10] = [
    ("--help", Takes::Nothing),
    ("--version", Takes::Nothing),
    ("--in", Takes::List),
    ("--limit", Takes::Count),
    ("--up", Takes::Count),
    ("--comments", Takes::Nothing),
    ("--json", Takes::Nothing),
    ("--to", Takes::List),
    ("--tag", Takes::List),
    ("--agent", Takes::Text),
];

/// A verb's row in SPECS: the flags its grammar line names, and how many words it takes.
pub(crate) struct Spec {
    pub(crate) flags: &'static [&'static str],
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    /// The message when fewer than `min` words come.
    pub(crate) missing: &'static str,
}

/// One row per verb, in `Verb::ALL` order, so `Verb as usize` indexes it.
pub(crate) const SPECS: [Spec; 6] = [
    Spec {
        flags: &["--in", "--limit", "--json"],
        min: 0,
        max: None,
        missing: "",
    },
    Spec {
        flags: &["--up", "--comments", "--limit", "--json"],
        min: 1,
        max: None,
        missing: "show needs at least one path or ref",
    },
    Spec {
        flags: &["--to", "--tag", "--agent", "--json"],
        min: 1,
        max: Some(1),
        missing: "log needs a stream name: memex log <stream> <<'EOF' … EOF",
    },
    Spec {
        flags: &["--agent", "--json"],
        min: 0,
        max: Some(0),
        missing: "",
    },
    Spec {
        flags: &["--limit", "--json"],
        min: 0,
        max: None,
        missing: "",
    },
    Spec {
        flags: &[],
        min: 0,
        max: Some(0),
        missing: "",
    },
];

/// The row of a verb in SPECS.
pub(crate) fn spec(verb: Verb) -> &'static Spec {
    &SPECS[verb as usize]
}

/// The name and what it takes, for a flag memex knows.
pub(crate) fn flag_entry(name: &str) -> Option<(&'static str, Takes)> {
    FLAGS.into_iter().find(|(flag, _)| *flag == name)
}

/// True when the verb's grammar line names the flag, or the flag is `--help`.
pub(crate) fn accepts(verb: Verb, flag: &'static str) -> bool {
    flag == "--help" || spec(verb).flags.contains(&flag)
}

/// True when the word is a flag that takes a value, given as the next word.
pub(crate) fn takes_value(word: &str) -> bool {
    let (name, inline) = split_inline(word);
    inline.is_none() && flag_entry(name).is_some_and(|(_, takes)| takes != Takes::Nothing)
}

/// Splits `--name=value` at its first equals sign. A word without one has no inline value.
pub(crate) fn split_inline(word: &str) -> (&str, Option<&str>) {
    match word.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (word, None),
    }
}
