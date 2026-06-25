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
//! - Node-pair cycle output for parametric algorithms
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
//! let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true);
//! assert!(result.is_some());
//! ```
//!
//! [`update_ok`]: NegCycleFinderQ::howard_pred

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Sub;

use crate::Graph;
use crate::Zero;

/// Negative cycle finder with constraints (predecessor / successor).
///
/// Extends the basic [`NegCycleFinder`](crate::NegCycleFinder) with:
/// - **Predecessor-based** Howard (`howard_pred`) — traditional Bellman–Ford
/// - **Successor-based** Howard (`howard_succ`) — reverse relaxation
/// - An **`update_ok`** callback that gates distance updates
/// - **Node-pair** cycle output for parametric algorithms
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
/// let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true);
/// assert!(result.is_some());
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

    /// Find a cycle in the given mapping (`pred` or `succ`).
    fn find_cycle(&self, point_to: &HashMap<G::Node, (G::Node, G::Weight)>) -> Option<G::Node> {
        let mut visited: HashMap<G::Node, G::Node> = HashMap::new();
        for vtx in self.graph.nodes() {
            if visited.contains_key(&vtx) {
                continue;
            }
            let mut utx = vtx;
            while !visited.contains_key(&utx) {
                visited.insert(utx, vtx);
                match point_to.get(&utx) {
                    None => break,
                    Some(&(prev, _)) => {
                        utx = prev;
                        if let Some(&root) = visited.get(&utx) {
                            if root == vtx {
                                return Some(utx);
                            }
                            break;
                        }
                    }
                }
            }
        }
        None
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
        let mut changed = false;
        for u in self.graph.nodes() {
            let du = *dist.get(&u).unwrap_or(&G::Weight::zero());
            for (v, w) in self.graph.neighbors(u) {
                let distance = du + get_weight(&w);
                let dv = *dist.get(&v).unwrap_or(&G::Weight::zero());
                if dv > distance && update_ok(&dv, &distance) {
                    dist.insert(v, distance);
                    self.pred.insert(v, (u, w));
                    changed = true;
                }
            }
        }
        changed
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
        let mut changed = false;
        for u in self.graph.nodes() {
            let du = *dist.get(&u).unwrap_or(&G::Weight::zero());
            for (v, w) in self.graph.neighbors(u) {
                let distance = *dist.get(&v).unwrap_or(&G::Weight::zero()) - get_weight(&w);
                if du < distance && update_ok(&du, &distance) {
                    dist.insert(u, distance);
                    self.succ.insert(u, (v, w));
                    changed = true;
                }
            }
        }
        changed
    }

    // ------------------------------------------------------------------
    // Cycle reconstruction
    // ------------------------------------------------------------------

    /// Reconstruct a cycle from the given mapping (as edge weights).
    fn cycle_list(
        &self,
        handle: G::Node,
        point_to: &HashMap<G::Node, (G::Node, G::Weight)>,
    ) -> Vec<G::Weight> {
        let mut vtx = handle;
        let mut cycle = Vec::new();
        loop {
            let &(u, w) = point_to.get(&vtx).unwrap();
            cycle.push(w);
            vtx = u;
            if vtx == handle {
                break;
            }
        }
        cycle
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
    pub fn is_negative<F>(
        &self,
        handle: G::Node,
        dist: &HashMap<G::Node, G::Weight>,
        get_weight: &F,
    ) -> bool
    where
        F: Fn(&G::Weight) -> G::Weight,
    {
        let mut vtx = handle;
        loop {
            let &(u, w) = self.pred.get(&vtx).unwrap();
            let dv = *dist.get(&vtx).unwrap_or(&G::Weight::zero());
            let du = *dist.get(&u).unwrap_or(&G::Weight::zero());
            if dv > du + get_weight(&w) {
                return true;
            }
            vtx = u;
            if vtx == handle {
                break;
            }
        }
        false
    }

    // ------------------------------------------------------------------
    // Howard's algorithm: predecessor-based (returns edge weights)
    // ------------------------------------------------------------------

    /// Predecessor-based Howard's algorithm with constraint.
    ///
    /// Returns the first negative cycle found as a list of edge weights,
    /// or `None` if no negative cycle exists.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    pub fn howard_pred<F, U>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
        update_ok: U,
    ) -> Option<Vec<G::Weight>>
    where
        F: Fn(&G::Weight) -> G::Weight,
        U: Fn(&G::Weight, &G::Weight) -> bool,
    {
        self.pred.clear();
        while self.relax_pred(dist, &get_weight, &update_ok) {
            if let Some(vtx) = self.find_cycle(&self.pred) {
                debug_assert!(self.is_negative(vtx, dist, &get_weight));
                return Some(self.cycle_list(vtx, &self.pred));
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // Howard's algorithm: successor-based (returns edge weights)
    // ------------------------------------------------------------------

    /// Successor-based Howard's algorithm with constraint.
    ///
    /// Returns the first negative cycle found as a list of edge weights,
    /// or `None` if no negative cycle exists.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    pub fn howard_succ<F, U>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
        update_ok: U,
    ) -> Option<Vec<G::Weight>>
    where
        F: Fn(&G::Weight) -> G::Weight,
        U: Fn(&G::Weight, &G::Weight) -> bool,
    {
        self.succ.clear();
        while self.relax_succ(dist, &get_weight, &update_ok) {
            if let Some(vtx) = self.find_cycle(&self.succ) {
                return Some(self.cycle_list(vtx, &self.succ));
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // Node-pair cycle helpers
    // ------------------------------------------------------------------

    /// Reconstruct a cycle as node-pair edges from the given mapping.
    fn cycle_list_node_pairs(
        &self,
        handle: G::Node,
        point_to: &HashMap<G::Node, (G::Node, G::Weight)>,
    ) -> Vec<(G::Node, G::Node)> {
        let mut vtx = handle;
        let mut cycle = Vec::new();
        loop {
            let &(u, _) = point_to.get(&vtx).unwrap();
            cycle.push((u, vtx));
            vtx = u;
            if vtx == handle {
                break;
            }
        }
        cycle
    }

    // ------------------------------------------------------------------
    // find_neg_cycle_pred (edge-data weight) — returns node pairs
    // ------------------------------------------------------------------

    /// Find one negative cycle (predecessor) returning node-pair edges.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    /// Returns the cycle as a `Vec` of `(Node, Node)` edges, or `None`.
    pub fn find_neg_cycle_pred<F, U>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
        update_ok: U,
    ) -> Option<Vec<(G::Node, G::Node)>>
    where
        F: Fn(&G::Weight) -> G::Weight,
        U: Fn(&G::Weight, &G::Weight) -> bool,
    {
        self.pred.clear();
        while self.relax_pred(dist, &get_weight, &update_ok) {
            if let Some(vtx) = self.find_cycle(&self.pred) {
                debug_assert!(self.is_negative(vtx, dist, &get_weight));
                return Some(self.cycle_list_node_pairs(vtx, &self.pred));
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // find_neg_cycle_succ (edge-data weight) — returns node pairs
    // ------------------------------------------------------------------

    /// Find one negative cycle (successor) returning node-pair edges.
    ///
    /// The `get_weight` closure receives a reference to the stored edge data.
    /// Returns the cycle as a `Vec` of `(Node, Node)` edges, or `None`.
    pub fn find_neg_cycle_succ<F, U>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
        update_ok: U,
    ) -> Option<Vec<(G::Node, G::Node)>>
    where
        F: Fn(&G::Weight) -> G::Weight,
        U: Fn(&G::Weight, &G::Weight) -> bool,
    {
        self.succ.clear();
        while self.relax_succ(dist, &get_weight, &update_ok) {
            if let Some(vtx) = self.find_cycle(&self.succ) {
                return Some(self.cycle_list_node_pairs(vtx, &self.succ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_from_edges;
    use std::collections::HashMap;

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
        let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_none());
    }

    #[test]
    fn test_q_pred_neg_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
    }

    // --- howard_succ tests ---

    #[test]
    fn test_q_succ_neg_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.howard_succ(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
    }

    // --- constraint tests ---

    #[test]
    fn test_q_pred_with_constraint_blocks_all() {
        // Graph has negative cycle, but update_ok blocks every update
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| false);
        assert!(result.is_none());
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
        let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
    }

    #[test]
    fn test_q_succ_string_nodes() {
        let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
        graph.insert("a", [("b", 1)].into());
        graph.insert("b", [("c", 1)].into());
        graph.insert("c", [("a", -3)].into());
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
        let result = ncfq.howard_succ(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
    }

    // --- find_neg_cycle_pred/succ (edge-data weight) ---

    #[test]
    fn test_q_find_neg_cycle_pred_node_pairs() {
        let graph = graph_from_edges(&[(0, 1, 2i32), (1, 2, 3), (2, 0, -6)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.find_neg_cycle_pred(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
        let cycle = result.unwrap();
        for &(u, v) in &cycle {
            assert_ne!(u, v);
        }
        assert!(cycle.len() >= 2);
    }

    #[test]
    fn test_q_find_neg_cycle_succ_node_pairs() {
        let graph = graph_from_edges(&[(0, 1, 2i32), (1, 2, 3), (2, 0, -6)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.find_neg_cycle_succ(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
        let cycle = result.unwrap();
        for &(u, v) in &cycle {
            assert_ne!(u, v);
        }
        assert!(cycle.len() >= 2);
    }

    #[test]
    fn test_q_find_neg_cycle_pred_blocked() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.find_neg_cycle_pred(&mut dist, |w| *w, |_, _| false);
        assert!(result.is_none());
    }

    #[test]
    fn test_q_find_neg_cycle_succ_blocked() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.find_neg_cycle_succ(&mut dist, |w| *w, |_, _| false);
        assert!(result.is_none());
    }

    #[test]
    fn test_q_find_neg_cycle_pred_no_neg_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, 1)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.find_neg_cycle_pred(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_none());
    }

    // --- empty graph ---

    #[test]
    fn test_q_empty_graph() {
        let graph: HashMap<i32, HashMap<i32, i32>> = HashMap::new();
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = HashMap::new();
        assert!(ncfq.howard_pred(&mut dist, |w| *w, |_, _| true).is_none());
        assert!(ncfq.howard_succ(&mut dist, |w| *w, |_, _| true).is_none());
    }
}
