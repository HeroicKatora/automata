use std::collections::{BTreeSet, HashSet, HashMap};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::{self, Write};
use std::iter::{Extend, FromIterator};

use super::{Alphabet, Ensure};
use super::dfa::Dfa;
use super::dot::{Family, Edge as DotEdge, GraphWriter, Node as DotNode};
use super::regex::{self, Regex, Op as RegOp};
use super::nondeterministic::NonDeterministic;

/// A node handle of an epsilon nfa.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Node(pub usize);

/// A node handle of a regex nfa.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct RegexNode(pub usize);

/// A non-deterministic automaton with epsilon transitions.
pub struct Nfa<A: Alphabet> {
    graph: NonDeterministic<A>,

    finals: HashSet<Node>,
}

pub struct NfaRegex<A: Alphabet>(A);

struct MultiMap<K: Hash + Eq, V> {
    inner: HashMap<K, Vec<V>>,
}

/// Symbol used during transformation to regex.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum EphermalSymbol {
    Start,
    Real(usize),
    End,
}

type EdgeKey = (EphermalSymbol, EphermalSymbol);

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
        let mut builder = NonDeterministic::builder();

        edge_iter.into_iter().for_each(
            |edge| builder.insert(edge.0, edge.1.as_ref(), edge.2));

        let finals = finals
            .into_iter()
            .map(Node)
            .collect();

        Nfa {
            graph: builder.finish(),
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
    pub fn to_regex(&self) -> Regex<A> {
        let mut cached = Regex::new().cached();

        // The epsilon symobl.
        let eps = cached.insert(RegOp::Epsilon);
        // Edge->handle list lut.
        let mut edges = MultiMap::default();

        use self::EphermalSymbol::{Start, Real, End};

        edges.insert((Start, Real(0)), eps);
        self.finals.iter().for_each(|&Node(real)| 
            edges.insert((Real(real), End), eps));

        for (real, node_edges) in self.graph.nodes() {
            for (symbol, target) in node_edges {
                let key = (Real(real), Real(target));
                let handle = match symbol {
                    Some(symbol) => cached.insert(RegOp::Match(*symbol)),
                    None => eps,
                };
                edges.insert(key, handle);
            }
        }

        // Remove intermediate nodes one-by-one
        (0..self.graph.nodes().len()).rev().for_each(|real_to_delete| {
            // 1. Merge phase
            edges.inner.values_mut().for_each(|alternatives| 
                if let Some(first) = alternatives.pop() {
                    let alt = alternatives.iter().cloned()
                        .fold(first, |alt1, alt2| 
                            cached.insert(RegOp::Or(alt1, alt2)));
                    alternatives.clear();
                    alternatives.push(alt);
                });


            // 2. Removal phase
            // 2.1 Separate all edges going to the node.
            let start_to = edges.inner.remove_entry(&(Start, Real(real_to_delete)));
            let to = (0..real_to_delete)
                .map(|from| (Real(from), Real(real_to_delete)))
                .filter_map(|key| edges.inner.remove_entry(&key))
                .chain(start_to)
                .filter_map(Self::get_single)
                .collect::<Vec<_>>();

            // 2.2 Separate all edges going out from the node.
            let from_to_end = edges.inner.remove_entry(&(Real(real_to_delete), End));
            let from = (0..real_to_delete)
                .map(|to| (Real(real_to_delete), Real(to)))
                .filter_map(|key| edges.inner.remove_entry(&key))
                .chain(from_to_end)
                .filter_map(Self::get_single)
                .collect::<Vec<_>>();

            // 2.3 Get the self-loop edge if it exists.
            let self_loop = edges.inner
                .remove_entry(&(Real(real_to_delete), Real(real_to_delete)))
                .and_then(Self::get_single);

            // ... and turn it into its `star` variant.
            let self_star = self_loop.map(|(_, handle)| cached.insert(RegOp::Star(handle)));

            // 2.4 Insert new paths for each going through.
            for ((from_node, _), from_handle) in to.iter().cloned() {
                for((_, to_node), to_handle) in from.iter().cloned() {
                    let from_to_handle = if let Some(self_star) = self_star {
                        let first_half = cached.insert(RegOp::Concat(from_handle, self_star));
                        cached.insert(RegOp::Concat(first_half, to_handle))
                    } else {
                        cached.insert(RegOp::Concat(from_handle, to_handle))
                    };
                    edges.insert((from_node, to_node), from_to_handle);
                }
            }
        });

        let start_to_end = edges.inner.remove_entry(&(Start, End))
            .expect("Ephermal start to end node should be the only left");
        let start_to_end = Self::get_single(start_to_end)
            .expect("Start to end path must exist").1;
        assert!(edges.inner.is_empty());

        let regex = cached.into_inner();
        assert_eq!(Some(start_to_end), regex.root(), "Start to end must be the regex root");
        regex
    }

    /// Convert to a dfa using the powerset construction.
    ///
    /// Since the alphabet can not be deduced purely from transitions, `alphabet_extension`
    /// provides a way to indicate additional symbols.
    pub fn into_dfa<I: IntoIterator<Item=A>>(self, alphabet_extension: I) -> Dfa<A> {
        // The epsilon transition closure of reachable nodes.
        let initial_state: BTreeSet<_> = self.epsilon_reach(Node(0));
        let alphabet = self.graph.alphabet()
            .iter().cloned()
            .chain(alphabet_extension)
            .collect::<Vec<_>>();

        let mut state_map = vec![(initial_state.clone(), 0)].into_iter().collect::<HashMap<_, _>>();
        let mut pending = vec![initial_state];
        let mut edges = Vec::new();
        let mut finals = Vec::new();

        while let Some(next) = pending.pop() {
            let from = state_map[&next];
            for ch in alphabet.iter().cloned() {
                let basic = next.iter().cloned()
                    .flat_map(|Node(idx)| {
                        let mut edges = self.graph.edges(idx).unwrap();
                        edges.restrict_to(Some(&ch));
                        edges.targets()
                    })
                    .collect::<HashSet<_>>();

                let closure = basic.into_iter()
                    .map(|state| self.epsilon_reach(Node(state)))
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

        for (from, edges) in self.graph.nodes() {
            for (label, to) in edges {
                let label = match label {
                    Some(ch) => format!("{}", ch),
                    None => "ε".into(),
                };

                let edge = DotEdge { 
                    label: Some(label.into()),
                    .. DotEdge::none()
                };

                writer.segment([from, to].iter().cloned(), Some(edge))?;
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
        // The epsilon transition closure of reachable nodes.
        let mut states: HashSet<_> = self.epsilon_reach(Node(0));

        for ch in sequence {
            let next = states.into_iter()
                .flat_map(|Node(idx)| {
                    let mut edges = self.graph.edges(idx).unwrap();
                    edges.restrict_to(Some(&ch));
                    edges.targets()
                })
                .collect::<HashSet<_>>();

            let epsilon_reach = next.into_iter()
                .map(|state| self.epsilon_reach(Node(state)))
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

        while let Some(Node(next)) = todo.pop() {
            let mut edges = self.graph.edges(next).unwrap();
            edges.restrict_to(None);
            edges.map(|(_, target)| Node(target))
                .filter(|&target| reached.insert_new(target))
                .for_each(|target| todo.push(target));
        }

        reached
    }

    fn get_single((key, mut val): (EdgeKey, Vec<regex::Handle>)) 
        -> Option<(EdgeKey, regex::Handle)> 
    {
        val.pop().map(|val| (key, val))
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
    pub fn into_nfa(self) -> Nfa<A> {
        unimplemented!()
    }
}

impl<A: Alphabet> From<Nfa<A>> for NfaRegex<A> {
    fn from(_automaton: Nfa<A>) -> Self {
        unimplemented!()
    }
}

impl<K: Hash + Eq, V> MultiMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        let mapped = self.inner.entry(key)
            .or_insert_with(Vec::new);
        mapped.push(value)
    }
}

impl<K: Hash + Eq, V> Default for MultiMap<K, V> {
    fn default() -> Self {
        MultiMap {
            inner: Default::default(),
        }
    }
}

impl<K: Hash + Eq, V> FromIterator<(K, V)> for MultiMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = (K, V)> 
    {
        let mut set = MultiMap::default();
        iter.into_iter()
            .for_each(|(key, value)| set.insert(key, value));
        set
    }
}

impl<K: Hash + Eq, V> Extend<(K, V)> for MultiMap<K, V> {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item = (K, V)> 
    {
        iter.into_iter()
            .for_each(|(key, value)| self.insert(key, value));
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

        let automaton = automaton.into_dfa(vec!['2']);

        assert!( automaton.contains("".chars()));
        assert!( automaton.contains("1".chars()));
        assert!( automaton.contains("1001".chars()));
        assert!( automaton.contains("0000".chars()));
        assert!(!automaton.contains("11".chars()));
        assert!(!automaton.contains("2".chars()));
    }
}
