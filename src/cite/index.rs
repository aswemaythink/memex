//! Builds the citation index (design §2.3): each doc's cites and the backlinks they give.
//! `build` is the full index that search's roster reads. `for_show` is what `show` reads: the backlinks of its targets and the cites
//! of its docs, found by one scan that resolves only the tokens that can name a target (`paths::Wanted`).
//! Cases: CITE-BARE-01, CITE-BARE-02, CITE-BARE-03, CITE-PREFIX-01, CITE-PREFIX-02, CITE-COLL-01, CITE-COLL-02, CITE-COLL-03, CITE-COLL-04, CITE-PATH-01, CITE-PATH-02, CITE-PATH-03, CITE-PATH-04, CITE-PATH-05, CITE-PATH-06, CITE-LINK-01, CITE-LINK-02, CITE-LINK-03, CITE-LINK-05, CITE-MAN-01, CITE-UNRES-01, CITE-FOLDER-01, CITE-FOLDER-02, CITE-FENCE-01, CITE-SELF-01, CITE-BACK-01, CITE-BACK-02, CITE-BACK-03, CITE-BACK-04, CITE-BACK-05, SRCH-ORPHAN-01, SRCH-ORPHAN-02

use std::collections::{BTreeMap, BTreeSet};

use super::paths::{self, is_intent, Hit, Names, Wanted};
use super::{Cite, Citer};
use crate::corpus::{label_of, parent_of, Corpus, Doc};
use crate::par;

/// The citations that docs make, and the backlinks that targets get (design §2.3).
#[derive(Debug, Default)]
pub struct Index {
    /// The citations of each citing doc, by its label, in first-appearance order. Only `for_show` fills it.
    cites: BTreeMap<String, Vec<Cite>>,
    /// The docs that cite each target, by the target's label, in label order of the citing doc.
    citers: BTreeMap<String, Vec<Citer>>,
}

/// The hits of one doc, reduced to the citations it makes and the targets it cites by prefix.
#[derive(Default)]
struct Gathered {
    /// The citations, in first-appearance order.
    cites: Vec<Cite>,
    /// The targets it cites directly.
    direct: BTreeSet<String>,
    /// The targets it cites by prefix, each with the prefix that names it first.
    families: BTreeMap<String, String>,
}

impl Index {
    /// The backlinks of every target in `corpus`, for search's roster (design §2.3). The docs are scanned and reduced on several
    /// threads, then recorded in label order, so each target's backlinks come out as a sequential scan gives them.
    pub fn build(corpus: &Corpus) -> Index {
        let docs: Vec<&Doc> = corpus.all_docs().collect();
        let names = Names::of(&docs);
        let gathered = par::map(&docs, |doc| {
            gather(corpus, doc, paths::hits_of(corpus, doc, &names, None))
        });
        let mut index = Index::default();
        for (doc, gathered) in docs.into_iter().zip(gathered) {
            index.record(doc, &gathered, |_| true);
        }
        index
    }

    /// The index that the `show` sections of several targets read (design §2.3): the cites of each doc labelled in `cites`, and the
    /// backlinks of each target labelled in `cited_by`. Each doc's tokens are scanned once, on several threads, and only those that
    /// can name one of the targets are resolved; the cites docs' tokens are all resolved. The backlinks are those that `build` lists
    /// for the same targets, and the cites are those a full scan of the doc makes. With no labels, no scan runs.
    pub fn for_show(corpus: &Corpus, cites: &[&str], cited_by: &[&str]) -> Index {
        let mut index = Index::default();
        if cites.is_empty() && cited_by.is_empty() {
            return index;
        }
        let docs: Vec<&Doc> = corpus.all_docs().collect();
        let names = Names::of(&docs);
        let targets: BTreeSet<&str> = cited_by.iter().copied().collect();
        let wanted = Wanted::of(corpus, &names, &targets);
        let gathered = par::map(&docs, |doc| {
            gather(
                corpus,
                doc,
                paths::hits_of(corpus, doc, &names, Some(&wanted)),
            )
        });
        for (doc, gathered) in docs.iter().zip(&gathered) {
            index.record(doc, gathered, |label| targets.contains(label));
        }
        let cited: BTreeSet<&str> = cites.iter().copied().collect();
        for label in cited {
            if let Some(list) = cites_of(corpus, &names, label) {
                index.cites.insert(label.to_string(), list);
            }
        }
        index
    }

    /// The label of the manifest doc at this root-relative path, when no intent doc holds the path (design §2.3). It reads only the
    /// docs' paths, so `show` finds it without building a citation index.
    pub fn manifest_label(corpus: &Corpus, rel: &str) -> Option<String> {
        let at_rel: Vec<&Doc> = corpus
            .all_docs()
            .filter(|doc| doc.path.as_deref() == Some(rel))
            .collect();
        if at_rel.iter().any(|doc| is_intent(doc)) {
            return None;
        }
        at_rel
            .into_iter()
            .find(|doc| !is_intent(doc))
            .map(|doc| doc.label.clone())
    }

    /// The citations a doc makes, in first-appearance order. Empty for a doc that cites nothing, and for a label that is no doc.
    pub fn cites(&self, label: &str) -> &[Cite] {
        self.cites.get(label).map_or(&[], Vec::as_slice)
    }

    /// The docs that cite a target, in label order. A doc that cites it only by prefix is marked with the prefix. Roster `<-N`
    /// counts the entries with no prefix, since prefix citations are not counted there (design §2.3).
    pub fn cited_by(&self, label: &str) -> &[Citer] {
        self.citers.get(label).map_or(&[], Vec::as_slice)
    }

