//! Which docs hold a search's `+` or `!` words, at tier 0 only (design §3.2 Operators).
//! Only tier 0 holds: the exact word or a stem-mate, or for an id or path the whole token or a longer one it begins at `-`, `/` or `.`.
//! Prefix and one-edit matches never hold, and the scores are never read or changed here.
//! Cases: SRCH-REQUIRE-01, SRCH-REQUIRE-02, SRCH-EXCLUDE-01, SRCH-EXCLUDE-02, SRCH-EXCLUDE-03

use super::tokenize;
use super::vocab::Posting;
use super::{Index, Probe};
use crate::par;

impl Index<'_> {
    /// The docs that pass a query's operator words (design §3.2 Operators): a doc passes when it holds every word of `must` and none
    /// of `must_not`, each at tier 0. The result has one entry per doc, in the order of the sources.
    pub fn passing(&self, must: &[String], must_not: &[String]) -> Vec<bool> {
        let words: Vec<String> = must.iter().chain(must_not).cloned().collect();
        let holding = self.holders(&words);
        (0..self.labels.len())
            .map(|doc| {
                (0..must.len()).all(|word| holding[word][doc])
                    && (0..must_not.len()).all(|word| !holding[must.len() + word][doc])
            })
            .collect()
    }

    /// Which docs hold each word at tier 0: `holders(words)[word][doc]` is true when that doc holds `words[word]`. No run is
    /// scanned when there are no words.
    fn holders(&self, words: &[String]) -> Vec<Vec<bool>> {
        if words.is_empty() {
            return Vec::new();
        }
        let probes: Vec<Probe> = words
            .iter()
            .map(|word| Probe::new(word, &self.stemmer))
            .collect();
        let runs: Vec<usize> = (0..self.runs.len()).collect();
        let mut holding = vec![vec![false; self.labels.len()]; words.len()];
        for found in par::map(&runs, |&run| self.holding_run(run, &probes)) {
            for (word, doc) in found {
                holding[word][doc] = true;
            }
        }
        holding
    }

    /// The (word, doc) pairs that one run holds at tier 0. A whole word holds through its exact term, and through each longer whole
    /// term that it begins; a plain word holds through each plain term with its stem.
    fn holding_run(&self, run: usize, probes: &[Probe]) -> Vec<(usize, usize)> {
        let vocab = &self.runs[run];
        let mut found = Vec::new();
        for (word, probe) in probes.iter().enumerate() {
            if probe.stem.is_none() {
                if let Some((_, id)) = vocab.get(probe.word) {
                    found.extend(docs_of(word, vocab.postings(id)));
                }
            }
        }
        let plain = probes.iter().any(|probe| probe.stem.is_some());
        for (term, id) in vocab.terms() {
            if tokenize::is_whole(term) {
                for (word, probe) in probes.iter().enumerate() {
                    if probe.matches_longer(term) {
                        found.extend(docs_of(word, vocab.postings(id)));
                    }
                }
            } else if plain {
                let stem = self.stemmer.stem(term);
                let stem: &str = &stem;
                for (word, probe) in probes.iter().enumerate() {
                    if probe.stem.as_deref() == Some(stem) {
                        found.extend(docs_of(word, vocab.postings(id)));
                    }
                }
            }
        }
        found
    }
}

/// The (word, doc) pairs of one word and the postings of a term that it holds.
fn docs_of(word: usize, postings: &[Posting]) -> impl Iterator<Item = (usize, usize)> + '_ {
    postings.iter().map(move |posting| (word, posting.doc))
}
