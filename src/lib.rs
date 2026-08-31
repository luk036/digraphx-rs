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
//! let has_cycle = ncf.howard(&mut dist, |w| *w).into_iter().next().is_some();
//! assert!(has_cycle);
//! ```

pub mod map_adapter;
pub mod mcf;
pub mod neg_cycle;
pub mod neg_cycle_q;
pub mod parametric;

#[cfg(feature = "std")]
pub mod logging;

pub mod prelude;

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Range;
use std::ops::Sub;
use std::pin::Pin;

use genawaiter::sync::Gen;

use crate::map_adapter::MapAdapter;

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

    /// Iterator over node identifiers (borrows from the graph).
    type Nodes<'a>: IntoIterator<Item = Self::Node>
    where
        Self: 'a;

    /// Iterator over (neighbor, weight) pairs for a given node (borrows from
    /// the graph).  Returning a borrowing iterator avoids the per-call `Vec`
    /// allocation that the previous `Vec::IntoIter`-based approach required.
    type Neighbors<'a>: IntoIterator<Item = (Self::Node, Self::Weight)>
    where
        Self: 'a;

    /// Return all nodes in the graph.
    fn nodes(&self) -> Self::Nodes<'_>;

    /// Return an iterator over the outgoing edges of `node`.
    fn neighbors(&self, node: Self::Node) -> Self::Neighbors<'_>;

    /// Return the number of nodes.
    fn num_nodes(&self) -> usize;
}

// ---------------------------------------------------------------------------
// Helper iterator types for HashMap / BTreeMap neighbours
// ---------------------------------------------------------------------------

/// Non-allocating neighbour iterator for `HashMap`-backed graphs.
///
/// Wraps the inner `Iter` and copies `(&N, &W)` → `(N, W)` per call without
/// collecting into an intermediate `Vec`.
#[derive(Debug, Clone)]
pub struct HashMapNeighborsIter<'a, N: 'a, W: 'a> {
    inner: std::collections::hash_map::Iter<'a, N, W>,
}

