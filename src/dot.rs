pub struct Node {
    /// Internal node marker, or None if automatically determined.
    mark: Option<usize>,

    /// A label to appear, or inserts the numeric ordering.
    label: Option<String>,
}

pub struct Edge;

/// Writes dot files with automatically chosen node names (to set attributes statelessly).
trait DotWrite {
    /// Set the default node information.
    fn node(&mut self, default_node: Node);

    /// Set the default edge attributes.
    fn edge(&mut self, default_edge: Edge);

    /// Add a line segment, that is two or more connected nodes.
    ///
    /// Panics: when the iterator returned less than two nodes.
    fn segment<I>(&mut self, iter: I, options: Option<Edge>) where I: Iterator<Item=usize>;

    /// Set node information or create a blank node.
    fn node(&mut self, node: Node);
}
