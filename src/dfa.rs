use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Debug};
use std::io::{self, Write};

use super::{Alphabet, Ensure};
use super::dot::{Family, Edge as DotEdge, GraphWriter, Node as DotNode};
use super::regex::Regex;

/// A node handle.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Node(pub usize);

pub struct Dfa<A: Alphabet> {
    /// Edges of the graph, each list sorted against the alphabet.
    edges: Vec<Vec<(A, Node)>>,

    /// Final or accepting states.
    finals: HashSet<Node>,

    /// A sorted list of the alphabet, to allow quick comparison.
    alphabet: Vec<A>,
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

        for (from, a, to) in edge_iter.into_iter() {
            edges.ensure_default(from + 1);
            edges.ensure_default(to + 1);
            check.ensure_default(from + 1);
            check.ensure_default(to + 1);
            
            edges[from].push((a.clone(), Node(to)));
            check[from].insert(a);
            states.insert(from);
            states.insert(to);
        }

        let finals = finals.into_iter()
            .inspect(|c| check.resize(c + 1, HashSet::new()))
            .map(Node)
            .collect();

        let alphabet = check.pop();
        if let Some(sample) = alphabet.as_ref() {
            if let Some(err) = check.iter().find(|&s| s != sample) {
                panic!("Different outgoing edges alphabet: {:?} vs {:?}", &sample, &err);
            }
        }

        let mut alphabet: Vec<_> = alphabet.unwrap().into_iter().collect();
        alphabet.sort_unstable();

        for edge_list in edges.iter_mut() {
            // There are never any duplicates and now the indices correspond to
            // the indices in the alphabet list.
            edge_list.sort_unstable();
        }

        Dfa {
            edges,
            finals,
            alphabet,
        }
    }

    /// Checks if the input word is contained in the language.
    pub fn contains<I: IntoIterator<Item=A>>(&self, sequence: I) -> bool {
        let mut sequence = sequence.into_iter();
        let mut state = 0;

        while let Some(ch) = sequence.next() {
            let edges = &self.edges[state];
            let Node(next) = edges.iter()
                .find(|e| e.0 == ch)
                .map(|e| e.1)
                .expect("Mismatch between DFA alphabet and word alphabet");
            state = next;
        }

        self.finals.contains(&Node(state))
    }

    pub fn to_regex(self) -> Regex<A> {
        unimplemented!()
    }

    pub fn write_to(&self, output: &mut Write) -> io::Result<()> 
        where for<'a> &'a A: Display
    {
        let mut writer = GraphWriter::new(output, Family::Directed, None)?;

        for (from, edges) in self.edges.iter().enumerate() {
            for (label, to) in edges.iter() {
                let edge = DotEdge { 
                    label: Some(format!("{}", label).into()),
                    .. DotEdge::none()
                };

                writer.segment([from, to.0].iter().cloned(), Some(edge))?;
            }
        }

        for Node(fin) in self.finals.iter().cloned() {
            let node = DotNode {
                peripheries: Some(2),
                .. DotNode::none()
            };
            writer.node(fin.into(), Some(node))?;
        }

        writer.end_into_inner().1
    }

    /// The alphabet is the set of symbols in words of that language.
    pub fn alphabet(&self) -> &[A] {
        self.alphabet.as_slice()
    }

    /// Minimize the automata into its language partition.
    ///
    /// Contrary to NFAs, the resulting automaton is guaranteed to be a minimal
    /// automaton exactly equivalent to the languages minimal DFA.
    pub fn minimized(&self) -> Self {
        unimplemented!()
    }

    /// Pairs two automata with a given binary boolean operation
    ///
    /// If there are no final states, returns `None`.
    pub fn pair(&self, rhs: &Self, decider: &Fn(bool, bool) -> bool) -> Option<Self> {
        assert!(self.alphabet() == rhs.alphabet(), "Automata alphabets differ");

        let mut assigned = HashMap::new();
        let mut working = vec![(0, 0, 0)];
        let mut edges: Vec<Vec<(A, Node)>> = Vec::new();
        let mut finals = HashSet::new();
        assigned.insert((0, 0), 0);

        while let Some((left, right, self_id)) = working.pop() {
            let decide = decider(
                self.finals.contains(&Node(left)),
                rhs.finals.contains(&Node(right)));

            if decide {
                finals.insert(Node(self_id));
            }

            edges.ensure_default(self_id + 1);
            let edges = &mut edges[self_id];

            for (pos, symbol) in self.alphabet().iter().enumerate() {
                // Gets updated when we encounter an existing node.
                let mut node_id = assigned.len();

                let new_left = self.edges[left][pos].1;
                let new_right = rhs.edges[right][pos].1;

                assigned.entry((new_left.0, new_right.0))
                    .and_modify(|&mut id| node_id = id)
                    .or_insert_with(|| {
                        working.push((new_left.0, new_right.0, node_id));
                        node_id
                    });

                edges.push((symbol.clone(), Node(node_id)));
            }
        }

        if finals.is_empty() {
            None
        } else {
            Some(Dfa {
                edges,
                finals,
                alphabet: self.alphabet.clone(),
            })
        }
    }

    /// Like `pair` but only determines if the result would be an empty automaton.
    ///
    /// This speeds up operations such as equivalence checks.
    pub fn pair_empty(&self, _rhs: &Self, _decider: &Fn(bool, bool) -> bool) -> bool {
        unimplemented!()
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
}
