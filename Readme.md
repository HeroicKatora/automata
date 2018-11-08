# Automata

A rust library to model several different automata and algorithms. In its
current form includes the following automata:

* Deterministic Finite Automata (DFA)
* Non-deterministic Finite Automata (NFA) with epsilon transitions

## Visualization & Debugging

```
cargo run --bin test
```

Contained is an exporter to the popular `dot` format. A separate test binary
also showcases construction and use of the exporter for example automata of each
family and try to immediately show the images. This is handled by invoking
`dot`–for conversion to `png`– and `feh`–for rendering the images–so make sure
those are installed for the best effect.

## NOTICE: Ownership transfer

This crate has recently changed ownership from
[gsingh93](https://github.com/gsingh93/rust-automata) and are now maintained
following a university course. Supplementary material on theoretical questions
might appear as part of the documentation.

## Tests

```
cargo test
```

All units are tested, a bit stricter than their interface requirements, by
automated unit tests. This includes automata construction, language recognition,
word rejection, and the exporter.

## TODO

There are more features planned and/or in development

* Regex
* Converters–all of (`dfa`, `nfa`, `regex`) are equivalent
* Joins, Compositions, Intersects, Differences, Equivalence checks
* Minimalization
* Modeling solutions to linear integer (in-)equalities with finite automata
* Finite-state Transducers–and compositions

