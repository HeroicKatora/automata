//! Strongly typed representation for writing `dot` files.
//!
//! See <https://graphviz.gitlab.io/_pages/doc/info/lang.html> for the full specification. Only
//! parts which were relevant have been translated. Some redundant parts have no representation
//! such as allowing multiple `[]` attribute lists behind a `node` or `edge` directive.
use std::borrow::Cow;
use std::fmt;
use std::io::{self, Write};

/// Optionally contains the possible node attributes.
#[derive(Clone, Default)]
pub struct Node {
    /// A label to appear, can be html or an escaped string.
    pub label: Option<Id>,

    /// Number of stacked polygon lines for the outer shape.
    ///
    /// Final/Accepting states in automaton are marked by two peripheral lines. The default value
    /// for this attribute is 1.
    pub peripheries: Option<usize>,
}

/// Optionally contains the possible edge attributes.
#[derive(Clone, Default)]
pub struct Edge {
    /// A label to appear, can be html or an escaped string.
    pub label: Option<Id>,
}

/// Writes dot files.
///
/// Node names are chosen automatically from the given index.
pub struct GraphWriter<W: Write> {
    inner: Option<W>,

    /// The edgeop must correspond to the chosen graph family.
    edgeop: Family,
}

#[derive(Clone, Copy, Debug)]
pub enum Family {
    Directed,
    Undirected,
}

/// An identifier, has several uses in the language (`ID`).
///
/// IDs representing attributes have a constant defined in this struct.
///
/// TODO: `node_id` is currently restricted to this, but could have port and another specifier.
#[derive(Clone, Debug)]
pub struct Id(IdEnum);

#[derive(Clone, Debug)]
enum IdEnum {
    /// A c-style identifier or a string numeral.
    ///
    /// Any string of alphabetic ([a-zA-Z\200-\377]) characters, underscores ('_') or digits ([0-9]), not beginning with a digit;
    ///
    /// A numeral `[-]?(.[0-9]+ | [0-9]+(.[0-9]*)? )`;
    Raw(Cow<'static, str>),

    /// A standard unsigned numeral, encoded to `(.[0-9]+ | [0-9]+(.[0-9]*)? )`;
    Numeral(usize),

    /// A standard signed numeral, encoded to `[-]?(.[0-9]+ | [0-9]+(.[0-9]*)? )`;
    INumeral(isize),

    /// A C string escaped string.
    ///
    /// Any double-quoted string ("...") possibly containing escaped quotes (\");
    Str(Cow<'static, str>),

    // An html escaped string.
    // Html(String),
}

/// Trait for structures that can be dumped as a dot graph.
pub trait DotGraph {
    /// Write a dot representation.
    fn dot_graph<W>(&self, to: W) -> io::Result<()>;
}

/// Extension to `std::io::Write` for writing dot graphs.
pub trait WriteDot: io::Write {
    fn write_dot<D: DotGraph>(&mut self, graph: D) -> io::Result<()> {
        graph.dot_graph(self)
    }
}

impl<W: Write> GraphWriter<W> {
    /// Begins writing a graph with the given parameters.
    pub fn new(mut inner: W, family: Family, name: Option<Id>) -> io::Result<Self> {
        if let Some(name) = name {
            write!(&mut inner, "{} {} {{\n", family.name(), name)?;
        } else {
            write!(&mut inner, "{} {{\n", family.name())?;
        }

        Ok(GraphWriter {
            inner: Some(inner),
            edgeop: family,
        })
    }

    /// Set the default node information.
    pub fn default_node(&mut self, default_node: Node) -> io::Result<()> {
        let fmt = self.inner.as_mut().unwrap();
        write!(fmt, "\tnode [{}];", default_node)
    }

    /// Set the default edge attributes.
    pub fn default_edge(&mut self, default_edge: Edge) -> io::Result<()> {
        let fmt = self.inner.as_mut().unwrap();
        write!(fmt, "\tedge [{}];", default_edge)
    }

    /// Add a line segment, that is two or more connected nodes.
    ///
    /// Panics: when the iterator returned less than two nodes.
    ///
    /// TODO: the spec allows adhoc subgraphs instead of node specifiers.
    pub fn segment<I, T>(&mut self, iter: I, options: Option<Edge>) -> io::Result<()> 
        where I: IntoIterator<Item=T>, T: Into<Id>
    {
        let fmt = self.inner.as_mut().unwrap();

        let mut iter = iter.into_iter();

        let begin = iter.next().unwrap();
        let end = iter.next().unwrap();

        write!(fmt, "\t{} {} {} ", begin.into(), self.edgeop.edgeop(), end.into())?;

        while let Some(next) = iter.next() {
            write!(fmt, "{} {} ", self.edgeop.edgeop(), next.into())?;
        }

        if let Some(options) = options {
            write!(fmt, "[{}];\n", options)
        } else {
            write!(fmt, ";\n")
        }
    }

    /// Set node information or create a new node.
    pub fn node(&mut self, id: Id, node: Option<Node>) -> io::Result<()> {
        let fmt = self.inner.as_mut().unwrap();

        write!(fmt, "\t{} ", id)?;

        if let Some(options) = node {
            write!(fmt, "[{}];\n", options)
        } else {
            write!(fmt, ";\n")
        }
    }

    /// In contrast to a simple drop, returns the inner writer.
    pub fn end_into_inner(mut self) -> (W, io::Result<()>) {
        let mut inner = self.inner.take().unwrap();
        let result = inner.write_all(b"}\n");

        (inner, result)
    }
}

impl<W: io::Write> Drop for GraphWriter<W> {
    fn drop(&mut self) {
        if let Some(writer) = self.inner.as_mut() {
            writer.write_all(b"}\n").unwrap();
        }
    }
}

impl<'a, W: Write> GraphWriter<&'a mut W> {
    pub fn subgraph(&mut self, _name: Option<String>) -> GraphWriter<&mut W> {
        unimplemented!()
    }
}

impl Node {
    /// A node with no attributes.
    ///
    /// May be used in constructors to default assign remaining members with `.. Node::none()`.
    pub fn none() -> Self {
        Node::default()
    }
}

impl Edge {
    /// An edge with no attributes.
    ///
    /// May be used in constructors to default assign remaining members with `.. Edge::none()`.
    pub fn none() -> Self {
        Edge::default()
    }
}

impl Family {
    /// The keyword at the top of the dot file.
    fn name(self) -> &'static str {
        match self {
            Family::Directed => "digraph",
            Family::Undirected => "graph",
        }
    }

    fn edgeop(self) -> &'static str {
        match self {
            Family::Directed => "->",
            Family::Undirected => "--",
        }
    }
}

