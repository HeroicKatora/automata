use super::regex::Regex;

pub struct NfaEps;

pub struct NfaRegex;

/// 
impl NfaEps {
    /// First collapse all output states (compress the automaton).
    ///     This is done by adding new initial/final state and
    ///     epsilon transition to/from the previous states.
    ///
    /// Then merge edges into (a+b) as much as possible
    ///
    ///                    /–a-\
    /// 0 --a+b-> 1   <  0 |    1
    ///                    \-b-/
    ///
    /// Then remove a single state (2), complicated:
    ///
    /// 0     3          0--(02)s*(23)-->1
    ///  \   /            \             /
    ///   \ /              \----\ /----/
    ///    2 = s      >          X
    ///   / \              /----/ \----\
    ///  /   \            /             \
    /// 1     4          1--(12)s*(24)-->4
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
    pub fn to_regex(self) -> Regex {
        unimplemented!()
    }
}

impl NfaRegex {
    /// General idea, local to edges:
    ///                          a
    ///                         /–\
    ///                         \ /
    /// 0 --a*--> 1   >  0 --e-> 2 --e-> 1
    ///
    /// 0 --ab--> 1   >  0 --a-> 2 --b-> 1
    ///
    ///                    /–a-\
    /// 0 --a+b-> 1   >  0 |    1
    ///                    \-b-/
    pub fn to_fnaeps(self) -> NfaEps {
        unimplemented!()
    }
}

impl From<NfaEps> for NfaRegex {
    fn from(automaton: NfaEps) -> Self {
        unimplemented!()
    }
}
