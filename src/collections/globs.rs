//! Matches the files under a manifest's roots against its globs (design §4.2 `include` and `merge`).
//! Every kind uses it. The roots are walked as the intent layer is (design §4.1 item 2), and the globs follow the gitignore rules.
//! Cases: MAN-MD-11, MAN-MD-13, MANK-JSON-06, MANK-FOLDER-04, MANK-FOLDER-08

use std::path::{Path, PathBuf};

use ignore::overrides::OverrideBuilder;

use super::manifest::Manifest;
use crate::walk::{self, Entry};

/// A file that a manifest's globs matched: its path from the root, and the path it is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matched {
    /// The path from the root, with `/` separators.
    pub rel: String,
    /// The path to read the file from.
    pub path: PathBuf,
}

/// The files under each root of `manifest` that `globs` match, sorted by path from the root, each listed once.
pub fn matched(manifest: &Manifest, root: &Path, globs: &[String]) -> Vec<Matched> {
    let mut found = Vec::new();
    for rel_root in &manifest.roots {
        let base = root.join(rel_root);
        let entries = walk::walk(&base);
        let files: Vec<&Entry> = entries.iter().filter(|entry| !entry.is_dir).collect();
        found.extend(hits(&base, globs, &files).into_iter().map(|file| Matched {
            rel: if rel_root.is_empty() {
                file.rel.clone()
            } else {
                format!("{rel_root}/{}", file.rel)
            },
            path: file.path.clone(),
        }));
    }
    found.sort_by(|left, right| left.rel.cmp(&right.rel));
    found.dedup_by(|left, right| left.rel == right.rel);
    found
}

/// The files among `files` that one of `globs` matches, read relative to `base` with the gitignore rules, so `*.md` matches a
/// name and `**/*.json` matches at any depth. A glob that does not parse matches nothing.
pub fn hits<'a>(base: &Path, globs: &[String], files: &[&'a Entry]) -> Vec<&'a Entry> {
    let mut builder = OverrideBuilder::new(base);
    for glob in globs {
        // A glob that does not parse is skipped, so it matches nothing.
        let _ = builder.add(glob);
    }
    let Ok(overrides) = builder.build() else {
        return Vec::new();
    };
    files
        .iter()
        .copied()
        .filter(|file| overrides.matched(&file.path, false).is_whitelist())
        .collect()
}
