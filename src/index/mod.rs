//! The in-memory BM25F index of one group of docs, with stems, prefixes and one-edit matches.
//! Cases: SRCH-RANK-01, SRCH-RANK-02, SRCH-WEIGHT-01, SRCH-WEIGHT-02, SRCH-STEM-01, SRCH-PREFIX-01, SRCH-PREFIX-02, SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-TIE-01, SRCH-TOKEN-01, SRCH-TOKEN-02, MAN-LOAD-13

pub mod bm25;
pub mod filter;
pub mod fuzzy;
pub mod tokenize;
mod vocab;

use std::collections::{BTreeMap, HashMap};
use std::ops::Bound;

use rust_stemmers::{Algorithm, Stemmer};

use crate::par;
use vocab::{Posting, Vocab};

/// The shortest query word that may match by prefix (PLAN §4).
const PREFIX_MIN: usize = 3;

/// The shortest query word that may match by one edit (PLAN §4).
const FUZZY_MIN: usize = 5;

/// The tiers of a match are indexes, best first: 0 for an exact or stem match, 1 for a prefix match, and `FUZZY` (2) for a one-edit
/// match (PLAN §4). `MULTIPLIER` gives each tier's multiplier, which scales a term's weighted frequency before it saturates.
const FUZZY: usize = 2;
const MULTIPLIER: [f64; 3] = [1.0, 0.5, 0.3];

/// The size that scores are rounded to before they are compared, so that ranking does not depend on the platform's libm (design §3.2).
const QUANTUM: f64 = 1e9;

/// A doc as the index reads it: its label, which breaks ties, and the text of its three fields.
pub struct Source<'a> {
    /// The label, which ranking breaks ties by.
    pub label: &'a str,
    /// The id field: the doc's id, or its path.
    pub id: &'a str,
    /// The title field: the doc's one-liner.
    pub title: &'a str,
    /// The body field: the doc's text, cut by the caller where the index stops reading.
    pub body: &'a str,
}

/// One hit: the position of its doc among the sources, its score, and the index terms that matched a query word by one edit.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// The position of the doc among the sources given to `Index::build`.
    pub doc: usize,
    /// The BM25F score of the doc for the query.
    pub score: f64,
    /// The index terms that matched by one edit, sorted, with no repeats.
    pub fuzzy: Vec<String>,
}

/// The index of one group: the labels, the normalisers of each doc's fields, the postings of each term, the plain terms of
/// each stem, and the field weights that the group ranks with.
pub struct Index {
    labels: Vec<String>,
    norms: Vec<[f64; 3]>,
    vocab: BTreeMap<String, Vec<Posting>>,
    stems: HashMap<String, Vec<String>>,
    stemmer: Stemmer,
    weights: [f64; 3],
}

/// What one query word matched in one doc: its best tier, the weighted frequency of its terms in that tier, and the fuzzy terms.
#[derive(Clone, Default)]
struct Match {
    tier: Option<usize>,
    weight: f64,
    fuzzy: Vec<String>,
}

/// The running totals of a query: each doc's score, whether it matched, and its fuzzy terms.
struct Totals {
    scores: Vec<f64>,
    matched: Vec<bool>,
    fuzzy: Vec<Vec<String>>,
}

impl Totals {
    /// Empty totals for `docs` docs.
    fn new(docs: usize) -> Totals {
        Totals {
            scores: vec![0.0; docs],
            matched: vec![false; docs],
            fuzzy: vec![Vec::new(); docs],
        }
    }
}

impl Index {
    /// Indexes the sources in order. Each doc's tokens are counted per field, and the field lengths give the normalisers. The docs
    /// are cut into runs, one per thread, which are counted at once and then joined in order. The group ranks with `weights`, the
    /// field weights of its manifest or the defaults (design §4.2).
    pub fn build(sources: &[Source], weights: [f64; 3]) -> Index {
        let stemmer = Stemmer::create(Algorithm::English);
        let runs = par::map(&runs_of(sources), |(first, run)| count_run(*first, run));
        let (terms, lengths) = join_runs(runs);
        let vocab = terms.into_map();
        Index {
            labels: sources
                .iter()
                .map(|source| source.label.to_string())
                .collect(),
            norms: bm25::norms(&lengths),
            stems: stem_map(&vocab, &stemmer),
            vocab,
            stemmer,
            weights,
        }
    }

    /// The hits of a query, best first, with ties broken by label. A doc that no word matches is not a hit.
    pub fn search(&self, words: &[String]) -> Vec<Hit> {
        let mut totals = Totals::new(self.labels.len());
        for word in words {
            self.score_word(word, &mut totals);
        }
        self.ranked(totals)
    }

    /// Adds one query word's score to every doc that it matches.
    fn score_word(&self, word: &str, totals: &mut Totals) {
        let found = self.matches(word);
        let df = found.iter().filter(|hit| hit.tier.is_some()).count();
        let idf = bm25::idf(self.labels.len(), df);
        for (doc, found) in found.into_iter().enumerate() {
            let Some(tier) = found.tier else {
                continue;
            };
            totals.scores[doc] += idf * bm25::saturate(MULTIPLIER[tier] * found.weight);
            totals.matched[doc] = true;
            if tier == FUZZY {
                totals.fuzzy[doc].extend(found.fuzzy);
            }
        }
    }

    /// What a query word matches in each doc: its best tier, and the weighted frequency of its terms in that tier.
    fn matches(&self, word: &str) -> Vec<Match> {
        let mut found = vec![Match::default(); self.labels.len()];
        for (tier, terms) in self.candidates(word).iter().enumerate() {
            for term in terms {
                self.add_term(term, tier, &mut found);
            }
        }
        found
    }

