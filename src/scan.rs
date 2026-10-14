//! Reads one verb's arguments against its row of the grammar: words, flags and their values, each checked as design §3 says.
//! Cases: CLI-*, CLI-ARGS-01, CLI-ARGS-02, CLI-ARGS-03, CLI-ARGS-04, CLI-ARGS-05, CLI-BARE-02, CLI-FLAG-01, CLI-FLAG-02, CLI-FLAG-03, CLI-FLAG-04, CLI-FLAG-05, CLI-FLAG-07, CLI-FLAG-08, CLI-FLAG-09, CLI-FLAG-10, CLI-FLAG-11, CLI-FLAG-12, CLI-FLAG-13, CLI-HELP-02, CLI-HELP-03, CLI-HELP-04, CLI-HELP-05, CLI-HELP-06, CLI-HELP-07, CLI-HELP-08

use std::iter::Peekable;
use std::slice::Iter;

use crate::cli::{Command, Verb};
use crate::error::Error;
use crate::grammar::{accepts, flag_entry, spec, split_inline, Takes};
use crate::help;

/// A flag's parsed value. Each flag appears at most once, so a flag's value is found by its name.
enum Value {
    Switch,
    Count(usize),
    List(Vec<String>),
    Text(String),
}

/// The flags given to one verb, in the order they came.
#[derive(Default)]
struct Flags(Vec<(&'static str, Value)>);

impl Flags {
    fn get(&self, flag: &str) -> Option<&Value> {
        self.0
            .iter()
            .find(|(name, _)| *name == flag)
            .map(|(_, value)| value)
    }

    fn has(&self, flag: &str) -> bool {
        self.get(flag).is_some()
    }

    fn count(&self, flag: &str) -> Option<usize> {
        match self.get(flag) {
            Some(Value::Count(count)) => Some(*count),
            _ => None,
        }
    }

    fn list(&self, flag: &str) -> Option<Vec<String>> {
        match self.get(flag) {
            Some(Value::List(items)) => Some(items.clone()),
            _ => None,
        }
    }

    fn text(&self, flag: &str) -> Option<String> {
        match self.get(flag) {
            Some(Value::Text(text)) => Some(text.clone()),
            _ => None,
        }
    }
}

/// A verb's flags and words, once they have passed its grammar.
#[derive(Default)]
struct Args {
    flags: Flags,
    words: Vec<String>,
}

/// The arguments after a verb, read one at a time.
type Rest<'a> = Peekable<Iter<'a, String>>;

/// Reads one verb's arguments. A `--help` among them ends the reading at once, so it wins over later errors.
pub(crate) fn parse_verb(verb: Verb, words: &[String]) -> Result<Command, Error> {
    let mut args = Args::default();
    let mut rest = words.iter().peekable();
    let mut ended = false;
    while let Some(word) = rest.next() {
        if ended || !word.starts_with('-') {
            add_word(verb, &mut args, word)?;
        } else if word.as_str() == "--" {
            ended = true;
        } else {
            let (name, inline) = split_inline(word);
            if name == "--help" && inline.is_none() {
                return Ok(Command::Help(Some(verb)));
            }
            add_flag(verb, &mut args, name, inline, &mut rest)?;
        }
    }
    check_words(verb, &args)?;
    Ok(command(verb, args))
}

/// Adds one word, unless the verb already has all the words it takes.
fn add_word(verb: Verb, args: &mut Args, word: &str) -> Result<(), Error> {
    if spec(verb).max.is_some_and(|max| args.words.len() >= max) {
        return Err(fail(verb, format!("unexpected argument '{word}'")));
    }
    args.words.push(word.to_string());
    Ok(())
}

/// Adds one flag and its value, after checking the verb takes it and it is not repeated.
fn add_flag(
    verb: Verb,
    args: &mut Args,
    name: &str,
    inline: Option<&str>,
    rest: &mut Rest<'_>,
) -> Result<(), Error> {
    let Some((flag, takes)) = flag_entry(name) else {
        return Err(fail(verb, format!("unknown flag '{name}'")));
    };
    if !accepts(verb, flag) {
        let message = format!("misplaced flag '{name}': not a flag of {}", verb.name());
        return Err(fail(verb, message));
    }
    if args.flags.has(flag) {
        return Err(fail(verb, format!("repeated flag '{name}'")));
    }
    let value = match (takes, inline) {
        (Takes::Nothing, Some(_)) => {
            return Err(fail(verb, format!("flag '{name}' takes no value")));
        }
        (Takes::Nothing, None) => Value::Switch,
        (_, Some(raw)) => parse_value(verb, flag, takes, raw)?,
        (_, None) => parse_value(verb, flag, takes, take_value(verb, name, rest)?)?,
    };
    args.flags.0.push((flag, value));
    Ok(())
}

/// The next word as a flag's value. A word that starts with `-` is never a value.
fn take_value<'a>(verb: Verb, name: &str, rest: &mut Rest<'a>) -> Result<&'a str, Error> {
    rest.next_if(|next| !next.starts_with('-'))
        .map(String::as_str)
        .ok_or_else(|| fail(verb, format!("flag '{name}' needs a value")))
}

