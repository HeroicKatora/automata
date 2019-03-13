//! Like the tests but dumps the constructed automatons into an output folder.
extern crate automata;

use std::fs;
use std::process;

use automata::dfa::Dfa;
#[cfg(feature = "self_experiments")]
use automata::dma::{Dma, EdgeTarget, NewEdge, SimpleCreator};
use automata::nfa::Nfa;

fn main() {
    fs::create_dir_all("./output")
        .expect("Failed to create output directory");
    
    dfa();
    nfa();
    #[cfg(feature = "self_experiments")]
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

#[cfg(feature = "self_experiments")]
fn dma() {
    // $ is the new symbol to stay connected to the invalid sink.
    let mut automaton = Dma::new(&['a', 'b', 'c', '$']);
    let standard = automaton.standard_transition();
    // 0: standard, 1: init, 2: push, 3: unpush, 4: finish.
    let init_transition = automaton.new_transition(SimpleCreator {
        is_final: false,
        label: "init".into(),
        edge: |alph| match alph {
            'a' => NewEdge {
                // a is the creating edge, with push creator.
                target: EdgeTarget::SelfCycle,
                kind: Some(2.into()),
            },
            'b' => NewEdge {
                // b targets the one we are coming from, with finish transition.
                target: EdgeTarget::Target('a'),
                kind: Some(4.into()),
            },
            'c' | '$' => NewEdge {
                // '$' targets the invalid sink.
                target: EdgeTarget::Target('$'),
                kind: Some(0.into()),
            },
            _ => unreachable!("Never called outside alphabet"),
        },
    });

    // the push transition.
    automaton.new_transition(SimpleCreator {
        is_final: false,
        label: "a_push".into(),
        edge: |alph| match alph {
            'a' => NewEdge {
                // a is the next creating edge.
                target: EdgeTarget::SelfCycle,
                kind: Some(2.into()),
            },
            'b' => NewEdge {
                // b targets the one we are coming from, with unpush transition.
                target: EdgeTarget::From,
                kind: Some(3.into()),
            },
            'c' | '$' => NewEdge {
                // '$' targets the invalid sink in every node.
                target: EdgeTarget::Target('$'),
                kind: Some(0.into()),
            },
            _ => unreachable!("Never called outside alphabet"),
        },
    });

    // the unpush transition.
    automaton.new_transition(SimpleCreator {
        is_final: false,
        label: "b_push".into(),
        edge: |alph| match alph {
            'a' => NewEdge {
                // a is invalid, only bs from now on. This is why we have the $ symbol.
                target: EdgeTarget::Target('$'),
                kind: Some(0.into()),
            },
            'b' => NewEdge {
                // b the next unpush transition. **Copy** the type of transition that is there.
                target: EdgeTarget::Target('b'),
                kind: None,
            },
            'c' => NewEdge {
                target: EdgeTarget::From,
                kind: Some(5.into()),
            },
            '$' => NewEdge {
                // '$' targets the invalid sink.
                target: EdgeTarget::Target('$'),
                kind: Some(0.into()),
            },
            _ => unreachable!("Never called outside alphabet"),
        },
    });

    // the final transition.
    automaton.new_transition(SimpleCreator {
        is_final: true,
        label: "b_finish".into(),
        edge: |alph| match alph {
            'c' => NewEdge {
                target: EdgeTarget::From,
                kind: Some(5.into()),
            },
            // all following content is invalid.
            _ => NewEdge {
                target: EdgeTarget::Target('$'),
                kind: Some(0.into()),
            },
        }
    });

    automaton.new_state(true, &[
        (0.into(), init_transition),
        (1.into(), standard), // Into garbage state
        (1.into(), standard),
        (1.into(), standard),
    ]);
    automaton.new_state(false, &[ // Garbage state
        (1.into(), standard),
        (1.into(), standard),
        (1.into(), standard),
        (1.into(), standard),
    ]);

    let mut output = Vec::new();
    automaton.write_to(&mut output).unwrap();
    fs::write("./output/dma.dot", &mut output)
        .expect("Failed to write dfa dot file");

    {
        output.clear();
        let mut run = automaton.run();
        assert!(run.matches("".chars()).unwrap());
        run.write_to(&mut output).unwrap();
        fs::write("./output/dma_eps.dot", &mut output)
            .expect("Failed to write dma run dot file");
    }

    {
        output.clear();
        let mut run = automaton.run();
        assert!(run.matches("aaabbb".chars()).unwrap());
        run.write_to(&mut output).unwrap();
        fs::write("./output/dma_aaabbb.dot", &mut output)
            .expect("Failed to write dma run dot file");
    }
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
