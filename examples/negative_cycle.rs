//! Negative cycle detection example

use digraphx_rs::find_negative_cycle;
use petgraph::Graph;

fn main() {
    println!("=== Example 1: Graph with negative cycle ===");
    let mut g_with_cycle = Graph::new();
    let a = g_with_cycle.add_node(());
    let b = g_with_cycle.add_node(());
    let c = g_with_cycle.add_node(());

    // Add edges creating a negative cycle: a -> b -> c -> a
    g_with_cycle.extend_with_edges([(a, b, 1.0), (b, c, 1.0), (c, a, -3.0)]);

    let cycle = find_negative_cycle(&g_with_cycle, a);
    match cycle {
        Some(nodes) => {
            println!("Negative cycle found: {:?}", nodes);
            println!("The total weight of this cycle is negative,");
            println!("making shortest paths undefined.");
        }
        None => println!("No negative cycle found."),
    }

    println!("\n=== Example 2: Graph without negative cycle ===");
    let mut g_no_cycle = Graph::new();
    let x = g_no_cycle.add_node(());
    let y = g_no_cycle.add_node(());
    let z = g_no_cycle.add_node(());

    // Add edges without negative cycle
    g_no_cycle.extend_with_edges([(x, y, 1.0), (y, z, 1.0), (z, x, 1.0)]);

    let cycle = find_negative_cycle(&g_no_cycle, x);
    match cycle {
        Some(nodes) => println!("Negative cycle found: {:?}", nodes),
        None => println!("No negative cycle found. All edge weights sum to non-negative."),
    }

    println!("\n=== Example 3: Simple two-node negative cycle ===");
    let mut g_simple = Graph::new();
    let n1 = g_simple.add_node(());
    let n2 = g_simple.add_node(());
    g_simple.extend_with_edges([(n1, n2, 1.0), (n2, n1, -2.0)]);

    let cycle = find_negative_cycle(&g_simple, n1);
    match cycle {
        Some(nodes) => println!("Negative cycle found: {:?}", nodes),
        None => println!("No negative cycle found."),
    }
}
