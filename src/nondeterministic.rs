use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::ops::Range;

use super::{Alphabet, Ensure};
use super::deterministic::{self, Deterministic};

pub struct NonDeterministic<A> {
    /// All visited characters, ordered.
    characters: Vec<A>,

    /// The edges of all nodes.
    edges: Vec<Edge>,

    /// Ranges of the edges of each node.
    ranges: Vec<Range<usize>>,
}

/// Dynamic representation of a non-deterministic graph.
///
/// As opposed to `NonDeterministic` this is optimized for making changes to the graph structure.
pub struct Builder<A> {
    /// All visited characters, unordered.
    characters: Vec<A>,

    /// The indices of the ordered list of characters.
    ///
    /// Changing the index of a char within `character` during mutation of the builder would
    /// require iterating all edges, i.e. be a large, potentially wasted effort. We nevertheless
    /// want an ordered list to bisect new characters. This list provides the bisectable ordering.
    ordered: Vec<Character>,

    /// Edges for each node, may contain duplicate entries for first component.
    edges: Vec<Vec<(Character, usize)>>,

    /// Stores epsilon transitions separately.
    ///
    /// This makes it easier to find the epsilon reachability graph.
    epsilons: Vec<Vec<usize>>,
}

/// Iterator over the outgoing edges of a node.
///
/// Should provides other access functions to facilitate traversing the graph or restricting to a
/// specific edge character label.
#[derive(Clone)]
pub struct Edges<'a, A> {
    graph: &'a NonDeterministic<A>,
    edges: &'a [Edge],
}

/// Iterator over all graph nodes.
#[derive(Clone)]
pub struct Nodes<'a, A> {
    node_id: usize,
    graph: &'a NonDeterministic<A>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    label: Label,
    target: usize,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Character(NonZeroUsize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Label(Option<Character>);

impl<A: Alphabet> NonDeterministic<A> {
    pub fn builder() -> Builder<A> {
        Builder::default()
    }

    pub fn edges(&self, node: usize) -> Option<Edges<A>> {
        let range = self.ranges.get(node)?;
        Some(Edges {
            graph: self,
            edges: self.edges.get(range.clone()).unwrap(),
        })
    }

    pub fn nodes(&self) -> Nodes<A> {
        Nodes { 
            node_id: 0,
            graph: self
        }
    }

    pub fn alphabet(&self) -> &[A] {
        self.characters.as_slice()
    }

    pub fn from_deterministic(det: &Deterministic<A>) -> Self {
        // alphabet is also sorted.
        let characters = det.alphabet().to_vec();
        let nodes = det.node_count();
        // Assume densely packed.
        let mut edges = Vec::with_capacity(nodes*characters.len());
        let mut ranges = Vec::with_capacity(nodes);

        for node in 0..nodes {
            let target = deterministic::Target::new(node).unwrap();
            let outgoing = &det[target];
            let outedges = outgoing.iter()
                .enumerate()
                .filter_map(|(id, edge)| edge.map(|target| Edge {
                    label: Label::character(id),
                    target: target.index(),
                }));
            let begin = edges.len();
            edges.extend(outedges);
            let end = edges.len();
            ranges.push(begin..end);
        }

        NonDeterministic {
            characters,
            edges,
            ranges,
        }
    }

    fn label(&self, ch: Option<&A>) -> Option<Label> {
        match ch {
            Some(ch) => self.characters
                .binary_search(ch)
                .map(Label::character)
                .ok(),
            None => Some(Label::EPSILON),
        }
    }

    fn unlabel(&self, label: Label) -> Option<&A> {
        label.index().map(|idx| &self.characters[idx])
    }
}

impl<A: Alphabet> Builder<A> {
    /// Insert a new edge, guarded by the specified character.
    pub fn insert(&mut self, from: usize, character: Option<&A>, to: usize) {
        self.ensure_nodes(from);
        self.ensure_nodes(to);
        if let Some(character) = character {
            let character = self.ensure_char(character);
            self.edges[from].push((character, to));
        } else {
            self.epsilons[from].push(to);
        }
    }

    /// Build a finalized graph.
    ///
    /// This step optimizes the data structure for querying of graph edges instead of insertion.
    pub fn finish(&self) -> NonDeterministic<A> {
        // Map for current to actually ordered character mapping.
        let character_label = self.ordered
            .iter()
            .enumerate()
            .map(|(index, character)| (*character, Label::character(index)))
            .collect::<HashMap<_, _>>();
        let mut characters = self.characters.clone();
        characters.sort();
        let characters = characters;

        let mut edges = Vec::new();
        let mut ranges = Vec::new();

        let per_node = self.edges.iter().zip(self.epsilons.iter());
        per_node.for_each(|(node_edges, node_epsilons)| {
            let start = edges.len();
            let end = start + node_epsilons.len() + node_edges.len();

            edges.extend(node_epsilons.iter().map(|&target| Edge {
                label: Label::EPSILON,
                target,
            }));
            edges.extend(node_edges.iter().map(|(character, target)| Edge {
                label: character_label[character],
                target: *target,
            }));

            edges[start..end].sort();
            ranges.push(start..end);
        });

        NonDeterministic {
            characters,
            edges,
            ranges,
        }
    }

    /// The `Character` or the index where to insert it into the ordered representation.
    ///
    /// In case of an insert, the new `Character` is given by the current length of the `character`
    /// vector.
    fn resolve_char(&self, character: &A) -> Result<Character, usize> {
        let index = self.ordered.binary_search_by_key(&character, 
            |character| &self.characters[character.index()])?;
        Ok(self.ordered[index])
    }

    fn ensure_char(&mut self, character: &A) -> Character {
        let resolve = self.resolve_char(character);
        resolve.unwrap_or_else(|index| {
            let new_char = Character::new(self.characters.len());
            self.characters.push(*character);
            self.ordered.insert(index, new_char);
            new_char
        })
    }

    fn ensure_nodes(&mut self, node: usize) {
        self.edges.ensure_with(node + 1, Vec::new);
        self.epsilons.ensure_with(node + 1, Vec::new);
    }
}

impl Character {
    pub fn new(index: usize) -> Self {
        Character(NonZeroUsize::new(index + 1).unwrap())
    }

    pub fn index(self) -> usize {
        self.0.get() - 1
    }
}

impl Label {
    const EPSILON: Label = Label(None);

    pub fn character(index: usize) -> Self {
        Label(Some(Character::new(index)))
    }

    pub fn index(self) -> Option<usize> {
        self.0.map(Character::index)
    }
}

impl<'a, A: Alphabet> Edges<'a, A> {
    /// Only iterate over edges labeled with the character.
    pub fn restrict_to(&mut self, character: Option<&A>) {
        let label = match self.graph.label(character) {
            Some(label) => label,
            None => return self.edges = &self.edges[0..0],
        };
        let begin = self.edges.iter()
            .position(|edge| edge.label >= label)
            .unwrap_or_else(|| self.edges.len());
        let end = self.edges.iter()
            .position(|edge| edge.label > label)
            .unwrap_or_else(|| self.edges.len());
        self.edges = &self.edges[begin..end];
    }

    pub fn targets(self) -> impl Iterator<Item=usize> + 'a {
        self.map(|(_, target)| target)
    }
}

