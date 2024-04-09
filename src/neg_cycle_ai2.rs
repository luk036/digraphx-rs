use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Iterator;

use petgraph::graph::DiGraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
// use petgraph::visit::{IntoEdges, IntoNeighbors, VisitMap, Visitable};
// use petgraph::Direction;

use num::rational::Ratio;

type Cycle<'a, V> = Vec<(&'a V, &'a V)>;

struct NegCycleFinder<'a, V, D> {
    pred: HashMap<&'a V, &'a V>,
    digraph: &'a DiGraph<&'a V, D>,
}

impl<'a, V, D> NegCycleFinder<'a, V, D>
where
    V: Eq + Hash,
    D: std::ops::Add<Output = D> + std::cmp::PartialOrd + std::ops::Sub<Output = D> + Copy,
{
    fn new(digraph: &'a DiGraph<&'a V, D>) -> Self {
        Self {
            pred: HashMap::new(),
            digraph,
        }
    }

    fn find_cycle(&mut self) -> Option<Cycle<'a, V>> {
        let mut visited = HashSet::new();
        for vtx in self
            .digraph
            .node_indices()
            .filter(|vtx| !visited.contains(vtx))
        {
            let mut utx = vtx;
            while !visited.contains(&utx) {
                visited.insert(utx);
                if let Some(&pred) = self.pred.get(&utx) {
                    utx = pred;
                    if visited.contains(&utx) {
                        if let Some(cycle) = self.build_cycle(&utx, &vtx) {
                            return Some(cycle);
                        }
                    }
                } else {
                    break;
                }
            }
        }
        None
    }

    fn build_cycle(&self, start: &V, end: &V) -> Option<Cycle<'a, V>> {
        let mut cycle = vec![];
        let mut utx = end;
        while utx != start {
            if let Some(&pred) = self.pred.get(utx) {
                cycle.push((pred, utx));
                utx = pred;
            } else {
                return None;
            }
        }
        cycle.reverse();
        Some(cycle)
    }

    pub fn relax2(&mut self, dist: &mut HashMap<&'a V, D>) -> bool {
        let mut changed = false;
        for edge in self.digraph.edge_references() {
            let utx = edge.source();
            let vtx = edge.target();
            let weight = *edge.weight();
            let distance = dist[utx] + weight;
            if dist[vtx] > distance {
                dist[vtx] = distance;
                self.pred.insert(vtx, utx);
                changed = true;
            }
        }
        changed
    }

    fn relax(
        &mut self,
        dist: &mut HashMap<&'a V, D>,
        edge: &petgraph::graph::EdgeReference<'_, D>,
    ) -> bool {
        let utx = edge.source();
        let vtx = edge.target();
        let weight = *edge.weight();
        let distance = dist[utx] + weight;
        if dist[vtx] > distance {
            dist.insert(vtx, distance);
            self.pred.insert(vtx, utx);
            true
        } else {
            false
        }
    }

    fn bellman_ford(&mut self, start: &V) -> Option<HashMap<&'a V, D>> {
        let mut dist = HashMap::new();
        let mut changed = true;
        for vtx in self.digraph.node_indices() {
            if vtx == *start {
                dist.insert(&self.digraph[vtx], Ratio::from_integer(0));
            } else {
                dist.insert(&self.digraph[vtx], Ratio::from_integer(i32::max_value()));
            }
        }
        for _ in 0..self.digraph.node_count() {
            if !changed {
                break;
            }
            changed = false;
            for edge in self.digraph.edge_references() {
                if self.relax(&mut dist, edge) {
                    changed = true;
                }
            }
        }
        if changed {
            self.find_cycle()
        } else {
            Some(dist)
        }
    }
}

fn main() {
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
    let dist = ncf.bellman_ford(&a).unwrap();
    println!("{:?}", dist);
}
