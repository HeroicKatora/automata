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
    pub target: State,

    /// The kind of transition.
    pub transition: Transition,
}

pub struct NewEdge<A> {
    /// How to determine the target state.
    pub target: EdgeTarget<A>,

    /// Which transition type to use.
    pub kind: Transition,
}

pub enum EdgeTarget<A> {
    /// That edge should be back to the new node.
    SelfCycle,

    /// The edge should point to some node connected to the target.
    Target(A),
}

pub trait CreatorFn<A> {
    fn is_final(&self) -> bool;
    fn edge(&self, character: A) -> NewEdge<A>;
}

pub enum Error {
    /// A character was not part of the alphabet.
    InvalidChar,

    /// An edge was found whose kind was never created.
    NoSuchEdge,

    /// Some state reference was wrong.
    NoSuchState,

    /// A creator was referenced but never registered.
    NoSuchCreator,
}

#[derive(Clone)]
pub struct Dma<A: Alphabet> {
    /// Alphabet for comparison.
    alphabet: Vec<A>,
    lut: HashMap<A, usize>,

    /// The number of states before each run.
    next_state: usize,

    /// Set of final states.
    final_states: HashSet<State>,

    /// |A| transitions for each state.
    edges: Vec<Edge>,

    /// The different transition types.
    transitions: Vec<TransitionKind>,

    /// The functions creating edges.
    creator: Vec<Arc<CreatorFn<A>>>,
}

pub struct Run<A: Alphabet> {
    backing: Dma<A>,
    state: State,
}

impl<A: Alphabet> Dma<A> {
    pub fn run(&self) -> Run<A> {
        assert!(self.next_state > 0, "Can not run an empty automaton");

        Run {
            backing: self.clone(),
            state: State(0),
        }
    }

    /// The character index.
    fn index(&self, character: A) -> Result<usize, Error> {
        self.lut.get(&character).cloned().ok_or(Error::InvalidChar)
    }

    /// Get the corresponding transition kind.
    fn edge(&self, state: State, character: usize) -> &Edge {
        let index = self.alphabet.len()*state.0 + character;
        self.edges.get(index).unwrap()
    }

    fn transition(&self, transition: Transition) -> Option<&TransitionKind> {
        self.transitions.get(transition.0)
    }

    fn creator(&self, index: usize) -> Option<Arc<CreatorFn<A>>> {
        self.creator.get(index).cloned()
    }

    fn new_state(&mut self, blueprint: State, creator: Arc<CreatorFn<A>>) -> Result<State, Error> {
        let tr_count = self.alphabet.len();
        let blueprint = blueprint.0;
        if blueprint >= self.next_state {
            return Err(Error::NoSuchState)
        }

        // We can retrieve our transitions from the blueprint state.
        let tr_start = tr_count*blueprint;
        assert!(self.edges.len() >= tr_start + tr_count);

        let new_state = State(self.next_state);
        let mut new_edges = vec![];
        for alph in self.alphabet.iter().cloned() {
            let NewEdge { target: new_target, kind } = creator.edge(alph);
            let target = match new_target {
                EdgeTarget::SelfCycle => new_state,
                EdgeTarget::Target(alph) => {
                    let index = self.index(alph)?;
                    assert!(index < tr_count);
                    self.edges[tr_start + index].target
                }
            };
            new_edges.push(Edge {
                target,
                transition: kind,
            });
        }

        self.edges.append(&mut new_edges);
        self.next_state += 1;
        if creator.is_final() {
            self.final_states.insert(new_state);
        }

        Ok(new_state)
    }
}

impl<A: Alphabet> Run<A> {
    pub fn next(&mut self, character: A) -> Result<(), Error> {
        let c = self.backing.index(character)?;
        let Edge { target, transition } = self.backing.edge(self.state, c).clone();
        let kind = self.backing.transition(transition).ok_or(Error::NoSuchEdge)?.clone();
        self.transition_to(target, kind)
    }

    pub fn is_final(&self) -> bool {
        self.backing.final_states.contains(&self.state)
    }

    fn transition_to(&mut self, target: State, kind: TransitionKind) -> Result<(), Error> {
        match kind {
            TransitionKind::Standard => Ok(self.state = target),
            TransitionKind::Creating { creator } => {
                let creator = self.backing.creator(creator.0).ok_or(Error::NoSuchCreator)?;
                self.create(target, creator)
            }
        }
    }

    fn create(&mut self, blueprint: State, creator: Arc<CreatorFn<A>>) -> Result<(), Error> {
        let new_state = self.backing.new_state(blueprint, creator)?;
        Ok(self.state = new_state)
    }
}