    /// Adds the postings of one index term to the matches of its tier. A doc that a better tier already matched keeps that
    /// tier, so a prefix or a fuzzy match counts only where no exact or stem match exists (PLAN §4).
    fn add_term(&self, term: &str, tier: usize, found: &mut [Match]) {
        let Some(postings) = self.vocab.get(term) else {
            return;
        };
        for posting in postings {
            let known = &mut found[posting.doc];
            if known.tier.is_some_and(|best| best < tier) {
                continue;
            }
            known.tier = Some(tier);
            known.weight += bm25::weight(posting.tf, self.norms[posting.doc], self.weights);
            if tier == FUZZY {
                known.fuzzy.push(term.to_string());
            }
        }
    }

    /// The index terms that a query word may match, in three lists: the stem-mates (exact or stem), the prefixes, and the
    /// one-edit variants. A whole id or path has only the exact list, which is the term itself.
    fn candidates(&self, word: &str) -> [Vec<&str>; 3] {
        if tokenize::is_whole(word) {
            let exact = self
                .vocab
                .get_key_value(word)
                .map(|(term, _)| term.as_str());
            return [exact.into_iter().collect(), Vec::new(), Vec::new()];
        }
        let stem = self.stemmer.stem(word);
        let stem_mates: Vec<&str> = self
            .stems
            .get(&*stem)
            .map_or_else(Vec::new, |terms| terms.iter().map(String::as_str).collect());
        [stem_mates, self.prefixed(word), self.one_edit(word)]
    }

    /// The plain terms that start with a query word of `PREFIX_MIN` characters or more.
    fn prefixed(&self, word: &str) -> Vec<&str> {
        if word.len() < PREFIX_MIN {
            return Vec::new();
        }
        self.vocab
            .range::<str, _>((Bound::Included(word), Bound::Unbounded))
            .map(|(term, _)| term.as_str())
            .take_while(|term| term.starts_with(word))
            .filter(|term| !tokenize::is_whole(term))
            .collect()
    }

    /// The plain terms one edit away from a query word of `FUZZY_MIN` characters or more. This scans the vocabulary once for the
    /// word, not once per doc.
    fn one_edit(&self, word: &str) -> Vec<&str> {
        if word.len() < FUZZY_MIN {
            return Vec::new();
        }
        self.vocab
            .keys()
            .map(String::as_str)
            .filter(|term| !tokenize::is_whole(term) && fuzzy::one_edit(term, word))
            .collect()
    }

    /// The hits in rank order: by score rounded to `QUANTUM`, highest first, then by label.
    fn ranked(&self, mut totals: Totals) -> Vec<Hit> {
        let mut hits: Vec<Hit> = (0..self.labels.len())
            .filter(|&doc| totals.matched[doc])
            .map(|doc| Hit {
                doc,
                score: totals.scores[doc],
                fuzzy: sorted(std::mem::take(&mut totals.fuzzy[doc])),
            })
            .collect();
        hits.sort_by(|a, b| {
            quantize(b.score)
                .cmp(&quantize(a.score))
                .then_with(|| self.labels[a.doc].cmp(&self.labels[b.doc]))
        });
        hits
    }
}

/// Adds one doc's terms to the vocabulary, and returns its token count in each field.
fn add_doc(terms: &mut Vocab, doc: usize, source: &Source) -> [f64; 3] {
    let mut length = [0.0; 3];
    for (field, text) in [source.id, source.title, source.body]
        .into_iter()
        .enumerate()
    {
        tokenize::for_each_token(text, |token| {
            length[field] += 1.0;
            terms.count(doc, field, token);
        });
    }
    length
}

/// The sources cut into runs of consecutive docs, one run per thread, each with the position of its first doc.
fn runs_of<'a, 'b>(sources: &'a [Source<'b>]) -> Vec<(usize, &'a [Source<'b>])> {
    let size = sources.len().div_ceil(par::threads()).max(1);
    sources
        .chunks(size)
        .enumerate()
        .map(|(run, docs)| (run * size, docs))
        .collect()
}

/// The vocabulary of one run of docs, and the token count of each doc in it, with its docs numbered from `first`.
fn count_run(first: usize, sources: &[Source]) -> (Vocab, Vec<[f64; 3]>) {
    let mut terms = Vocab::default();
    let lengths = sources
        .iter()
        .enumerate()
        .map(|(offset, source)| add_doc(&mut terms, first + offset, source))
        .collect();
    (terms, lengths)
}

/// The runs joined in order: one vocabulary, and the token count of every doc.
fn join_runs(runs: Vec<(Vocab, Vec<[f64; 3]>)>) -> (Vocab, Vec<[f64; 3]>) {
    let mut terms = Vocab::default();
    let mut lengths = Vec::new();
    for (run, counts) in runs {
        terms.append(run);
        lengths.extend(counts);
    }
    (terms, lengths)
}

/// The plain terms of the vocabulary by their stem, so that a query word finds its stem-mates in one lookup.
fn stem_map(
    vocab: &BTreeMap<String, Vec<Posting>>,
    stemmer: &Stemmer,
) -> HashMap<String, Vec<String>> {
    let mut stems: HashMap<String, Vec<String>> = HashMap::new();
    for term in vocab.keys().filter(|term| !tokenize::is_whole(term)) {
        stems
            .entry(stemmer.stem(term).into_owned())
            .or_default()
            .push(term.clone());
    }
    stems
}

/// A score rounded to the quantum, so that scores that differ only in the last digits of their floats compare equal.
fn quantize(score: f64) -> i64 {
    (score * QUANTUM).round() as i64
}

/// The terms in byte order, with no repeats.
fn sorted(mut terms: Vec<String>) -> Vec<String> {
    terms.sort();
    terms.dedup();
    terms
}
