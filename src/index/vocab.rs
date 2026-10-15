//! The vocabulary of one run of docs: each term's postings, counted in doc order.
//! A doc's postings are counted after those of the docs before it, so each term's postings stay in doc order. A group's runs are
//! never joined into one vocabulary: a query reads a term's postings from each run in run order, which is doc order too (perf P-IDX).
//! Cases: SRCH-RANK-01, SRCH-TOKEN-01

use std::collections::HashMap;

/// A doc that has a term, with the term's count in each field.
#[derive(Clone, Copy)]
pub struct Posting {
    pub doc: usize,
    pub tf: [u32; 3],
}

/// The terms of one run of docs: an id for each term, and the postings of each id in doc order. The docs are counted one after
/// another, so a doc's own posting of a term, when it has one, is the last of that term's postings.
#[derive(Default)]
pub struct Vocab {
    ids: HashMap<String, usize>,
    lists: Vec<Vec<Posting>>,
}

impl Vocab {
    /// Counts one occurrence of `term` in `field` of `doc`.
    pub fn count(&mut self, doc: usize, field: usize, term: &str) {
        let id = match self.ids.get(term) {
            Some(&id) => id,
            None => self.add_term(term),
        };
        let postings = &mut self.lists[id];
        match postings.last_mut() {
            Some(posting) if posting.doc == doc => posting.tf[field] += 1,
            _ => {
                let mut tf = [0; 3];
                tf[field] = 1;
                postings.push(Posting { doc, tf });
            }
        }
    }

    /// Every distinct term of the run with its id, in no particular order. A query scans these for its candidates.
    pub fn terms(&self) -> impl Iterator<Item = (&str, usize)> {
        self.ids.iter().map(|(term, id)| (term.as_str(), *id))
    }

    /// The term with its id when the run has that exact term.
    pub fn get(&self, term: &str) -> Option<(&str, usize)> {
        self.ids
            .get_key_value(term)
            .map(|(found, id)| (found.as_str(), *id))
    }

    /// The postings of a term id, in doc order.
    pub fn postings(&self, id: usize) -> &[Posting] {
        &self.lists[id]
    }

    /// Gives a new term the next id, with no postings yet.
    fn add_term(&mut self, term: &str) -> usize {
        let id = self.lists.len();
        self.lists.push(Vec::new());
        self.ids.insert(term.to_string(), id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two runs of docs, the first holding docs 0 and 1 and the second doc 2.
    fn two_runs() -> [Vocab; 2] {
        let mut first = Vocab::default();
        first.count(0, 0, "button");
        first.count(1, 2, "button");
        first.count(1, 1, "tab");
        let mut second = Vocab::default();
        second.count(2, 1, "button");
        second.count(2, 0, "tab");
        [first, second]
    }

    #[test]
    fn a_term_keeps_its_postings_in_doc_order_across_the_runs() {
        let runs = two_runs();
        let docs: Vec<usize> = runs
            .iter()
            .filter_map(|run| run.get("button").map(|(_, id)| run.postings(id)))
            .flatten()
            .map(|posting| posting.doc)
            .collect();
        assert_eq!(docs, [0, 1, 2]);
    }

    #[test]
    fn a_run_finds_the_terms_it_holds_and_no_others() {
        let runs = two_runs();
        assert!(runs[0].get("tab").is_some());
        assert!(runs[1].get("button").is_some());
        assert!(runs[1].get("missing").is_none());
        let mut terms: Vec<&str> = runs[0].terms().map(|(term, _)| term).collect();
        terms.sort_unstable();
        assert_eq!(terms, ["button", "tab"]);
    }
}
