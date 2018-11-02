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
* Converters–all currently planned automata classes are equivalent
* Minimalization
* Joins, Intersects, Differences, Equivalence checks
* Modeling solutions to linear integer (in-)equalities with finite automata

