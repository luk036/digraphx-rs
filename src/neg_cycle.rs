use petgraph::graph::{EdgeReference, NodeIndex};
use petgraph::prelude::*;
use petgraph::visit::EdgeRef;
use petgraph::visit::IntoNodeIdentifiers;
use std::collections::HashMap;
use std::ops::Add;

// use petgraph::visit::IntoNeighborsDirected;

/// The `NegCycleFinder` struct is used to find negative cycles in a directed graph.
///
/// Properties:
///
/// * `digraph`: The `digraph` property is a reference to a directed graph (`DiGraph`) that the
///             `NegCycleFinder` is operating on. It is annotated with a lifetime `'a`, indicating that the
///             reference is valid for a certain scope.
/// * `pred`: The `pred` property is a `HashMap` that maps a `NodeIndex` to a tuple containing the
///             previous node index and an `EdgeReference`. This is used to keep track of the predecessor node and
///             the edge that leads to that node during the process of finding negative cycles in a directed graph
#[derive(Debug)]
pub struct NegCycleFinder<'a, Value, Domain> {
    pub digraph: &'a DiGraph<Value, Domain>,
    pub pred: HashMap<NodeIndex, (NodeIndex, EdgeReference<'a, Domain>)>,
}

impl<'a, Value, Domain> NegCycleFinder<'a, Value, Domain>
where
    Domain: Add<Output = Domain> + PartialOrd + Copy,
{
    /// The `new` function creates a new `NegCycleFinder` object with an empty predecessor map.
    ///
    /// Arguments:
    ///
    /// * `digraph`: A reference to a directed graph (`DiGraph`) that the `NegCycleFinder` will operate on.
    ///
    /// Returns:
    ///
    /// The `new` function is returning an instance of the `NegCycleFinder<Value, Domain>` struct.
    /// Creates a new [`NegCycleFinder<Value, Domain>`].
    pub fn new(digraph: &'a DiGraph<Value, Domain>) -> Self {
        NegCycleFinder {
            digraph,
            pred: HashMap::new(),
        }
    }

    /// The `find_cycle` function in Rust returns the first node in a cycle found in a directed graph.
    ///
    /// Returns:
    ///
    /// The function `find_cycle` returns an `Option<NodeIndex>`.
    pub fn find_cycle(&self) -> Option<NodeIndex> {
        let mut visited = HashMap::new();
        for vtx in self.digraph.node_identifiers() {
            if visited.contains_key(&vtx) {
                continue;
            }
            let mut utx = vtx;
            while !visited.contains_key(&utx) {
                visited.insert(utx, vtx);
                if !self.pred.contains_key(&utx) {
                    break;
                }
                let result = *self.pred.get(&utx).unwrap();
                utx = result.0;
                if visited.contains_key(&utx) {
                    if visited[&utx] == vtx {
                        return Some(utx);
                    }
                    break;
                }
            }
        }
        None
    }

    /// The `relax` function updates the distances between nodes in a graph based on the weights of the
    /// edges, and returns a boolean indicating whether any distances were changed.
    ///
    /// Arguments:
    ///
    /// * `dist`: `dist` is a mutable reference to a slice of type `Domain`. It represents the distances from
    ///             a source node to each node in a graph.
    /// * `get_weight`: The `get_weight` parameter is a closure that takes an `EdgeReference<Domain>` as
    ///             input and returns a value of type `Domain`. This closure is used to calculate the weight of each edge
    ///             in the graph. The `EdgeReference<Domain>` represents a reference to an edge in the graph, and
    ///
    /// Returns:
    ///
    /// a boolean value.
    pub fn relax<Callable>(&mut self, dist: &mut [Domain], get_weight: Callable) -> bool
    where
        Callable: Fn(EdgeReference<Domain>) -> Domain,
    {
        let mut changed = false;
        for utx in self.digraph.node_identifiers() {
            for edge in self.digraph.edges(utx) {
                let vtx = edge.target();
                let weight = get_weight(edge);
                let distance = dist[utx.index()] + weight;
                if dist[vtx.index()] > distance {
                    dist[vtx.index()] = distance;
                    self.pred.insert(vtx, (utx, edge));
                    changed = true;
                }
            }
        }
        changed
    }

    /// The function `cycle_list` takes a node index as input and returns a vector of edge references
    /// that form a cycle in a graph.
    ///
    /// Arguments:
    ///
    /// * `handle`: The `handle` parameter is of type `NodeIndex`. It represents the starting node index
    ///             from which the cycle traversal will begin.
    ///
    /// Returns:
    ///
    /// The function `cycle_list` returns a vector of `EdgeReference` objects.
    fn cycle_list(&self, handle: NodeIndex) -> Vec<EdgeReference<'a, Domain>> {
        let mut vtx = handle;
        let mut cycle = Vec::new();
        loop {
            let (utx, edge) = self.pred[&vtx];
            cycle.push(edge);
            vtx = utx;
            if vtx == handle {
                break;
            }
        }
        cycle
    }

    /// The `howard` function implements Howard's algorithm for finding negative cycles in a directed
    /// graph.
    ///
    /// Arguments:
    ///
    /// * `dist`: `dist` is a mutable reference to an array of type `Domain`. This array is used to store the
    ///             distances from the source vertex to each vertex in the graph. The algorithm will update the
    ///             distances during the execution.
    /// * `get_weight`: `get_weight` is a closure that takes an `EdgeReference<Domain>` and returns the
    ///             weight of that edge. The `howard` function uses this closure to get the weight of each edge in
    ///             the graph.
    ///
    /// Returns:
    ///
    /// The `howard` function returns an `Option<Vec<EdgeReference<'a, Domain>>>`.
    /// Howard's algorithm for finding negative cycles
    ///
    /// # Examples
    ///
    /// ```
    /// use petgraph::prelude::*;
    /// use digraphx_rs::neg_cycle::NegCycleFinder;
    /// let digraph = DiGraph::<(), i32>::from_edges([
    ///     (0, 1, 1),
    ///     (0, 2, 1),
    ///     (0, 3, 1),
    ///     (1, 3, 1),
    ///     (2, 1, 1),
    ///     (3, 2, -3),
    /// ]);
    /// let mut ncf = NegCycleFinder::new(&digraph);
    /// let mut dist = [0, 0, 0, 0];
    /// let result = ncf.howard(&mut dist, |e| { *e.weight()});
    /// assert!(result.is_some());
    /// ```
    pub fn howard<F>(
        &mut self,
        dist: &mut [Domain],
        get_weight: F,
    ) -> Option<Vec<EdgeReference<'a, Domain>>>
    where
        F: Fn(EdgeReference<Domain>) -> Domain,
    {
        self.pred.clear();
        while self.relax(dist, &get_weight) {
            let v_opt = self.find_cycle();
            if let Some(vtx) = v_opt {
                return Some(self.cycle_list(vtx));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::rational::Ratio;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_neg_cycle1() {
        let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
            (0, 1, Ratio::new(1, 1)),
            (0, 2, Ratio::new(1, 1)),
            (0, 3, Ratio::new(1, 1)),
            (1, 3, Ratio::new(1, 1)),
            (2, 1, Ratio::new(1, 1)),
            (3, 2, Ratio::new(-3, 1)),
        ]);

        let mut ncf = NegCycleFinder::new(&digraph);
        let mut dist = [
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
        ];
        let result = ncf.howard(&mut dist, |e| *e.weight());
        assert!(result.is_some());
    }

    #[test]
    fn test_neg_cycle2() {
        let mut graph = DiGraph::new();
        let a = graph.add_node("a");
        let b = graph.add_node("b");
        let c = graph.add_node("c");
        let d = graph.add_node("d");
        let e = graph.add_node("e");
        let f = graph.add_node("f");
        let g = graph.add_node("g");
        let h = graph.add_node("h");
        let i = graph.add_node("i");
        graph.add_edge(a, b, Ratio::new(1, 1));
        graph.add_edge(a, c, Ratio::new(1, 1));
        graph.add_edge(b, d, Ratio::new(1, 1));
        graph.add_edge(c, d, Ratio::new(1, 1));
        graph.add_edge(d, e, Ratio::new(-3, 1));
        graph.add_edge(d, f, Ratio::new(1, 1));
        graph.add_edge(e, g, Ratio::new(1, 1));
        graph.add_edge(f, g, Ratio::new(1, 1));
        graph.add_edge(g, h, Ratio::new(1, 1));
        graph.add_edge(h, i, Ratio::new(1, 1));
        graph.add_edge(i, f, Ratio::new(1, 1));

        let mut ncf = NegCycleFinder::new(&graph);
        let mut dist = [
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
            Ratio::new(0, 1),
        ];
        let result = ncf.howard(&mut dist, |e| *e.weight());
        assert!(result.is_none());
    }
}
