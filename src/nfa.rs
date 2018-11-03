use std::collections::{BTreeSet, HashSet, HashMap};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::{self, Write};

use super::{Alphabet, Ensure};
use super::dfa::Dfa;
use super::dot::{Family, Edge as DotEdge, GraphWriter, Node as DotNode};
use super::regex::Regex;

/// A node handle of an epsilon nfa.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Node(pub usize);

/// A node handle of a regex nfa.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct RegexNode(pub usize);

/// A non-deterministic automaton with epsilon transitions.
pub struct Nfa<A: Alphabet> {
    /// Edges like a dfa but may contain duplicate entries for first component.
    edges: Vec<Vec<(A, Node)>>,

    /// Stores epsilon transitions separately.
    ///
    /// This makes it easier to find the epsilon reachability graph.
    epsilons: Vec<Vec<Node>>,

    finals: HashSet<Node>,
}

pub struct NfaRegex<A: Alphabet>(A);

trait InsertNew<T> {
    fn insert_new(&mut self, item: T) -> bool;
}

/// A non-deterministic finite epsilon automaton.
impl<A: Alphabet> Nfa<A> {
    /// Build a epsilon nfa from the connecting edges and final states.
    ///
    /// States are numbered in an arbitrary order, except the start label 0. Emulate multiple start
    /// states by creating epsilon transitions from the 0 state. Technically, the final state could
    /// also be collapsed into a single state but that is sometimes more tedious to work with
    /// (especially when transforming a regex in an nfa).
    pub fn from_edges<I, V>(edge_iter: I, finals: V) -> Nfa<A>
    where 
        I: IntoIterator<Item=(usize, Option<A>, usize)>,
        V: IntoIterator<Item=usize>, 
        A: Clone + Debug,
    {
        let mut edges = vec![Vec::new()];
        let mut epsilons = vec![Vec::new()];

        edge_iter.into_iter().for_each(|edge| match edge {
            (from, Some(label), to) => {
                edges.ensure_default(from + 1);
                edges.ensure_default(to + 1);
                epsilons.ensure_default(from + 1);
                epsilons.ensure_default(to + 1);

                edges[from].push((label, Node(to)));
            },
            (from, None, to) => {
                edges.ensure_default(from + 1);
                edges.ensure_default(to + 1);
                epsilons.ensure_default(from + 1);
                epsilons.ensure_default(to + 1);

                epsilons[from].push(Node(to));
            }
        });

        let finals = finals
            .into_iter()
            .map(Node)
            .collect();

        Nfa {
            edges,
            epsilons,
            finals,
        }
    }

    /// First collapse all output states (compress the automaton).
    ///     This is done by adding new initial/final state and
    ///     epsilon transition to/from the previous states.
    ///
    /// Then merge edges into (a+b) as much as possible
    ///
    /// ```text
    ///
    ///                    /–a–\
    /// 0 ––a+b–> 1   <  0 |    1
    ///                    \–b–/
    /// ```
    ///
    /// Then remove a single state (2), complicated:
    ///
    /// ```text
    /// 0     3          0––(02)s*(23)––>1
    ///  \   /            \             /
    ///   \ /              \––––\ /––––/
    ///    2 = s      >          X
    ///   / \              /––––/ \––––\
    ///  /   \            /             \
    /// 1     4          1––(12)s*(24)––>4
    /// ```
    ///
    /// relying on 2 not being a final nor initial state, and existance
    /// due to existance of two different pathas from inital to end with
    /// a common state (transitive paths were removed before). The regex
    /// grows by up to a factor or 3! with each remove state.
    ///
    /// '2' need not have 4 inputs or a self-loop. Note that 0,1,3,4 need
    /// not be unique! When 2 has other than 4 inputs, apply the path 
    /// transformation to all accordingly. Shortcut all paths through 2
    /// with new regexes combining all of 2s loops. When 2 does not have 
    /// a self-loop, just treat this as e or ignore it.
    ///
    /// => O(|V|³) length, preprocessing not even included (although its
    /// growth factor is smaller).
    pub fn to_regex(self) -> Regex<A> {
        unimplemented!()
    }

    /// Convert to a dfa using the powerset construction.
    ///
    /// Since the alphabet can not be deduced purely from transitions, `alphabet_extension`
    /// provides a way to indicate additional symbols.
    pub fn to_dfa<I: IntoIterator<Item=A>>(self, alphabet_extension: I) -> Dfa<A> {
        // The epsilon transition closure of reachable nodes.
        let initial_state: BTreeSet<_> = self.epsilon_reach(Node(0));
        let alphabet = self.edges.iter()
            .flat_map(|edges| edges.iter().map(|edge| edge.0))
            .chain(alphabet_extension.into_iter())
            .collect::<HashSet<A>>()
            .into_iter()
            .collect::<Vec<A>>();

        let mut state_map = vec![(initial_state.clone(), 0)].into_iter().collect::<HashMap<_, _>>();
        let mut pending = vec![initial_state];
        let mut edges = Vec::new();
        let mut finals = Vec::new();

        while let Some(next) = pending.pop() {
            let from = state_map.get(&next).unwrap().clone();
            for ch in alphabet.iter().cloned() {
                let basic = next.iter().cloned()
                    .flat_map(|Node(idx)| self.edges[idx].iter()
                        .filter(|edge| edge.0 == ch)
                        .map(|edge| edge.1))
                    .collect::<HashSet<_>>();
                let closure = basic.into_iter()
                    .map(|state| self.epsilon_reach(state))
                    .fold(BTreeSet::new(), |left, right| left.union(&right).cloned().collect());
                let is_final = closure.iter().any(|st| self.finals.contains(st));
                let new_index = state_map.len();

                let nr = if let Some(to) = state_map.get(&closure).cloned() {
                    to
                } else {
                    state_map.insert(closure.clone(), new_index);
                    pending.push(closure);
                    new_index
                };

                if is_final {
                    finals.push(nr)
                }

                edges.push((from, ch, nr));
            }
        }

        Dfa::from_edges(edges, finals)
    }

