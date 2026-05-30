//! # digraphx-rs
//!
//! Network optimization algorithms in Rust.
//!
//! A directed graph is treated as a **container of containers**:
//! - The outer container maps each node to its neighbors
//! - The inner container maps each neighbor to an edge weight
//!
//! This matches the Python `dict-of-dicts` and C++ `unordered_map<K, unordered_map<K, V>>`
//! patterns. Any type implementing the [`Graph`] trait can be used with the algorithms.
//!
//! ## Quick Start
//!
//! ```rust
//! use std::collections::HashMap;
//! use digraphx_rs::{Graph, NegCycleFinder};
//!
//! // A graph as HashMap<Node, HashMap<Node, Weight>>
//! let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
//! graph.insert("a", [("b", 1), ("c", 1)].into());
//! graph.insert("b", [("c", 1)].into());
//! graph.insert("c", [("a", -3)].into());
//!
//! let mut ncf = NegCycleFinder::new(&graph);
//! let mut dist: HashMap<&str, i32> =
//!     [("a", 0), ("b", 0), ("c", 0)].into();
//! let cycle = ncf.howard(&mut dist, |w| *w);
//! assert!(cycle.is_some());
//! ```

pub mod neg_cycle;
pub mod parametric;

#[cfg(feature = "std")]
pub mod logging;

pub mod prelude;

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;

// ---------------------------------------------------------------------------
// Graph trait — the graph as a container of containers
// ---------------------------------------------------------------------------

/// A directed graph viewed as a container of containers.
///
/// The basic assumption is that a graph maps each node to a container of its
/// outgoing edges.  Iterating over the outer container yields node–neighbors
/// pairs; iterating over the inner container yields (neighbor, weight) pairs.
///
/// # Provided implementations
///
/// | Container                          | Node    | Weight |
/// |------------------------------------|---------|--------|
/// | `HashMap<N, HashMap<N, W>>`       | `N`     | `W`    |
/// | `BTreeMap<N, BTreeMap<N, W>>`     | `N`     | `W`    |
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use digraphx_rs::Graph;
///
/// let g: HashMap<&str, HashMap<&str, i32>> =
///     [("a", [("b", 1)].into())].into();
/// assert_eq!(g.num_nodes(), 1);
/// ```
pub trait Graph {
    /// Identifier for a node (must be copyable, comparable, hashable).
    type Node: Copy + Eq + Hash;

    /// Edge weight (must support addition and ordering).
    type Weight: Copy + Add<Output = Self::Weight> + PartialOrd;

    /// Iterator over node identifiers.
    type Nodes: IntoIterator<Item = Self::Node>;

    /// Iterator over (neighbor, weight) pairs for a given node.
    type Neighbors: IntoIterator<Item = (Self::Node, Self::Weight)>;

    /// Return all nodes in the graph.
    fn nodes(&self) -> Self::Nodes;

    /// Return an iterator over the outgoing edges of `node`.
    fn neighbors(&self, node: Self::Node) -> Self::Neighbors;

    /// Return the number of nodes.
    fn num_nodes(&self) -> usize;
}

// ---------------------------------------------------------------------------
// Implementations for standard containers
// ---------------------------------------------------------------------------

/// Create a [`Graph`] from a slice of edges (node, neighbor, weight) triples.
/// Useful for small test graphs where writing out nested maps is tedious.
///
/// ```rust
/// use digraphx_rs::graph_from_edges;
/// use digraphx_rs::Graph;
/// let g = graph_from_edges(&[(0, 1, 1), (1, 2, 2), (2, 0, -3)]);
/// assert_eq!(g.num_nodes(), 3);
/// ```
pub fn graph_from_edges<N, W>(edges: &[(N, N, W)]) -> HashMap<N, HashMap<N, W>>
where
    N: Copy + Eq + Hash,
    W: Copy,
{
    let mut g: HashMap<N, HashMap<N, W>> = HashMap::new();
    for &(u, v, w) in edges {
        g.entry(u).or_default().insert(v, w);
        g.entry(v).or_default(); // ensure isolated nodes are present
    }
    g
}

// --- HashMap<N, HashMap<N, W>> -------------------------------------------

impl<N, W> Graph for HashMap<N, HashMap<N, W>>
where
    N: Copy + Eq + Hash,
    W: Copy + Add<Output = W> + PartialOrd,
{
    type Node = N;
    type Weight = W;
    type Nodes = std::vec::IntoIter<N>;
    type Neighbors = std::vec::IntoIter<(N, W)>;

    fn nodes(&self) -> Self::Nodes {
        self.keys().copied().collect::<Vec<_>>().into_iter()
    }

    fn neighbors(&self, node: N) -> Self::Neighbors {
        match self.get(&node) {
            Some(nbrs) => nbrs
                .iter()
                .map(|(&k, &v)| (k, v))
                .collect::<Vec<_>>()
                .into_iter(),
            None => Vec::new().into_iter(),
        }
    }

    fn num_nodes(&self) -> usize {
        self.len()
    }
}

// --- BTreeMap<N, BTreeMap<N, W>> -----------------------------------------

