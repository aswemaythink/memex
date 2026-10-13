//! Builds a case's sandbox (fixture copy, git setup, scrubbed environment) and runs its steps in it.
//! Cases: all (runner)

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::env::{base_env, set, with_git_identity, Env};
use super::transcript::StepRecord;
use super::{CaseSpec, GitMode, SetupStep, Step};

/// A case's sandbox: a fresh temp dir with repo/ and home/. It is deleted when dropped.
pub struct Sandbox {
    _dir: tempfile::TempDir,
    pub root: PathBuf,
    bin: PathBuf,
    /// The defaults plus the case's env: the environment of every step.
    case_env: Env,
}

impl Sandbox {
    /// Creates the sandbox, copies the fixture, makes repo/ a git repo if the case says so, and runs the setup steps.
    pub fn create(case_dir: &Path, spec: &CaseSpec, bin: &Path) -> Result<Sandbox, String> {
        let dir = tempfile::Builder::new()
            .prefix("memex-case-")
            .tempdir()
            .map_err(|e| io_failure("cannot create the sandbox", &e))?;
        let root = dir.path().to_path_buf();
        reject_memex_ancestor(&root)?;
        let fixture = case_dir.join("fixture");
        if fixture.is_dir() {
            copy_fixture(&fixture, &root)?;
        }
        let repo = root.join("repo");
        let home = root.join("home");
        fs::create_dir_all(&repo).map_err(|e| io_failure("cannot create repo/", &e))?;
        fs::create_dir_all(&home).map_err(|e| io_failure("cannot create home/", &e))?;
        fs::write(home.join(".gitconfig"), b"")
            .map_err(|e| io_failure("cannot write the empty git config", &e))?;

        let base = base_env(&root, bin, spec.memex_on_path)?;
        if let Some(mode) = spec.git {
            git_fixture(&repo, &base, mode == GitMode::Commit)?;
        }
        // Setup runs also get the fixed git identity and dates, so a setup commit has a stable hash.
        let setup_env = with_overrides(with_git_identity(&base), &spec.env);
        for (index, setup) in spec.setup.iter().enumerate() {
            run_setup(index + 1, setup, &repo, bin, &setup_env)?;
        }
        Ok(Sandbox {
            _dir: dir,
            root,
            bin: bin.to_path_buf(),
            case_env: with_overrides(base, &spec.env),
        })
    }

    /// Runs one step: memex with its args, in repo/ unless it names a cwd, with its stdin and env.
    pub fn run_step(&self, step: &Step, case_dir: &Path) -> Result<StepRecord, String> {
        let cwd = step.cwd.clone().unwrap_or_else(|| "repo".to_string());
        let dir = self.root.join(&cwd);
        if !dir.is_dir() {
            return Err(format!("step cwd {cwd} is not a folder in the sandbox"));
        }
        let env = with_overrides(self.case_env.clone(), &step.env);
        let stdin = match (&step.stdin, &step.stdin_file) {
            (Some(text), None) => text.clone().into_bytes(),
            (None, Some(file)) => fs::read(case_dir.join(file))
                .map_err(|e| io_failure(&format!("cannot read stdin_file {file}"), &e))?,
            (None, None) => Vec::new(),
            (Some(_), Some(_)) => return Err("a step sets both stdin and stdin_file".to_string()),
        };
        let out = run(self.bin.as_os_str(), &step.args, &dir, &env, stdin)?;
        Ok(StepRecord {
            args: step.args.clone(),
            cwd,
            env: step.env.clone(),
            exit: out.exit,
            stdout: out.stdout,
            stderr: out.stderr,
        })
    }
}

/// `env` with each of `overrides` set on top.
fn with_overrides(mut env: Env, overrides: &std::collections::BTreeMap<String, String>) -> Env {
    for (key, value) in overrides {
        set(&mut env, key, value);
    }
    env
}

/// Runs one setup argv in repo/. Its output is not golden'ed, and a nonzero exit fails the case.
fn run_setup(
    number: usize,
    setup: &SetupStep,
    repo: &Path,
    bin: &Path,
    env: &Env,
) -> Result<(), String> {
    let (program, args) = setup
        .run
        .split_first()
        .ok_or_else(|| format!("setup step {number} is empty"))?;
    let program: &OsStr = match program.as_str() {
        "git" => OsStr::new("git"),
        "memex" => bin.as_os_str(),
        other => {
            return Err(format!(
                "setup step {number} runs {other:?}, not git or memex"
            ))
        }
    };
    let out = run(program, args, repo, env, Vec::new())?;
    expect_success(&out, &format!("setup step {number}"))
}

