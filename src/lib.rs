mod deterministic;
mod nondeterministic;

pub mod dfa;
pub mod dot;
pub mod nfa;
pub mod regex;

use std::fmt::Debug;
use std::hash::Hash;

/// A generic alphabet.
///
/// `Eq`, `Ord`, and `Hash` are assumed to be provided for the finite set to
/// simplify data structures by allowing use of different map and set types.
///
/// An interesting case may be using `Option<A> where A: Alphabet` which
/// provides the possibility to consider an 'anything else' case and an actually
/// infinte alphabet of which the automaton just uses a finite set.
pub trait Alphabet: Hash + Eq + Debug + Clone + Copy + Ord { }

impl<T> Alphabet for T where T: Hash + Eq + Debug + Clone + Copy + Ord { }

/// Ensure the length of a container as if by resize(max(len, n)).
trait Ensure<T> {
    fn ensure_with<F>(&mut self, n: usize, new: F) where F: FnMut() -> T;

    fn ensure(&mut self, n: usize, item: T) where T: Clone {
        self.ensure_with(n, || item.clone())
    }

    fn ensure_default(&mut self, n: usize) where T: Default {
        self.ensure_with(n, T::default);
    }
}

impl<T: Clone> Ensure<T> for Vec<T> {
    fn ensure_with<F>(&mut self, n: usize, creator: F) 
        where F: FnMut() -> T
    {
        let new_len = self.len().max(n);
        self.resize_with(new_len, creator);
    }
}
