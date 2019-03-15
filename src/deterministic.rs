//! Datastructure to store deterministic, (finite) automata.
//!
//! Models graphs where each node has *at most* one outgoing edge for each character in a certain
//! alphabet. Through a simple utility check it can be used to also model graphs with exactly one
//! such edge.
use std::fmt::Display;
use std::iter::IntoIterator;
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::ops::Index;

use crate::Alphabet;
use crate::dot::{Edge, Family, GraphWriter};

pub struct Deterministic<A> {
    /// Characters of the underlying alphabet.
    alphabet: Vec<A>,

    /// Outgoing edges for each graph node, one space for each character.
    edges: Vec<Option<Target>>,

    /// The ide for the next node.
    next_id: usize,
}

pub struct Edges<'a, A> {
    alph: &'a [A],
    targets: &'a [Option<Target>],
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
            next_id: 0,
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

    pub fn edges(&mut self, target: Target) -> Option<Edges<A>> {
        unimplemented!()
    }

    pub fn write_to(&self, output: &mut Write) -> io::Result<()>
        where for<'a> &'a A: Display
    {
        let mut writer = GraphWriter::new(output, Family::Directed, None)?;

        for from in 0..self.next_id {
            for (label, to) in self.edges(Target::new(from).unwrap()).unwrap() {
                let edge = Edge {
                    label: Some(format!("{}", label).into()),
                    .. Edge::none()
                };

                writer.segment([from, to.index()].iter().cloned(), Some(edge))?;
            }
        }

        writer.end_into_inner().1
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

impl<A: Alphabet> Index<Target> for Deterministic<A> {
    type Output = [Option<Target>];

    fn index(&self, target: Target) -> &[Option<Target>] {
        self.edges(target).unwrap().targets
    }
}

impl<'a, A> Iterator for Edges<'a, A> {
    type Item = (&'a A, Target);

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