impl<'a, A: Alphabet> Nodes<'a, A> {
    fn todo(&self) -> usize {
        let len = self.graph.edges.len();
        len - self.node_id
    }
}

impl<'a, A: Alphabet> Iterator for Edges<'a, A> {
    type Item = (Option<&'a A>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (edge, tail) = self.edges.split_first()?;
        self.edges = tail;
        let character = self.graph.unlabel(edge.label);
        Some((character, edge.target))
    }
}

impl<'a, A: Alphabet> Iterator for Nodes<'a, A> {
    type Item = (usize, Edges<'a, A>);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.node_id;
        let edges = self.graph.edges(id)?;
        self.node_id += 1;
        Some((id, edges))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let todo = self.todo();
        (todo, Some(todo))
    }

    fn count(self) -> usize {
        self.todo()
    }
}

impl<'a, A: Alphabet> ExactSizeIterator for Nodes<'a, A> {
    fn len(&self) -> usize {
        self.todo()
    }
}

impl<A: Alphabet> Default for Builder<A> {
    fn default() -> Self {
        Builder {
            characters: Vec::new(),
            ordered: Vec::new(),
            edges: Vec::new(),
            epsilons: Vec::new(),
        }
    }
}

impl<A: Alphabet> From<Deterministic<A>> for NonDeterministic<A> {
    fn from(det: Deterministic<A>) -> Self {
        NonDeterministic::from_deterministic(&det)
    }
}

impl<'a, A: Alphabet> From<&'a Deterministic<A>> for NonDeterministic<A> {
    fn from(det: &'a Deterministic<A>) -> Self {
        NonDeterministic::from_deterministic(det)
    }
}
