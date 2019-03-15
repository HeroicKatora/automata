//! Datastructure to store deterministic, (finite) automata.
//!
//! Models graphs where each node has *at most* one outgoing edge for each character in a certain
//! alphabet. Through a simple utility check it can be used to also model graphs with exactly one
//! such edge.
use std::slice;
use std::fmt::Display;
use std::iter::{self, IntoIterator};
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut, Range};

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
    alphabet: &'a [A],
    targets: &'a [Option<Target>],
}

pub struct EdgesIter<'a, A> {
    alphabet: slice::Iter<'a, A>,
    targets: slice::Iter<'a, Option<Target>>,
}

pub struct EdgesMut<'a, A> {
    alphabet: &'a [A],
    targets: &'a mut [Option<Target>],
}

pub struct EdgesIterMut<'a, A> {
    alphabet: slice::Iter<'a, A>,
    targets: slice::IterMut<'a, Option<Target>>,
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

    /// Get the number of nodes in this graph.
    #[allow(unused)]
    pub fn node_count(&self) -> usize {
        self.next_id
    }

    /// Create a new node in the graph.
    ///
    /// Returns the id of the newly created node.
    ///
    /// # Panics
    /// When the new node id can not be represented.
    pub fn node(&mut self) -> Target {
        let count = self.char_count();
        self.edges.extend(iter::repeat(None).take(count));
        let id = self.next_id;
        self.next_id += 1;
        Target::new(id).expect("Maximum node count exceeded")
    }

    /// Get the outgoing edges of a node.
    pub fn edges(&self, target: Target) -> Option<Edges<A>> {
        let range = self.valid_edges_range(target)?;
        Some(Edges {
            alphabet: self.alphabet.as_slice(),
            targets: &self.edges[range],
        })
    }

    /// Iterate the edges of the specified node.
    ///
    /// Gives an empty iterator when the node is invalid or has no edges. Use `edges` to find out
    /// which of the two possibilites it is.
    pub fn iter_edges(&self, node: Target) -> EdgesIter<A> {
        let range = self.valid_edges_range(node)
            .unwrap_or(0..0);
        let edges = Edges {
            alphabet: self.alphabet.as_slice(),
            targets: &self.edges[range],
        };
        edges.into_iter()
    }

    /// Get a mutable reference to the outgoing edges of a node.
    pub fn edges_mut(&mut self, target: Target) -> Option<EdgesMut<A>> {
        let range = self.valid_edges_range(target)?;
        Some(EdgesMut {
            alphabet: self.alphabet.as_slice(),
            targets: &mut self.edges[range],
        })
    }

    /// Mutably iterate the edges of the specified node.
    ///
    /// Gives an empty iterator when the node is invalid or has no edges. Use `edges` to find out
    /// which of the two possibilites it is.
    pub fn iter_edges_mut(&mut self, node: Target) -> EdgesIterMut<A> {
        let range = self.valid_edges_range(node)
            .unwrap_or(0..0);
        let edges = EdgesMut {
            alphabet: self.alphabet.as_slice(),
            targets: &mut self.edges[range],
        };
        edges.into_iter()
    }

    /// Check that all edges refer to valid targets.
    pub fn is_complete(&self) -> bool {
        self.edges.iter().all(Option::is_some)
    }

    pub fn iter(&self) -> impl Iterator<Item=Target> {
        (0..self.next_id).map(|id| Target::new(id).unwrap())
    }

    fn valid_edges_range(&self, target: Target) -> Option<Range<usize>> {
        let idx = target.index();
        let count = self.char_count();
        if idx >= self.next_id {
            return None
        } else {
            // None of this overflows.
            let start = idx.checked_mul(count).unwrap();
            let end = start.checked_add(count).unwrap();
            Some(start..end)
        }
    }

    #[allow(unused)]
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
    pub const ZERO: Target = Target(unsafe { NonZeroUsize::new_unchecked(1) });

    /// Create the target representation.
    pub fn new(index: usize) -> Option<Self> {
        NonZeroUsize::new(index.wrapping_add(1)).map(Target)
    }

    /// Create the target, assuming it is valid.
    pub fn make(index: usize) -> Self {
        Self::new(index).unwrap()
    }

    /// Get edge target index with which this was created.
    pub fn index(self) -> usize {
        self.0.get().wrapping_sub(1)
    }
}

