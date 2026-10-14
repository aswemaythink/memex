//! Keeps the manifest roots that lie inside the repo root (design §4.2 Unreadable files).
//! It also parses the `roots` key. A root that resolves outside the repo root is left out with its warning, so no reader walks it.
//! Cases: MANK-ROOTS-01, MAN-LOAD-09, MAN-LOAD-10

use std::path::Path;

use serde_json::{Map, Value as Json};

use super::manifest::Manifest;
use crate::error;
use crate::walk::rel_string;

/// The manifest with only the roots that lie inside the repo root. A root that resolves outside it is left out, with the warning
/// `<collection>: root <root> leads outside the repo; skipped`, so no reader walks it (design §4.2 Unreadable files).
pub fn inside_repo(manifest: &Manifest, repo: &Path, warnings: &mut Vec<String>) -> Manifest {
    let mut usable = manifest.clone();
    usable.roots = manifest
        .roots
        .iter()
        .filter(|rel| {
            let outside = resolves_outside(&repo.join(rel), repo);
            if outside {
                warnings.push(error::warning(&format!(
                    "{}: root {rel} leads outside the repo; skipped",
                    manifest.name
                )));
            }
            !outside
        })
        .cloned()
        .collect();
    usable
}

/// True when `base` resolves, through its symbolic links, to a folder outside `repo`. Both are canonicalised with `dunce`, so a
/// symlink inside the repo stays inside. A path that cannot be resolved is not judged: the walk reads what it can (design §4.2).
fn resolves_outside(base: &Path, repo: &Path) -> bool {
    match (dunce::canonicalize(base), dunce::canonicalize(repo)) {
        (Ok(folder), Ok(home)) => !folder.starts_with(home),
        _ => false,
    }
}

/// The roots: present, and each a folder inside the root, written relative to it.
pub(super) fn roots_of(object: &Map<String, Json>) -> Result<Vec<String>, String> {
    match object.get("roots") {
        Some(Json::Array(items)) => items.iter().map(root_of).collect(),
        Some(_) => Err("key 'roots' must be a list of folders".to_string()),
        None => Err("missing key 'roots'".to_string()),
    }
}

/// One root, normalised to `/` separators; an absolute root, or one that climbs with `..`, is refused.
fn root_of(item: &Json) -> Result<String, String> {
    let text = item
        .as_str()
        .ok_or("key 'roots' must be a list of folders")?;
    rel_string(Path::new(text))
        .ok_or_else(|| format!("root '{text}' is not a folder inside the root"))
}

#[cfg(test)]
mod tests {
    use crate::collections::manifest::Kind;
    use crate::collections::testing;

    use super::inside_repo;

    /// A root that does not exist is kept, since its walk reads nothing, and it gives no warning (MAN-LOAD-01).
    #[test]
    fn a_missing_root_is_kept_with_no_warning() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let manifest = testing::manifest(Kind::Markdown, &["missing"]);
        let mut warnings = Vec::new();
        assert_eq!(
            inside_repo(&manifest, dir.path(), &mut warnings).roots,
            ["missing"]
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// MANK-ROOTS-01: a root that is a symlink to a folder outside the repo is left out with its warning, and the roots beside it
    /// stay. A symlink inside the repo to a folder inside it is kept.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_root_outside_the_repo_is_left_out_with_its_warning() {
        use std::fs;
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().expect("a temporary folder");
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join("units")).expect("a folder");
        fs::create_dir_all(dir.path().join("outside")).expect("a folder");
        symlink(dir.path().join("outside"), repo.join("linked")).expect("a symlink");
        symlink("units", repo.join("shortcut")).expect("a symlink");
        let manifest = testing::manifest(Kind::Markdown, &["linked", "shortcut", "units"]);
        let mut warnings = Vec::new();
        let usable = inside_repo(&manifest, &repo, &mut warnings);
        assert_eq!(usable.roots, ["shortcut", "units"]);
        assert_eq!(
            warnings,
            ["memex: warning: cases: root linked leads outside the repo; skipped"]
        );
    }
}
