use std::collections::HashMap;
use std::cmp::PartialOrd;
use std::hash::Hash;

trait ParametricAPI<Node, Edge, Ratio> {
    fn distance(&self, ratio: Ratio, edge: &Edge) -> Ratio;
    fn zero_cancel(&self, cycle: &Cycle) -> Ratio;
}

struct MaxParametricSolver<Node, Edge, Ratio, D, Cycle> {
    ncf: NegCycleFinder,
    omega: ParametricAPI<Node, Edge, Ratio>,
}

impl<Node, Edge, Ratio, D, Cycle> MaxParametricSolver<Node, Edge, Ratio, D, Cycle> {
    fn run(&self, dist: &mut HashMap<Node, D>, ratio: Ratio) -> (Ratio, Cycle) {
        // Implementation here
    }
}

fn set_default(digraph: &mut GraphMut, weight: &str, value: Domain) {
    // Implementation here
}

struct CycleRatioAPI<Node, Edge, Ratio, D> {
    digraph: HashMap<Node, HashMap<Node, HashMap<String, D>>>,
    result_type: D,
}

impl<Node, Edge, Ratio, D> CycleRatioAPI<Node, Edge, Ratio, D> {
    fn distance(&self, ratio: Ratio, edge: &mut HashMap<String, D>) -> Ratio {
        // Implementation here
    }

    fn zero_cancel(&self, cycle: &Cycle) -> Ratio {
        // Implementation here
    }
}

struct MinCycleRatioSolver<Node, Edge, Ratio> {
    digraph: Graph,
}

impl<Node, Edge, Ratio> MinCycleRatioSolver<Node, Edge, Ratio> {
    fn run(&self, dist: &mut HashMap<Node, Domain>, r0: Ratio) -> (Ratio, Cycle) {
        // Implementation here
    }
}

fn test_cycle_ratio_raw() {
    // Test implementation here
}

fn test_cycle_ratio() {
    // Test implementation here
}

fn test_cycle_ratio_timing() {
    // Test implementation here
}

fn test_cycle_ratio_tiny_graph() {
    // Test implementation here
}