    /// Write the nfa into the dot format.
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

        for (from, edges) in self.epsilons.iter().enumerate() {
            for to in edges.iter() {
                let edge = DotEdge {
                    label: Some("ε".into()),
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

    /// Checks if the input word is contained in the language.
    ///
    /// The check is realized by performing a dynamic powerset construction of the resulting dfa.
    /// For simplicity, previous states are not stored anywhere, resulting in abysmal expected
    /// runtime for anything but the smallest cases.
    ///
    /// Consider converting the nfa to an equivalent dfa before querying, especially when
    /// performing multiple successive queries.
    pub fn contains<I: IntoIterator<Item=A>>(&self, sequence: I) -> bool {
        let mut sequence = sequence.into_iter();

        // The epsilon transition closure of reachable nodes.
        let mut states: HashSet<_> = self.epsilon_reach(Node(0));

        while let Some(ch) = sequence.next() {
            let next = states.into_iter()
                .flat_map(|Node(idx)| self.edges[idx].iter()
                      .filter(|edge| edge.0 == ch)
                      .map(|edge| edge.1))
                .collect::<HashSet<_>>();
            let epsilon_reach = next.iter().cloned()
                .map(|state| self.epsilon_reach(state))
                .fold(HashSet::new(), |left, right| left.union(&right).cloned().collect());
            states = epsilon_reach;
        }

        !states.is_disjoint(&self.finals)
    }

    /// All the state reachable purely by epsilon transitions.
    fn epsilon_reach<R>(&self, start: Node) -> R 
        where R: Default + InsertNew<Node>
    {
        let mut reached = R::default();
        let mut todo = Vec::new();

        reached.insert_new(start);
        todo.push(start);

        while let Some(next) = todo.pop() {
            self.epsilons[next.0].iter()
                .filter(|&&target| reached.insert_new(target))
                .for_each(|&target| todo.push(target));
        }

        reached
    }
}

/// A non-deterministic finite automaton with regex transition guards.
impl<A: Alphabet> NfaRegex<A> {
    /// General idea, local to edges:
    /// ```text
    ///                          a
    ///                         /–\
    ///                         \ /
    /// 0 ––a*––> 1   >  0 ––e–> 2 ––e–> 1
    ///
    /// 0 ––ab––> 1   >  0 ––a–> 2 ––b–> 1
    ///
    ///                    /–a–\
    /// 0 ––a+b–> 1   >  0 |    1
    ///                    \–b–/
    /// ```
    pub fn to_nfa(self) -> Nfa<A> {
        unimplemented!()
    }
}

impl<A: Alphabet> From<Nfa<A>> for NfaRegex<A> {
    fn from(automaton: Nfa<A>) -> Self {
        unimplemented!()
    }
}

impl<T> InsertNew<T> for BTreeSet<T> where T: Eq + Ord {
    fn insert_new(&mut self, item: T) -> bool {
        self.insert(item)
    }
}

impl<T> InsertNew<T> for HashSet<T> where T: Eq + Hash {
    fn insert_new(&mut self, item: T) -> bool {
        self.insert(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_format() {
        let automaton = Nfa::from_edges(vec![
            (0, Some('0'), 0),
            (0, None, 1),
            (0, Some('1'), 1),
            (1, Some('0'), 0),
        ], vec![1]);

        let mut output = Vec::new();
        automaton.write_to(&mut output)
            .expect("failed to format to dot file");
        let output = String::from_utf8(output)
            .expect("output should be utf8 encoded");
        assert_eq!(output, r#"digraph {
	0 -> 0 [label=0,];
	0 -> 1 [label=1,];
	1 -> 0 [label=0,];
	0 -> 1 [label="ε",];
	1 [peripheries=2,];
}
"#);
    }

    #[test]
    fn contains() {
        let automaton = Nfa::from_edges(vec![
            (0, Some('0'), 0),
            (0, None, 1),
            (0, Some('1'), 1),
            (1, Some('0'), 0),
        ], vec![1]);

        assert!( automaton.contains("".chars()));
        assert!( automaton.contains("1".chars()));
        assert!( automaton.contains("1001".chars()));
        assert!( automaton.contains("0000".chars()));
        assert!(!automaton.contains("11".chars()));
        assert!(!automaton.contains("2".chars()));
    }

    #[test]
    fn convert_to_dfa() {
        let automaton = Nfa::from_edges(vec![
            (0, Some('0'), 0),
            (0, None, 1),
            (0, Some('1'), 1),
            (1, Some('0'), 0),
        ], vec![1]);

        let automaton = automaton.to_dfa(vec!['2']);

        assert!( automaton.contains("".chars()));
        assert!( automaton.contains("1".chars()));
        assert!( automaton.contains("1001".chars()));
        assert!( automaton.contains("0000".chars()));
        assert!(!automaton.contains("11".chars()));
        assert!(!automaton.contains("2".chars()));
    }
}
