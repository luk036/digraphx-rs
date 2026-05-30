use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Div;

use crate::Graph;
use crate::NegCycleFinder;
use crate::Zero;

/// Interface for parametric network problems.
///
/// Implementations provide:
/// - `distance(ratio, weight)` — computes the effective edge distance given
///   a parameter `ratio` and the stored edge weight.
/// - `zero_cancel(cycle)` — given a cycle (list of edge weights), computes
///   the ratio value that would make the cycle's total distance zero.
///
/// # Example
///
/// ```rust
/// use digraphx_rs::parametric::ParametricAPI;
///
/// struct MyAPI;
///
/// impl ParametricAPI<i32> for MyAPI {
///     fn distance(&self, ratio: &i32, weight: &i32) -> i32 {
///         *weight - *ratio
///     }
///     fn zero_cancel(&self, cycle: &[i32]) -> i32 {
///         let s: i32 = cycle.iter().sum();
///         s / cycle.len() as i32
///     }
/// }
/// ```
pub trait ParametricAPI<W> {
    /// Compute the effective edge distance given the current ratio.
    fn distance(&self, ratio: &W, weight: &W) -> W;

    /// Compute the ratio that makes the cycle's total distance zero.
    fn zero_cancel(&self, cycle: &[W]) -> W;
}

/// Maximum parametric solver.
///
/// Solves the parametric network problem:
///
/// ```text
/// max  r
/// s.t. dist[v] - dist[u] ≤ distance(edge, r)  ∀ edge(u,v) ∈ G
/// ```
///
/// Generic over any graph type `G` implementing [`Graph`].
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use digraphx_rs::{Graph, graph_from_edges};
/// use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
///
/// struct MinCycle;
///
/// impl ParametricAPI<i32> for MinCycle {
///     fn distance(&self, r: &i32, w: &i32) -> i32 { *w - *r }
///     fn zero_cancel(&self, cycle: &[i32]) -> i32 {
///         cycle.iter().sum::<i32>() / cycle.len() as i32
///     }
/// }
///
/// let graph = graph_from_edges(&[
///     (0, 1, 5i32), (0, 2, 1),
///     (1, 0, 1), (1, 2, 1),
///     (2, 1, 1), (2, 0, 1),
/// ]);
/// let mut solver = MaxParametricSolver::new(&graph, MinCycle);
/// let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
/// let mut ratio = 100i32;
/// solver.run(&mut dist, &mut ratio);
/// assert_eq!(ratio, 1);
/// ```
pub struct MaxParametricSolver<'a, G: Graph, P> {
    ncf: NegCycleFinder<'a, G>,
    omega: P,
}

impl<'a, G, P> MaxParametricSolver<'a, G, P>
where
    G: Graph,
    G::Weight: Add<Output = G::Weight> + PartialOrd + Copy + Div<Output = G::Weight> + Zero,
    G::Node: Copy + Eq + Hash,
    P: ParametricAPI<G::Weight>,
{
    /// Create a new solver.
    pub fn new(graph: &'a G, omega: P) -> Self {
        MaxParametricSolver {
            ncf: NegCycleFinder::new(graph),
            omega,
        }
    }

    /// Run the parametric solver.
    ///
    /// Updates `ratio` in place to the maximum feasible value and returns
    /// the critical cycle (the cycle that determines the optimal ratio).
    /// Returns an empty vector if no cycle was necessary.
    pub fn run(
        &mut self,
        dist: &mut HashMap<G::Node, G::Weight>,
        ratio: &mut G::Weight,
    ) -> Vec<G::Weight> {
        let mut cycle = Vec::new();

        loop {
            // Reset distances to zero
            for d in dist.values_mut() {
                *d = G::Weight::zero();
            }
            // Create fresh finder to reset predecessor map
            let graph = self.ncf.graph();
            self.ncf = NegCycleFinder::new(graph);

            let get_weight = |w: &G::Weight| self.omega.distance(ratio, w);
            if let Some(ci) = self.ncf.howard(dist, get_weight) {
                let ri = self.omega.zero_cancel(&ci);
                if ri < *ratio {
                    cycle = ci;
                    *ratio = ri;
                    continue;
                }
            }
            break;
        }
        cycle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_from_edges;

    struct MinCycleRatio;

    impl ParametricAPI<i32> for MinCycleRatio {
        fn distance(&self, r: &i32, w: &i32) -> i32 {
            *w - *r
        }
        fn zero_cancel(&self, cycle: &[i32]) -> i32 {
            cycle.iter().sum::<i32>() / cycle.len() as i32
        }
    }

    #[test]
    fn test_parametric_simple() {
        let graph = graph_from_edges(&[
            (0, 1, 5i32),
            (0, 2, 1),
            (1, 0, 1),
            (1, 2, 1),
            (2, 1, 1),
            (2, 0, 1),
        ]);
        let mut solver = MaxParametricSolver::new(&graph, MinCycleRatio);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let mut ratio = 100i32;
        solver.run(&mut dist, &mut ratio);
        assert_eq!(ratio, 1);
    }

    #[test]
    fn test_parametric_negative_cycle() {
        // Edge weights: [1, 1, -3] → cycle sum = -1, zero_cancel = -1/3 = 0 (i32)
        let graph = graph_from_edges(&[(0, 1, 1i32), (1, 2, 1), (2, 0, -3)]);
        let mut solver = MaxParametricSolver::new(&graph, MinCycleRatio);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0), (2, 0)].into();
        let mut ratio = 100i32;
        solver.run(&mut dist, &mut ratio);
        // -1/3 truncates to 0 in integer arithmetic
        assert_eq!(ratio, 0);
    }

    #[test]
    fn test_parametric_no_cycle() {
        let graph = graph_from_edges(&[(0, 1, 1i32)]);
        let mut solver = MaxParametricSolver::new(&graph, MinCycleRatio);
        let mut dist: HashMap<i32, i32> = [(0, 0), (1, 0)].into();
        let mut ratio = 100i32;
        solver.run(&mut dist, &mut ratio);
        assert_eq!(ratio, 100);
    }
}
