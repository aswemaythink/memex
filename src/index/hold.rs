//! Which docs hold a search's `+` or `!` words, at tier 0 only (design §3.2 Operators).
//! Only tier 0 holds: the exact word or a stem-mate, or for an id or path the whole token or a longer one it begins at `-`, `/` or `.`.
//! `Index::scan_run` finds them with prefix and one-edit matches off, so a prefix or a one-edit match never holds, and the scores are never read or changed here.
//! Cases: SRCH-REQUIRE-01, SRCH-REQUIRE-02, SRCH-EXCLUDE-01, SRCH-EXCLUDE-02, SRCH-EXCLUDE-03, SRCH-EXCLUDE-07

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

    /// Which docs hold each word at tier 0: `holders(words)[word][doc]` is true when that doc holds `words[word]`. Each run is
    /// scanned once for all the words by `scan_run`, with prefix and one-edit matches off, so its candidates are tier 0 only; each
    /// candidate's postings mark the docs it holds. No run is scanned when there are no words.
    fn holders(&self, words: &[String]) -> Vec<Vec<bool>> {
        if words.is_empty() {
            return Vec::new();
        }
        let probes: Vec<Probe> = words
            .iter()
            .map(|word| Probe {
                prefix: false,
                fuzzy: false,
                ..Probe::new(word, &self.stemmer)
            })
            .collect();
        let runs: Vec<usize> = (0..self.runs.len()).collect();
        let mut holding = vec![vec![false; self.labels.len()]; words.len()];
        for candidate in par::map(&runs, |&run| self.scan_run(run, &probes))
            .into_iter()
            .flatten()
        {
            for posting in self.runs[candidate.run].postings(candidate.id) {
                holding[candidate.word][posting.doc] = true;
            }
        }
        holding
    }
}
