//! Reads a search's terms and their `+` and `!` operators into filters and words (design §3.2).
//! A word is bare, required by `+` (it must be held, and it ranks as a bare word does) or excluded by `!` (no hit may hold it).
//! Cases: SRCH-REQUIRE-01, SRCH-REQUIRE-02, SRCH-REQUIRE-03, SRCH-REQUIRE-04, SRCH-EXCLUDE-01, SRCH-EXCLUDE-02, SRCH-EXCLUDE-03, SRCH-EXCLUDE-04, SRCH-EXCLUDE-05, SRCH-EXCLUDE-06

use std::collections::BTreeSet;

use crate::error::Error;
use crate::index::filter::{self, Filter};
use crate::index::tokenize::query_tokens;

/// The operator that a term begins with (design §3.2 Operators).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// No operator: a word ranks the docs, and a filter keeps the docs it holds.
    Bare,
    /// `+`: the word must be held, and it ranks as a bare word does; a filter is the same as a bare one.
    Require,
    /// `!`: the docs that hold the word are left out; a filter keeps the docs it drops.
    Exclude,
}

/// A term that is not a filter: as typed, with the operator it begins with, and the rest of it, which holds the words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The term as typed, which the zero-hit hint names.
    pub typed: String,
    /// The operator that the term begins with.
    pub op: Op,
    /// The term without its operator, which its words are read from.
    pub rest: String,
}

/// The words of a query, each listed once in each list: those that rank the docs, those that a hit must hold, and those that it
/// must not hold.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Words {
    /// The words that rank the docs: the bare words and the `+` words, in the order given.
    pub ranked: Vec<String>,
    /// The words that each hit must hold: the `+` words.
    pub required: Vec<String>,
    /// The words that no hit may hold: the `!` words.
    pub excluded: Vec<String>,
}

/// Reads the terms of a query into its filters and its literal terms, in the order given. Each term's operator comes off first, and
/// a `!` makes a filter its complement. A term that is not a filter is a literal, kept as typed.
pub fn read(
    terms: &[String],
    scope: &BTreeSet<String>,
) -> Result<(Vec<Filter>, Vec<Literal>), Error> {
    let mut filters = Vec::new();
    let mut literals = Vec::new();
    for term in terms {
        let (op, rest) = operator_of(term);
        match filter::classify(rest, scope, op == Op::Exclude)? {
            Some(found) => filters.push(found),
            None => literals.push(Literal {
                typed: term.clone(),
                op,
                rest: rest.to_string(),
            }),
        }
    }
    Ok((filters, literals))
}

/// The operator that a term begins with, and the rest of the term. Only a term's first character can be an operator, so a `+` or
/// a `!` later in the term is part of it.
fn operator_of(term: &str) -> (Op, &str) {
    if let Some(rest) = term.strip_prefix('+') {
        (Op::Require, rest)
    } else if let Some(rest) = term.strip_prefix('!') {
        (Op::Exclude, rest)
    } else {
        (Op::Bare, term)
    }
}

impl Words {
    /// The words of the literal terms: every token of each term, in order, each with its term's operator applied. A required word
    /// also ranks, so `+word` ranks as `word` does (design §3.2 Operators).
    pub fn of(literals: &[Literal]) -> Words {
        let mut words = Words::default();
        for literal in literals {
            for token in query_tokens(&literal.rest) {
                match literal.op {
                    Op::Bare => push_once(&mut words.ranked, token),
                    Op::Require => {
                        push_once(&mut words.ranked, token.clone());
                        push_once(&mut words.required, token);
                    }
                    Op::Exclude => push_once(&mut words.excluded, token),
                }
            }
        }
        words
    }

    /// True when no term holds a word, so that the query has nothing to rank or list the docs by.
    pub fn is_empty(&self) -> bool {
        self.ranked.is_empty() && self.excluded.is_empty()
    }
}

/// Adds a word to a list unless the list has it already.
fn push_once(list: &mut Vec<String>, word: String) {
    if !list.contains(&word) {
        list.push(word);
    }
}
