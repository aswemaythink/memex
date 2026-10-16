//! Finds each kind's findings over the scope, in the order of design §3.5d.
//! The kinds are headers, readmes, one-liners, cites and ids. Each is one function over `Ctx`, so `length` (third) and `thin` (fifth) join the list as one function each. Each list is
//! sorted by label and cut to `--limit` (design §3.3 Limits), and a kind with no finding is left out.
//! Cases: CHK-HEADERS-01, CHK-HEADERS-02, CHK-HEADERS-03, CHK-READMES-01, CHK-READMES-02, CHK-READMES-03, CHK-ONELINERS-01, CHK-ONELINERS-02, CHK-ONELINERS-03, CHK-ONELINERS-04, CHK-CITES-01, CHK-CITES-02, CHK-CITES-03, CHK-CITES-04, CHK-CITES-05, CHK-CITES-06, CHK-IDS-01, CHK-IDS-02, CHK-LIMIT-01, CHK-LIMIT-02, CHK-LIMIT-03, CHK-JSON-01, CHK-JSON-03

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::scope::Scope;
use crate::cite;
use crate::corpus::{label_of, readme_label, Corpus, Doc, Kind};
use crate::intent::header::has_family;
use crate::intent::{shared, Origin};
use crate::render::check::{Group, Item};
use crate::render::Listing;
use crate::walk::Entry;

/// The longest one-liner that a check allows, in characters (design §3.5d kind 4).
const ONE_LINER_MAX: usize = 100;

/// The fix line of `headers`: the legend of design §3.3 Gaps for a file with no header.
const NO_HEADER: &str =
    "(no header): add a comment block atop the file; its first sentence states the file's purpose";

/// The fix line of `readmes`: the legend of design §3.3 Gaps for a folder with no README.
const NO_README: &str =
    "(no README): add README.md here; line 1 is # Name, then a paragraph saying what the folder is";

/// The fix line of `one-liners` (design §3.5d).
const ONE_LINERS: &str = "(one-liners): shorten the first sentence to 100 characters or fewer";

/// The fix line of `cites` (design §3.5d).
const CITES: &str = "(cites): fix the reference or remove it";

/// The fix line of `ids` (design §3.5d).
const IDS: &str = "(ids): give each doc its own id";

/// What the kinds read: the corpus, the scope, the walked entries and the `--limit` of each list.
pub struct Ctx<'a> {
    /// The corpus of the root that is checked.
    pub corpus: &'a Corpus,
    /// The files and folders that the arguments name.
    pub scope: &'a Scope,
    /// Every walked entry of the root, the root included.
    pub tree: &'a [Entry],
    /// How many findings each kind lists.
    pub limit: usize,
}

