use std::io::{Result, Write};

pub struct Node {
    /// Internal node marker, or None if automatically determined.
    mark: Option<usize>,

    /// A label to appear, or inserts the numeric ordering.
    label: Option<String>,
}

pub struct Edge;

/// Writes dot files with automatically chosen node names (to set attributes statelessly).
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

impl<W: Write> GraphWriter<W> {
    pub fn new(inner: W, family: Family, name: Option<String>) -> Self {
        unimplemented!()
    }

    /// Set the default node information.
    pub fn default_node(&mut self, default_node: Node) -> Result<()> {
        unimplemented!()
    }

    /// Set the default edge attributes.
    pub fn default_edge(&mut self, default_edge: Edge) -> Result<()> {
        unimplemented!()
    }

    /// Add a line segment, that is two or more connected nodes.
    ///
    /// Panics: when the iterator returned less than two nodes.
    pub fn segment<I>(&mut self, iter: I, options: Option<Edge>) -> Result<()> 
        where I: IntoIterator<Item=usize>
    {
        unimplemented!()
    }

    /// Set node information or create a blank node.
    pub fn node(&mut self, node: Node) -> Result<()> {
        unimplemented!()
    }

    /// In contrast to a simple drop, returns the inner writer.
    pub fn end_into_inner(self) -> (W, Result<()>) {
        unimplemented!()
    }
}

impl<'a, W: Write> GraphWriter<&'a mut W> {
    pub fn subgraph(&mut self, name: Option<String>) -> GraphWriter<&mut W> {
        unimplemented!()
    }
}

impl Family {
    fn edgeop(self) -> &'static str {
        match self {
            Family::Directed => "->",
            Family::Undirected => "--",
        }
    }
}
