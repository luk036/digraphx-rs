//! Integration tests for digraphx-rs using generic graph containers.

use digraphx_rs::graph_from_edges;
use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
use digraphx_rs::NegCycleFinder;
use std::collections::HashMap;

#[test]
fn test_neg_cycle_hashmap_string() {
    let mut graph: HashMap<&str, HashMap<&str, i32>> = HashMap::new();
    graph.insert("a", [("b", 1), ("c", 1)].into());
    graph.insert("b", [("c", 1)].into());
    graph.insert("c", [("a", -3)].into());

    let mut ncf = NegCycleFinder::new(&graph);
    let mut dist: HashMap<&str, i32> = [("a", 0), ("b", 0), ("c", 0)].into();
    let result = ncf.howard(&mut dist, |w| *w);
    assert!(result.is_some());
}

#[test]
fn test_neg_cycle_graph_from_edges() {
    let graph = graph_from_edges(&[(0u32, 1u32, 1i32), (1, 2, 1), (2, 0, -3)]);
    let mut ncf = NegCycleFinder::new(&graph);
    let mut dist: HashMap<u32, i32> = [(0, 0), (1, 0), (2, 0)].into();
    let result = ncf.howard(&mut dist, |w| *w);
    assert!(result.is_some());
}

#[test]
fn test_no_neg_cycle_integration() {
    let graph = graph_from_edges(&[(0u32, 1u32, 1i32), (1, 2, 1), (2, 0, 1)]);
    let mut ncf = NegCycleFinder::new(&graph);
    let mut dist: HashMap<u32, i32> = [(0, 0), (1, 0), (2, 0)].into();
    let result = ncf.howard(&mut dist, |w| *w);
    assert!(result.is_none());
}

#[test]
fn test_parametric_solver_integration() {
    struct MinCycle;

    impl ParametricAPI<i32> for MinCycle {
        fn distance(&self, r: &i32, w: &i32) -> i32 {
            *w - *r
        }
        fn zero_cancel(&self, cycle: &[i32]) -> i32 {
            cycle.iter().sum::<i32>() / cycle.len() as i32
        }
    }

    let graph = graph_from_edges(&[
        (0i32, 1i32, 5i32),
        (0, 2, 1),
        (1, 0, 1),
        (1, 2, 1),
        (2, 1, 1),
        (2, 0, 1),
    ]);
    let mut solver = MaxParametricSolver::new(&graph, MinCycle);
    let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
    let mut ratio = 100i32;
    solver.run(&mut dist, &mut ratio);
    assert_eq!(ratio, 1);
}

#[test]
fn test_disconnected_graph() {
    let mut graph: HashMap<i32, HashMap<i32, f64>> = HashMap::new();
    graph.insert(0, [(1, 1.0)].into());
    graph.insert(1, HashMap::new());
    graph.insert(2, HashMap::new());

    let mut ncf = NegCycleFinder::new(&graph);
    let mut dist: HashMap<i32, f64> = [(0, 0.0), (1, 0.0), (2, 0.0)].into();
    let result = ncf.howard(&mut dist, |w| *w);
    assert!(result.is_none());
}

#[test]
fn test_complex_graph_multiple_components() {
    let mut graph: HashMap<i32, HashMap<i32, f64>> = HashMap::new();
    graph.insert(0, [(1, 1.0)].into());
    graph.insert(1, [(2, 2.0)].into());
    graph.insert(2, [(3, 3.0)].into());
    graph.insert(3, HashMap::new());

    let mut ncf = NegCycleFinder::new(&graph);
    let mut dist: HashMap<i32, f64> = [(0, 0.0), (1, 0.0), (2, 0.0), (3, 0.0)].into();
    let result = ncf.howard(&mut dist, |w| *w);
    assert!(result.is_none());
}
