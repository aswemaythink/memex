//! Carries out `memex init` (design §3.5b): the setup steps in order, then the atlas of the target.
//! Cases: INIT-FRESH-01, INIT-RERUN-01, INIT-SUB-01, INIT-SUB-02, INIT-INFO-01, INIT-INFO-02, INIT-INFO-03, INIT-NEST-01, INIT-NEST-02

mod agents;
mod files;
mod install;

use std::path::PathBuf;

use crate::clock::Timestamp;
use crate::error::{self, Error};
use crate::root;
use crate::verbs::{here, info};

/// What init prints, kept as it goes, so that a failure still shows the actions taken before it.
#[derive(Debug, Default)]
pub struct Report {
    /// The stdout: one line per action, then a blank line and the atlas of the target.
    pub stdout: String,
    /// The stderr: one line per warning.
    pub stderr: String,
}

/// What init did to one file or to the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    /// The file did not exist and was written.
    Created,
    /// The file was left as it was.
    Kept,
    /// The file existed and was rewritten.
    Updated,
}

impl Action {
    /// The word that starts the action's line.
    fn word(self) -> &'static str {
        match self {
            Action::Created => "created",
            Action::Kept => "kept",
            Action::Updated => "updated",
        }
    }
}

impl Report {
    /// Records one action on a file, as `created <label>`, `kept <label>` or `updated <label>`.
    fn action(&mut self, action: Action, label: &str) {
        self.line(&format!("{} {label}", action.word()));
    }

    /// Records a plain line on stdout, such as the PATH hint.
    fn line(&mut self, text: &str) {
        self.stdout.push_str(text);
        self.stdout.push('\n');
    }

    /// Records a warning on stderr, in memex's `memex: warning:` form.
    fn warn(&mut self, text: &str) {
        self.stderr.push_str(&error::warning(text));
        self.stderr.push('\n');
    }
}

/// Carries out `init` in the target folder, recording each action in `report`, and ends with the atlas of the target after a
/// blank line. The steps run in the order of design §3.5b. A failure leaves the actions before it in `report`.
pub fn run(report: &mut Report, now: Timestamp) -> Result<(), Error> {
    let target = target_dir()?;
    files::store(&target, report)?;
    agents::agents_md(&target, report)?;
    agents::claude_md(&target, report)?;
    install::run(&target, report)?;
    atlas(target, now, report)
}

/// The folder init works in (design §3.5b): the git toplevel of the cwd when the cwd is inside a worktree. Outside git it is the
/// root that discovery finds from the cwd, the nearest folder above it with a `.memex/`, so that init never creates a second store
/// that shadows one above it (INIT-NEST-01), else the cwd.
fn target_dir() -> Result<PathBuf, Error> {
    let cwd = here::current_dir()?;
    if let Some(top) = root::toplevel_of(&cwd) {
        return Ok(top);
    }
    Ok(root::nearest_memex(&cwd).unwrap_or(cwd))
}

/// Prints the atlas of the target after a blank line. It is read from the target as given, so the process cwd never changes,
/// and it ignores `$MEMEX_AGENT` (INIT-INFO-03).
fn atlas(target: PathBuf, now: Timestamp, report: &mut Report) -> Result<(), Error> {
    let text = info::text_in(target, now)?;
    report.line("");
    report.stdout.push_str(&text);
    Ok(())
}
