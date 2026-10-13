//! Carries out `memex log` (design §3.4 and §3.6): reads the body from stdin, writes the entry to the log store and returns the ref, or the path as JSON.
//! Cases: LOG-WRITE-01, LOG-WRITE-02, LOG-WRITE-03, LOG-WRITE-04, LOG-WRITE-05, LOG-WRITE-06, LOG-WRITE-07, LOG-WRITE-08, LOG-WRITE-09, LOG-WRITE-10, LOG-WRITE-11, LOG-WRITE-12, LOG-WRITE-13, DISC-ROOT-01, DISC-ROOT-02, DISC-ROOT-03, DISC-ROOT-04

use std::env;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::cli::Verb;
use crate::clock;
use crate::config;
use crate::error::{escape, Error};
use crate::logs::prune;
use crate::logs::write::{self, Entry};
use crate::root::{self, slash};
use crate::scan::fail;

/// The arguments of `memex log` once the grammar has passed them.
pub struct Request {
    /// The stream the entry goes to.
    pub stream: String,
    /// The `--to` ids.
    pub to: Vec<String>,
    /// The `--tag` names.
    pub tag: Vec<String>,
    /// The `--agent` id, if the flag was given.
    pub agent: Option<String>,
    /// True when `--json` was given.
    pub json: bool,
}

/// Writes the entry and returns what stdout should carry: `<stream>:<id>` on one line, or `{"ref","path"}` with `--json`.
/// Warnings about the config go to stderr as they are found. Errors are returned for the caller to report.
pub fn run(request: &Request) -> Result<String, Error> {
    check_stream(&request.stream)?;
    let body = read_body()?;
    let now = clock::now()?;
    let cwd = env::current_dir()
        .map_err(|failure| Error::io("cannot read the current folder", &failure))?;
    let place = root::locate(&cwd)?;
    let config = config::load(&place.owner);
    if let Some(line) = &config.warning {
        warn(line);
    }
    let head = root::head(&cwd);
    let entry = Entry {
        stream: request.stream.clone(),
        time: now,
        agent: request.agent.clone().or_else(env_agent),
        to: request.to.clone(),
        tags: request.tag.clone(),
        branch: head.branch,
        worktree: place.worktree.as_deref().map(root::slash),
        commit: head.commit,
        body,
    };
    let path = write::write_entry(&place, &entry)?;
    prune::run(&place.store, &config.keep_logs, now)?;
    Ok(output(&request.stream, &path, request.json))
}

/// A stream name is one to forty lowercase letters, digits or dashes, starting with a letter or digit (design §3.4).
/// Anything else would make the entry's folder land outside the store, so it is a usage error.
fn check_stream(stream: &str) -> Result<(), Error> {
    let mut chars = stream.chars();
    let first_ok =
        matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase() || ch.is_ascii_digit());
    let rest_ok = chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
    if first_ok && rest_ok && stream.len() <= 40 {
        return Ok(());
    }
    let message = format!(
        "invalid stream name '{stream}': use up to 40 lowercase letters, digits and dashes, starting with a letter or digit"
    );
    Err(fail(Verb::Log, message))
}

/// Reads all of stdin as the body. CRLF becomes LF, and a body that is blank is a usage error.
fn read_body() -> Result<String, Error> {
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .read_to_end(&mut bytes)
        .map_err(|failure| Error::io("cannot read the log body", &failure))?;
    let body = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
    if body.trim().is_empty() {
        return Err(fail(Verb::Log, "log body is empty".to_string()));
    }
    Ok(body)
}

/// The agent id from `$MEMEX_AGENT`. An empty value counts as unset.
fn env_agent() -> Option<String> {
    env::var_os("MEMEX_AGENT")
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
}

/// The stdout text of a successful log: the ref line, or the JSON object with `--json`.
fn output(stream: &str, path: &Path, json: bool) -> String {
    let id = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let reference = format!("{stream}:{id}");
    if json {
        let path = slash(path);
        format!(
            "{{\"ref\":\"{}\",\"path\":\"{}\"}}\n",
            escape(&reference),
            escape(&path)
        )
    } else {
        format!("{reference}\n")
    }
}

/// Prints one warning line on stderr. A failed write is ignored, as there is nowhere left to report it.
fn warn(line: &str) {
    let _ = writeln!(io::stderr().lock(), "{line}");
}
