mod dfa;
mod nfa;

trait Alphabet: Hash + Eq + Debug + Clone + Copy { }

impl<T> Alphabet for T where T: Hash + Eq + Debug + Clone + Copy { }

fn main() {
    println!("Hello, world!");
}
