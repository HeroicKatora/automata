use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Debug};
use std::io::{self, Write};

use crate::{Alphabet, Ensure};
use crate::deterministic::{Deterministic, Target};
use crate::dot::{Family, Edge as DotEdge, GraphWriter, Node as DotNode};
use crate::nfa::{self, Nfa};
use crate::regex::Regex;

/// A node handle.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Node(pub usize);

pub struct Dfa<A: Alphabet> {
    /// The deterministic graph, also stores the alphabet.
    graph: Deterministic<A>,

    /// Final or accepting states.
    finals: HashSet<Target>,
}

impl<A: Alphabet> Dfa<A> {
    /// Build a dfa from the connecting edges and final states.
    ///
    /// States are numbered in an arbitrary order, except the start label 0. The automaton will
    /// deduce the used alphabet subset automatically and test whether it has been used
    /// consistently.
    pub fn from_edges<I, V>(edge_iter: I, finals: V) -> Dfa<A>
    where 
        I: IntoIterator<Item=(usize, A, usize)>,
        V: IntoIterator<Item=usize>, 
        A: Clone + Debug,
    {
        let mut edges = vec![Vec::new()];
        let mut check = vec![HashSet::new()];
        let mut states = HashSet::new();
        states.insert(0);

        for (from, a, to) in edge_iter {
            edges.ensure_default(from + 1);
            edges.ensure_default(to + 1);
            check.ensure_default(from + 1);
            check.ensure_default(to + 1);
            
            edges[from].push((a, to));
            check[from].insert(a);
            states.insert(from);
            states.insert(to);
        }

        let finals = finals.into_iter()
            .inspect(|c| check.resize(c + 1, HashSet::new()))
            .map(Target::make)
            .collect();

        let alphabet = check.pop();
        if let Some(sample) = alphabet.as_ref() {
            if let Some(err) = check.iter().find(|&s| s != sample) {
                panic!("Different outgoing edges alphabet: {:?} vs {:?}", &sample, &err);
            }
        }

        let mut graph = Deterministic::new(alphabet.unwrap());

        for edge_list in edges.iter_mut() {
            // There are never any duplicates and now the indices correspond to
            // the indices in the alphabet list.
            edge_list.sort_unstable();
            let node = graph.node();
            let edges = graph.iter_edges_mut(node);
            let edge_list = edge_list.iter().cloned();

            for ((_, target), (_, edge_target)) in edges.zip(edge_list) {
                *target = Some(Target::make(edge_target));
            }
        }

        assert!(graph.is_complete());

        Dfa {
            graph,
            finals,
        }
    }

    /// Checks if the input word is contained in the language.
    pub fn contains<I: IntoIterator<Item=A>>(&self, sequence: I) -> bool {
        let mut state = Target::ZERO;

        for ch in sequence {
            let next = self.graph
                .edges(state).unwrap()
                [ch].unwrap();
            state = next;
        }

        self.finals.contains(&state)
    }

    pub fn write_to(&self, output: &mut Write) -> io::Result<()> 
        where for<'a> &'a A: Display
    {
        let mut writer = GraphWriter::new(output, Family::Directed, None)?;

        for from in self.graph.iter() {
            for (label, to) in self.graph.iter_edges(from) {
                let edge = DotEdge { 
                    label: Some(format!("{}", label).into()),
                    .. DotEdge::none()
                };

                writer.segment([from.index(), to.index()].iter().cloned(), Some(edge))?;
            }
        }

        for fin in self.finals.iter().cloned() {
            let node = DotNode {
                peripheries: Some(2),
                .. DotNode::none()
            };
            writer.node(fin.index().into(), Some(node))?;
        }

        writer.end_into_inner().1
    }

    /// The alphabet is the set of symbols in words of that language.
    pub fn alphabet(&self) -> &[A] {
        self.graph.alphabet()
    }

