//! Clock Skew Scheduling (CSS) objectives as one min-ratio engine.
//!
//! Every member of the survey's family — minimum clock period, EVEN, PROP,
//! and C-PROP — is the *same* parametric potential problem with a different
//! `(cost, time)` column. They differ only in the graph that is built, not in
//! the solver:
//!
//! | Problem | cost | time |
//! |---------|------|------|
//! | Min CP  | `-delay` (setup) / `+delay` (hold) | `1` / `0` |
//! | EVEN    | `delay` | `1` / `0` (pass-through) |
//! | PROP    | `cost`  | `time` |
//! | C-PROP  | `mu`    | `sigma` |
//!
//! [`css_ratio`] solves `min_C Σcost/Σtime` and is the single code path shared
//! by every objective; [`min_clock_period`] adds the negated-column transform
//! for the max-ratio case.
//!
//! Because [`crate::Graph`]'s edge weight doubles as the relaxation distance,
//! a CSS edge is a scalar [`CssWeight`] whose `value` field is that distance
//! and whose `cost` / `time` fields hold the raw columns.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Div;

use crate::parametric::ParametricAPI;
use crate::Graph;
use crate::NegCycleFinder;
use crate::Zero;

/// A CSS edge: a scalar `value` (the relaxation distance) carrying the raw
/// `(cost, time)` columns used by the ratio objective.
#[derive(Clone, Copy, Debug)]
pub struct CssWeight {
    /// Scalar consumed by the min-ratio search (meaningless on stored edges).
    pub value: f64,
    /// Cost column of the edge.
    pub cost: f64,
    /// Time column of the edge.
    pub time: f64,
}

impl CssWeight {
    /// Build a stored CSS edge from its raw `(cost, time)` columns.
    #[inline]
    pub fn new(cost: f64, time: f64) -> Self {
        CssWeight {
            value: 0.0,
            cost,
            time,
        }
    }

    /// EVEN column: `(delay, 1)`, or `(delay, 0)` for a pass-through edge.
    #[inline]
    pub fn even(delay: f64, pass_through: bool) -> Self {
        CssWeight::new(delay, if pass_through { 0.0 } else { 1.0 })
    }

    /// Minimum-clock-period setup column: `(-delay, 1)`.
    #[inline]
    pub fn setup(delay: f64) -> Self {
        CssWeight::new(-delay, 1.0)
    }

    /// Minimum-clock-period hold / pass-through column: `(delay, 0)`.
    #[inline]
    pub fn hold(delay: f64) -> Self {
        CssWeight::new(delay, 0.0)
    }

    /// The optimal ratio as a plain `f64`.
    #[inline]
    pub fn ratio(&self) -> f64 {
        self.value
    }
}

impl PartialEq for CssWeight {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for CssWeight {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Zero for CssWeight {
    #[inline]
    fn zero() -> Self {
        CssWeight {
            value: 0.0,
            cost: 0.0,
            time: 0.0,
        }
    }
}

impl Add for CssWeight {
    type Output = CssWeight;
    #[inline]
    fn add(self, other: Self) -> Self {
        CssWeight {
            value: self.value + other.value,
            cost: self.cost + other.cost,
            time: self.time + other.time,
        }
    }
}

impl Div for CssWeight {
    type Output = CssWeight;
    #[inline]
    fn div(self, other: Self) -> Self {
        CssWeight {
            value: self.value / other.value,
            ..self
        }
    }
}

/// Parametric API over `(cost, time)` CSS columns.
struct CssApi;

impl ParametricAPI<CssWeight> for CssApi {
    #[inline]
    fn distance(&self, ratio: &CssWeight, w: &CssWeight) -> CssWeight {
        CssWeight {
            value: w.cost - ratio.value * w.time,
            cost: w.cost,
            time: w.time,
        }
    }

