//! The in-memory BM25F index of one group of docs, with stems, prefixes and one-edit matches.
//! A query builds only what it needs: one scan over each run's distinct terms finds the candidates of every query word, and only
//! those terms' postings are read, run by run in doc order. The full join of the runs into one vocabulary, and the stem map of
//! every term, are gone (perf P-IDX); the matches, weights and ranking are the same.
//! Cases: SRCH-RANK-01, SRCH-RANK-02, SRCH-WEIGHT-01, SRCH-WEIGHT-02, SRCH-STEM-01, SRCH-PREFIX-01, SRCH-PREFIX-02, SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-TIE-01, SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-BOUNDARY-01, SRCH-BOUNDARY-02, MAN-LOAD-13, SRCH-UNICODE-01, SRCH-UNICODE-02, SRCH-UNICODE-03

pub mod bm25;
pub mod filter;
pub mod fuzzy;
mod hold;
pub mod tokenize;
mod vocab;

use std::mem::ManuallyDrop;

use rust_stemmers::{Algorithm, Stemmer};

use crate::par;
use vocab::{Posting, Vocab};

/// The shortest query word that may match by prefix.
const PREFIX_MIN: usize = 3;

/// The shortest query word that may match by one edit.
const FUZZY_MIN: usize = 5;

/// The tiers of a match are indexes, best first: 0 for an exact or stem match, 1 for a prefix match, and `FUZZY` (2) for a one-edit
/// match. `MULTIPLIER` gives each tier's multiplier, which scales a term's weighted frequency before it saturates.
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

/// The index of one group: the labels, the normalisers of each doc's fields, the distinct terms of each run of docs with their
/// postings, the stemmer that query words and terms are stemmed with, and the field weights that the group ranks with.
pub struct Index<'a> {
    labels: Vec<&'a str>,
    norms: Vec<[f64; 3]>,
    /// Never freed: a search builds one index per group and the verb ends with the process, and freeing the many small term strings
    /// and postings of a large group costs about 20 ms for nothing (perf P-IDX).
    runs: ManuallyDrop<Vec<Vocab>>,
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

/// A term that query word `word` (its position in the query) may match at a tier, and where the term's postings are: its id in run `run`.
struct Candidate<'a> {
    word: usize,
    tier: usize,
    term: &'a str,
    run: usize,
    id: usize,
}

/// A query word as the scan reads it: its stem, which only a plain word has (a whole id or path matches itself and the longer
/// ids and paths it begins), and whether it may match by prefix or by one edit.
struct Probe<'w> {
    word: &'w str,
    stem: Option<String>,
    prefix: bool,
    fuzzy: bool,
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

