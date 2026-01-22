//! # digraphx-rs
//!
//! Network optimization algorithms in Rust.
//!
//! ## Features
//!
//! - **Bellman-Ford** - Shortest path algorithm with support for negative edge weights
//! - **Negative Cycle Detection** - Find and report cycles with negative total weight
//! - **Parametric Algorithms** - Maximum cycle ratio and related optimization problems
//! - **Howard's Algorithm** - Efficient negative cycle detection for large graphs
//!
//! ## Quick Start
//!
//! ```rust
//! use digraphx_rs::bellman_ford;
//! use petgraph::Graph;
//!
//! let mut g = Graph::new();
//! let a = g.add_node(());
//! let b = g.add_node(());
//! g.extend_with_edges([(a, b, 4.0)]);
//!
//! let paths = bellman_ford(&g, a).unwrap();
//! println!("Distances: {:?}", paths.distances);
//! ```
//!
//! ## Modules
//!
//! - [`neg_cycle`] - Negative cycle detection algorithms
//! - [`parametric`] - Parametric optimization algorithms
//!
//! # digraphx-rs
//!
//! Network optimization algorithms in Rust.
//!
//! ## Features
//!
//! - **Bellman-Ford** - Shortest path algorithm with support for negative edge weights
//! - **Negative Cycle Detection** - Find and report cycles with negative total weight
//! - **Parametric Algorithms** - Maximum cycle ratio and related optimization problems
//! - **Howard's Algorithm** - Efficient negative cycle detection for large graphs
//!
//! ## Quick Start
//!
//! ```rust
//! use digraphx_rs::bellman_ford;
//! use petgraph::Graph;
//!
//! let mut g = Graph::new();
//! let a = g.add_node(());
//! let b = g.add_node(());
//! g.extend_with_edges([(a, b, 4.0)]);
//!
//! let paths = bellman_ford(&g, a).unwrap();
//! println!("Distances: {:?}", paths.distances);
//! ```
//!
//! ## Modules
//!
//! - [`neg_cycle`] - Negative cycle detection algorithms
//! - [`parametric`] - Parametric optimization algorithms
//!
//! Bellman-Ford algorithms.

pub mod neg_cycle;
pub mod parametric;

pub mod prelude;
// pub mod min_cycle_ratio_ai;

use petgraph::prelude::*;

use petgraph::algo::{FloatMeasure, NegativeCycle};
use petgraph::visit::{
    IntoEdges, IntoNodeIdentifiers, NodeCount, NodeIndexable, VisitMap, Visitable,
};

#[derive(Debug, Clone)]
pub struct Paths<NodeId, EdgeWeight> {
    pub distances: Vec<EdgeWeight>,
    pub predecessors: Vec<Option<NodeId>>,
}

