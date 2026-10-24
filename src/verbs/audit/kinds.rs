//! Finds each kind's findings over the scope, in the order of design §3.5d.
//! The kinds are headers, readmes, length, one-liners, thin, cites, ids and roots, which `roots.rs` finds. Each is one function over `Ctx`; `headers` reads the
//! corpus's head facts, and `length` and `thin` the line counts that `counts.rs` makes from them. Each list is sorted by label (`thin` by its header share) and cut to `--limit` (design §3.3 Limits),
//! and a kind with no finding is left out. `thin` is listed but is not a finding (design §3.5d).
//! Cases: AUD-HEADERS-01, AUD-HEADERS-02, AUD-HEADERS-03, AUD-READMES-01, AUD-READMES-02, AUD-READMES-03, AUD-READMES-04, AUD-READMES-05, AUD-LENGTH-01, AUD-LENGTH-02, AUD-LENGTH-03, AUD-LENGTH-04, AUD-LENGTH-05, AUD-LENGTH-06, AUD-LENGTH-07, AUD-ONELINERS-01, AUD-ONELINERS-02, AUD-ONELINERS-03, AUD-ONELINERS-04, AUD-THIN-01, AUD-THIN-02, AUD-THIN-03, AUD-THIN-04, AUD-THIN-05, AUD-THIN-06, AUD-THIN-07, AUD-CITES-01, AUD-CITES-02, AUD-CITES-03, AUD-CITES-04, AUD-CITES-05, AUD-CITES-06, AUD-IDS-01, AUD-IDS-02, AUD-LIMIT-01, AUD-LIMIT-02, AUD-LIMIT-03, AUD-JSON-01, AUD-JSON-03, AUD-REFINE-01, AUD-REFINE-02, AUD-REFINE-03, AUD-REFINE-04

use std::collections::BTreeSet;

use super::counts::{self, Counted};
use super::roots;
use super::scope::Scope;
use crate::cite;
use crate::corpus::{label_of, parent_of, readme_label, Corpus, Doc, Kind};
use crate::intent::coverage::{self, has_header};
use crate::intent::header::has_family;
use crate::intent::{shared, Origin};
use crate::render::audit::{Group, Item};
use crate::render::Listing;
use crate::walk::Entry;

/// The longest one-liner that an audit allows, in characters (design §3.5d kind 4).
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

/// The most code lines a file may have before `length` lists it (design §3.5d kind 3).
const LENGTH_MAX: u32 = 500;

/// The fix line of `length` (design §3.5d kind 3).
const LENGTH: &str =
    "(length): split the file; give the part with its own purpose its own file and header";

/// What `length` lists, in its heading (design §3.5d kind 3).
const LENGTH_WHAT: &str = "over 500 code lines";

/// The fewest lines a file may have to be listed under `thin` (design §3.5d kind 5).
const THIN_MIN: u32 = 80;

/// What `thin` lists, in its heading (design §3.5d kind 5).
const THIN_WHAT: &str = "files of 80 lines or more, lowest header share first";

/// What the kinds read: the corpus, the scope, the walked entries and the `--limit` of each list.
pub struct Ctx<'a> {
    /// The corpus of the root that is audited.
    pub corpus: &'a Corpus,
    /// The files and folders that the arguments name.
    pub scope: &'a Scope,
    /// Every walked entry of the root, the root included.
    pub tree: &'a [Entry],
    /// How many findings each kind lists.
    pub limit: usize,
}

/// The findings of every kind over the scope, in §3.5d order, with each kind that has none left out. The line counts are read once,
/// for `length` and `thin`. `thin` is listed among the kinds, but it is a pointer and not a finding (design §3.5d).
pub fn all(ctx: &Ctx<'_>) -> Vec<Group> {
    let counted = counts::in_scope(ctx.corpus, ctx.scope);
    [
        headers(ctx),
        readmes(ctx),
        length(ctx, &counted),
        one_liners(ctx),
        thin(ctx, &counted),
        cites(ctx),
        ids(ctx),
        roots::roots(ctx),
    ]
    .into_iter()
    .filter(|group| group.listing.total > 0)
    .collect()
}

/// A kind's group: its items sorted by label (the sort keeps equal labels in the order they came), cut to the limit.
pub(super) fn group(
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
        finding: true,
        listing: Listing::of(&items, ctx.limit),
    }
}

