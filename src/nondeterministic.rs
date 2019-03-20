use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::ops::Range;

use super::Alphabet;

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

struct Edge {
    character: Label,
    target: usize,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Character(NonZeroUsize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Label(Option<Character>);

impl<A: Alphabet> Builder<A> {
    pub fn node(&mut self) -> usize {
        let id = self.edges.len();
        self.edges.push(vec![]);
        self.epsilons.push(vec![]);
        id
    }

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
                character: Label::EPSILON,
                target,
            }));
            edges.extend(node_edges.iter().map(|(character, target)| Edge {
                character: character_label[character],
                target: *target,
            }));

            ranges.push(start..end);

            unimplemented!("Sort the range by character")
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
        if node >= self.edges.len() {
            self.edges.resize_with(node, Vec::new);
            self.epsilons.resize_with(node, Vec::new);
        }
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
}
