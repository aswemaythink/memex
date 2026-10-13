//! Writes one log entry to the store (design §3.4 and §4.3): memex's frontmatter with the agent's lines merged into it, the body verbatim, a file name from the title's slug, and never an overwrite.
//! Cases: LOG-WRITE-01, LOG-WRITE-02, LOG-WRITE-03, LOG-WRITE-04, LOG-WRITE-05, LOG-WRITE-06, LOG-WRITE-08, LOG-FRONT-01, LOG-BODY-02

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::clock::Timestamp;
use crate::error::Error;
use crate::root::{slash, Place};

/// The `.gitignore` that goes beside a new store, so logs can never be committed by accident (design §4.1 item 3).
const GITIGNORE: &[u8] = b"logs/\ncache/\n";

/// The longest slug, in characters (design §3.4).
const SLUG_MAX: usize = 60;

/// One log entry as the verb gathered it. Each optional field is written only when it is known.
pub struct Entry {
    /// The stream folder the entry goes to, already checked against the stream grammar.
    pub stream: String,
    /// The instant of the entry, from the clock.
    pub time: Timestamp,
    /// The agent id, from `--agent` or `$MEMEX_AGENT`.
    pub agent: Option<String>,
    /// The `--to` ids, in the order given.
    pub to: Vec<String>,
    /// The `--tag` names, in the order given.
    pub tags: Vec<String>,
    /// The branch of HEAD, when it has one.
    pub branch: Option<String>,
    /// The toplevel of a linked worktree, `/`-separated.
    pub worktree: Option<String>,
    /// The short commit of HEAD, when there is one.
    pub commit: Option<String>,
    /// The agent's own frontmatter lines, verbatim, written after memex's keys inside the same block.
    pub front: Vec<String>,
    /// The body after any agent frontmatter, with CRLF already normalised to LF.
    pub body: String,
}

/// Writes `entry` into the place's store and returns the path of the new file. The store is created on
/// first use, and a new store gets its `.gitignore`.
pub fn write_entry(place: &Place, entry: &Entry) -> Result<PathBuf, Error> {
    prepare_store(place)?;
    let dir = place.store.join(&entry.stream);
    fs::create_dir_all(&dir).map_err(|failure| io_error("cannot create", &dir, &failure))?;
    let stem = format!("{}-{}", entry.time.stamp(), slug_of(&title_of(&entry.body)));
    let text = render(entry);
    let mut suffix = 1;
    loop {
        let name = match suffix {
            1 => format!("{stem}.md"),
            _ => format!("{stem}-{suffix}.md"),
        };
        if let Some(path) = write_new(&dir, &name, text.as_bytes())? {
            return Ok(path);
        }
        suffix += 1;
    }
}

/// The title of a body: its first non-blank line, with leading `#`s and spaces removed. Empty when there is no such line.
/// The body is the text after any agent frontmatter, so the block is never read as a title.
pub fn title_of(body: &str) -> String {
    body.lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim_start_matches(['#', ' ']).to_string())
        .unwrap_or_default()
}

/// The slug of a title: its ASCII letters and digits in lowercase, each run of other characters as one
/// dash, trimmed of dashes and at most 60 characters, or `entry` when nothing is left.
pub fn slug_of(title: &str) -> String {
    let mut dashed = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            dashed.push(ch.to_ascii_lowercase());
        } else if !dashed.ends_with('-') {
            dashed.push('-');
        }
    }
    let kept: String = dashed.trim_matches('-').chars().take(SLUG_MAX).collect();
    let slug = kept.trim_end_matches('-');
    if slug.is_empty() {
        "entry".to_string()
    } else {
        slug.to_string()
    }
}

/// The file's text: memex's frontmatter in its fixed key order, then the agent's lines in their own order, then the body
/// verbatim (design §4.3).
fn render(entry: &Entry) -> String {
    let mut lines = vec![
        "---".to_string(),
        format!("stream: {}", entry.stream),
        format!("time: {}", entry.time.rfc3339()),
    ];
    if let Some(agent) = &entry.agent {
        lines.push(format!("agent: {agent}"));
    }
    if !entry.to.is_empty() {
        lines.push(format!("to: [{}]", entry.to.join(", ")));
    }
    if !entry.tags.is_empty() {
        lines.push(format!("tags: [{}]", entry.tags.join(", ")));
    }
    if let Some(branch) = &entry.branch {
        lines.push(format!("branch: {branch}"));
    }
    if let Some(worktree) = &entry.worktree {
        lines.push(format!("worktree: {worktree}"));
    }
    if let Some(commit) = &entry.commit {
        lines.push(format!("commit: {commit}"));
    }
    lines.extend(entry.front.iter().cloned());
    lines.push("---".to_string());
    let mut text = lines.join("\n");
    text.push('\n');
    text.push_str(&entry.body);
    text
}

/// Creates the store on first use. When the store folder is new, its `.gitignore` is written beside it,
/// unless the store lives in a bare repo's git dir. An existing `.gitignore` is never touched.
fn prepare_store(place: &Place) -> Result<(), Error> {
    if place.store.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(&place.store)
        .map_err(|failure| io_error("cannot create", &place.store, &failure))?;
    match place.store.parent() {
        Some(memex_dir) if !place.in_git_dir => {
            write_new(memex_dir, ".gitignore", GITIGNORE).map(|_| ())
        }
        _ => Ok(()),
    }
}

/// Writes `bytes` to `dir/name` through a dot-prefixed temporary file, so a reader never sees a partial
/// entry. Returns the path, or None when the name is already taken, in which case nothing is written.
fn write_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<Option<PathBuf>, Error> {
    let target = dir.join(name);
    let mut temp = tempfile::Builder::new()
        .prefix(".")
        .tempfile_in(dir)
        .map_err(|failure| io_error("cannot create a temporary file in", dir, &failure))?;
    temp.write_all(bytes)
        .map_err(|failure| io_error("cannot write", &target, &failure))?;
    match temp.persist_noclobber(&target) {
        Ok(_) => Ok(Some(target)),
        Err(failed) if failed.error.kind() == io::ErrorKind::AlreadyExists => Ok(None),
        Err(failed) => Err(io_error("cannot write", &target, &failed.error)),
    }
}

/// An I/O error that names what failed and the path, never the OS text (design §4.4).
fn io_error(what: &str, path: &Path, failure: &io::Error) -> Error {
    Error::io(&format!("{what} {}", slash(path)), failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_follow_the_design_rules() {
        assert_eq!(slug_of("Show walks hub READMEs!"), "show-walks-hub-readmes");
        assert_eq!(slug_of("  --  "), "entry");
        assert_eq!(slug_of("Café au lait"), "caf-au-lait");
        let long = slug_of(&"ab-".repeat(40));
        assert!(long.len() <= SLUG_MAX);
        assert!(!long.ends_with('-'));
    }

    #[test]
    fn titles_skip_blank_lines_and_leading_marks() {
        assert_eq!(title_of("## Heading\nbody"), "Heading");
        assert_eq!(title_of("\n  \n# Real\n"), "Real");
        assert_eq!(title_of("#\n"), "");
        assert_eq!(title_of(" \n\t\n"), "");
    }
}
