//! Finds the main checkout's copy of a root that a linked worktree lacks (design §4.2).
//! The copy is the same root-relative path under the main checkout, and the layout files alone say where that is, so no git is asked.
//! Cases: MANK-WT-01, MANK-WT-02, MANK-WT-03, MANK-WT-05

use std::path::{Path, PathBuf};

use crate::layout::{git_entry_is_file, layout_of, scan, Layout};

/// The folder that `repo` has in the main checkout, when `repo` lies in a linked worktree whose layout places it: the same path under
/// the main checkout (design §4.2 Roots in a linked worktree). None for the main checkout, a submodule, a worktree of a bare repo, a
/// folder outside any repo, and when that path is `repo` itself.
pub fn counterpart(repo: &Path) -> Option<PathBuf> {
    let (top, _) = scan(repo);
    let top = top?;
    if !git_entry_is_file(&top) {
        return None;
    }
    let Layout::Linked(common) = layout_of(&top) else {
        return None;
    };
    let rel = repo.strip_prefix(&top).ok()?;
    let found = common.parent()?.join(rel);
    (found.as_path() != repo).then_some(found)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::counterpart;

    /// The folder of a linked worktree maps to the same path under the main checkout, whose own folder maps to nothing: its `.git`
    /// is a folder, so it is the main checkout (design §4.2 Roots in a linked worktree).
    #[test]
    fn a_linked_worktree_folder_maps_to_the_same_path_in_the_main_checkout() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let top = dunce::canonicalize(dir.path()).expect("a canonical folder");
        let main = top.join("main");
        let linked = main.join(".git").join("worktrees").join("wt");
        fs::create_dir_all(&linked).expect("a linked git dir");
        fs::write(linked.join("commondir"), b"../..\n").expect("a commondir");
        let wt = top.join("wt");
        fs::create_dir_all(wt.join("sub")).expect("a worktree folder");
        fs::write(wt.join(".git"), format!("gitdir: {}\n", linked.display())).expect("a file");
        let expected: PathBuf = main.join("sub");
        assert_eq!(counterpart(&wt.join("sub")), Some(expected));
        assert_eq!(counterpart(&main.join("sub")), None);
    }

    /// Outside any repo there is no main checkout to borrow from.
    #[test]
    fn a_folder_outside_any_repo_has_no_counterpart() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        assert_eq!(counterpart(dir.path()), None);
    }
}
