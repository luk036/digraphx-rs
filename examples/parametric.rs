//! Parametric algorithm example

use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
use num::rational::Ratio;
use petgraph::graph::{DiGraph, EdgeReference};

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

fn main() {
    // Create a directed graph with weights
    let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
        (0, 1, Ratio::new(1, 1)),
        (1, 2, Ratio::new(1, 1)),
        (2, 0, Ratio::new(1, 1)),
    ]);

    // Create solver with custom ratio implementation
    let mut solver = MaxParametricSolver::new(&digraph, MyRatio {});

    // Initialize distances and ratio
    let mut dist = [Ratio::new(0, 1), Ratio::new(0, 1), Ratio::new(0, 1)];
    let mut ratio = Ratio::new(1_000_000, 1); // Start with large ratio

    // Run the parametric solver
    let cycle = solver.run(&mut dist, &mut ratio);

    println!("Minimum ratio found: {}", ratio);
    println!("Cycle: {:?}", cycle);

    // Example 2: Negative cycle
    println!("\n=== Example with negative cycle ===");
    let digraph_neg = DiGraph::<(), Ratio<i32>>::from_edges([
        (0, 1, Ratio::new(1, 1)),
        (1, 2, Ratio::new(-5, 1)),
        (2, 0, Ratio::new(1, 1)),
    ]);

    let mut solver2 = MaxParametricSolver::new(&digraph_neg, MyRatio {});
    let mut dist2 = [Ratio::new(0, 1), Ratio::new(0, 1), Ratio::new(0, 1)];
    let mut ratio2 = Ratio::new(1_000_000, 1);

    let cycle2 = solver2.run(&mut dist2, &mut ratio2);

    println!("Minimum ratio found: {}", ratio2);
    println!("Cycle: {:?}", cycle2);
}
