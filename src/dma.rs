//! A deterministic, self-modifying automata.
//!
//! Tests a new kind of automata maybe capable of recognizing `a^nb^nc^n` (i.e. more powerful than
//! context free) but still O(n) space and O(n) time.
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::IntoIterator;
use std::sync::Arc;

use crate::dot::{Family, GraphWriter, Node};
use super::Alphabet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Creator(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Transition(pub usize);

#[derive(Clone, Copy)]
enum TransitionKind {
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
    pub kind: Option<Transition>,
}

pub enum EdgeTarget<A> {
    /// That edge should be back to the new node.
    SelfCycle,

    /// The edge should point to some node connected to the target.
    Target(A),
}

pub trait CreatorFn<A>: fmt::Debug {
    fn is_final(&self) -> bool;
    fn edge(&self, character: A) -> NewEdge<A>;
}

pub struct SimpleCreator<F> {
    pub is_final: bool,
    pub label: String,
    pub edge: F,
}

#[derive(Clone, Copy, Debug)]
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
    pub fn new(alphabet: &[A]) -> Self {
        Dma {
            alphabet: Vec::from(alphabet),
            lut: alphabet.iter().cloned().enumerate().map(|(idx, c)| (c, idx)).collect(),
            next_state: 0,
            final_states: HashSet::new(),
            edges: Vec::new(),
            transitions: vec![TransitionKind::Standard],
            creator: Vec::new(),
        }
    }

    /// The alphabet (not necessarily in normal order).
    pub fn alphabet(&self) -> &[A] {
        self.alphabet.as_slice()
    }

    /// Begin a new run with this machine.
    pub fn run(&self) -> Run<A> {
        assert!(self.next_state > 0, "Can not run an empty automaton");

        Run {
            backing: self.clone(),
            state: State(0),
        }
    }

    /// Create a new kind of transition with the specified creator.
    ///
    /// Note that the transition count is incremental for your convenience. The caller does not
    /// need to use the return value.
    pub fn new_transition<C: CreatorFn<A> + 'static>(&mut self, creator: C) -> Transition {
        let creator = self.new_creator_impl(creator);
        self.new_transition_impl(creator)
    }

    fn new_transition_impl<C: Into<Creator>>(&mut self, tr: C) -> Transition {
        let new_id = Transition(self.transitions.len());
        self.transitions.push(TransitionKind::Creating { creator: tr.into() });
        new_id
    }

    fn new_creator_impl<C: CreatorFn<A> + 'static>(&mut self, creator: C) -> Creator {
        let new_id = Creator(self.creator.len());
        self.creator.push(Arc::new(creator));
        new_id
    }

    /// Get the transition index of the default transition kind that does not create a node.
    pub fn standard_transition(&self) -> Transition {
        Transition(0)
    }

    /// Supply edges for a new state in the alphabet order.
    ///
    /// # Panics
    ///
    /// When the edge count is not consistent with the alphabet.
    pub fn new_state(&mut self, is_final: bool, edges: &[(State, Transition)]) -> State {
        assert!(edges.len() == edges.len());

        self.add_state(is_final, edges.iter().map(|&(target, transition)| Edge {
            target,
            transition,
        }))
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

    fn derive_state(&mut self, blueprint: State, creator: Creator) -> Result<State, Error> {
        let own_kind = Transition(creator.0 + 1);
        let creator = self.creator(creator.0)
            .ok_or(Error::NoSuchCreator)?;
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
            let (target, alt_kind) = match new_target {
                EdgeTarget::SelfCycle => (new_state, own_kind),
                EdgeTarget::Target(alph) => {
                    let index = self.index(alph)?;
                    assert!(index < tr_count);
                    let edge = self.edges[tr_start + index];
                    (edge.target, edge.transition)
                }
            };
            let kind = kind.unwrap_or(alt_kind);
            new_edges.push(Edge {
                target,
                transition: kind,
            });
        }

        let new_state = self.add_state(creator.is_final(), new_edges.drain(..));
        Ok(new_state)
    }

    fn add_state<E>(&mut self, is_final: bool, edges: E) -> State 
        where E: IntoIterator<Item=Edge>
    {
        let new_state = State(self.next_state);
        self.edges.extend(edges);
        self.next_state += 1;
        if is_final {
            self.final_states.insert(new_state);
        }
        new_state
    }

    fn print<W: std::io::Write>(&self, mut dot: GraphWriter<W>) -> std::io::Result<()> {
        struct PrTransition<'a, A: Alphabet>(&'a Dma<A>, Option<&'a TransitionKind>);

        impl<'a, A: Alphabet> fmt::Debug for PrTransition<'a, A> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self.1 {
                    None => write!(f, "Invalid"),
                    Some(TransitionKind::Standard) => write!(f, "Standard"),
                    Some(TransitionKind::Creating { creator }) => match self.0.creator.get(creator.0) {
                        None => write!(f, "Invalid creator"),
                        Some(creator) => write!(f, "({:?})", creator),
                    }
                }
            }
        }

        let tr_count = self.alphabet.len();
        assert!(self.edges.len() >= self.next_state*tr_count);

        for from in 0..self.next_state {
            if self.final_states.contains(&State(from)) {
                dot.node(from.into(), Some(Node {
                    label: None,
                    peripheries: Some(2),
                }))?;
            }

            for (i, edge) in self.edges[from*tr_count..from*tr_count + tr_count].iter().enumerate() {
                let transition = PrTransition(self, self.transitions.get(edge.transition.0));
                dot.segment(
                    [from, edge.target.0].into_iter().cloned(),
                    Some(format!("{:?}:  {:?}", 
                            self.alphabet[i],
                            transition,
                        ).into()))?;
            }
        }

        Ok(())
    }

    pub fn write_to(&self, write: impl std::io::Write) -> std::io::Result<()> {
        self.print(GraphWriter::new(write, Family::Directed, None)?)
    }
}

