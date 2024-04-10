use std::collections::{HashMap, HashSet};
use std::hash::Hash;

struct NegCycleFinder<Node, Edge, Domain>
where
    Node: Hash + Eq,
    Edge: Hash + Eq,
    Domain: PartialEq + PartialOrd + Clone,
{
    digraph: HashMap<Node, HashMap<Node, Edge>>,
    pred: HashMap<Node, (Node, Edge)>,
}

impl<Node, Edge, Domain> NegCycleFinder<Node, Edge, Domain>
where
    Node: Hash + Eq + Clone,
    Edge: Hash + Eq + Clone,
    Domain: PartialEq + PartialOrd + Clone,
{
    fn new(digraph: HashMap<Node, HashMap<Node, Edge>>) -> NegCycleFinder<Node, Edge, Domain> {
        NegCycleFinder {
            digraph,
            pred: HashMap::new(),
        }
    }

    fn find_cycle(&self) -> HashSet<Node> {
        let mut visited: HashMap<Node, Node> = HashMap::new();
        let mut result: HashSet<Node> = HashSet::new();

        for vtx in self.digraph.keys() {
            if !visited.contains_key(vtx) {
                let mut utx = vtx.clone();
                visited.insert(utx.clone(), vtx.clone());

                while self.pred.contains_key(&utx) {
                    let (next_node, _) = self.pred[&utx].clone();
                    if visited.contains_key(&next_node) {
                        if visited[&next_node] == *vtx {
                            result.insert(next_node.clone());
                        }
                        break;
                    }
                    visited.insert(next_node.clone(), vtx.clone());
                    utx = next_node;
                }
            }
        }

        result
    }

    fn relax(
        &self,
        dist: &mut HashMap<Node, Domain>,
        get_weight: &dyn Fn(Edge) -> Domain,
    ) -> bool {
        let mut changed = false;

        for (utx, neighbors) in &self.digraph {
            for (vtx, edge) in neighbors {
                let distance = dist[utx].clone() + get_weight(edge.clone());
                if dist[vtx] > distance {
                    dist.insert(vtx.clone(), distance);
                    self.pred.insert(vtx.clone(), (utx.clone(), edge.clone()));
                    changed = true;
                }
            }
        }

        changed
    }

    fn howard(
        &mut self,
        dist: &mut HashMap<Node, Domain>,
        get_weight: &dyn Fn(Edge) -> Domain,
    ) -> Vec<Vec<Edge>> {
        let mut result: Vec<Vec<Edge>> = vec![];
        let mut found = false;

        while !found && self.relax(dist, get_weight) {
            for vtx in self.find_cycle() {
                assert!(self.is_negative(&vtx, dist, get_weight));
                found = true;
                result.push(self.cycle_list(vtx));
            }
        }

        result
    }

    fn cycle_list(&self, handle: Node) -> Vec<Edge> {
        let mut vtx = handle;
        let mut cycle: Vec<Edge> = vec![];

        loop {
            let (utx, edge) = self.pred[&vtx].clone();
            cycle.push(edge.clone());
            vtx = utx;
            if vtx == handle {
                break;
            }
        }

        cycle
    }

    fn is_negative(
        &self,
        handle: &Node,
        dist: &HashMap<Node, Domain>,
        get_weight: &dyn Fn(Edge) -> Domain,
    ) -> bool {
        let mut vtx = handle.clone();

        while {
            let (utx, edge) = self.pred[&vtx].clone();
            if dist[&vtx] > dist[&utx] + get_weight(edge.clone()) {
                return true;
            }

            vtx = utx;
            vtx != handle
        } {}

        false
    }
}

fn main() {
    let mut digraph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
    let mut inner1: HashMap<&str, i32> = HashMap::new();
    inner1.insert("b", 7);
    inner1.insert("c", 5);

    let mut inner2: HashMap<&str, i32> = HashMap::new();
    inner2.insert("a", 0);
    inner2.insert("c", 3);

    let mut inner3: HashMap<&str, i32> = HashMap::new();
    inner3.insert("b", 1);
    inner3.insert("a", 2);

    digraph.insert("a", inner1);
    digraph.insert("b", inner2);
    digraph.insert("c", inner3);

    let mut dist: HashMap<&str, i32> = HashMap::new();
    dist.insert("a", 0);
    dist.insert("b", 0);
    dist.insert("c", 0);

    let mut finder = NegCycleFinder::new(digraph);
    let result = finder.howard(&mut dist, &|edge| *edge);
    println!("{:?}", result);
}
