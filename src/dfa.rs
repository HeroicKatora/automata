use std::collections::Set;

use super::regex::Regex;

/// A node handle.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Node(pub usize);

struct Dfa<Alphabet: Eq + Hash> {
    edges: Vec<Vec<(Alphabet, Node)>>,
    final: Vec<Node>,
}

impl<A: Eq + Hash> Dfa<A> {
    pub fn from_edges<I, V>(edges: I, final: V) -> Dfa 
    where 
        I: IntoIterator<Item=(usize, Alphabet, usize)>,
        V: Into<Vec<usize>>, 
        A: Clone + Debug,
    {
        let mut edges = Vec::new();
        let mut check = Vec::new();

        for (from, a, to) in edges.into() {
            edges.resize(from + 1, Vec::new());
            check.resize(from + 1, Set::new());
            
            check.insert(a.clone());
            edges.push((a, to));
        }

        if let Some(check) = check.pop() {
            if let Some(err) = check.iter().find(|s| s != &check) {
                panic!("Different outgoing edges alphabet: {:?} vs {:?}", &check, &err);
            }
        }

        unimplemented!()
    }

    pub fn to_regex(self) -> Regex {
        unimplemented!()
    }
}

