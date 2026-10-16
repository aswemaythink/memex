//! Matches a json or folder manifest's files against its globs (design §4.2 `include` and `merge`).
//! The roots are walked as the intent layer is (design §4.1 item 2), and the globs follow the gitignore rules.
//! Cases: MANK-JSON-06, MANK-FOLDER-04, MANK-FOLDER-08

use std::path::Path;

use ignore::overrides::OverrideBuilder;

use crate::walk::Entry;

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
