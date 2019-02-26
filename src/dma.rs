//! A deterministic, self-modifying automata.
//!
//! Tests a new kind of automata maybe capable of recognizing `a^nb^nc^n` (i.e. more powerful than
//! context free) but still O(n) space and O(n) time.
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::Alphabet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Creator(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Transition(pub usize);

#[derive(Clone, Copy)]
pub enum TransitionKind {
    Standard,
    Creating {
        /// Index of the creator function.
        creator: Creator,
    },
}

#[derive(Clone, Copy)]
struct Edge {
    /// Where this leads to.
    target: State,

    /// The kind of transition.
    transition: Transition,
}

pub struct NewEdge<A> {
    target: EdgeTarget<A>,
    kind: Transition,
}

pub enum EdgeTarget<A> {
    /// That edge should be back to the new node.
    SelfCycle,

    /// The edge should point to some node connected to the target.
    Target(A),
}

#[derive(Clone)]
pub struct Dma<A: Alphabet> {
    /// Alphabet for comparison.
    alphabet: Vec<A>,
    lut: HashMap<A, usize>,

    /// The number of states before each run.
    initial_states: usize,

    /// Set of final states.
    final_states: HashSet<State>,

    /// |A| transitions for each state.
    connected: Vec<Edge>,

    /// The different transition types.
    transitions: Vec<TransitionKind>,

    /// The functions creating edges.
    creator: Vec<Arc<Fn(A) -> NewEdge<A>>>,
}

pub struct Run<A: Alphabet> {
    backing: Dma<A>,
    state: State,
}

impl<A: Alphabet> Dma<A> {
    pub fn run(&self) -> Run<A> {
        assert!(self.initial_states > 0, "Can not run an empty automaton");

        Run {
            backing: self.clone(),
            state: State(0),
        }
    }

    /// The character index.
    fn index(&self, character: &A) -> Option<usize> {
        self.lut.get(character).cloned()
    }

    /// Get the corresponding transition kind.
    fn edge(&self, state: State, character: usize) -> &Edge {
        let index = self.alphabet.len()*state.0 + character;
        self.connected.get(index).unwrap()
    }

    fn creator(&self, index: usize) -> &Fn(A) -> NewEdge<A> {
        &**self.creator.get(index).unwrap()
    }
}

impl<A: Alphabet> Run<A> {
    pub fn next(&mut self, character: A) {
        unimplemented!()
    }

    pub fn is_final(&self) -> bool {
        self.backing.final_states.contains(&self.state)
    }
}
