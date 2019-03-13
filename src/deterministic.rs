//! Datastructure to store deterministic, (finite) automata.
//!
//! Models graphs where each node has *at most* one outgoing edge for each character in a certain
//! alphabet. Through a simple utility check it can be used to also model graphs with exactly one
//! such edge.
use std::iter::IntoIterator;
use std::num::NonZeroUsize;

use super::Alphabet;

pub struct Deterministic<A> {
    /// Characters of the underlying alphabet.
    alphabet: Vec<A>,

    /// Outgoing edges for each graph node, one space for each character.
    edges: Vec<Option<Target>>,
}

/// The target of an existing edge.
///
/// The internal representation makes use of `NonZero` optimization so that `Option<Target>` has
/// the same size and align as a `usize` itself.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Target(NonZeroUsize);

impl<A: Alphabet> Deterministic<A> {
    pub fn new<I>(alphabet: I) -> Self 
        where I: IntoIterator<Item=A>
    {
        let mut alphabet = alphabet.into_iter().collect::<Vec<_>>();
        alphabet.as_mut_slice().sort();
        alphabet.dedup();
        // Alphabet is now strictly monotonically increasing.
        Deterministic {
            alphabet,
            edges: vec![],
        }
    }

    /// Get a comparable view of the alphabet.
    ///
    /// The slice is ordered and deduplicated, so that it is a unique representation of the
    /// underlying alphabet.
    pub fn alphabet(&self) -> &[A] {
        self.alphabet.as_slice()
    }

    /// Get the size of the alphabet.
    pub fn char_count(&self) -> usize {
        self.alphabet.len()
    }

    /// Create a new node in the graph.
    ///
    /// Returns the id of the newly created node.
    pub fn node(&mut self) -> Target {
        unimplemented!()
    }
}

impl Target {
    /// Create the target representation.
    pub fn new(index: usize) -> Option<Self> {
        NonZeroUsize::new(index.wrapping_add(1)).map(Target)
    }

    /// Get edge target index with which this was created.
    pub fn index(self) -> usize {
        self.0.get().wrapping_sub(1)
    }
}