impl<A: Alphabet> Run<A> {
    pub fn next(&mut self, character: A) -> Result<(), Error> {
        let c = self.backing.index(character)?;
        let Edge { target, transition } = self.backing.edge(self.state, c).clone();
        let kind = self.backing.transition(transition).ok_or(Error::NoSuchEdge)?.clone();
        self.transition_to(target, kind)
    }

    pub fn matches(&mut self, iter: impl IntoIterator<Item=A>) -> Result<bool, Error> {
        iter.into_iter().try_for_each(|ch| self.next(ch))?;
        Ok(self.is_final())
    }

    pub fn is_final(&self) -> bool {
        self.backing.final_states.contains(&self.state)
    }

    fn print<W: std::io::Write>(&self, dot: GraphWriter<W>) -> std::io::Result<()> {
        self.backing.print(dot)
    }

    pub fn write_to(&self, write: impl std::io::Write) -> std::io::Result<()> {
        self.print(GraphWriter::new(write, Family::Directed, None)?)
    }

    fn transition_to(&mut self, target: State, kind: TransitionKind) -> Result<(), Error> {
        match kind {
            TransitionKind::Standard => Ok(self.state = target),
            TransitionKind::Creating { creator } => self.create(target, creator),
        }
    }

    fn create(&mut self, blueprint: State, creator: Creator) -> Result<(), Error> {
        let new_state = self.backing.derive_state(blueprint, creator)?;
        Ok(self.state = new_state)
    }
}

impl<F> fmt::Debug for SimpleCreator<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl<F, A> CreatorFn<A> for SimpleCreator<F>
    where A: Alphabet, F: Fn(A) -> NewEdge<A> 
{
    fn is_final(&self) -> bool {
        self.is_final
    }

    fn edge(&self, character: A) -> NewEdge<A> {
        (self.edge)(character)
    }
}

impl From<usize> for State {
    fn from(idx: usize) -> State {
        State(idx)
    }
}

impl From<usize> for Transition {
    fn from(idx: usize) -> Transition {
        Transition(idx)
    }
}

impl From<usize> for Creator {
    fn from(idx: usize) -> Creator {
        Creator(idx)
    }
}

impl From<Option<Creator>> for TransitionKind {
    fn from(c: Option<Creator>) -> Self {
        match c {
            None => TransitionKind::Standard,
            Some(c) => TransitionKind::Creating {
                creator:c
            },
        }
    }
}
