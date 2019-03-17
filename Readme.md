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

## Features

* Dfa
  - word membership test
  - automaton pairing
* Nfa
  - word membership (dynamic powerset)
  - conversion to regex
  - conversion to dfa
* Regex
  - construction & printing over general alphabet
  - nothing particularly interesting yet

## TODO

There are more features planned and/or in development

* Converters–all of (`dfa`, `nfa`, `regex`) are equivalent
  - Dfa -> Nfa
  - Regex -> Nfa
  - Regx -> Dfa (directly, no det step)
* Joins, Compositions, Intersects, Differences, Equivalence checks
* Minimization
  - Quotienting (Hopcroft)
  - Dualization, Brzozowski’s algorithm, maybe.
* Finite-state Transducers–and compositions
  - Join, Pre, Post
  - Membership, Projections
  - Presburger arithmetic, Modeling solutions to linear integer (in-)equalities with finite automata
* Finite length languages
  - Minimization
  - Construction from NFA
  - Projection, Join, Pre, Post
* Decision Diagrams
  - Intersect, complement
  - Minimization
* Infinite-word automata
  - Büchi, Co-Büchi
  - NBA, Regex
  - DBA
  - Muller
  - Rabin
  - Streett
  - Parity
* Algorithms on infinite automata
  - Union, Intersection
  - Complement
  - Emptiness
  - Emerson-Lei (liveliness)
* Second-order logic
  - Finite word
  - Infinite universes

