//! Bellman-Ford shortest path algorithm example

use digraphx_rs::bellman_ford;
use petgraph::Graph;

fn main() {
    // Create a directed graph with weighted edges
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());
    let d = g.add_node(());

    // Add edges with weights
    g.extend_with_edges([(a, b, 4.0), (b, c, 3.0), (a, c, 10.0), (c, d, 2.0)]);

    // Find shortest paths from node 'a'
    let paths = bellman_ford(&g, a).unwrap();

    println!("Shortest path distances from node a:");
    println!("  a: {} (source)", paths.distances[a.index()]);
    println!("  b: {}", paths.distances[b.index()]);
    println!("  c: {}", paths.distances[c.index()]);
    println!("  d: {}", paths.distances[d.index()]);

    // Access predecessors to reconstruct paths
    println!("\nPath reconstruction (predecessors):");
    for (i, pred) in paths.predecessors.iter().enumerate() {
        println!("  Node {}: {:?}", i, pred);
    }

    // Example: Reconstruct path to node d
    println!("\nShortest path from a to d:");
    let mut path = Vec::new();
    let mut current = Some(d);
    while let Some(node) = current {
        path.push(node);
        current = paths.predecessors[node.index()];
        if current == Some(a) {
            path.push(a);
            break;
        }
    }
    path.reverse();
    println!("  Path: {:?}", path);
}