impl<'a> Index<'a> {
    /// Indexes the sources in order. Each doc's tokens are counted per field, and the field lengths give the normalisers. The docs
    /// are cut into runs, one per thread, which are counted at once and kept as they are. The group ranks with `weights`, the field
    /// weights of its manifest or the defaults (design §4.2).
    pub fn build(sources: &[Source<'a>], weights: [f64; 3]) -> Index<'a> {
        let counted = par::map(&runs_of(sources), |(first, run)| count_run(*first, run));
        let (runs, counts): (Vec<Vocab>, Vec<Vec<[f64; 3]>>) = counted.into_iter().unzip();
        let lengths: Vec<[f64; 3]> = counts.into_iter().flatten().collect();
        Index {
            labels: sources.iter().map(|source| source.label).collect(),
            norms: bm25::norms(&lengths),
            runs: ManuallyDrop::new(runs),
            stemmer: Stemmer::create(Algorithm::English),
            weights,
        }
    }

    /// The hits of a query, best first, with ties broken by label. A doc that no word matches is not a hit.
    pub fn search(&self, words: &[String]) -> Vec<Hit> {
        let mut totals = Totals::new(self.labels.len());
        for candidates in self.candidates(words) {
            let found = self.matches(&candidates);
            self.score_word(found, &mut totals);
        }
        self.ranked(totals)
    }

    /// Adds one query word's score to every doc that it matches, given what the word matched in each doc.
    fn score_word(&self, found: Vec<Match>, totals: &mut Totals) {
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

    /// What a word's candidates match in each doc: its best tier, and the weighted frequency of its terms in that tier. The
    /// candidates come in tier order and then byte order of their terms, so each doc's sum is taken in the same order as before.
    fn matches(&self, candidates: &[Candidate]) -> Vec<Match> {
        let mut found = vec![Match::default(); self.labels.len()];
        for candidate in candidates {
            let postings = self.runs[candidate.run].postings(candidate.id);
            self.add_term(candidate, postings, &mut found);
        }
        found
    }

    /// Adds one candidate's postings in one run to the matches of its tier. A doc that a better tier already matched keeps that
    /// tier, so a prefix or a fuzzy match counts only where no exact or stem match exists.
    fn add_term(&self, candidate: &Candidate, postings: &[Posting], found: &mut [Match]) {
        for posting in postings {
            let known = &mut found[posting.doc];
            if known.tier.is_some_and(|best| best < candidate.tier) {
                continue;
            }
            known.tier = Some(candidate.tier);
            known.weight += bm25::weight(posting.tf, self.norms[posting.doc], self.weights);
            if candidate.tier == FUZZY {
                known.fuzzy.push(candidate.term.to_string());
            }
        }
    }

    /// The candidates of each query word: its terms in every run, sorted by tier, then by term in byte order, then by run. The runs
    /// are scanned on several threads, in the order of the runs.
    fn candidates(&self, words: &[String]) -> Vec<Vec<Candidate<'_>>> {
        let probes: Vec<Probe> = words
            .iter()
            .map(|word| Probe::new(word, &self.stemmer))
            .collect();
        let runs: Vec<usize> = (0..self.runs.len()).collect();
        let mut per_word: Vec<Vec<Candidate>> = (0..words.len()).map(|_| Vec::new()).collect();
        for candidate in par::map(&runs, |&run| self.scan_run(run, &probes))
            .into_iter()
            .flatten()
        {
            per_word[candidate.word].push(candidate);
        }
        for candidates in &mut per_word {
            candidates.sort_unstable_by_key(|c| (c.tier, c.term, c.run));
        }
        per_word
    }

    /// The candidates of every query word in one run. A whole id or path is looked up by its exact term, and each whole term is
    /// checked against the longer ids and paths that a word begins (design §3.2). A plain term is stemmed once, and only when some
    /// query word is plain, and it is a candidate at each tier it reaches.
    fn scan_run(&self, run: usize, probes: &[Probe]) -> Vec<Candidate<'_>> {
        let vocab = &self.runs[run];
        let mut found = Vec::new();
        for (word, probe) in probes.iter().enumerate() {
            if probe.stem.is_none() {
                if let Some((term, id)) = vocab.get(probe.word) {
                    found.push(Candidate {
                        word,
                        tier: 0,
                        term,
                        run,
                        id,
                    });
                }
            }
        }
        let plain = probes.iter().any(|probe| probe.stem.is_some());
        for (term, id) in vocab.terms() {
            if tokenize::is_whole(term) {
                for (word, probe) in probes.iter().enumerate() {
                    if probe.matches_longer(term) {
                        found.push(Candidate {
                            word,
                            tier: 0,
                            term,
                            run,
                            id,
                        });
                    }
                }
            } else if plain {
                let stem = self.stemmer.stem(term);
                for (word, probe) in probes.iter().enumerate() {
                    for tier in probe.tiers(term, &stem).into_iter().flatten() {
                        found.push(Candidate {
                            word,
                            tier,
                            term,
                            run,
                            id,
                        });
                    }
                }
            }
        }
        found
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
                .then_with(|| self.labels[a.doc].cmp(self.labels[b.doc]))
        });
        hits
    }
}

impl<'w> Probe<'w> {
    /// The probe of a query word: a whole id or path has no stem, so it matches whole terms only, its exact term by lookup and the
    /// longer ones by `matches_longer`; a plain word may match by prefix from `PREFIX_MIN` characters and by one edit from `FUZZY_MIN`.
    fn new(word: &'w str, stemmer: &Stemmer) -> Probe<'w> {
        let whole = tokenize::is_whole(word);
        Probe {
            word,
            stem: (!whole).then(|| stemmer.stem(word).into_owned()),
            prefix: !whole && !shorter_than(word, PREFIX_MIN),
            fuzzy: !whole && !shorter_than(word, FUZZY_MIN),
        }
    }

    /// True when a whole term is a longer whole id or path that this whole word begins, with the next character `-`, `/` or `.`
    /// (design §3.2). The exact term is not one: `scan_run` finds it by lookup, so it is not counted twice.
    fn matches_longer(&self, term: &str) -> bool {
        self.stem.is_none()
            && term
                .strip_prefix(self.word)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|next| matches!(next, '-' | '/' | '.'))
    }

    /// The tiers at which a plain term matches this word, each `Some` in tier order: a stem-mate, a prefix, or a one-edit variant.
    /// A prefix includes the word itself, since the word starts with itself; the best tier then keeps it at its stem tier.
    fn tiers(&self, term: &str, stem: &str) -> [Option<usize>; 3] {
        [
            (self.stem.as_deref() == Some(stem)).then_some(0),
            (self.prefix && term.starts_with(self.word)).then_some(1),
            (self.fuzzy && fuzzy::one_edit(term, self.word)).then_some(2),
        ]
    }
}

/// Adds one doc's terms to a run's vocabulary, and returns its token count in each field.
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

/// The distinct terms of one run of docs, and the token count of each doc in it, with its docs numbered from `first`.
fn count_run(first: usize, sources: &[Source]) -> (Vocab, Vec<[f64; 3]>) {
    let mut terms = Vocab::default();
    let lengths = sources
        .iter()
        .enumerate()
        .map(|(offset, source)| add_doc(&mut terms, first + offset, source))
        .collect();
    (terms, lengths)
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

/// True when a word has fewer than `min` characters. Lengths count characters, not bytes, so `café` has 4 (design §3.2).
fn shorter_than(word: &str, min: usize) -> bool {
    word.chars().take(min).count() < min
}
