use std::num::NonZeroUsize;
use std::ops::Range;

pub struct NonDeterministic<A> {
    character: Vec<A>,
    edges: Vec<Edge>,
    nodes: Vec<Range<usize>>,
}

struct Edge {
    character: Label,
    target: usize,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Character(NonZeroUsize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Label(Option<Character>);

/// Dynamic representation of a non-deterministic graph.
///
/// As opposed to `NonDeterministic` this is optimized for making changes to the graph structure.
pub struct Builder<A> {
    /// All visited characters, ordered.
    character: Vec<A>,

    /// Edges for each node, may contain duplicate entries for first component.
    edges: Vec<Vec<usize>>,

    /// Stores epsilon transitions separately.
    ///
    /// This makes it easier to find the epsilon reachability graph.
    epsilons: Vec<Vec<usize>>,
}

impl Character {
    pub fn new(index: usize) -> Self {
        Character(NonZeroUsize::new(index + 1).unwrap())
    }
}
