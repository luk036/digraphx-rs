use crate::parametric::{ParametricAPI, MaxParametricSolver};

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Div, Sub};
use std::cmp::PartialOrd;
use std::convert::From;
use std::fmt::Debug;
use std::iter::Sum;
use std::marker::Copy;
use std::num::ParseFloatError;
use std::str::FromStr;

use petgraph::graph::{DiGraph, EdgeReference};
use petgraph::prelude::*;
use petgraph::visit::EdgeRef;
use petgraph::algo::FloatMeasure;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct V(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct R(f64);

impl Sub for R {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        R(self.0 - other.0)
    }
}

impl Div for R {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        R(self.0 / other.0)
    }
}

impl From<f64> for R {
    fn from(f: f64) -> Self {
        R(f)
    }
}

impl FromStr for R {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>()?;
        Ok(R(f))
    }
}

// impl FloatMeasure for R {
//     fn is_negative(&self) -> bool {
//         self.0 < 0.0
//     }

//     fn is_positive(&self) -> bool {
//         self.0 > 0.0
//     }

//     fn is_zero(&self) -> bool {
//         self.0 == 0.0
//     }

//     fn min_positive_value() -> Self {
//         R(std::f64::EPSILON)
//     }

//     fn max_value() -> Self {
//         R(std::f64::MAX)
//     }

//     fn min_value() -> Self {
//         R(std::f64::MIN)
//     }

//     fn nan() -> Self {
//         R(std::f64::NAN)
//     }

//     fn infinity() -> Self {
//         R(std::f64::INFINITY)
//     }

//     fn neg_infinity() -> Self {
//         R(std::f64::NEG_INFINITY)
//     }

//     fn epsilon() -> Self {
//         R(std::f64::EPSILON)
//     }

//     fn abs_diff_eq(&self, other: &Self, max_diff: Self) -> bool {
//         (self.0 - other.0).abs() <= max_diff.0
//     }
// }

fn set_default<D: Copy + Debug + PartialOrd + Sub<Output=D> + Div<Output=D> + From<f64> + FloatMeasure>(
    grph: &mut DiGraph<V, D>, weight: &str, value: D
) {
    for u in grph.node_indices() {
        for e in grph.edges(u) {
            if grph.edge_weight_mut(e).unwrap().get(weight).is_none() {
                grph.edge_weight_mut(e).unwrap().insert(weight.to_string(), value);
            }
        }
    }
}

struct CycleRatio<'a, D: 'a + Copy + Debug + PartialOrd + Sub<Output=D> + Div<Output=D> + From<f64>> {
    grph: &'a DiGraph<V, HashMap<String, D>>,
}

impl<'a, D: 'a + Copy + Debug + PartialOrd + Sub<Output=D> + Div<Output=D> + From<f64>> ParametricAPI<HashMap<String, D>, R> for CycleRatio<'a, R> {
    fn distance(&self, ratio: &R, e: &EdgeReference<HashMap<String, D>>) -> R {
        let cost = *e.weight().get("cost").unwrap();
        let time = *e.weight().get("time").unwrap();
        cost - ratio * time
    }

    fn zero_cancel(&self, cycle: Vec<EdgeIndex<V>>) -> R {
        let total_cost = cycle.iter().map(|e| *self.grph.edge_weight(*e).unwrap().get("cost").unwrap()).sum::<D>();
        let total_time = cycle.iter().map(|e| *self.grph.edge_weight(*e).unwrap().get("time").unwrap()).sum::<D>();
        total_cost / total_time
    }
}

/**
 * @brief Minimum Cycle Ratio Solver
 *
 * The minimum cycle ratio (MCR) problem is a fundamental problem in the
 * analysis of directed graphs. Given a directed graph, the MCR problem seeks to
 * find the cycle with the minimum ratio of the sum of edge weights to the
 * number of edges in the cycle. In other words, the MCR problem seeks to find
 * the "tightest" cycle in the graph, where the tightness of a cycle is measured
 * by the ratio of the total weight of the cycle to its length.
 *
 * The MCR problem has many applications in the analysis of discrete event
 * systems, such as digital circuits and communication networks. It is closely
 * related to other problems in graph theory, such as the shortest path problem
 * and the maximum flow problem. Efficient algorithms for solving the MCR
 * problem are therefore of great practical importance.
 *
 * @tparam DiGraph
 * @tparam Ratio
 */
struct MinCycleRatioSolver<'a, D: 'a + Copy + Debug + PartialOrd + Sub<Output=D> + Div<Output=D> + From<f64> + FloatMeasure> {
    grph: &'a DiGraph<V, HashMap<String, D>>,
}

impl<'a, D: 'a + Copy + Debug + PartialOrd + Sub<Output=D> + Div<Output=D> + From<f64> + FloatMeasure> MinCycleRatioSolver<'a, D> {
    fn run(&self, dist: &mut HashMap<V, R>, r0: R) -> (R, Vec<EdgeIndex<V>>) {
        let omega = CycleRatio { grph: self.grph };
        let mut solver = MaxParametricSolver::new(self.grph, omega);
        let (ratio, cycle) = solver.run(dist, r0);
        (ratio, cycle)
    }
}

fn main() {
    let mut grph = DiGraph::<V, HashMap<String, R>>::new();
    let a = grph.add_node(V(0));
    let b = grph.add_node(V(1));
    let c = grph.add_node(V(2));
    grph.add_edge(a, b, HashMap::new());
    grph.add_edge(b, c, HashMap::new());
    grph.add_edge(c, a, HashMap::new());
    set_default(&mut grph, "cost", R::from(1.0));
    set_default(&mut grph, "time", R::from(1.0));
    let mut dist = HashMap::new();
    dist.insert(V(0), R::from(0.0));
    dist.insert(V(1), R::from(1.0));
    dist.insert(V(2), R::from(2.0));
    let solver = MinCycleRatioSolver { grph: &grph };
    let (ratio, cycle) = solver.run(&mut dist, R::from(0.0));
    // println!("{:?} {:?}", ratio, cycle);
}

