//! Reads the docs of a `markdown` manifest: each matched Markdown file, with its frontmatter as fields (design §4.2). S2.4 fills it in.
//! Cases: none (stub; MAN-* in S2.4 pins it)

use std::path::Path;

use super::manifest::Manifest;
use crate::corpus::Doc;

/// The docs of a markdown manifest. None are read yet.
pub fn docs(_manifest: &Manifest, _root: &Path) -> Vec<Doc> {
    Vec::new()
}
