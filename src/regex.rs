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

pub enum Op<A: Alphabet> {
    Epsilon,
    Match(A),
    Star(Handle),
    Or(Handle, Handle),
    Concat(Handle, Handle),
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
}

impl<A: Alphabet> Default for Regex<A> {
    fn default() -> Self {
        Self::new()
    }
}
