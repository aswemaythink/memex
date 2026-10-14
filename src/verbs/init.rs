//! Carries out `memex init` (design §3.5b): the setup steps in order, then the atlas of the target.
//! Cases: INIT-FRESH-01, INIT-RERUN-01, INIT-SUB-01, INIT-SUB-02, INIT-INFO-01, INIT-INFO-02

mod agents;
mod files;
mod install;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::root::slash;
use crate::verbs::info;

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
        self.stderr.push_str(&format!("memex: warning: {text}\n"));
    }
}

/// Carries out `init` in the target folder, recording each action in `report`, and ends with the atlas of the target after a
/// blank line. The steps run in the order of design §3.5b. A failure leaves the actions before it in `report`.
pub fn run(report: &mut Report) -> Result<(), Error> {
    let target = target_dir()?;
    files::store(&target, report)?;
    agents::agents_md(&target, report)?;
    agents::claude_md(&target, report)?;
    install::run(&target, report)?;
    atlas(&target, report)
}

/// The folder init works in: the git toplevel of the cwd when the cwd is inside a worktree, else the cwd (design §3.5b).
fn target_dir() -> Result<PathBuf, Error> {
    let cwd = env::current_dir()
        .map_err(|failure| Error::io("cannot read the current folder", &failure))?;
    let cwd = dunce::canonicalize(&cwd)
        .map_err(|failure| Error::io("cannot resolve the current folder", &failure))?;
    Ok(git_toplevel(&cwd).unwrap_or(cwd))
}

/// The toplevel of the worktree that git reports for `cwd`, canonical. None outside a worktree, in a bare repo, or without git.
fn git_toplevel(cwd: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    dunce::canonicalize(text.lines().next()?).ok()
}

/// Prints the atlas of the target after a blank line. `info` reads the cwd, so the cwd becomes the target first.
fn atlas(target: &Path, report: &mut Report) -> Result<(), Error> {
    env::set_current_dir(target)
        .map_err(|failure| Error::io(&format!("cannot enter {}", slash(target)), &failure))?;
    let text = info::run(&info::Request {
        agent: None,
        json: false,
    })?;
    report.line("");
    report.stdout.push_str(&text);
    Ok(())
}
