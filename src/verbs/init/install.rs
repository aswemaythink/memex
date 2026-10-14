//! Installs the running binary when no memex is on PATH (design §3.5b item 3), and hints at PATH.
//! Cases: INIT-INSTALL-01, INIT-INSTALL-02, INIT-INSTALL-03, INIT-INSTALL-04, INIT-PATH-01, INIT-PATH-02

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::files::{io_error, label, lists, write_temp};
use super::{Action, Report};
use crate::error::Error;
use crate::root::slash;

/// The file name of the installed binary on Windows.
#[cfg(windows)]
const EXE: &str = "memex.exe";
/// The file name of the installed binary on unix.
#[cfg(not(windows))]
const EXE: &str = "memex";

/// Installs the running binary into the install folder when no memex is on PATH, then prints the PATH hint when that folder is
/// not on PATH. With a memex on PATH, nothing is reported (design §3.5b item 3).
pub fn run(target: &Path, report: &mut Report) -> Result<(), Error> {
    if memex_on_path() {
        return Ok(());
    }
    let dir = install_dir()?;
    let bytes = running_binary()?;
    let action = install(&dir, &bytes)?;
    report.action(action, &label(target, &dir.join(EXE)));
    if !on_path(&dir) {
        report.line(&format!("add to PATH: {}", slash(&dir)));
    }
    Ok(())
}

/// The bytes of the running binary, which the install copies.
fn running_binary() -> Result<Vec<u8>, Error> {
    let source = env::current_exe()
        .map_err(|failure| Error::io("cannot find the running memex", &failure))?;
    fs::read(&source).map_err(|failure| io_error("cannot read", &source, &failure))
}

/// Writes the binary into `dir`: created when there is none, kept when the file holds the same bytes, and replaced when they differ.
fn install(dir: &Path, bytes: &[u8]) -> Result<Action, Error> {
    let dest = dir.join(EXE);
    if !lists(dir, EXE)? {
        fs::create_dir_all(dir).map_err(|failure| io_error("cannot create", dir, &failure))?;
        write_binary(dir, bytes)?;
        return Ok(Action::Created);
    }
    let current = fs::read(&dest).map_err(|failure| io_error("cannot read", &dest, &failure))?;
    if current == bytes {
        return Ok(Action::Kept);
    }
    write_binary(dir, bytes)?;
    Ok(Action::Updated)
}

/// Writes the binary to `dir/EXE` through a temp file in `dir`, made executable where that means something, and then
/// replaces whatever file is at that name. A failed replace leaves the old binary whole; on Windows that is what happens while
/// the old one is running, so a refusal says it may be.
fn write_binary(dir: &Path, bytes: &[u8]) -> Result<(), Error> {
    let dest = dir.join(EXE);
    let temp = write_temp(dir, bytes, None)?;
    make_executable(temp.path())?;
    temp.persist(&dest).map_err(|failed| {
        if failed.error.kind() == io::ErrorKind::PermissionDenied {
            let what = format!("cannot replace {}, which may be running", slash(&dest));
            Error::io(&what, &failed.error)
        } else {
            io_error("cannot write", &dest, &failed.error)
        }
    })?;
    Ok(())
}

/// The folder the binary is installed to: `$HOME/.local/bin` on unix. Its variable is read, not the known-folder API, so tests can
/// redirect it (design §3.5b item 3).
#[cfg(not(windows))]
fn install_dir() -> Result<PathBuf, Error> {
    Ok(env_dir("HOME")?.join(".local").join("bin"))
}

/// The folder the binary is installed to: `%LOCALAPPDATA%\Programs\memex` on Windows, read from the variable (design §3.5b item 3).
#[cfg(windows)]
fn install_dir() -> Result<PathBuf, Error> {
    Ok(env_dir("LOCALAPPDATA")?.join("Programs").join("memex"))
}

/// The absolute folder in the variable `name`. An unset, empty or relative value is an environment error (exit 3), never a
/// folder under the cwd. Case: INIT-INSTALL-04
fn env_dir(name: &str) -> Result<PathBuf, Error> {
    match env::var_os(name).map(PathBuf::from) {
        Some(dir) if dir.is_absolute() => Ok(dir),
        _ => Err(Error::io_message(format!(
            "the {name} variable is not an absolute folder, so memex cannot install itself"
        ))),
    }
}

/// Gives the temp file the mode of an installed program, rwxr-xr-x (unix).
#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|failure| io_error("cannot make executable", path, &failure))
}

/// A Windows program is run by its `.exe` name, so there is no mode to set.
#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), Error> {
    Ok(())
}

/// True when some folder on PATH lists a memex program. An unreadable folder on PATH holds none.
fn memex_on_path() -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| holds_program(&dir))
}

/// True when `dir` lists the binary's name and that entry is a program that can run.
fn holds_program(dir: &Path) -> bool {
    lists(dir, EXE).unwrap_or(false) && executable(&dir.join(EXE))
}

/// True when the path is a file that a shell can run: an execute bit is set (unix). On Windows the `.exe` name decides.
#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// On Windows the `.exe` name is what makes a file run, and the listing has already matched it.
#[cfg(not(unix))]
fn executable(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file())
}

/// True when `dir` is one of the folders on PATH.
fn on_path(dir: &Path) -> bool {
    env::var_os("PATH")
        .is_some_and(|path| env::split_paths(&path).any(|entry| same_folder(&entry, dir)))
}

/// True when two paths name one folder: the same text, or the same folder once resolved.
fn same_folder(left: &Path, right: &Path) -> bool {
    left == right
        || matches!(
            (dunce::canonicalize(left), dunce::canonicalize(right)),
            (Ok(left), Ok(right)) if left == right
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_is_on_path_when_it_is_named_or_resolves_to_the_same_folder() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let same = dir.path().join(".").join("");
        assert!(same_folder(dir.path(), &same));
        assert!(!same_folder(dir.path(), &dir.path().join("elsewhere")));
    }

    #[test]
    fn a_folder_with_no_memex_holds_no_program() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        assert!(!holds_program(dir.path()));
        assert!(!holds_program(&dir.path().join("missing")));
    }
}
