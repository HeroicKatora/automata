use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::io::{self, Write};

use super::{Alphabet, Ensure};
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

    finals: Vec<Node>,
}

pub struct NfaRegex<A: Alphabet>(A);

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

    /// All the state reachable purely by epsilon transitions.
    fn epsilon_reach(&self, start: Node) -> HashSet<Node> {
        let mut reached = HashSet::new();
        let mut todo = Vec::new();

        reached.insert(start);
        todo.push(start);

        while let Some(next) = todo.pop() {
            self.epsilons[next.0].iter()
                .filter(|&&target| reached.insert(target))
                .map(|&target| todo.push(target));
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