/// \[Generic\] Compute shortest paths from node `source` to all other.
///
/// Using the [Bellman–Ford algorithm][bf]; negative edge costs are
/// permitted, but the graph must not have a cycle of negative weights
/// (in that case it will return an error).
///
/// On success, return one vec with path costs, and another one which points
/// out the predecessor of a node along a shortest path. The vectors
/// are indexed by the graph's node indices.
///
/// # Complexity
///
/// - **Time**: O(V * E) where V is the number of vertices and E is the number of edges
/// - **Space**: O(V) for the distance and predecessor arrays
///
/// [bf]: https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::bellman_ford;
/// use petgraph::prelude::*;
///
/// let mut g = Graph::new();
/// let a = g.add_node(()); // node with no weight
/// let b = g.add_node(());
/// let c = g.add_node(());
/// let d = g.add_node(());
/// let edge = g.add_node(());
/// let f = g.add_node(());
/// g.extend_with_edges(&[
///     (0, 1, 2.0),
///     (0, 3, 4.0),
///     (1, 2, 1.0),
///     (1, 5, 7.0),
///     (2, 4, 5.0),
///     (4, 5, 1.0),
///     (3, 4, 1.0),
/// ]);
///
/// // Graph represented with the weight of each edge
/// //
/// //     2       1
/// // a ----- b ----- c
/// // | 4     | 7     |
/// // d       f       | 5
/// // | 1     | 1     |
/// // \------ edge ------/
///
/// let path = bellman_ford(&g, a);
/// assert!(path.is_ok());
/// let path = path.unwrap();
/// assert_eq!(path.distances, vec![    0.0,     2.0,    3.0,    4.0,     5.0,     6.0]);
/// assert_eq!(path.predecessors, vec![None, Some(a),Some(b),Some(a), Some(d), Some(edge)]);
///
/// // Node f (indice 5) can be reach from a with a path costing 6.
/// // Predecessor of f is Some(edge) which predecessor is Some(d) which predecessor is Some(a).
/// // Thus the path from a to f is a <-> d <-> edge <-> f
///
/// let graph_with_neg_cycle = Graph::<(), f32, Undirected>::from_edges(&[
///         (0, 1, -2.0),
///         (0, 3, -4.0),
///         (1, 2, -1.0),
///         (1, 5, -25.0),
///         (2, 4, -5.0),
///         (4, 5, -25.0),
///         (3, 4, -1.0),
/// ]);
///
/// assert!(bellman_ford(&graph_with_neg_cycle, NodeIndex::new(0)).is_err());
///
/// // Graph with no edges
/// let mut g_no_edges = Graph::<(), f32, Directed>::new();
/// let n0 = g_no_edges.add_node(());
/// let path_no_edges = bellman_ford(&g_no_edges, n0);
/// assert!(path_no_edges.is_ok());
/// assert_eq!(path_no_edges.unwrap().distances, vec![0.0]);
///
/// // Disconnected graph
/// let mut g_disconnected = Graph::new();
/// let dn0 = g_disconnected.add_node(());
/// let dn1 = g_disconnected.add_node(());
/// let dn2 = g_disconnected.add_node(());
/// g_disconnected.add_edge(dn0, dn1, 1.0);
/// let path_disconnected = bellman_ford(&g_disconnected, dn0);
/// assert!(path_disconnected.is_ok());
/// let unwrapped_path = path_disconnected.unwrap();
/// assert_eq!(unwrapped_path.distances, vec![0.0, 1.0, f32::INFINITY]);
/// assert_eq!(unwrapped_path.predecessors, vec![None, Some(dn0), None]);
/// ```
pub fn bellman_ford<G>(
    g: G,
    source: G::NodeId,
) -> Result<Paths<G::NodeId, G::EdgeWeight>, NegativeCycle>
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable,
    G::EdgeWeight: FloatMeasure,
{
    let ix = |i| g.to_index(i);

    // Step 1 and Step 2: initialize and relax
    let (distances, predecessors) = bellman_ford_initialize_relax(g, source);

    // Step 3: check for negative weight cycle
    for i in g.node_identifiers() {
        for edge in g.edges(i) {
            let j = edge.target();
            let w = *edge.weight();
            if distances[ix(i)] + w < distances[ix(j)] {
                return Err(NegativeCycle(()));
            }
        }
    }

    Ok(Paths {
        distances,
        predecessors,
    })
}

/// \[Generic\] Find the path of a negative cycle reachable from node `source`.
///
/// Using the [find_negative_cycle][nc]; will search the Graph for negative cycles using
/// [Bellman–Ford algorithm][bf]. If no negative cycle is found the function will return `None`.
///
/// If a negative cycle is found from source, return one vec with a path of `NodeId`s.
///
/// # Complexity
///
/// - **Time**: O(V * E) where V is number of vertices and E is number of edges
/// - **Space**: O(V) for distance and predecessor arrays
///
/// The time complexity of this algorithm should be the same as the Bellman-Ford.
///
/// [nc]: https://blogs.asarkar.com/assets/docs/algorithms-curated/Negative-Weight%20Cycle%20Algorithms%20-%20Huang.pdf
/// [bf]: https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::find_negative_cycle;
/// use petgraph::prelude::*;
///
/// let graph_with_neg_cycle = Graph::<(), f32, Directed>::from_edges(&[
///         (0, 1, 1.),
///         (0, 2, 1.),
///         (0, 3, 1.),
///         (1, 3, 1.),
///         (2, 1, 1.),
///         (3, 2, -3.),
/// ]);
///
/// let path = find_negative_cycle(&graph_with_neg_cycle, NodeIndex::new(0));
/// assert_eq!(
///     path,
///     Some([NodeIndex::new(1), NodeIndex::new(3), NodeIndex::new(2)].to_vec())
/// );
///
/// // Graph with no negative cycle
/// let graph_no_neg_cycle = Graph::<(), f32, Directed>::from_edges(&[
///         (0, 1, 1.),
///         (1, 2, 1.),
///         (2, 0, 1.),
/// ]);
/// let path_no_neg_cycle = find_negative_cycle(&graph_no_neg_cycle, NodeIndex::new(0));
/// assert!(path_no_neg_cycle.is_none());
/// ```
pub fn find_negative_cycle<G>(g: G, source: G::NodeId) -> Option<Vec<G::NodeId>>
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable + Visitable,
    G::EdgeWeight: FloatMeasure,
{
    let ix = |i| g.to_index(i);
    let mut path = Vec::<G::NodeId>::new();

    // Step 1: initialize and relax
    let (distance, predecessor) = bellman_ford_initialize_relax(g, source);

    // Step 2: Check for negative weight cycle
    'outer: for i in g.node_identifiers() {
        for edge in g.edges(i) {
            let j = edge.target();
            let w = *edge.weight();
            if distance[ix(i)] + w < distance[ix(j)] {
                // Step 3: negative cycle found
                let mut node = j;
                let mut path_set = g.visit_map();
                while path_set.visit(node) {
                    node = predecessor[ix(node)].unwrap();
                }

                let mut cycle_node = node;
                loop {
                    path.push(cycle_node);
                    cycle_node = predecessor[ix(cycle_node)].unwrap();
                    if cycle_node == node {
                        path.push(cycle_node);
                        break;
                    }
                }
                path.reverse();
                path.pop();
                // We are done here
                break 'outer;
            }
        }
    }
    if !path.is_empty() {
        // Users will probably need to follow the path of the negative cycle
        // so it should be in the reverse order than it was found by the algorithm.
        path.reverse();
        Some(path)
    } else {
        None
    }
}

