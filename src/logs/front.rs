//! Splits an agent's own frontmatter off the top of a body and checks its keys (design §3.4 Extra fields and §4.2 flat YAML): a memex key or a repeated key is refused, and every other line is kept as the agent wrote it.
//! Cases: LOG-FRONT-01, LOG-FRONT-02, LOG-FRONT-03, LOG-FRONT-04, LOG-FRONT-05, LOG-FRONT-06

use crate::cli::Verb;
use crate::error::Error;
use crate::scan::fail;

/// The keys memex writes itself (design §3.4 and §4.3). An agent block may not set them.
const MEMEX_KEYS: [&str; 8] = [
    "stream", "time", "agent", "to", "tags", "branch", "worktree", "commit",
];

/// The line that opens and closes a frontmatter block.
const FENCE: &str = "---";

/// A body split at its agent frontmatter.
#[derive(Debug, PartialEq, Eq)]
pub struct Split {
    /// The block's lines without their line ends, in the order written. Empty when the body has no block.
    pub lines: Vec<String>,
    /// The body after the closing fence, verbatim. The whole body when there is no block.
    pub rest: String,
}

/// Splits `body` when its first line is exactly `---`: the block runs to the next line that is exactly `---`. A block
/// that never closes, a memex key, or a key that repeats, is a usage error.
pub fn split(body: &str) -> Result<Split, Error> {
    let mut pieces = body.split_inclusive('\n');
    let first = pieces.next().unwrap_or("");
    if fence_text(first) != FENCE {
        return Ok(Split {
            lines: Vec::new(),
            rest: body.to_string(),
        });
    }
    let mut offset = first.len();
    let mut lines = Vec::new();
    for piece in pieces {
        offset += piece.len();
        if fence_text(piece) == FENCE {
            check_keys(&lines)?;
            return Ok(Split {
                lines,
                rest: body[offset..].to_string(),
            });
        }
        lines.push(fence_text(piece).to_string());
    }
    Err(fail(
        Verb::Log,
        "frontmatter block is not closed".to_string(),
    ))
}

/// A line's text without its line end: one `\n`, and a `\r` before it.
fn fence_text(piece: &str) -> &str {
    let line = piece.strip_suffix('\n').unwrap_or(piece);
    line.strip_suffix('\r').unwrap_or(line)
}

/// Refuses a memex key, or a key that repeats, among the block's lines. Only the key lines are checked.
fn check_keys(lines: &[String]) -> Result<(), Error> {
    let mut seen: Vec<&str> = Vec::new();
    for key in lines.iter().filter_map(|line| key_of(line)) {
        if MEMEX_KEYS.contains(&key) {
            let message = format!("frontmatter key '{key}' is set by memex");
            return Err(fail(Verb::Log, message));
        }
        if seen.contains(&key) {
            let message = format!("frontmatter key '{key}' is repeated");
            return Err(fail(Verb::Log, message));
        }
        seen.push(key);
    }
    Ok(())
}

/// The key a line sets: an identifier at column 0 with a colon straight after it (design §4.2). Any other line is a continuation.
fn key_of(line: &str) -> Option<&str> {
    let (name, _) = line.split_once(':')?;
    let mut chars = name.chars();
    let opens = matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic() || ch == '_');
    let rest_ok = chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    (opens && rest_ok).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leading_block_is_split_off_and_the_rest_kept() {
        let split = split("---\r\nstatus: x\r\n---\n# T\n").unwrap();
        assert_eq!(split.lines, vec!["status: x".to_string()]);
        assert_eq!(split.rest, "# T\n");
    }

    #[test]
    fn a_body_without_a_leading_fence_is_all_rest() {
        let split = split("# T\n---\nx\n").unwrap();
        assert!(split.lines.is_empty());
        assert_eq!(split.rest, "# T\n---\nx\n");
    }

    #[test]
    fn a_block_may_close_on_the_last_line() {
        let split = split("---\nstatus: x\n---").unwrap();
        assert_eq!(split.rest, "");
    }

    #[test]
    fn keys_are_column_zero_identifiers() {
        assert_eq!(key_of("status: x"), Some("status"));
        assert_eq!(key_of("my-key_2: x"), Some("my-key_2"));
        assert_eq!(key_of("  time: 1"), None);
        assert_eq!(key_of("- item: x"), None);
        assert_eq!(key_of("no colon"), None);
    }
}