impl<A: Alphabet> Edges<'_, A> {
    #[allow(unused)]
    pub fn target(&self, ch: A) -> Result<Option<Target>, ()> {
        self.alphabet.binary_search(&ch).map_err(|_| ())
            .map(|idx| self.targets[idx].clone())
    }
}

impl<A: Alphabet> EdgesMut<'_, A> {
    #[allow(unused)]
    pub fn target(&self, ch: A) -> Result<Option<Target>, ()> {
        self.alphabet.binary_search(&ch).map_err(|_| ())
            .map(|idx| self.targets[idx].clone())
    }

    pub fn target_mut(&mut self, ch: A) -> Result<&mut Option<Target>, ()> {
        let targets = &mut self.targets;
        self.alphabet.binary_search(&ch).map_err(|_| ())
            .map(move |idx| &mut targets[idx])
    }
}

impl<A: Alphabet> Index<Target> for Deterministic<A> {
    type Output = [Option<Target>];

    fn index(&self, target: Target) -> &[Option<Target>] {
        self.edges(target).unwrap().targets
    }
}

impl<'a, A: Alphabet> Index<A> for Edges<'a, A> {
    type Output = Option<Target>;

    fn index(&self, ch: A) -> &Option<Target> {
        let idx = edge_unwrap(self.alphabet.binary_search(&ch));
        &self.targets[idx]
    }
}

impl<'a, A: Alphabet> Index<A> for EdgesMut<'a, A> {
    type Output = Option<Target>;

    fn index(&self, ch: A) -> &Option<Target> {
        let idx = edge_unwrap(self.alphabet.binary_search(&ch));
        &self.targets[idx]
    }
}

impl<'a, A: Alphabet> IndexMut<A> for EdgesMut<'a, A> {
    fn index_mut(&mut self, ch: A) -> &mut Option<Target> {
        edge_unwrap(self.target_mut(ch))
    }
}

fn edge_unwrap<T, E>(result: Result<T, E>) -> T where E: std::fmt::Debug {
    result.expect("Mismatch between deterministic alphabet and character")
}

impl<'a, A> IntoIterator for Edges<'a, A> {
    type IntoIter = EdgesIter<'a, A>;
    type Item = <EdgesIter<'a, A> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        let Edges { alphabet, targets} = self;
        EdgesIter {
            alphabet: alphabet.iter(),
            targets: targets.iter(),
        }
    }
}

impl<'a, A> IntoIterator for EdgesMut<'a, A> {
    type IntoIter = EdgesIterMut<'a, A>;
    type Item = <EdgesIterMut<'a, A> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        let EdgesMut { alphabet, targets} = self;
        EdgesIterMut {
            alphabet: alphabet.iter(),
            targets: targets.iter_mut(),
        }
    }
}

impl<'a, A> Iterator for EdgesIter<'a, A> {
    type Item = (&'a A, Target);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = self.alphabet.next();
            let target = self.targets.next();
            match (ch, target) {
                (None, None) => return None,
                (Some(ch), Some(Some(target))) => return Some((ch, *target)),
                (Some(_), Some(None)) => (),
                _ => unreachable!("Alphabet and target have same length"),
            }
        }
    }
}

impl<'a, A> Iterator for EdgesIterMut<'a, A> {
    type Item = (&'a A, &'a mut Option<Target>);

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.alphabet.next()?;
        let target = self.targets.next().unwrap();
        Some((ch, target))
    }
}
