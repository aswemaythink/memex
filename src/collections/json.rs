//! Reads the docs of a `json` manifest: each entry of the array at `at` in every matched file is one doc (design §4.2). S2.5 fills it in.
//! Cases: none (stub; MAN-* in S2.5 pins it)

use std::path::Path;

use super::manifest::Manifest;
use crate::corpus::Doc;

/// The docs of a json manifest. None are read yet.
pub fn docs(_manifest: &Manifest, _root: &Path) -> Vec<Doc> {
    Vec::new()
}
