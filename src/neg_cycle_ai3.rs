use petgraph::graph::DiGraph;
// use petgraph::visit::IntoNeighborsDirected;

/// Negative cycle finder using simple f64 weights.
struct NegCycleFinder<'a> {
    grph: &'a DiGraph<(), f64>,
    pred: std::collections::HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex>,
}

impl<'a> NegCycleFinder<'a> {
    /// Create a new negative cycle finder.
    fn new(grph: &'a DiGraph<(), f64>) -> Self {
        Self {
            grph,
            pred: std::collections::HashMap::new(),
        }
    }

    /// Find a cycle using the predecessor map.
    fn find_cycle(&self) -> impl Iterator<Item = petgraph::graph::NodeIndex> + '_ {
        let mut visited = std::collections::HashMap::new();
        self.grph
            .node_indices()
            .filter(move |&v| !visited.contains_key(&v))
            .filter_map(move |v| {
                let mut u = v;
                while !visited.contains_key(&u) {
                    visited.insert(u, v);
                    u = self.pred.get(&u);
                    if visited.contains_key(&u) {
                        if visited[&u] == v {
                            return Some(u);
                        }
                        break;
                    }
                }
                None
            })
    }

    /// Relax edges in the graph.
    fn relax<F>(&mut self, dist: &mut [f64], get_weight: F) -> bool
    where
        F: Fn((petgraph::graph::NodeIndex, petgraph::graph::NodeIndex)) -> f64,
    {
        let mut changed = false;
        for u in self.grph.node_indices() {
            for v in self
                .grph
                .neighbors_directed(u, petgraph::Direction::Outgoing)
            {
                let wt = get_weight((u, v));
                let d = dist[u.index()] + wt;
                if dist[v.index()] > d {
                    dist[v.index()] = d;
                    self.pred.insert(v, u);
                    changed = true;
                }
            }
        }
        changed
    }

    /// Find a negative cycle in the graph.
    fn find_neg_cycle<F>(
        &mut self,
        dist: &mut [f64],
        get_weight: F,
    ) -> Option<Vec<(petgraph::graph::NodeIndex, petgraph::graph::NodeIndex)>>
    where
        F: Fn((petgraph::graph::NodeIndex, petgraph::graph::NodeIndex)) -> f64,
    {
        self.pred.clear();
        let mut found = false;
        while !found && self.relax(dist, &get_weight) {
            for v in self.find_cycle() {
                found = true;
                return Some(self.cycle_list(v));
            }
        }
        None
    }

    /// Extract the cycle edges from a starting node.
    fn cycle_list(
        &self,
        handle: petgraph::graph::NodeIndex,
    ) -> Vec<(petgraph::graph::NodeIndex, petgraph::graph::NodeIndex)> {
        let mut v = handle;
        let mut cycle = Vec::new();
        loop {
            let u = self.pred[&v];
            cycle.push((u, v));
            v = u;
            if v == handle {
                break;
            }
        }
        cycle
    }
}
