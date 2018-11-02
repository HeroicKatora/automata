pub mod dfa;
pub mod dot;
pub mod nfa;
pub mod regex;

use std::fmt::Debug;
use std::hash::Hash;

trait Alphabet: Hash + Eq + Debug + Clone + Copy { }

impl<T> Alphabet for T where T: Hash + Eq + Debug + Clone + Copy { }
