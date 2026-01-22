//! Integration tests for digraphx-rs

use digraphx_rs::{
    bellman_ford, find_negative_cycle,
    neg_cycle::NegCycleFinder,
    parametric::{MaxParametricSolver, ParametricAPI},
};
use num::rational::Ratio;
use petgraph::{
    graph::{DiGraph, EdgeReference},
    Graph,
};

#[test]
fn test_bellman_ford_integration() {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.extend_with_edges([(a, b, 4.0), (b, c, 3.0), (a, c, 10.0)]);

    let paths = bellman_ford(&g, a).unwrap();
    assert_eq!(paths.distances, vec![0.0, 4.0, 7.0]);
    assert_eq!(paths.predecessors, vec![None, Some(a), Some(b)]);
}

#[test]
fn test_negative_cycle_detection_integration() {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.extend_with_edges([(a, b, 1.0), (b, c, 1.0), (c, a, -3.0)]);

    let cycle = find_negative_cycle(&g, a).unwrap();
    assert_eq!(cycle.len(), 3);
}

#[test]
fn test_no_negative_cycle_integration() {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.extend_with_edges([(a, b, 1.0), (b, c, 1.0), (c, a, 1.0)]);

    let cycle = find_negative_cycle(&g, a);
    assert!(cycle.is_none());
}

#[test]
fn test_disconnected_graph_integration() {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, 1.0);

    let paths = bellman_ford(&g, a).unwrap();
    assert_eq!(paths.distances[a.index()], 0.0);
    assert_eq!(paths.distances[b.index()], 1.0);
    assert_eq!(paths.distances[c.index()], f32::INFINITY);
}

#[test]
fn test_neg_cycle_finder_howard() {
    let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
        (0, 1, Ratio::new(1, 1)),
        (0, 2, Ratio::new(1, 1)),
        (0, 3, Ratio::new(1, 1)),
        (1, 3, Ratio::new(1, 1)),
        (2, 1, Ratio::new(1, 1)),
        (3, 2, Ratio::new(-3, 1)),
    ]);

    let mut ncf = NegCycleFinder::new(&digraph);
    let mut dist = [
        Ratio::new(0, 1),
        Ratio::new(0, 1),
        Ratio::new(0, 1),
        Ratio::new(0, 1),
    ];
    let result = ncf.howard(&mut dist, |e| *e.weight());

    assert!(result.is_some());
}

#[test]
fn test_parametric_solver_integration() {
    #[derive(Debug)]
    struct MyRatio {}

    impl<V> ParametricAPI<V, Ratio<i32>> for MyRatio
    where
        V: Eq + Clone,
    {
        fn distance(&self, ratio: &Ratio<i32>, edge: &EdgeReference<Ratio<i32>>) -> Ratio<i32> {
            *edge.weight() - *ratio
        }

        fn zero_cancel(&self, cycle: &[EdgeReference<Ratio<i32>>]) -> Ratio<i32> {
            let mut total_weight = Ratio::new(0, 1);
            for edge in cycle {
                total_weight += *edge.weight();
            }
            total_weight / Ratio::from_integer(cycle.len() as i32)
        }
    }

    let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
        (0, 1, Ratio::new(1, 1)),
        (1, 2, Ratio::new(1, 1)),
        (2, 0, Ratio::new(1, 1)),
    ]);

    let mut solver = MaxParametricSolver::new(&digraph, MyRatio {});
    let mut dist = [Ratio::new(0, 1), Ratio::new(0, 1), Ratio::new(0, 1)];
    let mut ratio = Ratio::new(1_000_000, 1);

    let _cycle = solver.run(&mut dist, &mut ratio);

    assert_eq!(ratio, Ratio::new(1, 1));
}

#[test]
fn test_complex_graph_with_multiple_components() {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());
    let d = g.add_node(());

    g.extend_with_edges([(a, b, 1.0), (b, c, 2.0), (c, d, 3.0)]);

    let paths = bellman_ford(&g, a).unwrap();
    assert_eq!(paths.distances, vec![0.0, 1.0, 3.0, 6.0]);
}
