//! Reads the docs of a `folder` manifest: each folder that holds its marker file is one doc, with the marker's fields and its merged files (design §4.2). S2.5 fills it in.
//! Cases: none (stub; MAN-* in S2.5 pins it)

use std::path::Path;

use super::manifest::Manifest;
use crate::corpus::Doc;

/// The docs of a folder manifest. None are read yet.
pub fn docs(_manifest: &Manifest, _root: &Path) -> Vec<Doc> {
    Vec::new()
}