/// The walked files in scope with a comment family, no header and at least one byte (design §3.5d kind 1), as the corpus's head
/// facts give them. A file that cannot be read has none and is not judged, since its read already warned (design §4.2).
fn headers(ctx: &Ctx<'_>) -> Group {
    let items = coverage::expected(ctx.corpus)
        .filter(|(rel, has)| !has && ctx.scope.holds_file(rel))
        .map(|(rel, _)| Item {
            reference: rel.to_string(),
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

/// The walked folders in scope with no README.md whose subtree holds a file with a comment family, the root included (design §3.5d
/// kind 2). A folder of data alone is not a unit and needs none. A folder the walk could not list is not judged, since its children,
/// and so any README in it, are unknown.
fn readmes(ctx: &Ctx<'_>) -> Group {
    let code = code_folders(ctx.tree);
    let items = ctx
        .tree
        .iter()
        .filter(|entry| entry.is_dir && ctx.scope.holds_folder(&entry.rel))
        .filter(|entry| code.contains(entry.rel.as_str()))
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

/// The folders whose subtree holds a walked file with a comment family, the folder itself included, and the root `""` when any does
/// (design §3.5d kind 2). Each such file marks its folders up to the root; a folder already marked ends the climb, since the folders
/// above it were marked with it.
fn code_folders(tree: &[Entry]) -> BTreeSet<&str> {
    let mut folders = BTreeSet::new();
    for entry in tree
        .iter()
        .filter(|entry| !entry.is_dir && has_family(&entry.path))
    {
        let mut folder = parent_of(&entry.rel);
        while let Some(rel) = folder {
            if !folders.insert(rel) {
                break;
            }
            folder = parent_of(rel);
        }
    }
    folders
}

/// The files in scope whose code lines are more than 500, each with its code lines (design §3.5d kind 3).
fn length(ctx: &Ctx<'_>, counted: &[Counted]) -> Group {
    let items = counted
        .iter()
        .filter(|file| file.counts.code() > LENGTH_MAX)
        .map(|file| Item {
            reference: file.rel.clone(),
            detail: Some(format!("{} code lines", file.counts.code())),
        })
        .collect();
    group(ctx, "length", LENGTH_WHAT, Some(LENGTH), items)
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

/// The files in scope of 80 lines or more that have a header, by header lines over total lines, lowest first and ties by path (design
/// §3.5d kind 5). It is a pointer and not a finding: it is listed but not counted, and it has no fix line.
fn thin(ctx: &Ctx<'_>, counted: &[Counted]) -> Group {
    let mut picks: Vec<&Counted> = counted
        .iter()
        .filter(|file| file.counts.total >= THIN_MIN && has_header(ctx.corpus, &file.rel))
        .collect();
    picks.sort_by(|left, right| {
        let (left_header, left_total) =
            (u64::from(left.counts.header), u64::from(left.counts.total));
        let (right_header, right_total) = (
            u64::from(right.counts.header),
            u64::from(right.counts.total),
        );
        (left_header * right_total)
            .cmp(&(right_header * left_total))
            .then_with(|| left.rel.cmp(&right.rel))
    });
    let items: Vec<Item> = picks
        .iter()
        .map(|file| Item {
            reference: file.rel.clone(),
            detail: Some(format!(
                "{}/{} header lines",
                file.counts.header, file.counts.total
            )),
        })
        .collect();
    Group {
        kind: "thin",
        what: THIN_WHAT,
        fix: None,
        finding: false,
        listing: Listing::of(&items, ctx.limit),
    }
}

/// The explicit references that resolve to nothing, in the READMEs, headers and collection docs in scope (design §3.5d kind 6).
/// Logs are not audited. Each doc is read for itself, so two docs that share a label each give theirs, and a reference that both
/// make is listed once.
fn cites(ctx: &Ctx<'_>) -> Group {
    let docs: Vec<&Doc> = ctx
        .corpus
        .all_docs()
        .filter(|doc| !ctx.corpus.is_log(doc) && in_scope(ctx.scope, doc))
        .collect();
    let unresolved = cite::Index::unresolved(ctx.corpus, &docs);
    let mut seen = BTreeSet::new();
    let items = docs
        .iter()
        .zip(unresolved)
        .flat_map(|(doc, refs)| refs.into_iter().map(|raw| (doc.label.as_str(), raw)))
        .filter(|pair| seen.insert(pair.clone()))
        .map(|(label, raw)| Item {
            reference: label.to_string(),
            detail: Some(format!("→ {raw}")),
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