impl<'a, N: Copy, W: Copy> Iterator for HashMapNeighborsIter<'a, N, W> {
    type Item = (N, W);
    #[inline(always)]
    fn next(&mut self) -> Option<(N, W)> {
        self.inner.next().map(|(&k, &v)| (k, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Sum type that handles both "node found" and "node not found" without
/// allocating a temporary [`HashMap`] for the empty case.
#[derive(Debug, Clone)]
pub enum HashMapNeighbors<'a, N: 'a, W: 'a> {
    /// The node exists and we iterate its neighbours.
    Found(HashMapNeighborsIter<'a, N, W>),
    /// The node does not exist — empty iteration.
    NotFound,
}

impl<'a, N: Copy, W: Copy> Iterator for HashMapNeighbors<'a, N, W> {
    type Item = (N, W);
    #[inline(always)]
    fn next(&mut self) -> Option<(N, W)> {
        match self {
            HashMapNeighbors::Found(iter) => iter.next(),
            HashMapNeighbors::NotFound => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            HashMapNeighbors::Found(iter) => iter.size_hint(),
            HashMapNeighbors::NotFound => (0, Some(0)),
        }
    }
}

/// Non-allocating neighbour iterator for `BTreeMap`-backed graphs.
#[derive(Debug, Clone)]
pub struct BTreeMapNeighborsIter<'a, N: 'a, W: 'a> {
    inner: std::collections::btree_map::Iter<'a, N, W>,
}

impl<'a, N: Copy, W: Copy> Iterator for BTreeMapNeighborsIter<'a, N, W> {
    type Item = (N, W);
    #[inline(always)]
    fn next(&mut self) -> Option<(N, W)> {
        self.inner.next().map(|(&k, &v)| (k, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Sum type for "found" / "not found" without allocating a temporary.
#[derive(Debug, Clone)]
pub enum BTreeMapNeighbors<'a, N: 'a, W: 'a> {
    Found(BTreeMapNeighborsIter<'a, N, W>),
    NotFound,
}

impl<'a, N: Copy, W: Copy> Iterator for BTreeMapNeighbors<'a, N, W> {
    type Item = (N, W);
    #[inline(always)]
    fn next(&mut self) -> Option<(N, W)> {
        match self {
            BTreeMapNeighbors::Found(iter) => iter.next(),
            BTreeMapNeighbors::NotFound => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BTreeMapNeighbors::Found(iter) => iter.size_hint(),
            BTreeMapNeighbors::NotFound => (0, Some(0)),
        }
    }
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

    type Nodes<'a>
        = std::iter::Copied<std::collections::hash_map::Keys<'a, N, HashMap<N, W>>>
    where
        N: 'a,
        W: 'a;

    type Neighbors<'a>
        = HashMapNeighbors<'a, N, W>
    where
        N: 'a,
        W: 'a;

    #[inline]
    fn nodes(&self) -> Self::Nodes<'_> {
        self.keys().copied()
    }

    fn neighbors(&self, node: N) -> Self::Neighbors<'_> {
        match self.get(&node) {
            Some(nbrs) => HashMapNeighbors::Found(HashMapNeighborsIter { inner: nbrs.iter() }),
            None => HashMapNeighbors::NotFound,
        }
    }

    #[inline]
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

    type Nodes<'a>
        = std::iter::Copied<
        std::collections::btree_map::Keys<'a, N, std::collections::BTreeMap<N, W>>,
    >
    where
        N: 'a,
        W: 'a;

    type Neighbors<'a>
        = BTreeMapNeighbors<'a, N, W>
    where
        N: 'a,
        W: 'a;

    #[inline]
    fn nodes(&self) -> Self::Nodes<'_> {
        self.keys().copied()
    }

    fn neighbors(&self, node: N) -> Self::Neighbors<'_> {
        match self.get(&node) {
            Some(nbrs) => BTreeMapNeighbors::Found(BTreeMapNeighborsIter { inner: nbrs.iter() }),
            None => BTreeMapNeighbors::NotFound,
        }
    }

    #[inline]
    fn num_nodes(&self) -> usize {
        self.len()
    }
}

// ---------------------------------------------------------------------------
// Array-based graph representations (nodes are `usize` indices)
// ---------------------------------------------------------------------------

/// Create an array-based graph from a slice of edges (node, neighbor, weight)
/// triples.  Returns a `Vec<Vec<(usize, W)>>` adjacency list.
///
/// This is the array-based equivalent of [`graph_from_edges`], matching the
/// `vector<vector<pair<size_t, W>>>` convention used in the C++ digraphx-cpp.
///
/// ```rust
/// use digraphx_rs::{Graph, graph_from_edges_array};
///
/// let g = graph_from_edges_array(&[(0, 1, 1), (1, 2, 2), (2, 0, -3)]);
/// assert_eq!(g.num_nodes(), 3);
/// ```
pub fn graph_from_edges_array<W>(edges: &[(usize, usize, W)]) -> Vec<Vec<(usize, W)>>
where
    W: Copy,
{
    if edges.is_empty() {
        return Vec::new();
    }
    let max_node = edges.iter().map(|&(u, v, _)| u.max(v)).max().unwrap_or(0);
    // Pre-compute out-degree so each inner Vec is allocated with exact capacity.
    let mut out_deg = vec![0usize; max_node + 1];
    for &(u, _, _) in edges {
        out_deg[u] += 1;
    }
    let mut g: Vec<Vec<(usize, W)>> = out_deg.iter().map(|&d| Vec::with_capacity(d)).collect();
    for &(u, v, w) in edges {
        g[u].push((v, w));
    }
    g
}

// --- Vec<Vec<(usize, W)>> --------------------------------------------------
//
// Standard adjacency list: the outer index is the source node, and each inner
// vector stores (neighbor, weight) pairs.  Matches the C++ convention of
// `vector<vector<pair<size_t, W>>>` wrapped by `MapAdapter`.

impl<W> Graph for Vec<Vec<(usize, W)>>
where
    W: Copy + Add<Output = W> + PartialOrd,
{
    type Node = usize;
    type Weight = W;

    type Nodes<'a>
        = Range<usize>
    where
        W: 'a;

    type Neighbors<'a>
        = std::iter::Copied<std::slice::Iter<'a, (usize, W)>>
    where
        W: 'a;

    #[inline]
    fn nodes(&self) -> Self::Nodes<'_> {
        0..self.len()
    }

    fn neighbors(&self, node: usize) -> Self::Neighbors<'_> {
        self.get(node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    #[inline]
    fn num_nodes(&self) -> usize {
        self.len()
    }
}

// --- MapAdapter<Vec<(usize, W)>> -------------------------------------------
//
// Same adjacency-list layout but with the outer Vec wrapped in a MapAdapter.
// This mirrors the C++ pattern where `MapAdapter<vector<vector<pair<K,V>>>>`
// provides the graph interface.  Here `MapAdapter<Vec<(usize, W)>>` stores
// the adjacency list in its inner Vec (`lst: Vec<Vec<(usize, W)>>`).

impl<W> Graph for MapAdapter<Vec<(usize, W)>>
where
    W: Copy + Add<Output = W> + PartialOrd,
{
    type Node = usize;
    type Weight = W;

    type Nodes<'a>
        = Range<usize>
    where
        W: 'a;

    type Neighbors<'a>
        = std::iter::Copied<std::slice::Iter<'a, (usize, W)>>
    where
        W: 'a;

    #[inline]
    fn nodes(&self) -> Self::Nodes<'_> {
        0..self.len()
    }

    fn neighbors(&self, node: usize) -> Self::Neighbors<'_> {
        self.lst
            .get(node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    #[inline]
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

        type Nodes<'b>
            = petgraph::graph::NodeIndices
        where
            PetGraph<'a, V, E>: 'b;

        type Neighbors<'b>
            = PetNeighbors<'b, E>
        where
            PetGraph<'a, V, E>: 'b;

        #[inline]
        fn nodes(&self) -> Self::Nodes<'_> {
            self.0.node_indices()
        }

        fn neighbors(&self, node: NodeIndex) -> Self::Neighbors<'_> {
            PetNeighbors {
                iter: self.0.edges(node),
            }
        }

        #[inline]
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
        #[inline]
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

/// Shared cycle-finding logic used by both [`NegCycleFinder`] and
/// [`NegCycleFinderQ`].  Returns **all** cycle start nodes found in the
/// predecessor / successor graph so callers can inspect every candidate.
pub(crate) fn find_cycles_in<G: Graph>(
    graph: &G,
    point_to: &HashMap<G::Node, (G::Node, G::Weight)>,
) -> Vec<G::Node>
where
    G::Node: Copy + Eq + Hash,
    G::Weight: Copy,
{
    let mut result = Vec::new();
    let mut visited: HashMap<G::Node, G::Node> = HashMap::new();
    for vtx in graph.nodes() {
        if visited.contains_key(&vtx) {
            continue;
        }
        let mut utx = vtx;
        visited.insert(utx, vtx);
        loop {
            match point_to.get(&utx) {
                None => break,
                Some(&(prev, _)) => {
                    utx = prev;
                    if let Some(&root) = visited.get(&utx) {
                        if root == vtx {
                            result.push(utx);
                        }
                        break;
                    }
                    visited.insert(utx, vtx);
                }
            }
        }
    }
    result
}

/// Trait for additive identity.
pub trait Zero: Sized {
    fn zero() -> Self;
}

// ---------------------------------------------------------------------------
// Shared relaxation / cycle-reconstruction cores
// ---------------------------------------------------------------------------
//
// These are the building blocks shared by `NegCycleFinder` (neg_cycle.rs) and
// `NegCycleFinderQ` (neg_cycle_q.rs).  The Howard iteration skeleton is a
// Template Method: the relaxation pass and the negativity check are injected
// as strategy closures, while the relax-to-fixpoint + find-cycle + yield loop
// lives here once.

/// Predecessor relaxation (Bellman–Ford style) with an `update_ok` gate.
///
/// Generic over `U` so the gate is monomorphized (static dispatch) rather than
/// erased to a trait object — the gate is invoked once per edge in the hot loop.
pub(crate) fn relax_pred_core<G, F, U>(
    graph: &G,
    dist: &mut HashMap<G::Node, G::Weight>,
    get_weight: &F,
    update_ok: &U,
    pred: &mut HashMap<G::Node, (G::Node, G::Weight)>,
) -> bool
where
    G: Graph,
    G::Weight: Add<Output = G::Weight> + PartialOrd + Copy + Zero,
    G::Node: Copy + Eq + Hash,
    F: Fn(&G::Weight) -> G::Weight,
    U: Fn(&G::Weight, &G::Weight) -> bool,
{
    let mut changed = false;
    for utx in graph.nodes() {
        let du = *dist.get(&utx).unwrap_or(&G::Weight::zero());
        for (vtx, w) in graph.neighbors(utx) {
            let distance = du + get_weight(&w);
            let dv = *dist.get(&vtx).unwrap_or(&G::Weight::zero());
            if dv > distance && update_ok(&dv, &distance) {
                dist.insert(vtx, distance);
                pred.insert(vtx, (utx, w));
                changed = true;
            }
        }
    }
    changed
}

/// Successor relaxation (reverse Bellman–Ford style) with an `update_ok` gate.
///
/// Generic over `U` so the gate is monomorphized (static dispatch).
pub(crate) fn relax_succ_core<G, F, U>(
    graph: &G,
    dist: &mut HashMap<G::Node, G::Weight>,
    get_weight: &F,
    update_ok: &U,
    succ: &mut HashMap<G::Node, (G::Node, G::Weight)>,
) -> bool
where
    G: Graph,
    G::Weight: Add<Output = G::Weight> + Sub<Output = G::Weight> + PartialOrd + Copy + Zero,
    G::Node: Copy + Eq + Hash,
    F: Fn(&G::Weight) -> G::Weight,
    U: Fn(&G::Weight, &G::Weight) -> bool,
{
    let mut changed = false;
    for utx in graph.nodes() {
        let du = *dist.get(&utx).unwrap_or(&G::Weight::zero());
        for (vtx, w) in graph.neighbors(utx) {
            let distance = *dist.get(&vtx).unwrap_or(&G::Weight::zero()) - get_weight(&w);
            if du < distance && update_ok(&du, &distance) {
                dist.insert(utx, distance);
                succ.insert(utx, (vtx, w));
                changed = true;
            }
        }
    }
    changed
}

/// Reconstruct a cycle from the given point-to map (as edge weights).
pub(crate) fn cycle_list_from<Node, Weight>(
    point_to: &HashMap<Node, (Node, Weight)>,
    handle: Node,
) -> Vec<Weight>
where
    Node: Copy + Eq + Hash,
    Weight: Copy,
{
    let mut vtx = handle;
    let mut cycle = Vec::new();
    loop {
        let &(utx, w) = point_to.get(&vtx).unwrap();
        cycle.push(w);
        vtx = utx;
        if vtx == handle {
            break;
        }
    }
    cycle
}

/// Check whether the cycle starting at `handle` in `point_to` is negative.
pub(crate) fn is_negative_cycle<Node, Weight, F>(
    point_to: &HashMap<Node, (Node, Weight)>,
    handle: Node,
    dist: &HashMap<Node, Weight>,
    get_weight: &F,
) -> bool
where
    Node: Copy + Eq + Hash,
    Weight: Add<Output = Weight> + PartialOrd + Copy + Zero,
    F: Fn(&Weight) -> Weight,
{
    let mut vtx = handle;
    loop {
        let &(utx, w) = point_to.get(&vtx).unwrap();
        let dv = *dist.get(&vtx).unwrap_or(&Weight::zero());
        let du = *dist.get(&utx).unwrap_or(&Weight::zero());
        if dv > du + get_weight(&w) {
            return true;
        }
        vtx = utx;
        if vtx == handle {
            break;
        }
    }
    false
}

/// Template Method: Howard's policy-iteration skeleton.
///
/// Repeatedly relaxes to a fixpoint, finds cycles in the point-to map, and
/// yields each cycle as a list of edge weights.  The relaxation pass (`relax`)
/// and the optional negativity check (`check`) are injected as strategies;
/// both receive the point-to map as a parameter so callers do not capture it.
/// The `update_ok` gate is baked into the `relax` closure (not passed as a
/// trait object) so the per-edge call stays statically dispatched.
#[allow(clippy::type_complexity)]
pub(crate) fn howard_search<'b, G, F, R, C>(
    graph: &'b G,
    dist: &'b mut HashMap<G::Node, G::Weight>,
    get_weight: F,
    point_to: &'b mut HashMap<G::Node, (G::Node, G::Weight)>,
    relax: R,
    check: C,
) -> Gen<Vec<G::Weight>, (), Pin<Box<dyn std::future::Future<Output = ()> + 'b>>>
where
    G: Graph,
    G::Weight: Add<Output = G::Weight> + PartialOrd + Copy + Zero,
    G::Node: Copy + Eq + Hash,
    F: Fn(&G::Weight) -> G::Weight + 'b,
    R: Fn(
            &mut HashMap<G::Node, G::Weight>,
            &F,
            &mut HashMap<G::Node, (G::Node, G::Weight)>,
        ) -> bool
        + 'b,
    C: Fn(G::Node, &HashMap<G::Node, G::Weight>, &F, &HashMap<G::Node, (G::Node, G::Weight)>) + 'b,
{
    Gen::new(
        |co| -> Pin<Box<dyn std::future::Future<Output = ()> + 'b>> {
            Box::pin(async move {
                point_to.clear();
                let mut found = false;
                while !found && relax(&mut *dist, &get_weight, &mut *point_to) {
                    let cycles = crate::find_cycles_in(graph, point_to);
                    for vtx in cycles {
                        check(vtx, dist, &get_weight, point_to);
                        found = true;
                        co.yield_(cycle_list_from(point_to, vtx)).await;
                    }
                }
            })
        },
    )
}