impl<N, W> Graph for std::collections::BTreeMap<N, std::collections::BTreeMap<N, W>>
where
    N: Copy + Eq + Hash + Ord,
    W: Copy + Add<Output = W> + PartialOrd,
{
    type Node = N;
    type Weight = W;
    type Nodes = std::vec::IntoIter<N>;
    type Neighbors = std::vec::IntoIter<(N, W)>;

    fn nodes(&self) -> Self::Nodes {
        self.keys().copied().collect::<Vec<_>>().into_iter()
    }

    fn neighbors(&self, node: N) -> Self::Neighbors {
        match self.get(&node) {
            Some(nbrs) => nbrs
                .iter()
                .map(|(&k, &v)| (k, v))
                .collect::<Vec<_>>()
                .into_iter(),
            None => Vec::new().into_iter(),
        }
    }

    fn num_nodes(&self) -> usize {
        self.len()
    }
}

// ---------------------------------------------------------------------------
// petgraph adapter (behind "petgraph" feature gate)
// ---------------------------------------------------------------------------

#[cfg(feature = "petgraph")]
pub mod petgraph_adapter {
    use super::*;
    use petgraph::graph::DiGraph;
    use petgraph::graph::NodeIndex;
    use petgraph::visit::EdgeRef;

    /// Adapter that lets a [`DiGraph`] be used as a [`Graph`].
    ///
    /// ```rust
    /// #[cfg(feature = "petgraph")]
    /// {
    ///     use petgraph::graph::DiGraph;
    ///     use digraphx_rs::{Graph, PetGraph};
    ///
    ///     let mut g = DiGraph::new();
    ///     let a = g.add_node(());
    ///     let b = g.add_node(());
    ///     g.add_edge(a, b, 1.0);
    ///
    ///     let pg = PetGraph(&g);
    ///     assert_eq!(pg.num_nodes(), 2);
    /// }
    /// ```
    pub struct PetGraph<'a, V, E>(pub &'a DiGraph<V, E>);

    impl<'a, V, E> Graph for PetGraph<'a, V, E>
    where
        V: 'a,
        E: Copy + Add<Output = E> + PartialOrd + 'a,
    {
        type Node = NodeIndex;
        type Weight = E;
        type Nodes = std::vec::IntoIter<NodeIndex>;
        type Neighbors = PetNeighbors<'a, E>;

        fn nodes(&self) -> Self::Nodes {
            self.0.node_indices().collect::<Vec<_>>().into_iter()
        }

        fn neighbors(&self, node: NodeIndex) -> Self::Neighbors {
            PetNeighbors {
                iter: self.0.edges(node),
            }
        }

        fn num_nodes(&self) -> usize {
            self.0.node_count()
        }
    }

    /// Iterator over petgraph edges, yielding (neighbor, weight).
    pub struct PetNeighbors<'a, E> {
        iter: petgraph::graph::Edges<'a, E, petgraph::Directed>,
    }

    impl<'a, E> Iterator for PetNeighbors<'a, E>
    where
        E: Copy,
    {
        type Item = (NodeIndex, E);
        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(|e| (e.target(), *e.weight()))
        }
    }
}

#[cfg(feature = "petgraph")]
pub use petgraph_adapter::PetGraph;

// ---------------------------------------------------------------------------
// Zero helper (internal)
// ---------------------------------------------------------------------------

/// Trait for additive identity.
pub trait Zero: Sized {
    fn zero() -> Self;
}

impl Zero for i32 {
    fn zero() -> Self {
        0
    }
}
impl Zero for f32 {
    fn zero() -> Self {
        0.0
    }
}
impl Zero for f64 {
    fn zero() -> Self {
        0.0
    }
}
impl<T: num::Integer + Clone> Zero for num::rational::Ratio<T> {
    fn zero() -> Self {
        num::rational::Ratio::new(
            <T as num::traits::Zero>::zero(),
            <T as num::traits::One>::one(),
        )
    }
}

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use neg_cycle::NegCycleFinder;
pub use neg_cycle::NegCycleFinderQ;
pub use parametric::{MaxParametricSolver, ParametricAPI};

/// Cycle type: a sequence of node IDs.
pub type Cycle<N> = Vec<N>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_trait_hashmap_nodes() {
        let g: HashMap<&str, HashMap<&str, i32>> =
            [("a", [("b", 1)].into()), ("b", [("a", 2)].into())].into();
        let nodes: Vec<_> = g.nodes().collect();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&"a"));
        assert!(nodes.contains(&"b"));
    }

    #[test]
    fn test_graph_trait_hashmap_neighbors() {
        let g: HashMap<&str, HashMap<&str, i32>> = [("a", [("b", 5)].into())].into();
        let nbrs: Vec<_> = g.neighbors("a").collect();
        assert_eq!(nbrs, vec![("b", 5)]);
    }

    #[test]
    fn test_graph_from_edges() {
        let g = graph_from_edges(&[(0, 1, 1), (1, 2, 2), (2, 0, -3)]);
        assert_eq!(g.num_nodes(), 3);
        let nbrs: Vec<_> = g.neighbors(0).collect();
        assert_eq!(nbrs, vec![(1, 1)]);
    }

    #[test]
    fn test_empty_graph() {
        let g: HashMap<i32, HashMap<i32, f64>> = HashMap::new();
        assert_eq!(g.num_nodes(), 0);
        assert!(g.nodes().collect::<Vec<_>>().is_empty());
        assert!(g.neighbors(0).collect::<Vec<_>>().is_empty());
    }
}
