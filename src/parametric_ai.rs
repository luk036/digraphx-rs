use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Div;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Neg;

use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use petgraph::algo::NegativeCycle;

use num::traits::Float;
use num::traits::Zero;
use num::traits::One;
use num::rational::Ratio;

type Cycle<'a, V> = Vec<(&'a V, &'a V)>;

trait ParametricAPI<V> {
    fn distance<R: Float>(&self, ratio: Ratio<R>, edge: (&V, &V)) -> Ratio<R>;
    fn zero_cancel<R: Float>(&self, cycle: Cycle<V>) -> Ratio<R>;
}

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
    fn get_weight(&self, edge: &petgraph::graph::EdgeReference<W>) -> Ratio<P> {
        let (u, v) = self.grph.edge_endpoints(edge.id()).unwrap();
        self.omega.distance(self.ratio, (self.grph.node_weight(u).unwrap(), self.grph.node_weight(v).unwrap()))
    }

    fn find_neg_cycle(&self) -> Option<Cycle<'a, V>> {
        let mut cycle = None;
        let mut dist = self.dist.clone();
        let mut neg_cycle = NegativeCycle::new(&self.grph, Some(&mut dist));
        if let Some(edge) = neg_cycle.next_edge() {
            cycle = Some(neg_cycle.find_cycle(edge));
        }
        cycle
    }

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

