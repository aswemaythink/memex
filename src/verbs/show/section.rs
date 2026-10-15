//! Builds what `memex show` prints for one target: a folder, a file or a doc (design §3.3).
//! A folder or a file that cannot be read gives its io error, which `show.rs` reports as that ref's error.
//! Its cites and cited by come from the citation index of its one target (`cite::Index::for_show`), not from the index of every doc.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-FILE-03, SHOW-MD-01, SHOW-MD-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-ROOT-01, SHOW-ROOT-02, SHOW-ROOT-03, SHOW-LOG-01, SHOW-LOG-02, SHOW-UP-01, SHOW-UP-02, SHOW-LIMIT-01, SHOW-LIMIT-02, SHOW-LINE-01, SHOW-JSON-02, SHOW-JSON-03, SHOW-IO-01, SHOW-IO-02, CASE-JSON-01, CASE-SECT-01, CITE-BACK-01, CITE-BACK-03, CITE-BACK-04, CITE-COLL-02, CITE-FOLDER-02, CITE-JSON-01, CITE-JSON-02, CITE-MAN-01, CITE-PREFIX-01, CITE-PREFIX-02, CITE-SELF-01, CITE-UNRES-01, SHOW-OTHER-01

use std::fs::File;
use std::io;

use crate::cite::{self, Citer};
use crate::corpus::{self, parent_of, readme_label, Child, Corpus, Doc, Kind};
use crate::intent::cases;
use crate::render::{CaseParts, Citations, Fields, Link, Listing, Ref, Section};
use crate::resolve::Target;
use crate::text;

/// The settings that every section is built with.
#[derive(Clone)]
pub struct Options {
    /// The most READMEs in a chain, or None for the whole chain.
    pub up: Option<usize>,
    /// The most rows of a children list or a cited-by list.
    pub limit: usize,
    /// The root that holds the target, when the target lies in another root. Its citations are then not computed, since they name
    /// labels of that root (design §3.1). None for a target of the current root.
    pub other_root: Option<String>,
}

/// The section of one resolved target. A folder or a file that cannot be read gives its io error (design §3.3).
pub fn section_for(
    corpus: &Corpus,
    target: &Target<'_>,
    options: &Options,
) -> Result<Section, io::Error> {
    match target {
        Target::Path {
            rel,
            kind: Kind::Dir,
        } => folder_section(corpus, rel, options),
        Target::Path {
            rel,
            kind: Kind::File,
        } => file_section(corpus, rel, options),
        Target::Doc(doc) => Ok(doc_section(corpus, doc, options)),
    }
}

/// The section of a folder: its README or `(no README)`, its children other than that README, its citations, and the chain
/// above it. A folder that the walk could not list is an io error for its ref, and no part of its section is built.
fn folder_section(corpus: &Corpus, rel: &str, options: &Options) -> Result<Section, io::Error> {
    if let Some(kind) = corpus.unlistable(rel) {
        return Err(io::Error::from(kind));
    }
    let label = corpus::label_of(rel);
    let readme = corpus.doc(&readme_label(rel));
    let children = listed_children(corpus, rel);
    Ok(Section {
        label: label.clone(),
        title: readme.map(|doc| doc.title.clone()),
        collection: None,
        id: None,
        path: Some(label.clone()),
        text: readme.map(|doc| doc.text.clone()),
        missing: "(no README)",
        fields: None,
        children: Some(Listing::of(&children, options.limit)),
        citations: citations(
            corpus,
            options,
            readme.map(|doc| doc.label.as_str()),
            &label,
        ),
        chain: chain(corpus, parent_of(rel), options),
    })
}

/// The children that a folder's section lists: all of them but its own README.md, which the body has already printed
/// (design §3.3).
fn listed_children(corpus: &Corpus, rel: &str) -> Vec<Child> {
    let own = readme_label(rel);
    corpus
        .children(rel)
        .into_iter()
        .filter(|child| !(child.kind == Kind::File && child.rel == own))
        .collect()
}