    fn zero_cancel(&self, cycle: &[CssWeight]) -> CssWeight {
        let cost: f64 = cycle.iter().map(|e| e.cost).sum();
        let time: f64 = cycle.iter().map(|e| e.time).sum();
        CssWeight {
            value: if time != 0.0 {
                cost / time
            } else {
                f64::INFINITY
            },
            cost: 0.0,
            time: 0.0,
        }
    }
}

/// Minimum cost-to-time ratio cycle over generic `(cost, time)` columns.
///
/// Returns the critical cycle; `r0` is updated in place to the minimum of
/// `Σcost / Σtime` over all cycles. This is the single code path for every CSS
/// objective — the wrappers below only choose the `(cost, time)` columns.
pub fn css_ratio<G>(
    gra: &G,
    dist: &mut HashMap<G::Node, CssWeight>,
    r0: &mut CssWeight,
) -> Vec<CssWeight>
where
    G: Graph<Weight = CssWeight>,
    G::Node: Copy + Eq + Hash,
{
    let api = CssApi;
    let mut ncf = NegCycleFinder::new(gra);
    let mut ratio = *r0;
    let mut ratio_best = *r0;
    let mut cycle_best = Vec::new();
    let mut cycle_opt = Vec::new();

    loop {
        let current = ratio;
        let get_weight = |w: &CssWeight| api.distance(&current, w);
        for ci in ncf.howard(dist, get_weight) {
            let ri = api.zero_cancel(&ci);
            if ratio_best > ri {
                ratio_best = ri;
                cycle_best = ci;
            }
        }
        if ratio_best >= ratio {
            break;
        }
        cycle_opt = std::mem::take(&mut cycle_best);
        ratio = ratio_best;
    }

    *r0 = ratio;
    cycle_opt
}

/// Minimum feasible clock period (the *maximum* cycle ratio).
///
/// Edges are built with [`CssWeight::setup`] / [`CssWeight::hold`]. The
/// smallest feasible period is the maximum cycle ratio, so it is computed as
/// the negated minimum ratio of the transformed columns.
pub fn min_clock_period<G>(
    gra: &G,
    dist: &mut HashMap<G::Node, CssWeight>,
    tcp: &mut CssWeight,
) -> Vec<CssWeight>
where
    G: Graph<Weight = CssWeight>,
    G::Node: Copy + Eq + Hash,
{
    let cycle = css_ratio(gra, dist, tcp);
    tcp.value = -tcp.value;
    cycle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_from_edges;

    fn run<G>(gra: &G, edges: usize, ratio: f64) -> f64
    where
        G: Graph<Weight = CssWeight>,
        G::Node: Copy + Eq + Hash,
    {
        let mut dist: HashMap<G::Node, CssWeight> = HashMap::new();
        for node in gra.nodes() {
            dist.insert(node, CssWeight::zero());
        }
        let _ = edges;
        let mut r0 = CssWeight {
            value: ratio,
            cost: 0.0,
            time: 0.0,
        };
        css_ratio(gra, &mut dist, &mut r0);
        r0.ratio()
    }

    #[test]
    fn test_min_clock_period() {
        let gra = graph_from_edges(&[
            (0u32, 1u32, CssWeight::setup(7.0)),
            (0, 2, CssWeight::hold(2.0)),
            (1, 0, CssWeight::hold(4.0)),
            (1, 2, CssWeight::hold(0.0)),
            (2, 0, CssWeight::setup(3.0)),
        ]);
        let mut dist: HashMap<u32, CssWeight> = HashMap::new();
        for node in gra.nodes() {
            dist.insert(node, CssWeight::zero());
        }
        let mut tcp = CssWeight {
            value: 1.0e6,
            cost: 0.0,
            time: 0.0,
        };
        let cycle = min_clock_period(&gra, &mut dist, &mut tcp);
        assert!(!cycle.is_empty());
        assert!((tcp.ratio() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_min_clock_period_alt() {
        let gra = graph_from_edges(&[
            (0u32, 1u32, CssWeight::setup(2.0)),
            (0, 2, CssWeight::hold(1.5)),
            (1, 2, CssWeight::setup(3.0)),
            (1, 0, CssWeight::hold(2.0)),
            (2, 0, CssWeight::setup(4.0)),
            (2, 1, CssWeight::hold(3.0)),
        ]);
        let mut dist: HashMap<u32, CssWeight> = HashMap::new();
        for node in gra.nodes() {
            dist.insert(node, CssWeight::zero());
        }
        let mut tcp = CssWeight {
            value: 1.0e6,
            cost: 0.0,
            time: 0.0,
        };
        min_clock_period(&gra, &mut dist, &mut tcp);
        assert!((tcp.ratio() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_even() {
        let tcp = 6.5;
        let gra = graph_from_edges(&[
            (0u32, 1u32, CssWeight::even(tcp - 7.0, false)),
            (0, 2, CssWeight::even(2.0, false)),
            (1, 0, CssWeight::even(4.0, false)),
            (1, 2, CssWeight::even(0.0, true)),
            (2, 0, CssWeight::even(tcp - 3.0, false)),
        ]);
        let beta = run(&gra, 0, 1.0e6);
        assert!((beta - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_even_multi_node() {
        let tcp = 7.5;
        let gra = graph_from_edges(&[
            (0u32, 2u32, CssWeight::even(6.0, false)),
            (0, 3, CssWeight::even(6.0, false)),
            (1, 2, CssWeight::even(6.0, false)),
            (1, 5, CssWeight::even(0.0, true)),
            (2, 0, CssWeight::even(tcp - 7.0, false)),
            (2, 1, CssWeight::even(tcp - 9.0, false)),
            (2, 3, CssWeight::even(6.0, false)),
            (3, 4, CssWeight::even(8.0, false)),
            (3, 2, CssWeight::even(tcp - 6.0, false)),
            (3, 0, CssWeight::even(tcp - 6.0, false)),
            (4, 3, CssWeight::even(tcp - 8.0, false)),
            (4, 5, CssWeight::even(3.0, false)),
            (5, 4, CssWeight::even(tcp - 3.0, false)),
        ]);
        let beta = run(&gra, 0, 1.0e6);
        assert!((beta - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_even_raw_weights() {
        let tcp = 7.5;
        let gra = graph_from_edges(&[
            (0u32, 2u32, CssWeight::even(6.0, false)),
            (0, 3, CssWeight::even(6.0, false)),
            (1, 2, CssWeight::even(6.0, false)),
            (1, 4, CssWeight::even(tcp - 3.0, false)),
            (2, 0, CssWeight::even(tcp - 7.0, false)),
            (2, 1, CssWeight::even(tcp - 9.0, false)),
            (2, 3, CssWeight::even(6.0, false)),
            (3, 4, CssWeight::even(8.0, false)),
            (3, 2, CssWeight::even(tcp - 6.0, false)),
            (3, 0, CssWeight::even(tcp - 6.0, false)),
            (4, 3, CssWeight::even(tcp - 8.0, false)),
            (4, 1, CssWeight::even(3.0, false)),
        ]);
        let beta = run(&gra, 0, 1.0e6);
        assert!((beta - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_prop() {
        let tcp = 7.5;
        let gra = graph_from_edges(&[
            (0u32, 3u32, CssWeight::new(6.0, 2.5)),
            (0, 2, CssWeight::new(6.0, 3.1)),
            (1, 2, CssWeight::new(6.0, 2.5)),
            (1, 4, CssWeight::new(tcp - 3.0, 1.1)),
            (2, 0, CssWeight::new(tcp - 7.0, 1.5)),
            (2, 1, CssWeight::new(tcp - 9.0, 4.1)),
            (2, 3, CssWeight::new(6.0, 2.5)),
            (3, 0, CssWeight::new(tcp - 6.0, 3.1)),
            (3, 2, CssWeight::new(tcp - 6.0, 3.1)),
            (3, 4, CssWeight::new(8.0, 1.5)),
            (4, 1, CssWeight::new(3.0, 1.0)),
            (4, 3, CssWeight::new(tcp - 8.0, 4.1)),
        ]);
        let beta = run(&gra, 0, 1.0e6);
        assert!((beta - 0.32258064516129037).abs() < 1e-9, "beta = {beta}");
    }

    #[test]
    fn test_prop_alt() {
        let tcp = 4.5;
        let gra = graph_from_edges(&[
            (0u32, 1u32, CssWeight::new(tcp - 2.0, 3.1)),
            (0, 2, CssWeight::new(1.5, 0.7)),
            (1, 2, CssWeight::new(tcp - 3.0, 4.1)),
            (1, 0, CssWeight::new(2.0, 2.2)),
            (2, 0, CssWeight::new(tcp - 4.0, 5.1)),
            (2, 1, CssWeight::new(3.0, 1.5)),
        ]);
        let beta = run(&gra, 0, 1.0e6);
        assert!((beta - 0.3448275862069039).abs() < 1e-9);
    }

    #[test]
    fn test_cprop() {
        let gra = graph_from_edges(&[
            (0u32, 1u32, CssWeight::new(4.0, 1.0)),
            (1, 0, CssWeight::new(2.0, 1.0)),
        ]);
        let beta = run(&gra, 0, 1.0e6);
        assert!((beta - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_min_clock_period_ratio_graph() {
        let gra = graph_from_edges(&[
            (0u32, 3u32, CssWeight::setup(6.0)),
            (0, 2, CssWeight::setup(7.0)),
            (1, 2, CssWeight::setup(9.0)),
            (1, 4, CssWeight::hold(3.0)),
            (2, 0, CssWeight::hold(6.0)),
            (2, 1, CssWeight::hold(6.0)),
            (2, 3, CssWeight::setup(6.0)),
            (3, 4, CssWeight::setup(8.0)),
            (3, 0, CssWeight::hold(6.0)),
            (3, 2, CssWeight::hold(6.0)),
            (4, 1, CssWeight::setup(3.0)),
            (4, 3, CssWeight::hold(8.0)),
        ]);
        let mut dist: HashMap<u32, CssWeight> = HashMap::new();
        for node in gra.nodes() {
            dist.insert(node, CssWeight::zero());
        }
        let mut tcp = CssWeight {
            value: 1.0e6,
            cost: 0.0,
            time: 0.0,
        };
        min_clock_period(&gra, &mut dist, &mut tcp);
        assert!((tcp.ratio() - 6.5).abs() < 1e-9);
    }
}
