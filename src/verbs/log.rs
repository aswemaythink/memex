//! Carries out `memex log` (design §3.4 and §3.6).
//! It checks the stream, the flag values and the body, writes the entry to the log store, and returns the ref, or the path as JSON.
//! The place and HEAD come from one `root::locate_for_write`, so HEAD's branch rides on git's probe call (design §4.4 git).
//! Cases: LOG-WRITE-01, LOG-WRITE-02, LOG-WRITE-03, LOG-WRITE-04, LOG-WRITE-05, LOG-WRITE-06, LOG-WRITE-07, LOG-WRITE-08, LOG-WRITE-09, LOG-WRITE-10, LOG-WRITE-11, LOG-WRITE-12, LOG-WRITE-13, LOG-WRITE-15, LOG-BODY-01, LOG-BODY-02, LOG-BODY-03, LOG-BODY-04, LOG-BODY-05, LOG-BODY-06, LOG-FRONT-01, LOG-FRONT-02, LOG-FRONT-03, LOG-FRONT-04, LOG-FRONT-05, LOG-FRONT-06, LOG-STREAM-01, LOG-STREAM-02, LOG-STREAM-03, LOG-STREAM-04, LOG-STREAM-05, LOG-FLAG-01, LOG-FLAG-02, LOG-FLAG-03, LOG-FLAG-04, LOG-FLAG-05, LOG-FLAG-06, DISC-ROOT-01, DISC-ROOT-02, DISC-ROOT-03, DISC-ROOT-04, DISC-NOGIT-01, DISC-NOGIT-03

use std::env;
use std::path::Path;

use crate::clock::Timestamp;
use crate::config;
use crate::error::{self, escape, Error};
use crate::logs::write::{self, Entry};
use crate::logs::{body, front, prune, stream, values};
use crate::root::{self, slash};

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
/// Every refusal comes before anything is written. Warnings about the config go to stderr as they are found.
pub fn run(request: &Request, now: Timestamp) -> Result<String, Error> {
    stream::check_grammar(&request.stream)?;
    stream::check_reserved(&request.stream)?;
    let to = values::list("--to", &request.to)?;
    let tags = values::list("--tag", &request.tag)?;
    let agent = values::agent(request.agent.as_deref(), values::env_agent().as_deref())?;
    let text = body::read_stdin()?;
    let split = front::split(&text)?;
    body::check_title(&split.rest)?;
    let cwd = env::current_dir()
        .map_err(|failure| Error::io("cannot read the current folder", &failure))?;
    let (place, head) = root::locate_for_write(&cwd)?;
    stream::check_collections(&request.stream, &cwd, &place.owner)?;
    let config = config::load(&place.owner);
    error::warn(&config.warnings);
    let entry = Entry {
        stream: request.stream.clone(),
        time: now,
        agent,
        to,
        tags,
        branch: head.branch,
        worktree: place.worktree.as_deref().map(root::slash),
        commit: head.commit,
        front: split.lines,
        body: split.rest,
    };
    let path = write::write_entry(&place, &entry)?;
    // Pruning follows the write, so the entry just written is counted and never deleted (design §4.3 Cleanup).
    if config.keep.is_some() || config.cap.is_some() {
        prune::run(&place.store, config.keep, config.cap, now, &path);
    }
    Ok(output(&request.stream, &path, request.json))
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
