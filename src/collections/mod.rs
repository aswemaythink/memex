//! The collections of design §4.2: the manifest that describes one, the flat-YAML frontmatter reader, and the readers of the three kinds. S2.0 holds the types and the stubs; the loader and the kinds come in S2.4 and S2.5.
//! Cases: none (module map; MAN-* in S2.4 and S2.5 pins the kinds)

pub mod folder;
pub mod frontmatter;
pub mod json;
pub mod manifest;
pub mod markdown;
