//! Writes one log entry to the store (design §3.4 and §4.3).
//! memex's frontmatter with the agent's lines merged into it, the body verbatim, a file name from the title's slug, and never an overwrite. The store itself may be a symlink, which is followed; a symlinked stream folder is refused. A stream folder that a concurrent prune removes is made again (design §4.3 Vanished entries).
//! Cases: LOG-WRITE-01, LOG-WRITE-02, LOG-WRITE-03, LOG-WRITE-04, LOG-WRITE-05, LOG-WRITE-06, LOG-WRITE-08, LOG-WRITE-14, LOG-WRITE-15, LOG-FRONT-01, LOG-BODY-02, DISC-STORE-01, DISC-STORE-02, PORT-MODE-01

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

use crate::atomic;
use crate::clock::Timestamp;
use crate::error::Error;
use crate::root::{slash, store_listed, Place};

/// The `.gitignore` that goes beside a new store, so logs can never be committed by accident (design §4.1 items 3 and 4). init
/// writes the same bytes into a new `.memex/`.
pub const GITIGNORE: &[u8] = b"logs/\ncache/\n";

/// The longest slug, in characters (design §3.4).
const SLUG_MAX: usize = 60;

/// The most attempts when a concurrent prune removes a stream folder under a write (design §4.3 Vanished entries).
const ATTEMPTS: u32 = 3;

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
    make_folder(&dir)?;
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
fn slug_of(title: &str) -> String {
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
    if store_exists(&place.store)? {
        return Ok(());
    }
    fs::create_dir_all(&place.store)
        .map_err(|failure| Error::io_at("cannot create", &place.store, &failure))?;
    match place.store.parent() {
        Some(memex_dir) if !place.in_git_dir => {
            write_new(memex_dir, ".gitignore", GITIGNORE).map(|_| ())
        }
        _ => Ok(()),
    }
}

/// True when the store is listed under its parent by its exact name and is a folder. A store listed as a file is not one, so
/// the create that follows reports it as it always has (design §4.4 Name matching).
fn store_exists(store: &Path) -> Result<bool, Error> {
    let listed =
        store_listed(store).map_err(|failure| Error::io_at("cannot read", store, &failure))?;
    Ok(listed && store.is_dir())
}

/// Creates `dir` and everything above it, and refuses a `dir` that is a symlink: the store may be a link, but no folder
/// below it is followed (the rule prune keeps too). A folder a concurrent prune removes after the create is made again (design §4.3).
fn make_folder(dir: &Path) -> Result<(), Error> {
    let mut attempt = 1;
    loop {
        fs::create_dir_all(dir).map_err(|failure| Error::io_at("cannot create", dir, &failure))?;
        let failure = match fs::symlink_metadata(dir) {
            Ok(meta) if meta.is_dir() => return Ok(()),
            Ok(_) => {
                return Err(Error::io_message(format!(
                    "cannot write to {}: it is a symlink, and memex follows none inside the store",
                    slash(dir)
                )))
            }
            Err(failure) => failure,
        };
        if failure.kind() != io::ErrorKind::NotFound || attempt == ATTEMPTS {
            return Err(Error::io_at("cannot read", dir, &failure));
        }
        attempt += 1;
    }
}

/// Writes `bytes` to `dir/name` through a dot-prefixed temporary file, so a reader never sees a partial
/// entry. Returns the path, or None when the name is already taken, in which case nothing is written.
fn write_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<Option<PathBuf>, Error> {
    let target = dir.join(name);
    let mut temp = temp_in(dir)?;
    temp.write_all(bytes)
        .map_err(|failure| Error::io_at("cannot write", &target, &failure))?;
    match temp.persist_noclobber(&target) {
        Ok(_) => Ok(Some(target)),
        Err(failed) if taken(&target, &failed.error) => Ok(None),
        Err(failed) => Err(Error::io_at("cannot write", &target, &failed.error)),
    }
}

/// True when a failed persist means the name is taken: the error says so, or something exists at the name in any form,
/// which `symlink_metadata` sees without following it. Windows reports a folder there as a different error (design §4.4).
fn taken(target: &Path, failure: &io::Error) -> bool {
    failure.kind() == io::ErrorKind::AlreadyExists || fs::symlink_metadata(target).is_ok()
}

/// A dot-prefixed temporary file in `dir`, made with the mode a new file gets, so an entry is an ordinary file (design §4.4 File
/// modes). A concurrent prune can remove the stream folder under the create, so `temp_with` retries it.
fn temp_in(dir: &Path) -> Result<NamedTempFile, Error> {
    temp_with(dir, |dir| atomic::temp_in(dir, None))
}

/// Makes the temporary file with `create`. While `dir` is not listed under its parent, any failure makes the folder again and
/// tries once more (macOS says InvalidInput mid-removal), three attempts in all (design §4.3 Vanished entries). A listed folder
/// is not made again, so its failure is returned.
fn temp_with(
    dir: &Path,
    create: impl Fn(&Path) -> io::Result<NamedTempFile>,
) -> Result<NamedTempFile, Error> {
    let mut attempt = 1;
    loop {
        let failure = match create(dir) {
            Ok(temp) => return Ok(temp),
            Err(failure) => failure,
        };
        if attempt == ATTEMPTS || listed(dir) {
            return Err(Error::io_at(
                "cannot create a temporary file in",
                dir,
                &failure,
            ));
        }
        make_folder(dir)?;
        attempt += 1;
    }
}

