//! The collections of design §4.2: manifests, frontmatter, and the readers of the three kinds.
//! Cases: none (module map; manifest.rs and markdown.rs cite their MAN-* ids, and folder and json wait for S2.5)

pub mod folder;
pub mod frontmatter;
pub mod json;
pub mod manifest;
pub mod markdown;
