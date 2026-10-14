//! The vocabulary of an index: each term's postings, counted from the docs and joined by shard.
//! Each term's postings stay in doc order, since a shard joins its term's runs in the order of their docs.
//! Cases: SRCH-RANK-01, SRCH-TOKEN-01

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, PoisonError};

use crate::par;

/// A doc that has a term, with the term's count in each field.
#[derive(Clone, Copy)]
pub struct Posting {
    pub doc: usize,
    pub tf: [u32; 3],
}

/// A term with its postings, in doc order: the postings of one run, or of one shard's runs joined in order.
pub type Term = (String, Vec<Posting>);

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

    /// Splits the terms into `shards` buckets: each term goes, with its postings, to the bucket that `shard_of` gives it. The buckets
    /// of every run are then joined shard by shard, so that each shard can be joined on a thread of its own.
    pub fn into_shards(self, shards: usize) -> Vec<Vec<Term>> {
        let mut lists = self.lists;
        let mut buckets: Vec<Vec<Term>> = (0..shards).map(|_| Vec::new()).collect();
        for (term, id) in self.ids {
            let postings = std::mem::take(&mut lists[id]);
            buckets[shard_of(&term, shards)].push((term, postings));
        }
        buckets
    }

    /// Gives a new term the next id, with no postings yet.
    fn add_term(&mut self, term: &str) -> usize {
        let id = self.lists.len();
        self.lists.push(Vec::new());
        self.ids.insert(term.to_string(), id);
        id
    }
}

/// Joins the buckets of some runs into one vocabulary. `runs[r][s]` holds run `r`'s terms of shard `s`, with the runs in the order of
/// their docs, so each term's postings come out in doc order. The shards are joined on several threads, and the result is in byte
/// order of its terms.
pub fn join(runs: Vec<Vec<Vec<Term>>>, shards: usize) -> BTreeMap<String, Vec<Posting>> {
    let mut by_shard: Vec<Vec<Vec<Term>>> = (0..shards).map(|_| Vec::new()).collect();
    for run in runs {
        for (slot, bucket) in by_shard.iter_mut().zip(run) {
            slot.push(bucket);
        }
    }
    let slots: Vec<Mutex<Vec<Vec<Term>>>> = by_shard.into_iter().map(Mutex::new).collect();
    let (joined, repeats): (Vec<Vec<Term>>, Vec<Vec<Term>>) =
        par::map(&slots, |slot| join_shard(take_runs(slot)))
            .into_iter()
            .unzip();
    // The repeats are freed on this thread: freed on the shard threads, their many small frees contended for the allocator.
    drop(repeats);
    joined.into_iter().flatten().collect()
}

/// The buckets of one shard, taken out of their slot by the thread that joins the shard.
fn take_runs(slot: &Mutex<Vec<Vec<Term>>>) -> Vec<Vec<Term>> {
    let mut runs = slot.lock().unwrap_or_else(PoisonError::into_inner);
    std::mem::take(&mut *runs)
}

/// One shard's buckets, in the order of their runs, joined into its terms in byte order, each with its postings in doc order. A term
/// that an earlier run already has gives its postings to that term, and is returned among the repeats.
fn join_shard(runs: Vec<Vec<Term>>) -> (Vec<Term>, Vec<Term>) {
    let mut ids: HashMap<String, Vec<Posting>> = HashMap::new();
    let mut repeats = Vec::new();
    for (term, postings) in runs.into_iter().flatten() {
        match ids.get_mut(&term) {
            Some(list) => {
                list.extend_from_slice(&postings);
                repeats.push((term, postings));
            }
            None => {
                ids.insert(term, postings);
            }
        }
    }
    let mut terms: Vec<Term> = ids.into_iter().collect();
    terms.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    (terms, repeats)
}

/// The shard of a term. The hash is the same in every run, so a term's postings from all the runs meet in one shard.
fn shard_of(term: &str, shards: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    term.hash(&mut hasher);
    (hasher.finish() % shards as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two runs of docs, the first holding docs 0 and 1 and the second doc 2, joined through `shards` shards.
    fn joined(shards: usize) -> BTreeMap<String, Vec<Posting>> {
        let mut first = Vocab::default();
        first.count(0, 0, "button");
        first.count(1, 2, "button");
        first.count(1, 1, "tab");
        let mut second = Vocab::default();
        second.count(2, 1, "button");
        second.count(2, 0, "tab");
        join(
            vec![first.into_shards(shards), second.into_shards(shards)],
            shards,
        )
    }

    #[test]
    fn a_term_keeps_its_postings_in_doc_order_whatever_the_shard_count() {
        for shards in 1..=4 {
            let terms = joined(shards);
            let docs: Vec<usize> = terms["button"].iter().map(|posting| posting.doc).collect();
            assert_eq!(docs, [0, 1, 2]);
            let names: Vec<&str> = terms.keys().map(String::as_str).collect();
            assert_eq!(names, ["button", "tab"]);
        }
    }
}