/// True when `dir` is still listed under its parent by its exact name, which `store_listed` checks for any folder (design §4.4
/// Name matching). A failed listing is not one.
fn listed(dir: &Path) -> bool {
    store_listed(dir).unwrap_or(false)
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
    fn a_stream_folder_removed_before_the_write_is_made_again() {
        let store = tempfile::tempdir().expect("a temporary store");
        let dir = store.path().join("impl");
        make_folder(&dir).expect("a stream folder");
        // What a concurrent prune does when it empties the stream: the folder is gone before the temp file.
        fs::remove_dir(&dir).expect("the folder removed");
        let written = write_new(&dir, "20261013-120000-x.md", b"x\n").expect("a write");
        assert_eq!(written, Some(dir.join("20261013-120000-x.md")));
        assert_eq!(
            fs::read(dir.join("20261013-120000-x.md")).expect("the entry"),
            b"x\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_stream_folder_is_refused_and_a_symlinked_store_is_followed() {
        let root = tempfile::tempdir().expect("a temporary folder");
        let elsewhere = root.path().join("elsewhere");
        fs::create_dir(&elsewhere).expect("a target folder");
        let store = root.path().join("store");
        std::os::unix::fs::symlink(&elsewhere, &store).expect("a symlinked store");
        make_folder(&store.join("impl")).expect("a stream under a symlinked store");
        assert!(elsewhere.join("impl").is_dir());
        std::os::unix::fs::symlink(&elsewhere, store.join("alias")).expect("a symlinked stream");
        assert!(make_folder(&store.join("alias")).is_err());
    }

    #[test]
    fn a_folder_at_the_target_name_takes_it_and_the_entry_is_written_as_2() {
        let root = tempfile::tempdir().expect("a temporary folder");
        let store = root.path().join(".memex").join("logs");
        let place = Place {
            owner: root.path().to_path_buf(),
            store: store.clone(),
            in_git_dir: false,
            worktree: None,
        };
        let dir = store.join("impl");
        let folder = dir.join("20261013-120000-x.md");
        fs::create_dir_all(&folder).expect("a folder at the target name");
        let entry = Entry {
            stream: "impl".to_string(),
            time: crate::clock::parse_stamp("20261013-120000").expect("a stamp"),
            agent: None,
            to: Vec::new(),
            tags: Vec::new(),
            branch: None,
            worktree: None,
            commit: None,
            front: Vec::new(),
            body: "# X\n".to_string(),
        };
        let path = write_entry(&place, &entry).expect("an entry");
        assert_eq!(path, dir.join("20261013-120000-x-2.md"));
        assert!(folder.is_dir(), "the folder is left as it was");
    }

    #[cfg(unix)]
    #[test]
    fn an_entry_gets_0644_before_the_umask_and_not_the_private_one() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().expect("a temporary folder");
        let dir = root.path().join("impl");
        make_folder(&dir).expect("a stream folder");
        let entry = write_new(&dir, "20261013-120000-x.md", b"x\n")
            .expect("a write")
            .expect("the entry");
        fs::write(root.path().join("plain"), b"x\n").expect("a plain file");
        let mode = |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        // A file that File::create makes gets 0o666 less the umask, which recovers the umask of this run.
        let umask = 0o666 & !mode(&root.path().join("plain"));
        assert_eq!(mode(&entry), 0o644 & !umask);
    }

    #[test]
    fn titles_skip_blank_lines_and_leading_marks() {
        assert_eq!(title_of("## Heading\nbody"), "Heading");
        assert_eq!(title_of("\n  \n# Real\n"), "Real");
        assert_eq!(title_of("#\n"), "");
        assert_eq!(title_of(" \n\t\n"), "");
    }

    #[test]
    fn a_create_that_fails_while_the_folder_is_gone_is_tried_three_times() {
        let store = tempfile::tempdir().expect("a temporary store");
        let dir = store.path().join("impl");
        let calls = std::cell::Cell::new(0);
        // Each attempt finds the folder gone, as a prune that empties it leaves it.
        let gone = |dir: &Path| -> io::Result<NamedTempFile> {
            calls.set(calls.get() + 1);
            let _ = fs::remove_dir(dir);
            Err(io::Error::from(io::ErrorKind::InvalidInput))
        };
        assert!(temp_with(&dir, gone).is_err());
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn a_create_that_fails_while_the_folder_is_listed_is_not_retried() {
        let store = tempfile::tempdir().expect("a temporary store");
        let dir = store.path().join("impl");
        make_folder(&dir).expect("a stream folder");
        let calls = std::cell::Cell::new(0);
        let refused = |_: &Path| -> io::Result<NamedTempFile> {
            calls.set(calls.get() + 1);
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        };
        assert!(temp_with(&dir, refused).is_err());
        assert_eq!(calls.get(), 1);
    }
}
