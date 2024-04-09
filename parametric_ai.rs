 Here's a rough conversion of the given Python code into Rust using the Petgraph library for graph data structures. Please note that this is not a complete or perfect translation as there are some differences between Python and Rust syntax and design patterns. Also, the given Python code lacks proper error handling and edge cases, so those should be added to the Rust version if needed.

First, we need to create some types and functions that correspond to the Python types and classes:

```rust
use std::collections::{HashMap, Vec};
use petgraph::prelude::*;
use petgraph::graph::NodeInfo;

type EdgeWeight = Domain; // Define Domain type for edge weights

#[derive(Clone)]
struct Node<T: Ord + 'static> {
    id: T,
    info: NodeInfo,
}

#[derive(Debug, Clone)]
enum Cycle {
    Empty,
    Nodes(Vec<Node<usize>>),
}

#[derive(Debug, Clone)]
struct Domain {
    low: f64,
    high: f64,
}

impl PartialOrd for Domain {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Domain {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.low.partial_cmp(&other.low).unwrap_or(self.high.cmp(&other.high))
    }
}

#[derive(Debug)]
struct Edge<T: Ord + 'static> {
    id: T,
    weight: EdgeWeight,
}

impl<T: Ord + 'static> Edge<T> {
    fn new(id1: T, id2: T, weight: EdgeWeight) -> Self {
        Edge { id1, id2, weight }
    }
}

#[derive(Debug)]
struct Graph<T: Ord + 'static> {
    nodes: HashMap<T, Node<T>>,
    edges: Vec<Edge<T>>,
}

impl<T: Ord + 'static> Graph<T> {
    fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, id: T) {
        let node = Node {
            id,
            info: NodeInfo::default(),
        };
        self.nodes.insert(id, node);
    }

    fn add_edge(&mut self, id1: T, id2: T, weight: EdgeWeight) {
        let edge = Edge::new(id1, id2, weight);
        self.edges.push(edge);
    }
}
```

Now we can implement the `ParametricAPI`, `NegCycleFinder`, and `MaxParametricSolver` in Rust:

```rust
struct ParametricAPI<N, E, R> {
    distance: fn(&R, &Edge<usize>) -> Domain,
    zero_cancel: fn(&Vec<Node<usize>>) -> Domain,
}

impl<N: Ord + 'static, E: Clone, R: Ord + 'static> ParametricAPI<N, E, R> {
    fn new(distance: fn(&R, &E) -> Domain, zero_cancel: fn(&Vec<Node<usize>>) -> Domain) -> Self {
        ParametricAPI { distance, zero_cancel }
    }
}

struct NegCycleFinder<'a, N, E>
where
    N: Ord + 'static,
    E: Clone,
{
    graph: &'a mut Graph<N>,
    dist: Vec<Domain>,
    prev: Vec<Option<usize>>,
}

impl<'a, N: Ord + 'static, E: Clone> NegCycleFinder<'a, N, E> {
    fn new(graph: &'a mut Graph<N>) -> Self {
        NegCycleFinder {
            graph,
            dist: Vec::new(),
            prev: Vec::repeat(None, graph.nodes.len()),
        }
    }

    fn howard(&mut self) -> Option<Vec<Node<usize>>> {
        let mut stack = vec![self.graph.nodes[&self.graph.nodes.keys().next().unwrap()].id];
        self.dist[stack[0]] = Domain::new(f64::MIN, f64::MAX);

        while let Some(cur_node) = stack.pop() {
            for edge in &self.graph.edges {
                if edge.id1 == cur_node || edge.id2 == cur_node {
                    continue;
                }

                let new_dist = (self.graph.nodes[&edge.id1].info.index() as i32)
                    .checked_sub(self.graph.nodes[&cur_node].info.index() as i32)
                    .map(|x| self.dist[x] + edge.weight);
                if let Some(new_dist) = new_dist {
                    if self.dist[edge.id1] > new_dist {
                        self.dist[edge.id1] = new_dist;
                        self.prev[edge.id1] = Some(cur_node as usize);
                        stack.push(edge.id1);
                    }
                }
            }
        }

        if self.dist[self.graph.nodes.len() as usize - 1].low >= 0. {
            Some(vec![self.graph.nodes[&self.prev[self.graph.nodes.len() as usize - 1].unwrap()]])
        } else {
            None
        }
    }
}

struct MaxParametricSolver<N, E>
where
    N: Ord + 'static,
    E: Clone,
{
    graph: Graph<N>,
    parametric_api: ParametricAPI<N, E, f64>,
    cycle_finder: NegCycleFinder<'_, N, E>,
}

impl<N: Ord + 'static, E: Clone> MaxParametricSolver<N, E> {
    fn new(graph: Graph<N>, parametric_api: ParametricAPI<N, E, f64>) -> Self {
        MaxParametricSolver {
            graph,
            parametric_api,
            cycle_finder: NegCycleFinder::new(&mut graph),
        }
    }

    fn solve(&mut self) -> (f64, Cycle<Vec<Node<usize>>>) {
        let mut r_min = f64::MAX;
        let mut c_min: Option<Vec<Node<usize>>> = None;

        while let Some(cycle) = self.cycle_finder.howard() {
            let ri = (self.parametric_api.zero_cancel)(cycle);
            if r_min > ri {
                r_min = ri;
                c_min = Some(cycle);
            }
        }

        if self.cycle_finder.dist[self.graph.nodes.len() as usize - 1].low >= 0. {
            (r_min, c_min.unwrap())
        } else {
            (f64::MIN, Cycle::Empty)
        }
    }
}
```

Finally, we can use the `MaxParametricSolver` to find the minimum ratio and its corresponding cycle:

```rust
fn main() {
    let mut graph = Graph::<usize>::new();

    // Add nodes and edges with their weights

    let parametric_api = ParametricAPI::new(|r, e| Domain::new(r.abs(), r.abs() * 2.), |n1, n2| n1 < n2);
    let mut solver = MaxParametricSolver::new(graph, parametric_api);

    let (min_ratio, cycle) = solver.solve();

    println!("Minimum ratio: {}", min_ratio);
    if cycle.is_empty() {
        println!("No cycle found");
    } else {
        println!("Cycle:");
        for node in &cycle {
            println!("{}", node);
        }
    }
}
```

