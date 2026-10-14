//! The vocabulary of an index: each term's postings, counted from the docs and joined in doc order.
//! Cases: SRCH-RANK-01, SRCH-TOKEN-01

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};

/// A doc that has a term, with the term's count in each field.
pub struct Posting {
    pub doc: usize,
    pub tf: [u32; 3],
}

/// The terms of some docs: an id for each term, and the postings of each id in doc order. The docs are counted one after another,
/// so a doc's own posting of a term, when it has one, is the last of that term's postings.
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

    /// Adds the terms of `run`, whose docs come after every doc of this vocabulary. Joining runs in the order of their docs gives
    /// each term its postings in doc order, as counting the docs one after another would.
    pub fn append(&mut self, run: Vocab) {
        let mut lists = run.lists;
        for (term, id) in run.ids {
            let postings = std::mem::take(&mut lists[id]);
            match self.ids.entry(term) {
                Entry::Occupied(known) => self.lists[*known.get()].extend(postings),
                Entry::Vacant(slot) => {
                    slot.insert(self.lists.len());
                    self.lists.push(postings);
                }
            }
        }
    }

    /// Gives a new term the next id, with no postings yet.
    fn add_term(&mut self, term: &str) -> usize {
        let id = self.lists.len();
        self.lists.push(Vec::new());
        self.ids.insert(term.to_string(), id);
        id
    }

    /// The vocabulary in byte order of its terms, each with its postings.
    pub fn into_map(self) -> BTreeMap<String, Vec<Posting>> {
        let mut lists = self.lists;
        self.ids
            .into_iter()
            .map(|(term, id)| (term, std::mem::take(&mut lists[id])))
            .collect()
    }
}
