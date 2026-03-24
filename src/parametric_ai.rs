use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use petgraph::algo::NegativeCycle;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;

use num::rational::Ratio;
use num::traits::Float;
use num::traits::One;
use num::traits::Zero;

/// A cycle represented as a list of vertex pairs.
type Cycle<'a, V> = Vec<(&'a V, &'a V)>;

/// Trait for parametric cycle ratio calculations.
///
/// Provides methods to compute the distance (cost - ratio * time) for edges
/// and find zero-canceling ratios for cycles.
trait ParametricAPI<V> {
    /// Compute the parametric distance for an edge.
    fn distance<R: Float>(&self, ratio: Ratio<R>, edge: (&V, &V)) -> Ratio<R>;
    /// Find the ratio where the cycle cost equals zero.
    fn zero_cancel<R: Float>(&self, cycle: Cycle<V>) -> Ratio<R>;
}

/// Parametric cycle ratio solver using a directed graph.
///
/// Solves the maximum cycle ratio problem using parametric search.
struct MaxParametric<'a, V, W, P> {
    grph: &'a DiGraph<V, W>,
    ratio: Ratio<P>,
    omega: &'a dyn ParametricAPI<V>,
    dist: HashMap<V, Ratio<P>>,
}

impl<'a, V, W, P> MaxParametric<'a, V, W, P>
where
    V: Eq + Hash,
    W: Add<Output = W> + Sub<Output = W> + PartialOrd + Copy,
    P: Float + Zero + One + PartialOrd + Copy,
{
    /// Get the parametric weight for an edge.
    fn get_weight(&self, edge: &petgraph::graph::EdgeReference<W>) -> Ratio<P> {
        let (u, v) = self.grph.edge_endpoints(edge.id()).unwrap();
        self.omega.distance(
            self.ratio,
            (
                self.grph.node_weight(u).unwrap(),
                self.grph.node_weight(v).unwrap(),
            ),
        )
    }

    /// Find a negative cycle in the graph.
    fn find_neg_cycle(&self) -> Option<Cycle<'a, V>> {
        let mut cycle = None;
        let mut dist = self.dist.clone();
        let mut neg_cycle = NegativeCycle::new(&self.grph, Some(&mut dist));
        if let Some(edge) = neg_cycle.next_edge() {
            cycle = Some(neg_cycle.find_cycle(edge));
        }
        cycle
    }

    /// Run the parametric solver to find the minimum ratio cycle.
    fn run(&mut self) -> (Ratio<P>, Cycle<'a, V>) {
        let mut r_min = self.ratio;
        let mut c_min = vec![];
        let mut cycle = vec![];
        loop {
            if let Some(ci) = self.find_neg_cycle() {
                let ri = self.omega.zero_cancel(ci.clone());
                if r_min > ri {
                    r_min = ri;
                    c_min = ci;
                }
            } else {
                break;
            }
            if r_min >= self.ratio {
                break;
            }
            cycle = c_min.clone();
            self.ratio = r_min;
        }
        (self.ratio, cycle)
    }
}
