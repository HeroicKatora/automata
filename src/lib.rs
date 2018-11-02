pub mod dfa;
pub mod dot;
pub mod nfa;
pub mod regex;

use std::fmt::Debug;
use std::hash::Hash;

pub trait Alphabet: Hash + Eq + Debug + Clone + Copy { }

impl<T> Alphabet for T where T: Hash + Eq + Debug + Clone + Copy { }

/// Ensure the length of a container as if by resize(max(len, n)).
trait Ensure<T: Clone> {
    fn ensure(&mut self, n: usize, item: T);
    fn ensure_default(&mut self, n: usize) where T: Default {
        self.ensure(n, T::default());
    }
}

impl<T: Clone> Ensure<T> for Vec<T> {
    fn ensure(&mut self, n: usize, item: T) {
        let new_len = self.len().max(n);
        self.resize(new_len, item);
    }
}
