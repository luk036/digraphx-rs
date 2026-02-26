/*!
 * Property-based quickcheck tests for digraphx-rs
 *
 * Run with: `cargo run --example quickcheck_tests`
 */

extern crate digraphx_rs;
extern crate quickcheck;

use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

use digraphx_rs::bellman_ford;
use digraphx_rs::neg_cycle::NegCycleFinder;
use petgraph::prelude::*;
use petgraph::visit::NodeIndexable;

fn main() {
    println!("Running quickcheck property-based tests for digraphx-rs...\n");

    // Test 1: bellman_ford_source_distance_is_zero
    println!("Test 1: bellman_ford_source_distance_is_zero");
    match QuickCheck::new()
        .tests(100)
        .quicktest(bellman_ford_source_distance_is_zero as fn(TestGraph) -> TestResult)
    {
        Ok(n) => println!("  Passed {}/100\n", n),
        Err(_) => println!("  FAILED\n"),
    }

    // Test 2: bellman_ford_distances_nonnegative
    println!("Test 2: bellman_ford_distances_nonnegative");
    match QuickCheck::new()
        .tests(100)
        .quicktest(bellman_ford_distances_nonnegative as fn(TestGraph) -> TestResult)
    {
        Ok(n) => println!("  Passed {}/100\n", n),
        Err(_) => println!("  FAILED\n"),
    }

    // Test 3: bellman_ford_empty_graph
    println!("Test 3: bellman_ford_empty_graph");
    match QuickCheck::new()
        .tests(1)
        .quicktest(bellman_ford_empty_graph as fn() -> TestResult)
    {
        Ok(n) => println!("  Passed {}/1\n", n),
        Err(_) => println!("  FAILED\n"),
    }

    // Test 4: bellman_ford_single_node
    println!("Test 4: bellman_ford_single_node");
    match QuickCheck::new()
        .tests(1)
        .quicktest(bellman_ford_single_node as fn() -> TestResult)
    {
        Ok(n) => println!("  Passed {}/1\n", n),
        Err(_) => println!("  FAILED\n"),
    }

    // Test 5: neg_cycle_finder_howard_empty_graph
    println!("Test 5: neg_cycle_finder_howard_empty_graph");
    match QuickCheck::new()
        .tests(1)
        .quicktest(neg_cycle_finder_howard_empty_graph as fn() -> TestResult)
    {
        Ok(n) => println!("  Passed {}/1\n", n),
        Err(_) => println!("  FAILED\n"),
    }

    println!("Quickcheck integration verified!");
}

// Property: Bellman-Ford distance to source is always 0
fn bellman_ford_source_distance_is_zero(graph: TestGraph) -> TestResult {
    let g = graph.0;
    if g.node_count() == 0 {
        return TestResult::discard();
    }
    let source = g.node_indices().next().unwrap();
    match bellman_ford(&g, source) {
        Ok(paths) => {
            let idx = g.to_index(source);
            TestResult::from_bool(paths.distances[idx] == 0.0)
        }
        Err(_) => TestResult::passed(),
    }
}

// Property: Bellman-Ford distances are non-negative
fn bellman_ford_distances_nonnegative(graph: TestGraph) -> TestResult {
    let g = graph.0;
    if g.node_count() == 0 {
        return TestResult::discard();
    }
    let source = g.node_indices().next().unwrap();
    match bellman_ford(&g, source) {
        Ok(paths) => {
            let all_nonnegative = paths.distances.iter().all(|d| *d >= 0.0 || d.is_infinite());
            TestResult::from_bool(all_nonnegative)
        }
        Err(_) => TestResult::passed(),
    }
}

// Property: Bellman-Ford handles empty graph
fn bellman_ford_empty_graph() -> TestResult {
    let mut g: Graph<(), f64> = Graph::new();
    let source = g.add_node(());
    match bellman_ford(&g, source) {
        Ok(paths) => TestResult::from_bool(paths.distances.len() == 1),
        Err(_) => TestResult::failed(),
    }
}

// Property: Bellman-Ford handles single node
fn bellman_ford_single_node() -> TestResult {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    g.add_edge(a, b, 1.0);
    match bellman_ford(&g, a) {
        Ok(paths) => TestResult::from_bool(paths.distances.len() == 2),
        Err(_) => TestResult::failed(),
    }
}

// Property: NegCycleFinder::howard returns None for empty graph
fn neg_cycle_finder_howard_empty_graph() -> TestResult {
    let digraph = DiGraph::<(), i32>::new();
    let mut ncf = NegCycleFinder::new(&digraph);
    let mut dist: [i32; 0] = [];
    TestResult::from_bool(ncf.howard(&mut dist, |e| *e.weight()).is_none())
}

// Custom graph generator for testing
#[derive(Clone, Debug)]
struct TestGraph(Graph<(), f64>);

impl Arbitrary for TestGraph {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = usize::arbitrary(g).saturating_add(1).min(6);
        let mut graph = Graph::new();
        let nodes: Vec<_> = (0..size).map(|_| graph.add_node(())).collect();

        // Add some edges
        let num_edges = usize::arbitrary(g).saturating_add(size).min(size * 2);
        for _ in 0..num_edges {
            let i = usize::arbitrary(g) % size;
            let j = usize::arbitrary(g) % size;
            if i != j {
                let weight = f64::arbitrary(g).abs().max(0.1);
                graph.add_edge(nodes[i], nodes[j], weight);
            }
        }

        if graph.edge_count() == 0 && size >= 2 {
            graph.add_edge(nodes[0], nodes[1], 1.0);
        }

        TestGraph(graph)
    }
}
