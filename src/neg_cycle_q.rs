//! Negative cycle detection with constraints using Howard's method.
//!
//! This module extends the basic negative cycle detection to support constrained
//! optimization problems. It implements both predecessor and successor versions
//! of Howard's algorithm, allowing for more flexible cycle detection strategies.
//!
//! Key features:
//! - Support for distance update constraints via [`update_ok`] callbacks
//! - Both predecessor-based and successor-based algorithms
//! - Flexible constraint handling for complex optimization problems
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashMap;
//! use digraphx_rs::{graph_from_edges, NegCycleFinderQ};
//!
//! let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
//! let mut ncfq = NegCycleFinderQ::new(&graph);
//! let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
//! let cycles: Vec<_> = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true).into_iter().collect();
//! assert!(!cycles.is_empty());
//! ```
//!
//! [`update_ok`]: NegCycleFinderQ::howard_pred

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Sub;
use std::pin::Pin;

use genawaiter::sync::Gen;

use crate::Graph;
use crate::Zero;

/// Negative cycle finder with constraints (predecessor / successor).
///
/// Extends the basic [`NegCycleFinder`](crate::NegCycleFinder) with:
/// - **Predecessor-based** Howard (`howard_pred`) — traditional Bellman–Ford
/// - **Successor-based** Howard (`howard_succ`) — reverse relaxation
/// - An **`update_ok`** callback that gates distance updates
///
/// Generic over any graph type `G` that implements [`Graph`].
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use digraphx_rs::{graph_from_edges, NegCycleFinderQ};
///
/// let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
/// let mut ncfq = NegCycleFinderQ::new(&graph);
/// let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
///
/// // allow-all constraint
/// let cycles: Vec<_> = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true).into_iter().collect();
/// assert!(!cycles.is_empty());
/// ```
pub struct NegCycleFinderQ<'a, G: Graph> {
    graph: &'a G,
    pred: HashMap<G::Node, (G::Node, G::Weight)>,
    succ: HashMap<G::Node, (G::Node, G::Weight)>,
}