    /// The number of docs that cite a target directly: `cited_by` without its prefix citations, which is the `<-N` of a roster
    /// (design §2.3).
    pub fn direct_count(&self, label: &str) -> usize {
        self.cited_by(label)
            .iter()
            .filter(|citer| citer.via.is_none())
            .count()
    }

    /// Records the backlinks that one doc's hits give to the targets that `keep` admits: a direct citation, or a prefix citation
    /// that the doc makes no direct one for. Docs that share a label are next to each other in the scan, so a label's first doc
    /// keeps its backlink and a repeat is not listed twice.
    fn record(&mut self, doc: &Doc, gathered: &Gathered, keep: impl Fn(&str) -> bool) {
        for label in gathered.direct.iter().filter(|label| keep(label)) {
            add_citer(self.citers.entry(label.clone()).or_default(), doc, None);
        }
        for (label, prefix) in &gathered.families {
            if keep(label) && !gathered.direct.contains(label) {
                let via = Some(format!("{prefix}*"));
                add_citer(self.citers.entry(label.clone()).or_default(), doc, via);
            }
        }
    }
}

/// The citations of the doc labelled `label`: those of its first doc that makes any. Its tokens are all resolved, since its
/// unresolved refs are citations too (design §2.3).
fn cites_of(corpus: &Corpus, names: &Names, label: &str) -> Option<Vec<Cite>> {
    corpus
        .labelled(label)
        .iter()
        .map(|doc| gather(corpus, doc, paths::hits_of(corpus, doc, names, None)).cites)
        .find(|cites| !cites.is_empty())
}

/// The label of the folder that a README is the doc of. A README never cites its own folder (design §2.3, self-citation).
fn own_folder(doc: &Doc) -> Option<String> {
    let path = doc.path.as_deref()?;
    doc.is_readme()
        .then(|| label_of(parent_of(path).unwrap_or("")))
}

/// Reduces one doc's hits: each target once, in first-appearance order, with the doc's own label and folder left out. It reads
/// only the corpus, so the docs are reduced on the worker threads.
fn gather(corpus: &Corpus, doc: &Doc, hits: Vec<Hit>) -> Gathered {
    let own = own_folder(doc);
    let is_self = |label: &str| label == doc.label || own.as_deref() == Some(label);
    let mut gathered = Gathered::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for hit in hits {
        match hit {
            Hit::Target(label) if is_self(&label) => {}
            Hit::Target(label) => add_target(corpus, label, &mut gathered, &mut seen),
            Hit::Unresolved(raw) => add_unresolved(raw, &mut gathered, &mut seen),
            Hit::Family { label, prefix } => {
                if !is_self(&label) {
                    gathered.families.entry(label).or_insert(prefix);
                }
            }
        }
    }
    gathered
}

/// Adds a target that the doc cites directly: its citation the first time it appears, and its label to the direct targets.
fn add_target(
    corpus: &Corpus,
    label: String,
    gathered: &mut Gathered,
    seen: &mut BTreeSet<String>,
) {
    if seen.insert(label.clone()) {
        gathered.cites.push(Cite {
            title: paths::title_of(corpus, &label),
            reference: label.clone(),
            resolved: true,
        });
    }
    gathered.direct.insert(label);
}

/// Adds an unresolved ref, as a citation that names nothing, the first time it appears.
fn add_unresolved(raw: String, gathered: &mut Gathered, seen: &mut BTreeSet<String>) {
    if seen.insert(raw.clone()) {
        gathered.cites.push(Cite {
            reference: raw,
            title: String::new(),
            resolved: false,
        });
    }
}

/// Adds the backlink that `doc` makes to a target's list, unless the previous entry is the same doc.
fn add_citer(list: &mut Vec<Citer>, doc: &Doc, via: Option<String>) {
    if list.last().is_none_or(|last| last.label != doc.label) {
        list.push(Citer {
            label: doc.label.clone(),
            title: doc.title.clone(),
            via,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// The case folders of tests/cases/cite, each a small intent layer, loaded as corpora.
    fn fixtures() -> Vec<Corpus> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/cite");
        let mut corpora = Vec::new();
        for entry in fs::read_dir(root).expect("the cite fixtures") {
            let path = entry.expect("a fixture entry").path();
            if path.is_dir() {
                corpora.push(Corpus::load(&path, None));
            }
        }
        corpora
    }

    /// The labels that a `show` can print in a corpus: every doc's, and every walked entry's with its folders.
    fn every_label(corpus: &Corpus) -> Vec<String> {
        let mut labels: BTreeSet<String> = corpus.all_docs().map(|doc| doc.label.clone()).collect();
        for rel in corpus.files() {
            labels.insert(rel.to_string());
            let mut at = rel;
            while let Some(parent) = parent_of(at) {
                labels.insert(label_of(parent));
                at = parent;
            }
        }
        labels.into_iter().collect()
    }

    #[test]
    fn a_batched_index_lists_each_target_as_the_full_index_and_a_single_target_index_do() {
        let mut checked = 0;
        for corpus in fixtures() {
            let full = Index::build(&corpus);
            let labels = every_label(&corpus);
            let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
            let batched = Index::for_show(&corpus, &refs, &refs);
            for label in &labels {
                let single = Index::for_show(&corpus, &[label.as_str()], &[label.as_str()]);
                assert_eq!(batched.cited_by(label), full.cited_by(label), "{label}");
                assert_eq!(batched.cited_by(label), single.cited_by(label), "{label}");
                assert_eq!(batched.cites(label), single.cites(label), "{label}");
                checked += 1;
            }
        }
        assert!(checked > 100, "only {checked} targets were checked");
    }
}
