use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;

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
}
