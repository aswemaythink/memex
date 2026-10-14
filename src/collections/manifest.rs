//! Holds a collection manifest's keys (design §4.2) and dispatches its docs to the reader of its kind. The loader comes in S2.4.
//! Cases: none (stub; MAN-* in S2.4 and S2.5 pins it)

use std::collections::BTreeMap;
use std::path::Path;

use super::{folder, json, markdown};
use crate::corpus::Doc;

/// How a manifest's roots are read (design §4.2 `kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Each matched Markdown file is one doc.
    Markdown,
    /// Each folder that holds the marker file is one doc.
    Folder,
    /// Each entry of the array at `at` is one doc.
    Json,
}

/// A collection manifest with every key of design §4.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The label prefix, which must equal the file stem.
    pub name: String,
    /// Shown by `info`.
    pub description: Option<String>,
    /// The skill that builds the collection.
    pub owner: Option<String>,
    /// How the roots are read.
    pub kind: Kind,
    /// The root-relative folders the collection reads.
    pub roots: Vec<String>,
    /// Globs under the roots; markdown only.
    pub include: Vec<String>,
    /// The field or rule that gives each doc its id.
    pub id: Option<String>,
    /// The field that gives each doc its one-liner.
    pub title: Option<String>,
    /// The ranking boost of each field.
    pub weights: BTreeMap<String, u32>,
    /// The file whose presence defines a doc; folder only.
    pub marker: Option<String>,
    /// The globs appended to a folder doc's body; folder only.
    pub merge: Vec<String>,
    /// The JSON Pointer to the entries array; json only.
    pub at: Option<String>,
}

/// The manifests under `.memex/collections/`. None are loaded before S2.4.
pub fn load(_root: &Path) -> Vec<Manifest> {
    Vec::new()
}

/// The docs of one manifest, read by the reader of its kind.
pub fn docs(manifest: &Manifest, root: &Path) -> Vec<Doc> {
    match manifest.kind {
        Kind::Markdown => markdown::docs(manifest, root),
        Kind::Folder => folder::docs(manifest, root),
        Kind::Json => json::docs(manifest, root),
    }
}