impl Zero for i32 {
    #[inline]
    fn zero() -> Self {
        0
    }
}
impl Zero for f32 {
    #[inline]
    fn zero() -> Self {
        0.0
    }
}
impl Zero for f64 {
    #[inline]
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
pub use neg_cycle_q::NegCycleFinderQ;
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

    // --- Array-based graph tests ---

    #[test]
    fn test_graph_vec_vec_nodes() {
        let g: Vec<Vec<(usize, i32)>> = vec![vec![(1, 1)], vec![(0, 2)]];
        let nodes: Vec<_> = g.nodes().collect();
        assert_eq!(nodes, vec![0, 1]);
    }

    #[test]
    fn test_graph_vec_vec_neighbors() {
        let g: Vec<Vec<(usize, i32)>> = vec![vec![(1, 5)]];
        let nbrs: Vec<_> = g.neighbors(0).collect();
        assert_eq!(nbrs, vec![(1, 5)]);
    }

    #[test]
    fn test_graph_vec_vec_empty() {
        let g: Vec<Vec<(usize, f64)>> = vec![];
        assert_eq!(g.num_nodes(), 0);
        assert!(g.nodes().collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn test_graph_vec_vec_neg_cycle() {
        let g = graph_from_edges_array(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncf = NegCycleFinder::new(&g);
        let mut dist: HashMap<usize, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(ncf.howard(&mut dist, |w| *w).into_iter().next().is_some());
    }

    #[test]
    fn test_graph_vec_vec_no_neg_cycle() {
        let g = graph_from_edges_array(&[(0, 1, 1i32), (1, 0, 1)]);
        let mut ncf = NegCycleFinder::new(&g);
        let mut dist: HashMap<usize, i32> = [(0, 0), (1, 0)].into();
        assert!(ncf.howard(&mut dist, |w| *w).into_iter().next().is_none());
    }

    #[test]
    fn test_graph_map_adapter_vec() {
        let adj: Vec<Vec<(usize, i32)>> = vec![vec![(1, 1), (2, 2)], vec![(2, 3)], vec![(0, -4)]];
        let g = MapAdapter::new(adj);
        assert_eq!(g.num_nodes(), 3);
        let nbrs: Vec<_> = g.neighbors(0).collect();
        assert_eq!(nbrs, vec![(1, 1), (2, 2)]);
    }

    #[test]
    fn test_graph_map_adapter_neg_cycle() {
        let adj: Vec<Vec<(usize, i32)>> = vec![vec![(1, 1)], vec![(2, 1)], vec![(0, -3)]];
        let g = MapAdapter::new(adj);
        let mut ncf = NegCycleFinder::new(&g);
        let mut dist: HashMap<usize, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(ncf.howard(&mut dist, |w| *w).into_iter().next().is_some());
    }

    #[test]
    fn test_graph_from_edges_array_basic() {
        let g = graph_from_edges_array(&[(0, 1, 1), (1, 2, 2), (2, 0, -3)]);
        assert_eq!(g.num_nodes(), 3);
        let nbrs: Vec<_> = g.neighbors(0).collect();
        assert_eq!(nbrs, vec![(1, 1)]);
    }

    #[test]
    fn test_graph_from_edges_array_empty() {
        let g: Vec<Vec<(usize, i32)>> = graph_from_edges_array(&[]);
        assert_eq!(g.num_nodes(), 0);
    }
}
