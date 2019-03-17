use std::collections::HashMap;
use std::fmt::Write;

use super::Alphabet;
use super::nfa::Nfa;

/// Represents regular expressions over some finite alphabet.
///
/// Optimizes storage and construction for reoccurring subexpressions to allow
/// polynomial time conversion from NFA. You can create (and keep) handles on 
/// subexpressions, then evaluate as if those subexpressions were at the root.
/// This relationship forms an acyclic graph.
pub struct Regex<A: Alphabet> {
    subs: Vec<Op<A>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Handle(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Op<A: Alphabet> {
    Epsilon,
    Match(A),
    Star(Handle),
    Or(Handle, Handle),
    Concat(Handle, Handle),
}

/// Provides access to creating new regex expressions with cached results.
///
/// ```
/// let mut regex = Regex::new();
/// let cached = regex.cached();
/// ```
pub struct Cached<A: Alphabet> {
    regex: Regex<A>,
    cache: HashMap<Op<A>, Handle>,
}

impl<A: Alphabet> Regex<A> {
    pub fn new() -> Self {
        Regex {
            subs: vec![],
        }
    }

    /// Idea:
    ///
    /// Is like a regex-labeled nfa with only one final state.
    pub fn to_nfa(self) -> Nfa<A> {
        unimplemented!()
    }

    /// Push a new operation as the regex root.
    ///
    /// It is not required that all regex states are reachable afterwards but all
    /// handles must point to existing operations. Returns a handle on the newly
    /// inserted operation.
    pub fn push(&mut self, op: Op<A>) -> Handle {
        match op {
            Op::Epsilon => (),
            Op::Match(_) => (),
            Op::Star(Handle(i)) => assert!(i < self.subs.len()),
            Op::Or(Handle(i), Handle(j)) => assert!(i < self.subs.len() && j < self.subs.len()),
            Op::Concat(Handle(i), Handle(j)) => assert!(i < self.subs.len() && j < self.subs.len()),
        }

        let handle = Handle(self.subs.len());
        self.subs.push(op);
        handle
    }

    /// Get a root to the regex.
    pub fn root(&self) -> Option<Handle> {
        self.subs.len().checked_sub(1).map(Handle)
    }

    pub fn cached(self) -> Cached<A> {
        Cached {
            regex: self,
            // TODO: prefill the hashmap with existing operations?
            cache: HashMap::new(),
        }
    }

    pub fn to_string(&self) -> String {

        let mut string = String::new();
        self.push_from_root(self.root().unwrap(), &mut string);
        string
    }

    fn push_from_root(&self, Handle(root): Handle, string: &mut String) {
        match self.subs[root] {
            Op::Epsilon => string.push_str("{e}"),
            Op::Match(a) => write!(string, "{{{:?}}}", a).unwrap(),
            Op::Star(sub) => {
                string.push('(');
                self.push_from_root(sub, string);
                string.push_str(")*");
            },
            Op::Or(a, b) => {
                string.push('(');
                self.push_from_root(a, string);
                string.push('|');
                self.push_from_root(b, string);
                string.push(')');
            },
            Op::Concat(a, b) => {
                self.push_from_root(a, string);
                self.push_from_root(b, string);
            },
        }
    }
}

impl<A: Alphabet> Cached<A> {
    pub fn new() -> Self {
        // TODO: can be faster if `cached` should fill the hashmap.
        Regex::new().cached()
    }

    /// Insert a new operation.
    ///
    /// Deduplicates same operations to also point to the same handle, so you can not generally
    /// assert that the returned handle is the new root of the regex.
    pub fn insert(&mut self, op: Op<A>) -> Handle {
        let regex = &mut self.regex;
        self.cache.entry(op)
            .or_insert_with(|| regex.push(op))
            .clone()
    }

    pub fn inner(&self) -> &Regex<A> {
        &self.regex
    }

    pub fn into_inner(self) -> Regex<A> {
        self.regex
    }
}

impl<A: Alphabet> Default for Regex<A> {
    fn default() -> Self {
        Self::new()
    }
}