    /// Minimize the automata into its language partition.
    ///
    /// NOT YET IMPLEMENTED!
    ///
    /// Contrary to NFAs, the resulting automaton is guaranteed to be a minimal
    /// automaton exactly equivalent to the languages minimal DFA.
    pub fn minimized(&self) -> Self {
        unimplemented!()
    }

    /// Pairs two automata with a given binary boolean operation
    ///
    /// If there are no final states, returns `None`.
    pub fn pair<F>(&self, rhs: &Self, decider: F) -> Option<Self>
        where F: Fn(bool, bool) -> bool
    {
        assert!(self.alphabet() == rhs.alphabet(), "Automata alphabets differ");

        let mut assigned = HashMap::new();
        let mut working = vec![(Target::ZERO, Target::ZERO, Target::ZERO)];
        let mut graph = Deterministic::new(self.alphabet().iter().cloned());
        let mut finals = HashSet::new();

        assigned.insert((Target::ZERO, Target::ZERO), Target::ZERO);
        graph.node();

        while let Some((left, right, self_id)) = working.pop() {
            let decide = decider(
                self.finals.contains(&left),
                rhs.finals.contains(&right));

            if decide {
                finals.insert(self_id);
            }

            let left_edges = self.graph.iter_edges(left);
            let right_edges = rhs.graph.iter_edges(right);

            for ((symbol, new_left), (_, new_right)) in left_edges.zip(right_edges) {
                let node_id = match assigned.entry((new_left, new_right)) {
                    Entry::Occupied(occupied) => *occupied.get(),
                    Entry::Vacant(vacant) => {
                        let new_id = graph.node();
                        working.push((new_left, new_right, new_id));
                        vacant.insert(new_id);
                        new_id
                    },
                };

                let mut edges = graph.edges_mut(self_id).unwrap();
                edges[*symbol] = Some(node_id);
            }
        }

        if finals.is_empty() {
            None
        } else {
            Some(Dfa {
                graph,
                finals
            })
        }
    }

    /// Like `pair` but only determines if the result would be an empty automaton.
    ///
    /// This speeds up operations such as equivalence checks. Equivalent to
    /// `empty` but terminates immediately whenever a state would be inserted
    /// into the set of final states. This is because the state would be reachable 
    /// by construction. Therefore we also need not record edges and state ids.
    ///
    /// Note that you can also use this as a universality test by inverting the
    /// decider function. A DFA is universal iff all of its reachable states are 
    /// final, which is the same as checking that in the complement all reachable
    /// states are non-final.
    pub fn pair_empty<F>(&self, rhs: &Self, decider: F) -> bool
        where F: Fn(bool, bool) -> bool
    {
        assert!(self.alphabet() == rhs.alphabet(), "Automata alphabets differ");

        let mut assigned = HashSet::new();
        let mut working = vec![(Target::ZERO, Target::ZERO)];
        assigned.insert((Target::ZERO, Target::ZERO));

        while let Some((left, right)) = working.pop() {
            let decide = decider(
                self.finals.contains(&left),
                rhs.finals.contains(&right));

            if decide {
                return false;
            }

            let left_edges = self.graph.iter_edges(left);
            let right_edges = rhs.graph.iter_edges(right);

            for ((_, new_left), (_, new_right)) in left_edges.zip(right_edges) {
                if assigned.insert((new_left, new_right)) {
                    working.push((new_left, new_right))
                }
            }
        }

        true
    }

    /// Get an equivalent nfa.
    pub fn to_nfa(&self) -> Nfa<A> {
        let graph = (&self.graph).into();
        let finals = self.finals.iter().cloned()
            .map(Target::index)
            .map(nfa::Node)
            .collect();
        Nfa::from_nondeterministic(graph, finals)
    }


    /// Turn this into an equivalent nfa.
    ///
    /// Compared to `to_nfa` this may be able to reuse some of the allocations and might be more
    /// efficient.
    pub fn into_nfa(self) -> Nfa<A> {
        // FIXME: maybe nondeterministic can spare a copy.
        self.to_nfa()
    }

