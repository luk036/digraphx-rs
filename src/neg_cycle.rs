use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::pin::Pin;

use genawaiter::sync::Gen;

use crate::Graph;
use crate::Zero;

/// Negative cycle finder using Howard's policy iteration method.
///
/// Generic over any graph type `G` that implements the [`Graph`] trait.
/// The graph is treated as a container of containers: the outer container
/// maps each node to its neighbors, the inner maps each neighbor to
/// an edge weight.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use digraphx_rs::{Graph, NegCycleFinder};
///
/// let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
/// graph.insert("a", [("b", 7), ("c", 5)].into());
/// graph.insert("b", [("a", 0), ("c", 3)].into());
/// graph.insert("c", [("a", 2), ("b", 1)].into());
///
/// let mut ncf = NegCycleFinder::new(&graph);
/// let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
/// let cycles: Vec<_> = ncf.howard(&mut dist, |w| *w).into_iter().collect();
/// assert!(cycles.is_empty()); // no negative cycle
/// ```
pub struct NegCycleFinder<'a, G: Graph> {
    graph: &'a G,
    pred: HashMap<G::Node, (G::Node, G::Weight)>,
}

impl<'a, G: Graph> NegCycleFinder<'a, G>
where
    G::Weight: Add<Output = G::Weight> + PartialOrd + Copy + Zero,
    G::Node: Copy + Eq + Hash,
{
    /// Return a reference to the underlying graph.
    #[inline]
    pub fn graph(&self) -> &'a G {
        self.graph
    }

    /// Create a new finder for the given graph.
    pub fn new(graph: &'a G) -> Self {
        NegCycleFinder {
            graph,
            pred: HashMap::new(),
        }
    }

    /// Perform one Bellman–Ford relaxation pass.
    ///
    /// For each edge $(u, v)$ in the graph, checks the triangle inequality:
    ///
    /// $$ d\[v\] > d\[u\] + w(u,v) $$
    ///
    /// and updates the predecessor map if so. Returns `true` if any distance was changed.
    #[inline]
    pub fn relax<F>(&mut self, dist: &mut HashMap<G::Node, G::Weight>, get_weight: &F) -> bool
    where
        F: Fn(&G::Weight) -> G::Weight,
    {
        crate::relax_pred_core(self.graph, dist, get_weight, &|_, _| true, &mut self.pred)
    }

    /// Howard's algorithm: find negative cycles using policy iteration.
    ///
    /// $$ \text{Repeat relax until fixpoint; yield every cycle found in the predecessor graph} $$
    ///
    /// A cycle is negative iff:
    ///
    /// $$ \sum_{C} w_{ij} < 0 $$
    ///
    /// Yields each cycle as a list of edge weights, lazily via a [`Gen`]
    /// (synchronous generator).  Loop over it directly:
    ///
    /// ```rust,ignore
    /// for cycle in ncf.howard(&mut dist, |w| *w) {
    ///     // each `cycle` is a Vec<Weight> forming a negative cycle
    /// }
    /// ```
    ///
    /// # Type parameters
    ///
    /// * `F` — weight-extraction closure (typically `\|w\| *w` when the
    ///   weight is the edge data itself, or a projection for structured
    ///   edge types).
    #[allow(clippy::type_complexity)]
    pub fn howard<'b, F>(
        &'b mut self,
        dist: &'b mut HashMap<G::Node, G::Weight>,
        get_weight: F,
    ) -> Gen<Vec<G::Weight>, (), Pin<Box<dyn std::future::Future<Output = ()> + 'b>>>
    where
        F: Fn(&G::Weight) -> G::Weight + 'b,
    {
        let graph = self.graph; // Copy: capture the graph ref, not `self`
                                // Gate baked into the closure: unconstrained → always allow updates.
        let relax = |d: &mut HashMap<G::Node, G::Weight>,
                     w: &F,
                     p: &mut HashMap<G::Node, (G::Node, G::Weight)>| {
            crate::relax_pred_core(graph, d, w, &|_, _| true, p)
        };
        let check = |_: G::Node,
                     _: &HashMap<G::Node, G::Weight>,
                     _: &F,
                     _: &HashMap<G::Node, (G::Node, G::Weight)>| {};
        crate::howard_search(self.graph, dist, get_weight, &mut self.pred, relax, check)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_from_edges;
    use std::collections::HashMap;

    #[inline]
    fn has_neg_cycle<G, F>(
        ncf: &mut NegCycleFinder<'_, G>,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
    ) -> bool
    where
        G: Graph,
        G::Weight: Add<Output = G::Weight> + PartialOrd + Copy + Zero,
        G::Node: Copy + Eq + Hash,
        F: Fn(&G::Weight) -> G::Weight,
    {
        ncf.howard(dist, get_weight).into_iter().next().is_some()
    }

    #[test]
    fn test_no_negative_cycle() {
        let graph = graph_from_edges(&[
            (0, 1, 7i32),
            (0, 2, 5),
            (1, 0, 0),
            (1, 2, 3),
            (2, 1, 1),
            (2, 0, 2),
        ]);
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(!has_neg_cycle(&mut ncf, &mut dist, |w| *w));
    }

    #[test]
    fn test_negative_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(has_neg_cycle(&mut ncf, &mut dist, |w| *w));
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<i32, HashMap<i32, i32>> = HashMap::new();
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = HashMap::new();
        assert!(!has_neg_cycle(&mut ncf, &mut dist, |w| *w));
    }

    #[test]
    fn test_string_nodes() {
        let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
        graph.insert("a", [("b", 1)].into());
        graph.insert("b", [("c", 1)].into());
        graph.insert("c", [("a", -3)].into());

        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
        assert!(has_neg_cycle(&mut ncf, &mut dist, |w| *w));
    }

    #[test]
    fn test_single_node() {
        let graph: HashMap<i32, HashMap<i32, i32>> = [(0, HashMap::new())].into();
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0)].into();
        assert!(!has_neg_cycle(&mut ncf, &mut dist, |w| *w));
    }

    #[test]
    fn test_ratio_weights() {
        use num::rational::Ratio;
        let graph = graph_from_edges(&[
            (0, 1, Ratio::new(1, 1)),
            (1, 2, Ratio::new(1, 1)),
            (2, 0, Ratio::new(-3, 1)),
        ]);
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, Ratio<i32>> = [
            (0, Ratio::new(0, 1)),
            (1, Ratio::new(0, 1)),
            (2, Ratio::new(0, 1)),
        ]
        .into();
        assert!(has_neg_cycle(&mut ncf, &mut dist, |w| *w));
    }

    #[test]
    fn test_multiple_cycles_yielded() {
        // A graph with two disjoint negative cycles
        let graph = graph_from_edges(&[
            (0, 1, -1i32),
            (1, 0, -1), // cycle 0-1-0: sum = -2
            (2, 3, -2i32),
            (3, 2, -2), // cycle 2-3-2: sum = -4
        ]);
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0), (3, 0)].into();
        let cycles: Vec<_> = ncf.howard(&mut dist, |w| *w).into_iter().collect();
        assert!(!cycles.is_empty());
    }
}
