use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Sub;

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
/// let result = ncf.howard(&mut dist, |w| *w);
/// assert!(result.is_none()); // no negative cycle
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
    pub fn relax<F>(&mut self, dist: &mut HashMap<G::Node, G::Weight>, get_weight: &F) -> bool
    where
        F: Fn(&G::Weight) -> G::Weight,
    {
        let mut changed = false;
        for u in self.graph.nodes() {
            let du = *dist.get(&u).unwrap_or(&G::Weight::zero());
            for (v, w) in self.graph.neighbors(u) {
                let distance = du + get_weight(&w);
                let dv = *dist.get(&v).unwrap_or(&G::Weight::zero());
                if dv > distance {
                    dist.insert(v, distance);
                    self.pred.insert(v, (u, w));
                    changed = true;
                }
            }
        }
        changed
    }

    /// Find a cycle in the predecessor graph.
    ///
    /// Returns the start node of a cycle if one exists.
    fn find_cycle(&self) -> Option<G::Node> {
        let mut visited: HashMap<G::Node, G::Node> = HashMap::new();
        for vtx in self.graph.nodes() {
            if visited.contains_key(&vtx) {
                continue;
            }
            let mut utx = vtx;
            while !visited.contains_key(&utx) {
                visited.insert(utx, vtx);
                match self.pred.get(&utx) {
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

    /// Reconstruct the cycle edges starting from `handle`.
    fn cycle_list(&self, handle: G::Node) -> Vec<G::Weight> {
        let mut vtx = handle;
        let mut cycle = Vec::new();
        loop {
            let &(u, w) = self.pred.get(&vtx).unwrap();
            cycle.push(w);
            vtx = u;
            if vtx == handle {
                break;
            }
        }
        cycle
    }

    /// Howard's algorithm: find a negative cycle in the graph.
    ///
    /// Returns `Some(cycle)` where `cycle` is a list of edge weights
    /// forming a negative cycle, or `None` if no negative cycle exists.
    ///
    /// # Type parameters
    ///
    /// * `F` — weight-extraction closure (typically `\|w\| *w` when the
    ///   weight is the edge data itself, or a projection for structured
    ///   edge types).
    pub fn howard<F>(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        get_weight: F,
    ) -> Option<Vec<G::Weight>>
    where
        F: Fn(&G::Weight) -> G::Weight,
    {
        self.pred.clear();
        while self.relax(dist, &get_weight) {
            if let Some(vtx) = self.find_cycle() {
                return Some(self.cycle_list(vtx));
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// NegCycleFinderQ — constrained version with pred/succ + update_ok
// ---------------------------------------------------------------------------

/// Negative cycle finder with constraints (predecessor / successor).
///
/// Extends the basic [`NegCycleFinder`] with:
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

    /// Predecessor relaxation (Bellman–Ford style) with constraint.
    ///
    /// For each edge $(u, v)$, updates $d\[v\]$ when:
    ///
    /// $$ d\[v\] > d\[u\] + w(u,v) $$
    ///
    /// AND $\text{update\_ok}(d_{\text{old}}, d_{\text{new}})$ is `true`.
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

    /// Successor relaxation (reverse Bellman–Ford style) with constraint.
    ///
    /// For each edge $(u, v)$, updates $d\[u\]$ when:
    ///
    /// $$ d\[u\] < d\[v\] - w(u,v) $$
    ///
    /// AND $\text{update\_ok}(d_{\text{old}}, d_{\text{new}})$ is `true`.
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

    /// Reconstruct a cycle from the given mapping.
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

    /// Check whether the cycle starting at `handle` is negative.
    ///
    /// A cycle is negative if for any edge $(u,v)$ on the cycle:
    ///
    /// $$ d\[v\] > d\[u\] + w(u,v) $$
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

    /// Predecessor-based Howard's algorithm with constraint.
    ///
    /// Returns the first negative cycle found as edge weights, or `None`.
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

    /// Successor-based Howard's algorithm with constraint.
    ///
    /// Returns the first negative cycle found as edge weights, or `None`.
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

    /// Find one negative cycle (predecessor) returning node-pair edges.
    ///
    /// Useful for parametric algorithms that need edge endpoints.
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

    /// Find one negative cycle (successor) returning node-pair edges.
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
        let result = ncf.howard(&mut dist, |w| *w);
        assert!(result.is_none());
    }

    #[test]
    fn test_negative_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncf.howard(&mut dist, |w| *w);
        assert!(result.is_some());
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<i32, HashMap<i32, i32>> = HashMap::new();
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = HashMap::new();
        let result = ncf.howard(&mut dist, |w| *w);
        assert!(result.is_none());
    }

    #[test]
    fn test_string_nodes() {
        let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
        graph.insert("a", [("b", 1)].into());
        graph.insert("b", [("c", 1)].into());
        graph.insert("c", [("a", -3)].into());

        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
        let result = ncf.howard(&mut dist, |w| *w);
        assert!(result.is_some());
    }

    #[test]
    fn test_single_node() {
        let graph: HashMap<i32, HashMap<i32, i32>> = [(0, HashMap::new())].into();
        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0)].into();
        let result = ncf.howard(&mut dist, |w| *w);
        assert!(result.is_none());
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
        let result = ncf.howard(&mut dist, |w| *w);
        assert!(result.is_some());
    }

    // --- NegCycleFinderQ tests ---

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

    #[test]
    fn test_q_succ_neg_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.howard_succ(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
    }

    #[test]
    fn test_q_pred_with_constraint_blocks_all() {
        // Graph has negative cycle, but update_ok blocks every update
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.howard_pred(&mut dist, |w| *w, |_, _| false);
        assert!(result.is_none());
    }

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

    #[test]
    fn test_q_find_neg_cycle_pred_node_pairs() {
        let graph = graph_from_edges(&[(0, 1, 2i32), (1, 2, 3), (2, 0, -6)]);
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let result = ncfq.find_neg_cycle_pred(&mut dist, |w| *w, |_, _| true);
        assert!(result.is_some());
        let cycle = result.unwrap();
        // Each element is a (u, v) node pair
        for &(u, v) in &cycle {
            assert_ne!(u, v); // no self-loop
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
    fn test_q_empty_graph() {
        let graph: HashMap<i32, HashMap<i32, i32>> = HashMap::new();
        let mut ncfq = NegCycleFinderQ::new(&graph);
        let mut dist: HashMap<i32, i32> = HashMap::new();
        assert!(ncfq.howard_pred(&mut dist, |w| *w, |_, _| true).is_none());
        assert!(ncfq.howard_succ(&mut dist, |w| *w, |_, _| true).is_none());
    }
}