/// The section of a file: its header, or its whole text for a Markdown file, and the chain above its folder. A README
/// starts its chain one folder higher, since its own text is the body. A file that cannot be read gives its io error.
fn file_section(corpus: &Corpus, rel: &str, options: &Options) -> Result<Section, io::Error> {
    let doc = corpus.doc(rel);
    let path = corpus.root.join(rel);
    let text = if rel.ends_with(".md") {
        Some(text::read(&path)?)
    } else {
        File::open(&path)?;
        doc.map(|doc| doc.text.clone())
    };
    let folder = parent_of(rel).unwrap_or("");
    let start = if is_readme(rel) {
        parent_of(folder)
    } else {
        Some(folder)
    };
    let key = file_key(corpus, options, rel);
    Ok(Section {
        label: rel.to_string(),
        title: doc.map(|doc| doc.title.clone()),
        collection: doc.map(|doc| doc.collection.clone()),
        id: Some(rel.to_string()),
        path: Some(rel.to_string()),
        text,
        missing: "(no header)",
        fields: None,
        children: None,
        citations: citations(
            corpus,
            options,
            Some(doc.map_or(key.as_str(), |doc| doc.label.as_str())),
            &key,
        ),
        chain: chain(corpus, start, options),
    })
}

/// The label that a file's citations and backlinks are keyed by: a manifest doc's `collection:id` label, else its path (design
/// §2.3). Only a listed root's citations are read, so another root's manifest is never looked up for this (design §3.1).
fn file_key(corpus: &Corpus, options: &Options, rel: &str) -> String {
    match &options.other_root {
        Some(_) => rel.to_string(),
        None => cite::Index::manifest_label(corpus, rel).unwrap_or_else(|| rel.to_string()),
    }
}

/// The section of a doc, such as a log or a case: its text, its fields, and its citations, with no chain.
fn doc_section(corpus: &Corpus, doc: &Doc, options: &Options) -> Section {
    Section {
        label: doc.label.clone(),
        title: Some(doc.title.clone()),
        collection: Some(doc.collection.clone()),
        id: Some(doc.id.clone()),
        path: doc.path.clone(),
        text: Some(doc.text.clone()),
        missing: "(no text)",
        fields: Some(doc_fields(corpus, doc)),
        children: None,
        // A case shows cited by and no cites (design §3.3 Case), so its JSON cites list is empty too.
        citations: citations(
            corpus,
            options,
            (doc.collection != corpus::CASES).then_some(doc.label.as_str()),
            &doc.label,
        ),
        chain: None,
    }
}

/// The fields of a doc: a case's group and the evidence its text names that the walk holds (design §2.2), or the
/// frontmatter of any other doc.
fn doc_fields(corpus: &Corpus, doc: &Doc) -> Fields {
    if doc.collection == corpus::CASES {
        Fields::Case(CaseParts {
            group: cases::group_of(doc),
            evidence: cases::evidence(doc, |rel| corpus.kind(rel) == Some(Kind::File)),
        })
    } else {
        Fields::Pairs(doc.fields.clone())
    }
}

/// The READMEs from `start` up to the root, nearest first, at most `--up` of them. None when `--up 0` prints no chain.
fn chain(corpus: &Corpus, start: Option<&str>, options: &Options) -> Option<Vec<Link>> {
    if options.up == Some(0) {
        return None;
    }
    let mut links: Vec<Link> = Vec::new();
    let mut folder = start;
    while let Some(dir) = folder {
        if options.up.is_some_and(|up| links.len() >= up) {
            break;
        }
        if let Some(doc) = corpus.doc(&readme_label(dir)) {
            links.push(Link {
                path: doc.label.clone(),
                text: doc.text.clone(),
            });
        }
        folder = parent_of(dir);
    }
    Some(links)
}

/// The citations of a target: the cites of the doc labelled `cites_label` (none for None, as for a case) and the refs that cite
/// `cited_by_label`, read from the index that `show` builds for this one target (design §2.3). For an entry of another root
/// neither is computed, since they name labels of that root, so the root is printed in their place (design §3.1).
fn citations(
    corpus: &Corpus,
    options: &Options,
    cites_label: Option<&str>,
    cited_by_label: &str,
) -> Citations {
    match &options.other_root {
        Some(root) => Citations::OtherRoot { root: root.clone() },
        None => {
            let index = cite::Index::for_show(corpus, cites_label, cited_by_label);
            Citations::Listed {
                cites: cites_label.map_or_else(Vec::new, |label| index.cites(label).to_vec()),
                cited_by: cited_by(index.cited_by(cited_by_label), options.limit),
            }
        }
    }
}

/// The docs that cite a target, each with its one-liner and the prefix it cites by, with the limit applied.
fn cited_by(citers: &[Citer], limit: usize) -> Listing<Ref> {
    let refs: Vec<Ref> = citers
        .iter()
        .map(|citer| Ref {
            label: citer.label.clone(),
            title: Some(citer.title.clone()),
            via: citer.via.clone(),
        })
        .collect();
    Listing::of(&refs, limit)
}

/// True for a README.md, at the root or in a folder.
fn is_readme(rel: &str) -> bool {
    rel == "README.md" || rel.ends_with("/README.md")
}