/// What a finished process left behind.
struct Outcome {
    /// None when a signal ended the process.
    exit: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Runs a program with exactly `env`, feeding it `stdin` and collecting both output streams.
fn run(
    program: &OsStr,
    args: &[String],
    cwd: &Path,
    env: &Env,
    stdin: Vec<u8>,
) -> Result<Outcome, String> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "cannot start {} ({:?})",
                program.to_string_lossy(),
                e.kind()
            )
        })?;
    // Feed stdin on its own thread so a child that writes a lot cannot deadlock against us.
    let feeder = child.stdin.take().map(|mut input| {
        std::thread::spawn(move || {
            let _ = input.write_all(&stdin);
        })
    });
    let output = child
        .wait_with_output()
        .map_err(|e| io_failure("cannot wait for a process", &e))?;
    if let Some(feeder) = feeder {
        let _ = feeder.join();
    }
    Ok(Outcome {
        exit: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

/// Fails unless the process exited 0. The failure carries its stderr.
fn expect_success(out: &Outcome, what: &str) -> Result<(), String> {
    if out.exit == Some(0) {
        return Ok(());
    }
    let how = match out.exit {
        Some(code) => format!("exit {code}"),
        None => "a signal".to_string(),
    };
    Err(format!(
        "{what} failed with {how}: {}",
        String::from_utf8_lossy(&out.stderr).trim_end()
    ))
}

/// Makes repo/ a git repository on branch main, and optionally commits the fixture with a fixed identity and date.
fn git_fixture(repo: &Path, base: &Env, commit: bool) -> Result<(), String> {
    let git = OsStr::new("git");
    let init = run(
        git,
        &args(&["init", "-q", "-b", "main"]),
        repo,
        base,
        Vec::new(),
    )?;
    expect_success(&init, "git init")?;
    if commit {
        let env = with_git_identity(base);
        let add = run(git, &args(&["add", "-A"]), repo, &env, Vec::new())?;
        expect_success(&add, "git add")?;
        // --allow-empty: an empty fixture still gives repo/ a first commit.
        let commit = run(
            git,
            &args(&["commit", "-q", "--allow-empty", "-m", "fixture"]),
            repo,
            &env,
            Vec::new(),
        )?;
        expect_success(&commit, "git commit")?;
    }
    Ok(())
}

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

/// Fails if the sandbox would sit under a directory that holds a .memex store.
fn reject_memex_ancestor(root: &Path) -> Result<(), String> {
    match root.ancestors().find(|dir| dir.join(".memex").is_dir()) {
        Some(_) => {
            Err("a .memex store sits above the sandbox; point TMPDIR outside it".to_string())
        }
        None => Ok(()),
    }
}

/// Copies the fixture into the sandbox root. A path part that starts with `dot_` becomes `.`.
fn copy_fixture(from: &Path, to: &Path) -> Result<(), String> {
    let entries = fs::read_dir(from).map_err(|e| io_failure("cannot read the fixture", &e))?;
    for entry in entries {
        let entry = entry.map_err(|e| io_failure("cannot read the fixture", &e))?;
        let name = rename_part(&entry.file_name().to_string_lossy());
        let target = to.join(&name);
        let kind = entry
            .file_type()
            .map_err(|e| io_failure("cannot read the fixture", &e))?;
        if kind.is_dir() {
            fs::create_dir_all(&target).map_err(|e| io_failure("cannot copy the fixture", &e))?;
            copy_fixture(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target)
                .map_err(|e| io_failure("cannot copy the fixture", &e))?;
        } else {
            return Err(format!(
                "the fixture holds {name:?}, which is neither a file nor a folder"
            ));
        }
    }
    Ok(())
}

/// Renames a path part that starts with `dot_` so it starts with `.`, and changes nothing else.
fn rename_part(part: &str) -> String {
    match part.strip_prefix("dot_") {
        Some(rest) => format!(".{rest}"),
        None => part.to_string(),
    }
}

/// A failure message that names what failed and the io::ErrorKind, never the raw OS text.
fn io_failure(what: &str, error: &io::Error) -> String {
    format!("{what} ({:?})", error.kind())
}
