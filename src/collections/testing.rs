//! Test helpers shared by the collection readers' tests: a temporary tree, a manifest and doc labels.
//! Cases: none (test helpers; the tests that use them cite the ids they pin)

use std::collections::BTreeMap;
use std::fs;

use super::manifest::{Kind, Manifest};
use crate::corpus::Doc;

/// A manifest named `cases` of `kind` over `roots`, with no keys set: a test sets the keys it uses.
pub(super) fn manifest(kind: Kind, roots: &[&str]) -> Manifest {
    Manifest {
        name: "cases".to_string(),
        description: None,
        owner: None,
        kind,
        roots: roots.iter().map(|root| root.to_string()).collect(),
        include: Vec::new(),
        id: None,
        title: None,
        weights: BTreeMap::new(),
        marker: None,
        merge: Vec::new(),
        at: None,
    }
}

/// A temporary folder holding each `(path, text)` pair, with its folders made.
pub(super) fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("a temporary folder");
    for (rel, text) in files {
        let path = root.path().join(rel);
        fs::create_dir_all(path.parent().expect("a folder")).expect("a folder");
        fs::write(path, text).expect("a file");
    }
    root
}

/// The labels of the docs, in order.
pub(super) fn labels(found: &[Doc]) -> Vec<&str> {
    found.iter().map(|doc| doc.label.as_str()).collect()
}
