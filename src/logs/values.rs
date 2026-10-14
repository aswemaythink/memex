//! Checks the values of `--to`, `--tag` and the agent id (design §3.4).
//! Each value matches one grammar, and a list drops its repeats, keeping the first place of each.
//! Cases: LOG-FLAG-01, LOG-FLAG-02, LOG-FLAG-03, LOG-FLAG-04, LOG-FLAG-05, LOG-FLAG-06

use crate::cli::Verb;
use crate::error::Error;
use crate::scan::fail;

/// The longest value, in bytes. The grammar is ASCII, so bytes are characters.
const VALUE_MAX: usize = 64;

/// The bytes a value may hold besides ASCII letters and digits.
const SYMBOLS: &[u8] = b"._@/+-";

/// The items of one list, each checked, with repeats dropped and each item kept where it first appears.
/// `flag` names the list, such as `--to`, in the error.
pub fn list(flag: &str, items: &[String]) -> Result<Vec<String>, Error> {
    let mut kept: Vec<String> = Vec::new();
    for item in items {
        check(flag, item)?;
        if !kept.contains(item) {
            kept.push(item.clone());
        }
    }
    Ok(kept)
}

/// The agent id: `--agent` when it is given, else `$MEMEX_AGENT` when it is set. Only the value that is used is checked.
pub fn agent(flag: Option<&str>, env: Option<&str>) -> Result<Option<String>, Error> {
    let (name, value) = match (flag, env) {
        (Some(value), _) => ("--agent", value),
        (None, Some(value)) => ("MEMEX_AGENT", value),
        (None, None) => return Ok(None),
    };
    check(name, value)?;
    Ok(Some(value.to_string()))
}

/// Refuses a value that does not match `[A-Za-z0-9._@/+-]{1,64}`. `name` is the flag or the variable that carried it.
fn check(name: &str, value: &str) -> Result<(), Error> {
    let fits = (1..=VALUE_MAX).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || SYMBOLS.contains(&byte));
    if fits {
        return Ok(());
    }
    let message = format!("invalid {name} value '{value}'");
    Err(fail(Verb::Log, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_are_dropped_in_first_order() {
        let items = vec!["b".to_string(), "a".to_string(), "b".to_string()];
        assert_eq!(list("--to", &items).unwrap(), vec!["b", "a"]);
    }

    #[test]
    fn the_grammar_holds_at_its_edges() {
        assert!(check("--to", &"a".repeat(64)).is_ok());
        assert!(check("--to", &"a".repeat(65)).is_err());
        assert!(check("--to", "Az09._@/+-").is_ok());
        assert!(check("--to", "a b").is_err());
        assert!(check("--to", "é").is_err());
    }

    #[test]
    fn only_the_agent_in_use_is_checked() {
        assert_eq!(
            agent(Some("cook-2"), Some("bad:one")).unwrap(),
            Some("cook-2".to_string())
        );
        let env = agent(None, Some("bad:one")).unwrap_err();
        assert_eq!(env.message, "invalid MEMEX_AGENT value 'bad:one'");
        assert_eq!(agent(None, None).unwrap(), None);
    }
}
