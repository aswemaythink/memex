//! Parses the argv grammar of design §3 into a typed Command.
//! Or into the usage error the grammar calls for.
//! Cases: CLI-*, CLI-BARE-01, CLI-VERB-01, CLI-VERB-02, CLI-VERB-03, CLI-FLAG-06, CLI-HELP-01, CLI-JSON-03, CLI-JSON-04, CLI-HINT-10, CLI-HINT-11

use std::ffi::OsString;

use crate::error::Error;
use crate::grammar::{flag_entry, split_inline, takes_value};
use crate::help;
use crate::scan::{fail, parse_verb};

/// The five verbs of design §3, in the order of the grammar block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    /// `memex search`.
    Search,
    /// `memex show`.
    Show,
    /// `memex log`.
    Log,
    /// `memex info`.
    Info,
    /// `memex init`.
    Init,
}

impl Verb {
    /// Every verb, in grammar order.
    pub const ALL: [Verb; 5] = [Verb::Search, Verb::Show, Verb::Log, Verb::Info, Verb::Init];

    /// The word that names this verb on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Verb::Search => "search",
            Verb::Show => "show",
            Verb::Log => "log",
            Verb::Info => "info",
            Verb::Init => "init",
        }
    }

    /// The verb a word names, if one does.
    pub fn from_name(word: &str) -> Option<Verb> {
        Verb::ALL.into_iter().find(|verb| verb.name() == word)
    }
}

/// A parsed invocation: a verb with its words and flags, or help, or the version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `memex search`, with its words and flags.
    Search {
        /// The words to search for.
        terms: Vec<String>,
        /// The `--in` values, split on commas.
        in_: Option<Vec<String>>,
        /// The value of `--limit`, if the flag was given.
        limit: Option<usize>,
        /// True when `--json` was given.
        json: bool,
    },
    /// `memex show`, with its refs and flags.
    Show {
        /// The refs to show, in order.
        refs: Vec<String>,
        /// The value of `--up`, if the flag was given.
        up: Option<usize>,
        /// The value of `--limit`, if the flag was given.
        limit: Option<usize>,
        /// True when `--json` was given.
        json: bool,
    },
    /// `memex log`, with its stream and flags.
    Log {
        /// The stream the entry goes to, such as `impl`.
        stream: String,
        /// The `--to` values, split on commas.
        to: Vec<String>,
        /// The `--tag` values, split on commas.
        tag: Vec<String>,
        /// The value of `--agent`, if the flag was given.
        agent: Option<String>,
        /// True when `--json` was given.
        json: bool,
    },
    /// `memex info`, with its flags.
    Info {
        /// The value of `--agent`, if the flag was given.
        agent: Option<String>,
        /// True when `--json` was given.
        json: bool,
    },
    /// `memex init`.
    Init,
    /// `--help`: the top-level help for None, otherwise one verb's help.
    Help(Option<Verb>),
    /// `--version`: the crate version.
    Version,
}

/// Parses the arguments after the program name. A usage error comes back as Err, for the caller to print.
pub fn parse(args: &[OsString]) -> Result<Command, Error> {
    let words = words_of(args);
    let leading_dashes = words.first().is_some_and(|word| word.as_str() == "--");
    let start = usize::from(leading_dashes);
    let Some(word) = words.get(start) else {
        return Err(Error::usage(String::new(), help::usage_block()));
    };
    if !leading_dashes && word.starts_with('-') {
        return top_flag(word, &words);
    }
    match Verb::from_name(word) {
        Some(verb) => parse_verb(verb, &words[start + 1..]),
        None => Err(unknown_verb(word)),
    }
}

/// True when `--json` comes before the first `--`. Then usage errors are JSON (design §3).
pub fn json_requested(args: &[OsString]) -> bool {
    words_of(args)
        .iter()
        .take_while(|word| word.as_str() != "--")
        .any(|word| word.as_str() == "--json" || word.starts_with("--json="))
}

/// A flag before the verb. `--help` and `--version` are the only ones allowed there.
fn top_flag(word: &str, words: &[String]) -> Result<Command, Error> {
    let (name, inline) = split_inline(word);
    match (name, inline) {
        ("--help", None) => Ok(Command::Help(None)),
        ("--version", None) => Ok(Command::Version),
        ("--help" | "--version", Some(_)) => {
            Err(at_top(format!("flag '{name}' takes no value"), words))
        }
        _ if flag_entry(name).is_some() => Err(at_top(
            format!("misplaced flag '{name}': flags go after the verb"),
            words,
        )),
        _ => Err(at_top(format!("unknown flag '{name}'"), words)),
    }
}

/// An error found before the verb. Its usage line is the verb's when a verb follows, else the whole block.
fn at_top(message: String, words: &[String]) -> Error {
    match verb_candidate(words) {
        Some(word) => match Verb::from_name(word) {
            Some(verb) => fail(verb, message),
            None => unknown_verb(word),
        },
        None => Error::usage(message, help::usage_block()),
    }
}

/// The first word that is neither a flag nor a flag's value, read by the same rules as `parse`.
fn verb_candidate(words: &[String]) -> Option<&str> {
    let mut rest = words.iter().peekable();
    let mut ended = false;
    while let Some(word) = rest.next() {
        if ended || !word.starts_with('-') {
            return Some(word.as_str());
        }
        if word.as_str() == "--" {
            ended = true;
        } else if takes_value(word) {
            rest.next_if(|next| !next.starts_with('-'));
        }
    }
    None
}

/// A usage error for a word that names no verb: the message names the next step (design §3.5c), then the whole grammar block.
fn unknown_verb(word: &str) -> Error {
    Error::usage(
        format!("unknown verb '{word}'; run memex --help"),
        help::usage_block(),
    )
}

/// The arguments as text. Non-UTF-8 bytes become replacement characters.
fn words_of(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn equals_form_gives_the_value() {
        let parsed = parse(&argv(&["search", "words", "--limit=5"]));
        let expected = Command::Search {
            terms: vec!["words".to_string()],
            in_: None,
            limit: Some(5),
            json: false,
        };
        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn lists_split_on_commas() {
        let parsed = parse(&argv(&["log", "impl", "--to", "a,b", "--tag", "x"]));
        let expected = Command::Log {
            stream: "impl".to_string(),
            to: vec!["a".to_string(), "b".to_string()],
            tag: vec!["x".to_string()],
            agent: None,
            json: false,
        };
        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn roster_needs_in_and_no_words() {
        let parsed = parse(&argv(&["search", "--in", "cases"]));
        let expected = Command::Search {
            terms: Vec::new(),
            in_: Some(vec!["cases".to_string()]),
            limit: None,
            json: false,
        };
        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn dashdash_makes_a_flag_a_word() {
        let parsed = parse(&argv(&["show", "--", "--up"]));
        let expected = Command::Show {
            refs: vec!["--up".to_string()],
            up: None,
            limit: None,
            json: false,
        };
        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn help_and_version_parse() {
        assert_eq!(
            parse(&argv(&["search", "--help"])),
            Ok(Command::Help(Some(Verb::Search)))
        );
        assert_eq!(parse(&argv(&["--version"])), Ok(Command::Version));
    }

    #[test]
    fn json_counts_only_before_dashdash() {
        assert!(json_requested(&argv(&["search", "--json"])));
        assert!(!json_requested(&argv(&["init", "--", "--json"])));
    }
}
