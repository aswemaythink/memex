//! Finds the `roots` kind of `audit` (design §3.5d kind 8): roots that cannot work as written.
//! A README whose `memex:` key names no manifest, a manifest with no root, and a root inside another root of the same manifest,
//! which reads the inner one's docs twice (design §4.2 Roots).
//! Cases: AUD-ROOTS-01, AUD-ROOTS-02

use super::kinds::{group, Ctx};
use crate::collections::manifest::{Kind, Manifest};
use crate::render::audit::{Group, Item};

/// The fix line of `roots` (design §3.5d kind 8).
const ROOTS: &str = "(roots): give each marked collection a manifest and each manifest a root, and nest no root inside another";

/// The findings of `roots` in scope: a README by its path, a nested root by its folder, and a manifest with no root only when the
/// scope holds the root, since its file lies in `.memex/`, which the walk skips.
pub fn roots(ctx: &Ctx<'_>) -> Group {
    let manifests: Vec<&Manifest> = ctx
        .corpus
        .manifests
        .iter()
        .map(|loaded| &loaded.manifest)
        .filter(|found| found.kind != Kind::Markdown)
        .collect();
    let mut items: Vec<Item> = ctx
        .corpus
        .marks
        .iter()
        .filter(|mark| !manifests.iter().any(|found| found.name == mark.name))
        .filter(|mark| ctx.scope.holds_file(&mark.readme))
        .map(|mark| item(mark.readme.clone(), format!("→ {}: no manifest", mark.name)))
        .collect();
    for found in manifests {
        if found.roots.is_empty() && ctx.scope.holds_folder("") {
            let file = format!(".memex/collections/{}.json", found.name);
            items.push(item(file, "no roots".to_string()));
        }
        items.extend(nested(ctx, found));
    }
    group(ctx, "roots", "collection roots", Some(ROOTS), items)
}

/// Each root of the manifest that lies inside another of its roots, with the first such outer root, when the scope holds it.
fn nested(ctx: &Ctx<'_>, found: &Manifest) -> Vec<Item> {
    found
        .roots
        .iter()
        .filter(|inner| ctx.scope.holds_folder(inner))
        .filter_map(|inner| {
            let outer = found.roots.iter().find(|outer| inside(inner, outer))?;
            let shown = if outer.is_empty() { "." } else { outer };
            Some(item(
                inner.clone(),
                format!("{}: inside {shown}", found.name),
            ))
        })
        .collect()
}

/// True when the folder `inner` lies below the folder `outer`; the root, `""`, holds every other folder.
fn inside(inner: &str, outer: &str) -> bool {
    inner != outer
        && (outer.is_empty()
            || inner
                .strip_prefix(outer)
                .is_some_and(|rest| rest.starts_with('/')))
}

/// One finding with its detail.
fn item(reference: String, detail: String) -> Item {
    Item {
        reference,
        detail: Some(detail),
    }
}

#[cfg(test)]
mod tests {
    use super::inside;

    /// A folder lies inside its ancestors and the root, and not inside itself or a sibling that shares its prefix.
    #[test]
    fn inside_means_strictly_below() {
        assert!(inside("a/b", "a"));
        assert!(inside("a", ""));
        assert!(!inside("a", "a"));
        assert!(!inside("ab", "a"));
        assert!(!inside("", ""));
    }
}