impl<'a, G: Graph> NegCycleFinderQ<'a, G>
where
    G::Weight: Add<Output = G::Weight> + Sub<Output = G::Weight> + PartialOrd + Copy + Zero,
    G::Node: Copy + Eq + Hash,
{
    /// Create a new constrained finder for the given graph.
    pub fn new(graph: &'a G) -> Self {
        NegCycleFinderQ {
            graph,
            pred: HashMap::new(),
            succ: HashMap::new(),
        }
    }

    // ------------------------------------------------------------------
    // Predecessor relaxation (edge-data weight)
    // ------------------------------------------------------------------

    /// Predecessor relaxation (Bellman–Ford style) with constraint.
    ///
    /// For each edge $(u, v)$, updates $d\[v\]$ when:
    ///
    /// $$ d\[v\] > d\[u\] + w(u,v) $$
    ///
    /// AND $\text{update\_ok}(d_{\text{old}}, d_{\text{new}})$ is `true`.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    #[inline]
    pub fn relax_pred<F, U>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: &F,
        update_ok: &U,
    ) -> bool
    where
        F: Fn(&G::Weight) -> G::Weight,
        U: Fn(&G::Weight, &G::Weight) -> bool,
    {
        crate::relax_pred_core(self.graph, dist, get_weight, update_ok, &mut self.pred)
    }

    // ------------------------------------------------------------------
    // Successor relaxation (edge-data weight)
    // ------------------------------------------------------------------

    /// Successor relaxation (reverse Bellman–Ford style) with constraint.
    ///
    /// For each edge $(u, v)$, updates $d\[u\]$ when:
    ///
    /// $$ d\[u\] < d\[v\] - w(u,v) $$
    ///
    /// AND $\text{update\_ok}(d_{\text{old}}, d_{\text{new}})$ is `true`.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    #[inline]
    pub fn relax_succ<F, U>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: &F,
        update_ok: &U,
    ) -> bool
    where
        F: Fn(&G::Weight) -> G::Weight,
        U: Fn(&G::Weight, &G::Weight) -> bool,
    {
        crate::relax_succ_core(self.graph, dist, get_weight, update_ok, &mut self.succ)
    }

    // ------------------------------------------------------------------
    // Negativity checks
    // ------------------------------------------------------------------

    /// Check whether the cycle starting at `handle` is negative.
    ///
    /// A cycle is negative if for any edge $(u,v)$ on the cycle:
    ///
    /// $$ d\[v\] > d\[u\] + w(u,v) $$
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    #[inline]
    pub fn is_negative<F>(
        &self,
        handle: G::Node,
        dist: &HashMap<G::Node, G::Weight>,
        get_weight: &F,
    ) -> bool
    where
        F: Fn(&G::Weight) -> G::Weight,
    {
        crate::is_negative_cycle(&self.pred, handle, dist, get_weight)
    }

    // ------------------------------------------------------------------
    // Howard's algorithm: predecessor-based (yields edge weights)
    // ------------------------------------------------------------------

    /// Predecessor-based Howard's algorithm with constraint.
    ///
    /// A cycle is negative iff:
    ///
    /// $$ \sum_{C} w_{ij} < 0 $$
    ///
    /// Yields cycles as lists of edge weights.  The returned iterator can be
    /// looped over directly.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    #[allow(clippy::type_complexity)]
    pub fn howard_pred<'b, F, U>(
        &'b mut self,
        dist: &'b mut HashMap<G::Node, G::Weight>,
        get_weight: F,
        update_ok: U,
    ) -> Gen<Vec<G::Weight>, (), Pin<Box<dyn std::future::Future<Output = ()> + 'b>>>
    where
        F: Fn(&G::Weight) -> G::Weight + 'b,
        U: Fn(&G::Weight, &G::Weight) -> bool + 'b,
    {
        let graph = self.graph; // Copy: capture the graph ref, not `self`
                                // Gate baked into the closure: predecessor relaxation with the user's gate.
        let relax = move |d: &mut HashMap<G::Node, G::Weight>,
                          w: &F,
                          p: &mut HashMap<G::Node, (G::Node, G::Weight)>| {
            crate::relax_pred_core(graph, d, w, &update_ok, p)
        };
        let check = |vtx: G::Node,
                     d: &HashMap<G::Node, G::Weight>,
                     w: &F,
                     p: &HashMap<G::Node, (G::Node, G::Weight)>| {
            debug_assert!(crate::is_negative_cycle(p, vtx, d, w));
        };
        crate::howard_search(self.graph, dist, get_weight, &mut self.pred, relax, check)
    }

    // ------------------------------------------------------------------
    // Howard's algorithm: successor-based (yields edge weights)
    // ------------------------------------------------------------------

    /// Successor-based Howard's algorithm with constraint.
    ///
    /// A cycle is negative iff:
    ///
    /// $$ \sum_{C} w_{ij} < 0 $$
    ///
    /// Yields cycles as lists of edge weights.  The returned iterator can be
    /// looped over directly.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    #[allow(clippy::type_complexity)]
    pub fn howard_succ<'b, F, U>(
        &'b mut self,
        dist: &'b mut HashMap<G::Node, G::Weight>,
        get_weight: F,
        update_ok: U,
    ) -> Gen<Vec<G::Weight>, (), Pin<Box<dyn std::future::Future<Output = ()> + 'b>>>
    where
        F: Fn(&G::Weight) -> G::Weight + 'b,
        U: Fn(&G::Weight, &G::Weight) -> bool + 'b,
    {
        let graph = self.graph; // Copy: capture the graph ref, not `self`
                                // Gate baked into the closure: successor relaxation with the user's gate.
        let relax = move |d: &mut HashMap<G::Node, G::Weight>,
                          w: &F,
                          p: &mut HashMap<G::Node, (G::Node, G::Weight)>| {
            crate::relax_succ_core(graph, d, w, &update_ok, p)
        };
        let check = |_: G::Node,
                     _: &HashMap<G::Node, G::Weight>,
                     _: &F,
                     _: &HashMap<G::Node, (G::Node, G::Weight)>| {};
        crate::howard_search(self.graph, dist, get_weight, &mut self.succ, relax, check)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_from_edges;
    use std::collections::HashMap;

    fn has_neg_cycle_pred<G, F>(
        ncfq: &mut NegCycleFinderQ<'_, G>,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
    ) -> bool
    where
        G: Graph,
        G::Weight: Add<Output = G::Weight> + Sub<Output = G::Weight> + PartialOrd + Copy + Zero,
        G::Node: Copy + Eq + Hash,
        F: Fn(&G::Weight) -> G::Weight,
    {
        ncfq.howard_pred(dist, get_weight, |_, _| true)
            .into_iter()
            .next()
            .is_some()
    }

    fn has_neg_cycle_succ<G, F>(
        ncfq: &mut NegCycleFinderQ<'_, G>,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
    ) -> bool
    where
        G: Graph,
        G::Weight: Add<Output = G::Weight> + Sub<Output = G::Weight> + PartialOrd + Copy + Zero,
        G::Node: Copy + Eq + Hash,
        F: Fn(&G::Weight) -> G::Weight,
    {
        ncfq.howard_succ(dist, get_weight, |_, _| true)
            .into_iter()
            .next()
            .is_some()
    }

    // --- howard_pred tests ---

    #[test]
    fn test_q_pred_no_neg_cycle() {
        let graph = graph_from_edges(&[
            (0, 1, 7i32),
            (0, 2, 5),
            (1, 0, 0),
            (1, 2, 3),
            (2, 1, 1),
            (2, 0, 2),
        ]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(!has_neg_cycle_pred(&mut ncfq, &mut dist, |w| *w));
    }

    #[test]
    fn test_q_pred_neg_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(has_neg_cycle_pred(&mut ncfq, &mut dist, |w| *w));
    }

    // --- howard_succ tests ---

    #[test]
    fn test_q_succ_neg_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        assert!(has_neg_cycle_succ(&mut ncfq, &mut dist, |w| *w));
    }

    // --- constraint tests ---

    #[test]
    fn test_q_pred_with_constraint_blocks_all() {
        // Graph has negative cycle, but update_ok blocks every update
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let cycles: Vec<_> = ncfq
            .howard_pred(&mut dist, |w| *w, |_, _| false)
            .into_iter()
            .collect();
        assert!(cycles.is_empty());
    }

    // --- string node tests ---

    #[test]
    fn test_q_pred_string_nodes() {
        let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
        graph.insert("a", [("b", 1)].into());
        graph.insert("b", [("c", 1)].into());
        graph.insert("c", [("a", -3)].into());
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
        assert!(has_neg_cycle_pred(&mut ncfq, &mut dist, |w| *w));
    }

    #[test]
    fn test_q_succ_string_nodes() {
        let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
        graph.insert("a", [("b", 1)].into());
        graph.insert("b", [("c", 1)].into());
        graph.insert("c", [("a", -3)].into());
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
        assert!(has_neg_cycle_succ(&mut ncfq, &mut dist, |w| *w));
    }

    // --- empty graph ---

    #[test]
    fn test_q_empty_graph() {
        let graph: HashMap<i32, HashMap<i32, i32>> = HashMap::new();
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = HashMap::new();
        assert!(ncfq
            .howard_pred(&mut dist, |w| *w, |_, _| true)
            .into_iter()
            .next()
            .is_none());
        assert!(ncfq
            .howard_succ(&mut dist, |w| *w, |_, _| true)
            .into_iter()
            .next()
            .is_none());
    }
}