impl Id {
    const LABEL: Id = Id(IdEnum::Raw(Cow::Borrowed("label")));
    const PERIPHERIES: Id = Id(IdEnum::Raw(Cow::Borrowed("peripheries")));
}

impl IdEnum {
    /// Constructs the ID representation for this string.
    ///
    /// Automatically chooses between raw ascii, digits and string encoded versions of identifiers,
    /// whichever has the least conversion and usage overhead.
    ///
    /// Panics: When the escaped string does not fit inside a string.
    fn from_string_like<T>(name: T) -> Self
        where T: Into<Cow<'static, str>> 
    {
        let name = name.into();

        let raw_identifier = |c: char| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_';
        let raw_identifier_begin = |c: &char| c.is_ascii_alphabetic() || *c == '_';

        if name.as_ref().is_empty() {
            return IdEnum::Str(name)
        }

        if name.as_ref().chars().all(|c| c.is_ascii_digit()) {
            return IdEnum::Raw(name)
        }

        if name.as_ref().chars().all(raw_identifier) && name.as_ref().chars().nth(0).filter(raw_identifier_begin).is_some() {
            return IdEnum::Raw(name)
        }

        // Simply escape single quotes once. Since the default charset is UTF-8, all other strings are fine.
        let quote_count = name.bytes().filter(|c| *c == b'"').count();
        let name = if quote_count > 0 {
            let mut string = name.into_owned();

            // Escape every '"' with '\'.
            //
            // This operation is safe since we only prepend b'\' (a valid UTF-8 sequence) to b'"'.
            //
            // More in-depth:
            // Because b'"' can only appear inside char boundaries in any other situation but as a
            // standalone b'"' character, the new sequence keeps all char boundaries intact and has
            // only inserted a new valid char sequence, b'\'. Hence the new string is still valid
            // UTF-8.
            //
            // This cannot be performed safely and efficiently, since we can only utilize
            // `String::insert` to add single characters but doing so would be O(n·m) where n is
            // the length of the string and m is the number of '"' chars. In comparison, this
            // operation is O(n) since we only move each character at most once.
            unsafe{
                let vec = string.as_mut_vec();
                let mut num_inserts = quote_count;

                assert!(num_inserts > 0, "contains at least one quote");
                assert!(vec.len() > 0, "contains at least one quote");
                let mut text_end = vec.len();

                // Controlled panic
                let new_len = vec.len().checked_add(num_inserts)
                    .expect("escaped string would not fit");

                // Add all the additional escape characters. We move them around as a contiguous
                // slice later, each time leaving behind the last slash where it belongs.
                vec.resize(new_len, b'\\');
                let mut new_end = new_len;

                let slice = vec.as_mut_slice();
                // Pointer arithmetic on the slice elements (u8) later can be done as usize
                // arithmetic with this base address without wrapping.
                let base_ptr = slice.as_ptr() as usize;

                // Copy & insert the new characters.
                while num_inserts > 0 {
                    let tail = slice[..text_end]
                        // Get all the text following the last '"'
                        .rsplit(|c| *c == b'"').next().unwrap()
                        .as_ptr() as usize;

                    assert!(tail > base_ptr, "at least one quote left infront");

                    // Calculate the index of the quote character
                    let quote_index = tail
                        .checked_sub(base_ptr).unwrap()
                        .checked_sub(1).unwrap();
                    let relative_end = text_end
                        .checked_sub(quote_index).unwrap();

                    // Move the uninitialized part infront of the slice. Remember that the slice of
                    // new characters consists only of slashes.
                    slice[quote_index..new_end].rotate_right(relative_end);

                    // Now leave behind one slash and set all new values.  Expecting clang magic to
                    // remove the unwrap because he can prove that `num_insert > 1` at this point.
                    num_inserts = num_inserts.checked_sub(1).unwrap();
                    new_end = quote_index + num_inserts;
                    text_end = quote_index;
                }
            }

            string.into()
        } else {
            name
        };

        IdEnum::Str(name)
    }
}