/// The findings of every kind over the scope, in §3.5d order, with each kind that has none left out. `length` and `thin` are
/// added to this list in their own slices.
pub fn all(ctx: &Ctx<'_>) -> Vec<Group> {
    let kinds: [fn(&Ctx<'_>) -> Group; 5] = [headers, readmes, one_liners, cites, ids];
    kinds
        .iter()
        .map(|kind| kind(ctx))
        .filter(|group| group.listing.total > 0)
        .collect()
}

/// A kind's group: its items sorted by label (the sort keeps equal labels in the order they came), cut to the limit.
fn group(
    ctx: &Ctx<'_>,
    kind: &'static str,
    what: &'static str,
    fix: Option<&'static str>,
    mut items: Vec<Item>,
) -> Group {
    items.sort_by(|left, right| left.reference.cmp(&right.reference));
    Group {
        kind,
        what,
        fix,
        listing: Listing::of(&items, ctx.limit),
    }
}

/// The walked files in scope with a comment family, no header and some text (design §3.5d kind 1). A file that cannot be opened
/// is not judged, since its read already warned (design §4.2).
fn headers(ctx: &Ctx<'_>) -> Group {
    let items = ctx
        .tree
        .iter()
        .filter(|entry| !entry.is_dir && ctx.scope.holds_file(&entry.rel))
        .filter(|entry| has_family(&entry.path) && !has_header(ctx.corpus, &entry.rel))
        .filter(|entry| is_non_empty(&entry.path))
        .map(|entry| Item {
            reference: entry.rel.clone(),
            detail: None,
        })
        .collect();
    group(
        ctx,
        "headers",
        "files with no header",
        Some(NO_HEADER),
        items,
    )
}

/// True when the graph holds a header doc for the file at this root-relative path.
fn has_header(corpus: &Corpus, rel: &str) -> bool {
    corpus.labelled(rel).iter().any(Doc::is_header)
}

/// True when the file opens and holds at least one byte. Only its metadata is asked for, never its text (design §3.5d, a ruling).
fn is_non_empty(path: &Path) -> bool {
    fs::File::open(path)
        .and_then(|file| file.metadata())
        .is_ok_and(|meta| meta.len() > 0)
}

/// The walked folders in scope with no README.md, the root included (design §3.5d kind 2). A folder the walk could not list is
/// not judged, since its children, and so any README in it, are unknown.
fn readmes(ctx: &Ctx<'_>) -> Group {
    let items = ctx
        .tree
        .iter()
        .filter(|entry| entry.is_dir && ctx.scope.holds_folder(&entry.rel))
        .filter(|entry| ctx.corpus.kind(&readme_label(&entry.rel)) != Some(Kind::File))
        .filter(|entry| ctx.corpus.unlistable(&entry.rel).is_none())
        .map(|entry| Item {
            reference: label_of(&entry.rel),
            detail: None,
        })
        .collect();
    group(
        ctx,
        "readmes",
        "folders with no README",
        Some(NO_README),
        items,
    )
}

/// The READMEs and headers in scope whose one-liner is over 100 characters, counted as Unicode scalar values (design §3.5d kind 4).
fn one_liners(ctx: &Ctx<'_>) -> Group {
    let items = ctx
        .corpus
        .all_docs()
        .filter(|doc| doc.is_readme() || doc.is_header())
        .filter(|doc| {
            doc.path
                .as_deref()
                .is_some_and(|path| ctx.scope.holds_file(path))
        })
        .filter_map(|doc| {
            let count = doc.title.chars().count();
            (count > ONE_LINER_MAX).then(|| Item {
                reference: doc.label.clone(),
                detail: Some(format!("{count} characters")),
            })
        })
        .collect();
    group(
        ctx,
        "one-liners",
        "over 100 characters",
        Some(ONE_LINERS),
        items,
    )
}

/// The explicit references that resolve to nothing, in the READMEs, headers and collection docs in scope (design §3.5d kind 6).
/// Logs are not checked. The citation index resolves each label's doc, so one that is named twice is read once.
fn cites(ctx: &Ctx<'_>) -> Group {
    let labels: BTreeSet<&str> = ctx
        .corpus
        .all_docs()
        .filter(|doc| !ctx.corpus.is_log(doc) && in_scope(ctx.scope, doc))
        .map(|doc| doc.label.as_str())
        .collect();
    let names: Vec<&str> = labels.iter().copied().collect();
    let index = cite::Index::for_show(ctx.corpus, &names, &[]);
    let items = labels
        .iter()
        .flat_map(|label| {
            index
                .cites(label)
                .iter()
                .filter(|cite| !cite.resolved)
                .map(|cite| Item {
                    reference: label.to_string(),
                    detail: Some(format!("→ {}", cite.reference)),
                })
        })
        .collect();
    group(ctx, "cites", "unresolved", Some(CITES), items)
}

/// The collection ids that more than one doc holds, when one of those docs is in scope (design §3.5d kind 7). The detail names
/// each file that holds one, once.
fn ids(ctx: &Ctx<'_>) -> Group {
    let items = shared::duplicates(ctx.corpus)
        .into_iter()
        .filter(|(_, docs)| docs.iter().any(|doc| in_scope(ctx.scope, doc)))
        .map(|(label, docs)| Item {
            reference: label,
            detail: Some(sources(&docs)),
        })
        .collect();
    group(ctx, "ids", "duplicate", Some(IDS), items)
}

/// The files that hold these docs, each once, in the order the docs come, joined by ` · ` (design §3.5d kind 7).
fn sources(docs: &[&Doc]) -> String {
    let mut files: Vec<String> = Vec::new();
    for doc in docs {
        let file = shared::source(doc);
        if !files.contains(&file) {
            files.push(file);
        }
    }
    files.join(" · ")
}

/// True when a doc lies in the scope: a doc with a path by that file, and a manifest's doc, which has none, by its folder (design
/// §4.2 and §3.5d).
fn in_scope(scope: &Scope, doc: &Doc) -> bool {
    match (&doc.path, &doc.origin) {
        (Some(path), _) => scope.holds_file(path),
        (None, Origin::Other { folder, .. }) => scope.holds_folder(folder),
        _ => false,
    }
}
