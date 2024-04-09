 To convert the given Python code into Rust using the `petgraph` library, we'll need to do the following:

1. Create new Rust data structures and functions that correspond to the Python classes and functions.
2. Use the `Petgraph` graph data structure provided by `petgraph`.
3. Write Rust code that initializes the graph and adds an edge between nodes.
4. Modify the existing code to print out the number of nodes and edges, as well as printing node neighbors.
5. Create a new function to initialize the `Vec` used for storing node data.

Here's the Rust equivalent:

```rust
use petgraph::prelude::{Direction, Edge, Graph, NodeIndex};
use std::collections::VecDeque;

struct TinyDiGraph {
    graph: Graph<i32, i32>,
    node_data: Vec<i32>,
}

impl TinyDiGraph {
    fn new(num_nodes: usize) -> Self {
        let mut node_data = Vec::new();
        node_data.resize(num_nodes, 0);

        TinyDiGraph {
            graph: Graph::new(),
            node_data,
        }
    }

    fn init_nodes(&mut self, num_nodes: usize) {
        self.graph.add_nodes(num_nodes as u64);
        self.node_data.resize(num_nodes, 0);
    }

    fn add_edge(&mut self, src: NodeIndex, dst: NodeIndex) {
        let edge = Edge::new(src, dst, 1);
        self.graph.add_edge(edge);
    }

    fn number_of_nodes(&self) -> usize {
        self.graph.node_count() as usize
    }

    fn number_of_edges(&self) -> usize {
        self.graph.edge_count() as usize
    }

    fn neighbors(&self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut neighbors = VecDeque::new();
        for neighbor in self.graph.neighbors(node) {
            neighbors.push_back(neighbor);
        }
        neighbors.into_vec()
    }
}

fn main() {
    let mut tiny_digraph = TinyDiGraph::new();
    tiny_digraph.init_nodes(1000);
    tiny_digraph.add_edge(2 as NodeIndex, 1 as NodeIndex);

    println!("Number of nodes: {}", tiny_digraph.number_of_nodes());
    println!(
        "Number of edges: {}",
        tiny_digraph.number_of_edges()
    );

    for node in tiny_digraph.graph.iter().filter(|n| n.data() != &None) {
        let neighbors = tiny_digraph.neighbors(*node);
        println!(
            "Node {}, neighbors: {:?}",
            node, neighbors
        );
    }
}
```

In this example, we create a `TinyDiGraph` struct that has an inner `Graph<i32, i32>` data structure and a `Vec<i32>` for storing node data. The `init_nodes`, `add_edge`, `number_of_nodes`, and `number_of_edges` functions have been implemented to correspond to their Python counterparts.

The main function initializes the graph, adds an edge between two nodes, and prints out the number of nodes and edges along with each node's neighbors.