/// Parses a flag's value by what the flag takes: a non-negative integer, a non-empty comma list, or a non-empty text.
fn parse_value(verb: Verb, flag: &'static str, takes: Takes, raw: &str) -> Result<Value, Error> {
    let bad = |why: &str| fail(verb, format!("bad value '{raw}' for '{flag}': {why}"));
    match takes {
        Takes::Nothing => Ok(Value::Switch),
        Takes::Count if !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_digit()) => raw
            .parse::<usize>()
            .map(Value::Count)
            .map_err(|_| bad("expected a non-negative integer")),
        Takes::Count => Err(bad("expected a non-negative integer")),
        Takes::List if raw.split(',').all(|item| !item.is_empty()) => {
            Ok(Value::List(raw.split(',').map(str::to_string).collect()))
        }
        Takes::List => Err(bad("expected comma-separated names, none empty")),
        Takes::Text if !raw.is_empty() => Ok(Value::Text(raw.to_string())),
        Takes::Text => Err(bad("expected a non-empty value")),
    }
}

/// Checks the words against the verb's row: the minimum, and the bare `search` that design §3.2 refuses.
fn check_words(verb: Verb, args: &Args) -> Result<(), Error> {
    let row = spec(verb);
    if args.words.len() < row.min {
        return Err(fail(verb, row.missing.to_string()));
    }
    if verb == Verb::Search && args.words.is_empty() && !args.flags.has("--in") {
        let message = "search needs words or filters; run memex info for an overview";
        return Err(fail(verb, message.to_string()));
    }
    Ok(())
}

/// Builds the typed Command for a verb whose arguments have passed the grammar.
fn command(verb: Verb, args: Args) -> Command {
    let Args { flags, words } = args;
    let json = flags.has("--json");
    match verb {
        Verb::Search => Command::Search {
            terms: words,
            in_: flags.list("--in"),
            limit: flags.count("--limit"),
            json,
        },
        Verb::Show => Command::Show {
            refs: words,
            up: flags.count("--up"),
            limit: flags.count("--limit"),
            json,
        },
        Verb::Log => Command::Log {
            stream: words.into_iter().next().unwrap_or_default(),
            to: flags.list("--to").unwrap_or_default(),
            tag: flags.list("--tag").unwrap_or_default(),
            agent: flags.text("--agent"),
            json,
        },
        Verb::Info => Command::Info {
            agent: flags.text("--agent"),
            json,
        },
        Verb::Init => Command::Init,
    }
}

/// A usage error for one verb: its message, then its grammar line.
pub(crate) fn fail(verb: Verb, message: String) -> Error {
    Error::usage(message, vec![format!("usage: {}", help::usage_line(verb))])
}
