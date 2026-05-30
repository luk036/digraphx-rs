//! Example usage of digraphx-rs with the generic graph API.

use digraphx_rs::NegCycleFinder;
use std::collections::HashMap;

fn main() {
    // A graph as HashMap<Node, HashMap<Node, Weight>> — the canonical
    // "container of containers" representation.
    let graph: HashMap<&str, HashMap<&str, i32>> = [
        ("a", [("b", 1), ("c", 1)].into()),
        ("b", [("c", 1)].into()),
        ("c", [("a", -3)].into()),
    ]
    .into();

    let mut ncf = NegCycleFinder::new(&graph);
    let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();

    match ncf.howard(&mut dist, |w| *w) {
        Some(cycle) => println!("Found negative cycle with edges: {:?}", cycle),
        None => println!("No negative cycle found"),
    }
}
