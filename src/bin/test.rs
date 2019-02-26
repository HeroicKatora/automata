//! Like the tests but dumps the constructed automatons into an output folder.
extern crate automata;

use std::fs;
use std::process;

use automata::dfa::Dfa;
use automata::dma::{Dma, EdgeTarget, NewEdge, SimpleCreator};
use automata::nfa::Nfa;

fn main() {
    fs::create_dir_all("./output")
        .expect("Failed to create output directory");
    
    dfa();
    nfa();
    dma();

    convert();
    // view();
}

fn dfa() {
    let automaton = Dfa::from_edges(vec![
        (0, '0', 0),
        (0, '1', 1),
        (1, '0', 2),
        (1, '1', 0),
        (2, '0', 1),
        (2, '1', 2),
    ], vec![1]);

    let mut output = Vec::new();
    automaton.write_to(&mut output).unwrap();
    fs::write("./output/dfa.dot", output)
        .expect("Failed to write dfa dot file");
}

fn nfa() {
    let automaton = Nfa::from_edges(vec![
        (0, Some('0'), 0),
        (0, None, 1),
        (0, Some('1'), 1),
        (1, Some('0'), 0),
    ], vec![1]);

    let mut output = Vec::new();
    automaton.write_to(&mut output).unwrap();
    fs::write("./output/nfa.dot", output)
        .expect("Failed to write dfa dot file");
}

fn dma() {
    let mut automaton = Dma::new(&['a', 'b']);
    let standard = automaton.standard_transition();
    let ctransition = automaton.new_transition(SimpleCreator {
        is_final: false,
        label: "pusher".into(),
        edge: |alph| {
            match alph {
                'a' => NewEdge {
                    target: EdgeTarget::SelfCycle,
                    kind: 0.into(),
                },
                'b' => NewEdge {
                    target: EdgeTarget::Target('a'),
                    kind: 1.into(),
                },
                _ => unreachable!("Never called outside alphabet"),
            }
        },
    });
    automaton.new_state(false, &[
        (0.into(), ctransition),
        (1.into(), standard), // Into garbage state
    ]);
    automaton.new_state(false, &[ // Garbage state
        (1.into(), standard),
        (1.into(), standard),
    ]);

    let mut output = Vec::new();
    automaton.write_to(&mut output).unwrap();
    fs::write("./output/dma.dot", output)
        .expect("Failed to write dfa dot file");
}

// Try to run `dot` for all files to convert to png, optionally.
fn convert() {
    let _: Result<(), _> = fs::read_dir("./output")
        .expect("Failed to iterate over output files")
        .filter_map(|path| path.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("dot"))
        .map(|path| process::Command::new("dot")
             .arg(path)
             .arg("-Tpng")
             .arg("-O")
             .spawn()
             .and_then(|mut child| child.wait())
             .map(|_exit| ()))
        .collect();
}

fn view() {
    // Try to spawn `feh` to view the output but it's not necessary.
    let _ = process::Command::new("feh")
        .arg("./output")
        .spawn();
}
