use super::Alphabet;
use super::nfa::Nfa;

/// Represents regular expressions over some finite alphabet.
pub struct Regex<A: Alphabet>(A);

impl<A: Alphabet> Regex<A> {
    /// Idea:
    ///
    /// Is like a regex-labeled nfa with only one final state.
    pub fn to_nfa(self) -> Nfa<A> {
        unimplemented!()
    }
}
