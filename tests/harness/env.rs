//! The environment every sandboxed run gets, with the sandbox defaults.
//! The inherited environment, scrubbed of memex, git and agent variables and of PATH entries holding a memex.
//! Cases: all (runner)

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The clock every case runs under (MEMEX_NOW). A step's env may override it.
const NOW: &str = "2026-10-13T12:00:00Z";
/// The date and identity of runner git commits, so commit hashes are stable.
const GIT_DATE: &str = "2026-10-13T11:00:00Z";
const IDENTITY_NAME: &str = "memex test";
const IDENTITY_EMAIL: &str = "test@memex.invalid";

/// A process environment: exactly these variables, nothing inherited.
pub type Env = BTreeMap<OsString, OsString>;

/// The inherited environment minus the variables a case must not see, plus the sandbox defaults.
pub fn base_env(root: &Path, bin: &Path, memex_on_path: bool) -> Result<Env, String> {
    let mut env: Env = std::env::vars_os()
        .filter(|(name, _)| !scrubbed(name))
        .collect();

    // Remove every PATH entry that holds a memex, so the dogfood install cannot leak in.
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();
    dirs.retain(|dir| !has_memex(dir));
    if memex_on_path {
        if let Some(bin_dir) = bin.parent() {
            dirs.insert(0, bin_dir.to_path_buf());
        }
    }
    let path =
        std::env::join_paths(&dirs).map_err(|_| "a PATH entry cannot be joined".to_string())?;
    set(&mut env, "PATH", &path);

    let home = root.join("home");
    set(&mut env, "HOME", home.as_os_str());
    set(&mut env, "USERPROFILE", home.as_os_str());
    set(
        &mut env,
        "XDG_CONFIG_HOME",
        home.join(".config").as_os_str(),
    );
    set(
        &mut env,
        "LOCALAPPDATA",
        home.join("AppData").join("Local").as_os_str(),
    );
    set(&mut env, "GIT_CONFIG_NOSYSTEM", "1");
    set(
        &mut env,
        "GIT_CONFIG_GLOBAL",
        home.join(".gitconfig").as_os_str(),
    );
    set(&mut env, "GIT_CEILING_DIRECTORIES", root.as_os_str());
    set(&mut env, "NO_COLOR", "1");
    set(&mut env, "MEMEX_NOW", NOW);
    // The C locale, so memex and git print the same bytes on every machine.
    set(&mut env, "LC_ALL", "C");
    set(&mut env, "LANG", "C");
    Ok(env)
}

/// The inherited variables a case must not see: MEMEX_*, GIT_*, CLAUDE* and AI_AGENT.
/// Windows names are case-insensitive, and its hidden `=C:`-style entries are not variables.
fn scrubbed(name: &OsStr) -> bool {
    let name = name.to_string_lossy();
    let name = if cfg!(windows) {
        name.to_ascii_uppercase()
    } else {
        name.into_owned()
    };
    name.starts_with('=')
        || name.starts_with("MEMEX_")
        || name.starts_with("GIT_")
        || name.starts_with("CLAUDE")
        || name == "AI_AGENT"
}

/// Sets a variable. On Windows names match without regard to case, so an old spelling is replaced.
pub fn set(env: &mut Env, key: &str, value: impl AsRef<OsStr>) {
    env.retain(|name, _| !same_name(name, key));
    env.insert(OsString::from(key), value.as_ref().to_os_string());
}

fn same_name(name: &OsStr, key: &str) -> bool {
    if cfg!(windows) {
        name.to_string_lossy().eq_ignore_ascii_case(key)
    } else {
        name == OsStr::new(key)
    }
}

/// True when the directory holds a memex executable: `memex` on Unix, and `memex.exe` or an npm `memex.cmd` shim on Windows,
/// where a bare `memex` file is not something the shell runs. init counts the same Windows names as installed.
fn has_memex(dir: &Path) -> bool {
    let names: &[&str] = if cfg!(windows) {
        &["memex.exe", "memex.cmd"]
    } else {
        &["memex", "memex.exe"]
    };
    names.iter().any(|name| is_executable(&dir.join(name)))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// `env` plus the fixed git identity and dates, for the git runs the runner makes itself.
pub fn with_git_identity(env: &Env) -> Env {
    let mut env = env.clone();
    for (key, value) in [
        ("GIT_AUTHOR_NAME", IDENTITY_NAME),
        ("GIT_AUTHOR_EMAIL", IDENTITY_EMAIL),
        ("GIT_AUTHOR_DATE", GIT_DATE),
        ("GIT_COMMITTER_NAME", IDENTITY_NAME),
        ("GIT_COMMITTER_EMAIL", IDENTITY_EMAIL),
        ("GIT_COMMITTER_DATE", GIT_DATE),
    ] {
        set(&mut env, key, value);
    }
    env
}
