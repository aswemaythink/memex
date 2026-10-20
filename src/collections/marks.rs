//! Loads the manifests over their roots, explicit and marked by READMEs (design §4.2 Roots).
//! A README whose frontmatter says `memex: <name>` marks its folder as a root of the manifest `<name>`; a manifest reads its own
//! `roots`, then the marked folders, each once. A mark that names no manifest is a warning, and `audit` lists it.
//! Cases: MAN-ROOTS-01, MAN-ROOTS-02, MAN-ROOTS-03, MAN-ROOTS-04, COLL-README-01

use std::path::Path;

use super::frontmatter::{self, Value};
use super::manifest::{self, Loaded, Manifest};
use crate::corpus::{parent_of, Doc};
use crate::error;
use crate::root::slash;

/// A README whose frontmatter holds a `memex:` key: the folder it marks and the collection it names (design §4.2 Roots).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    /// The root-relative path of the README.
    pub readme: String,
    /// The root-relative folder that holds it, which it marks; empty for the root.
    pub folder: String,
    /// The collection that the key names; empty when the key holds no single name.
    pub name: String,
}

impl Mark {
    /// The mark of the README `readme`, whose key names `name`.
    pub fn new(readme: &str, name: String) -> Mark {
        Mark {
            readme: readme.to_string(),
            folder: parent_of(readme).unwrap_or_default().to_string(),
            name,
        }
    }
}

/// The collection that a README's `memex:` key names, read from its whole text `text`: the key's value, or empty for a list.
pub fn name_in(text: &str) -> String {
    frontmatter::split(text)
        .fields
        .into_iter()
        .find(|(key, _)| key == "memex")
        .map(|(_, value)| match value {
            Value::Str(name) => name,
            Value::List(_) => String::new(),
        })
        .unwrap_or_default()
}

/// The manifests under `root` with their docs, each read over its own roots and then the folders that `marks` give it. A manifest
/// file that does not load adds its warning to `manifest_warnings`; a mark that names no manifest adds its warning to `warnings`.
pub fn load(
    root: &Path,
    marks: &[Mark],
    manifest_warnings: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> (Vec<Loaded>, Vec<Doc>) {
    let mut manifests = manifest::load(root, manifest_warnings);
    for found in &mut manifests {
        union(found, marks);
    }
    for mark in marks.iter().filter(|mark| !names_any(&manifests, mark)) {
        warnings.push(no_manifest(root, mark));
    }
    let (mut loaded, mut docs) = (Vec::new(), Vec::new());
    for found in manifests {
        let mut warnings = Vec::new();
        let read = manifest::docs(&found, root, &mut warnings);
        loaded.push(Loaded {
            manifest: found,
            count: read.len(),
            warnings,
        });
        docs.extend(read);
    }
    (loaded, docs)
}

/// Adds to the manifest's roots each folder a mark gives it that the roots do not already hold, in the marks' order.
fn union(found: &mut Manifest, marks: &[Mark]) {
    for mark in marks.iter().filter(|mark| mark.name == found.name) {
        if !found.roots.contains(&mark.folder) {
            found.roots.push(mark.folder.clone());
        }
    }
}

/// True when a manifest that loaded is the one the mark names.
fn names_any(manifests: &[Manifest], mark: &Mark) -> bool {
    manifests.iter().any(|found| found.name == mark.name)
}

/// The warning for a README whose key names no manifest (design §2.2).
fn no_manifest(root: &Path, mark: &Mark) -> String {
    error::warning(&format!(
        "{} is a README; its memex: key marks a root of '{name}', which has no manifest — add .memex/collections/{name}.json, or move that content into its own file",
        slash(&root.join(&mark.readme)),
        name = mark.name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::manifest::Kind;
    use crate::collections::testing;

    /// A README's key gives its name, and a list gives none.
    #[test]
    fn the_name_is_the_keys_single_value() {
        assert_eq!(name_in("---\nmemex: lib-cases\n---\n# A\n"), "lib-cases");
        assert_eq!(name_in("---\nmemex: [a, b]\n---\n# A\n"), "");
    }

    /// MAN-ROOTS-02: a marked folder joins the explicit roots once, after them, and a mark for another manifest is left out.
    #[test]
    fn marks_join_the_explicit_roots_once() {
        let mut found = testing::manifest(Kind::Folder, &["a"]);
        let marks = [
            Mark::new("a/README.md", "cases".to_string()),
            Mark::new("b/README.md", "cases".to_string()),
            Mark::new("c/README.md", "other".to_string()),
            Mark::new("README.md", "cases".to_string()),
        ];
        union(&mut found, &marks);
        assert_eq!(found.roots, ["a", "b", ""]);
    }
}
