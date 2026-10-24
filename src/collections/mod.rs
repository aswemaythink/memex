//! The collections of design §4.2: manifests, frontmatter, and the readers of the two manifest kinds.
//! Cases: none (module map; each reader cites its own MAN-* or MANK-* ids)

pub mod borrow;
pub mod folder;
pub mod frontmatter;
pub mod globs;
pub mod json;
pub mod manifest;
pub mod marks;
pub mod roots;
pub mod title;

#[cfg(test)]
mod testing;