impl From<Cow<'static, str>> for Id {
    fn from(id: Cow<'static, str>) -> Self {
        Id(IdEnum::from_string_like(id))
    }
}

impl From<&'static str> for Id {
    fn from(id: &'static str) -> Self {
        Id(IdEnum::from_string_like(Cow::from(id)))
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Id(IdEnum::from_string_like(Cow::from(id)))
    }
}

impl From<usize> for Id {
    fn from(id: usize) -> Self {
        Id(IdEnum::Numeral(id))
    }
}

impl From<isize> for Id {
    fn from(id: isize) -> Self {
        Id(IdEnum::INumeral(id))
    }
}

impl fmt::Display for IdEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            IdEnum::Raw(id) => write!(f, "{}", id),
            IdEnum::Numeral(id) => write!(f, "{}", id),
            IdEnum::INumeral(id) => write!(f, "{}", id),
            IdEnum::Str(id) => write!(f, "\"{}\"", id),
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

/// Formats the node attributes (`a_list` in specification terms).
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(label) = self.label.as_ref() {
            write!(f, "{}={},", Id::LABEL, label)?;
        }

        if let Some(peripheries) = self.peripheries.clone() {
            write!(f, "{}={},", Id::PERIPHERIES, peripheries)?;
        }

        Ok(())
    }
}

/// Formats the edge attributes (`a_list` in specification terms).
impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(label) = self.label.as_ref() {
            write!(f, "{}={},", Id::LABEL, label)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers() {
        assert_eq!(format!("{}", Id::from("abc")), "abc");
        assert_eq!(format!("{}", Id::from(0usize)), "0");
        assert_eq!(format!("{}", Id::from(-1isize)), "-1");
        assert_eq!(format!("{}", Id::from("123")), "123");
        assert_eq!(format!("{}", Id::from("a string with spaces")), r#""a string with spaces""#);
        assert_eq!(format!("{}", Id::from("\"")), r#""\"""#);
        assert_eq!(format!("{}", Id::from("")), r#""""#);
    }
}
