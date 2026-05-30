use digraphx_rs::graph_from_edges;
use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
/// Example: minimum cycle ratio using the generic container-of-containers API.
use std::collections::HashMap;

struct MinCycleRatio;

impl ParametricAPI<i32> for MinCycleRatio {
    fn distance(&self, r: &i32, w: &i32) -> i32 {
        *w - *r
    }
    fn zero_cancel(&self, cycle: &[i32]) -> i32 {
        cycle.iter().sum::<i32>() / cycle.len() as i32
    }
}

fn main() {
    // Build a graph using the convenient helper.
    let graph = graph_from_edges(&[
        (0, 1, 5),
        (0, 2, 1),
        (1, 0, 1),
        (1, 2, 1),
        (2, 1, 1),
        (2, 0, 1),
    ]);

    let mut solver = MaxParametricSolver::new(&graph, MinCycleRatio);
    let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
    let mut ratio = 100;
    solver.run(&mut dist, &mut ratio);

    println!("Minimum cycle ratio: {ratio}");
    assert_eq!(ratio, 1);
}
