//! Counts each in-scope file's lines for `length` and `thin` (design §3.5d Lines).
//! The files are read on several threads (`par.rs`), and each is read once for its head, then once more past the head only when it is longer.

use super::scope::Scope;
use crate::intent::header::{has_family, lines, read_head};
use crate::par;
use crate::walk::Entry;

/// One in-scope file with a comment family, with its line counts (design §3.5d Lines).
pub struct Counted {
    /// The root-relative path of the file, with `/` separators.
    pub rel: String,
    /// The lines of the file: its whole text, and the header and license blocks that its head holds.
    pub counts: lines::Counts,
}

/// The line counts of each walked file in the scope that has a comment family, in walk order. A file that cannot be read is left
/// out, since its read already warned (design §4.2).
pub fn in_scope(tree: &[Entry], scope: &Scope) -> Vec<Counted> {
    let files: Vec<&Entry> = tree
        .iter()
        .filter(|entry| !entry.is_dir && scope.holds_file(&entry.rel) && has_family(&entry.path))
        .collect();
    par::map(&files, |entry| count_one(entry))
        .into_iter()
        .flatten()
        .collect()
}

/// The counts of one walked file: its head, and its rest when the file is longer than the head (design §3.5d Lines).
fn count_one(entry: &Entry) -> Option<Counted> {
    let head = read_head(&entry.path).ok()?;
    let counts = lines::counts(&entry.path, &head).ok().flatten()?;
    Some(Counted {
        rel: entry.rel.clone(),
        counts,
    })
}