/// Perform Step 1 and Step 2 of the Bellman-Ford algorithm.
///
/// This function initializes distances and predecessors, then performs
/// the relaxation step of the Bellman-Ford algorithm.
///
/// # Complexity
///
/// - **Time**: O(V * E) where V is number of vertices and E is number of edges
/// - **Space**: O(V) for distance and predecessor arrays
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use digraphx_rs::bellman_ford_initialize_relax;
/// use petgraph::prelude::*;
///
/// let mut g = Graph::new();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.extend_with_edges(&[(a, b, 1.0), (b, c, 1.0), (a, c, 3.0)]);
/// let (distances, predecessors) = bellman_ford_initialize_relax(&g, a);
/// assert_eq!(distances[0], 0.0); // distance to source is 0
/// assert_eq!(distances[1], 1.0); // distance to b from a
/// assert_eq!(predecessors[0], None); // source has no predecessor
/// assert_eq!(predecessors[1], Some(a)); // predecessor of b is a
/// ```
#[inline(always)]
pub fn bellman_ford_initialize_relax<G>(
    g: G,
    source: G::NodeId,
) -> (Vec<G::EdgeWeight>, Vec<Option<G::NodeId>>)
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable,
    G::EdgeWeight: FloatMeasure,
{
    // Step 1: initialize graph
    let mut predecessor = vec![None; g.node_bound()];
    let mut distance = vec![<_>::infinite(); g.node_bound()];
    let ix = |i| g.to_index(i);
    distance[ix(source)] = <_>::zero();

    // Step 2: relax edges repeatedly
    for _ in 1..g.node_count() {
        let mut did_update = false;
        for i in g.node_identifiers() {
            for edge in g.edges(i) {
                let j = edge.target();
                let w = *edge.weight();
                if distance[ix(i)] + w < distance[ix(j)] {
                    distance[ix(j)] = distance[ix(i)] + w;
                    predecessor[ix(j)] = Some(i);
                    did_update = true;
                }
            }
        }
        if !did_update {
            break;
        }
    }
    (distance, predecessor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Graph;

    #[test]
    fn test_bellman_ford_simple() {
        let mut g = Graph::new();
        let a = g.add_node(());
        let b = g.add_node(());
        let c = g.add_node(());
        g.extend_with_edges([(a, b, 1.0), (b, c, 1.0), (a, c, 3.0)]);
        let path = bellman_ford(&g, a).unwrap();
        assert_eq!(path.distances, vec![0.0, 1.0, 2.0]);
        assert_eq!(path.predecessors, vec![None, Some(a), Some(b)]);
    }

    #[test]
    fn test_bellman_ford_negative_cycle() {
        let mut g = Graph::new();
        let a = g.add_node(());
        let b = g.add_node(());
        g.extend_with_edges([(a, b, 1.0), (b, a, -2.0)]);
        let path = bellman_ford(&g, a);
        assert!(path.is_err());
    }

    #[test]
    fn test_find_negative_cycle_simple() {
        let mut g = Graph::new();
        let a = g.add_node(());
        let b = g.add_node(());
        g.extend_with_edges([(a, b, 1.0), (b, a, -2.0)]);
        let cycle = find_negative_cycle(&g, a).unwrap();
        assert_eq!(cycle, vec![a, b]);
    }

    #[test]
    fn test_find_negative_cycle_none() {
        let mut g = Graph::new();
        let a = g.add_node(());
        let b = g.add_node(());
        g.extend_with_edges([(a, b, 1.0), (b, a, 2.0)]);
        let cycle = find_negative_cycle(&g, a);
        assert!(cycle.is_none());
    }
}