    /// Get an equivalent regex.
    pub fn to_regex(&self) -> Regex<A> {
        // This is not the best but still pretty efficient.
        self.to_nfa().to_regex()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_format() {
        let automaton = Dfa::from_edges(vec![
            (0, '0', 0),
            (0, '1', 1),
            (1, '0', 2),
            (1, '1', 0),
            (2, '0', 1),
            (2, '1', 2),
        ], vec![1]);

        let mut output = Vec::new();
        automaton.write_to(&mut output)
            .expect("failed to format to dot file");
        let output = String::from_utf8(output)
            .expect("output should be utf8 encoded");
        assert_eq!(output, r#"digraph {
	0 -> 0 [label=0,];
	0 -> 1 [label=1,];
	1 -> 2 [label=0,];
	1 -> 0 [label=1,];
	2 -> 1 [label=0,];
	2 -> 2 [label=1,];
	1 [peripheries=2,];
}
"#);
    }

    #[test]
    fn contains() {
        let automaton = Dfa::from_edges(vec![
            (0, '0', 0),
            (0, '1', 1),
            (1, '0', 2),
            (1, '1', 0),
            (2, '0', 1),
            (2, '1', 2),
        ], vec![1]);
        
        assert!( automaton.contains("1".chars()));
        assert!( automaton.contains("100".chars()));
        assert!(!automaton.contains("0".chars()));
        assert!(!automaton.contains("10".chars()));
        assert!(!automaton.contains("".chars()));
    }

    #[test]
    fn pairing() {
        // Accepts even length words
        let automaton_2 = Dfa::from_edges(vec![
            (0, '.', 1),
            (1, '.', 0),
        ], vec![0]);

        // Accepts words with `len(w) % 3 == 0`
        let automaton_3 = Dfa::from_edges(vec![
            (0, '.', 1),
            (1, '.', 2),
            (2, '.', 0),
        ], vec![0]);

        let accept_6_0 = automaton_2.pair(&automaton_3, &|lhs, rhs| lhs & rhs).unwrap();
        assert!( accept_6_0.contains("".chars()));
        assert!(!accept_6_0.contains(".".chars()));
        assert!(!accept_6_0.contains("..".chars()));
        assert!(!accept_6_0.contains("...".chars()));
        assert!(!accept_6_0.contains("....".chars()));
        assert!(!accept_6_0.contains(".....".chars()));
        assert!( accept_6_0.contains("......".chars()));

        let accept_6_1 = automaton_2.pair(&automaton_3, &|lhs, rhs| lhs | rhs).unwrap();
        assert!( accept_6_1.contains("".chars()));
        assert!(!accept_6_1.contains(".".chars()));
        assert!( accept_6_1.contains("..".chars()));
        assert!( accept_6_1.contains("...".chars()));
        assert!( accept_6_1.contains("....".chars()));
        assert!(!accept_6_1.contains(".....".chars()));
        assert!( accept_6_1.contains("......".chars()));
    }

    #[test]
    fn pairing_empty() {
        // Accepts even length words
        let automaton_even = Dfa::from_edges(vec![
            (0, '.', 1),
            (1, '.', 0),
        ], vec![0]);

        // Accepts odd length words
        let automaton_odd = Dfa::from_edges(vec![
            (0, '.', 1),
            (1, '.', 0),
        ], vec![1]);

        // Accepts words with `len(w) % 3 == 0`
        let automaton_3 = Dfa::from_edges(vec![
            (0, '.', 1),
            (1, '.', 2),
            (2, '.', 0),
        ], vec![0]);

        assert!(!automaton_even.pair_empty(&automaton_3, |lhs, rhs| lhs & rhs));
        assert!(!automaton_even.pair_empty(&automaton_3, |lhs, rhs| lhs | rhs));

        assert!( automaton_even.pair_empty(&automaton_odd, |lhs, rhs| lhs & rhs));
        assert!( automaton_even.pair_empty(&automaton_odd, |lhs, rhs| !(lhs | rhs)));
    }
}
