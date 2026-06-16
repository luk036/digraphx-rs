//! Benchmark suite for digraphx-rs using criterion
//!
//! All benchmarks use the container-of-containers graph API
//! (`HashMap<N, HashMap<N, W>>`) — no petgraph dependency.

use std::collections::HashMap;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use digraphx_rs::graph_from_edges;
use digraphx_rs::neg_cycle::NegCycleFinder;
use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
use num::rational::Ratio;

// ---------------------------------------------------------------------------
// Negative cycle detection benchmarks
// ---------------------------------------------------------------------------

fn bench_neg_cycle_small(c: &mut Criterion) {
    let graph = graph_from_edges(&[(0, 1, 1.0f64), (1, 2, 1.0), (2, 0, -3.0)]);

    c.bench_function("neg_cycle_small", |b| {
        b.iter(|| {
            let mut ncf = NegCycleFinder::new(black_box(&graph));
            let mut dist: HashMap<i32, f64> = [(0, 0.0), (1, 0.0), (2, 0.0)].into();
            ncf.howard(&mut dist, |w| *w)
        })
    });
}

fn bench_neg_cycle_no_cycle(c: &mut Criterion) {
    let graph = graph_from_edges(&[(0, 1, 4.0f64), (1, 2, 3.0), (0, 2, 10.0)]);

    c.bench_function("neg_cycle_no_cycle", |b| {
        b.iter(|| {
            let mut ncf = NegCycleFinder::new(black_box(&graph));
            let mut dist: HashMap<i32, f64> = HashMap::new();
            ncf.howard(&mut dist, |w| *w)
        })
    });
}

fn bench_neg_cycle_medium(c: &mut Criterion) {
    let mut edges = Vec::new();
    for i in 0..99 {
        edges.push((i, i + 1, 1.0f64));
        edges.push((i + 1, i, 2.0f64));
    }
    let graph = graph_from_edges(&edges);

    c.bench_function("neg_cycle_medium", |b| {
        b.iter(|| {
            let mut ncf = NegCycleFinder::new(black_box(&graph));
            let mut dist: HashMap<i32, f64> = (0..100).map(|i| (i, 0.0)).collect();
            ncf.howard(&mut dist, |w| *w)
        })
    });
}

// ---------------------------------------------------------------------------
// Howard's algorithm with Ratio weights
// ---------------------------------------------------------------------------

fn bench_howard_ratio(c: &mut Criterion) {
    let mut edges: Vec<(u32, u32, Ratio<i32>)> = Vec::new();
    for i in 0..100u32 {
        for j in 0..100u32 {
            let w = (i + j) % 10;
            edges.push((i, j, Ratio::new(w as i32, 1)));
        }
    }
    edges.truncate(500);
    let graph = graph_from_edges(&edges);
    let base_dist: HashMap<u32, Ratio<i32>> = (0..100).map(|i| (i, Ratio::new(0, 1))).collect();

    c.bench_function("howard_ratio", |b| {
        b.iter(|| {
            let mut ncf = NegCycleFinder::new(black_box(&graph));
            let mut dist = base_dist.clone();
            ncf.howard(&mut dist, |w| *w)
        })
    });
}

// ---------------------------------------------------------------------------
// Parametric solver benchmarks
// ---------------------------------------------------------------------------

struct MyRatio;

impl ParametricAPI<Ratio<i32>> for MyRatio {
    fn distance(&self, ratio: &Ratio<i32>, weight: &Ratio<i32>) -> Ratio<i32> {
        *weight - *ratio
    }

    fn zero_cancel(&self, cycle: &[Ratio<i32>]) -> Ratio<i32> {
        let mut total = Ratio::new(0, 1);
        for w in cycle {
            total += *w;
        }
        total / Ratio::from_integer(cycle.len() as i32)
    }
}

fn bench_parametric_solver(c: &mut Criterion) {
    let graph = graph_from_edges(&[
        (0u32, 1, Ratio::new(1, 1)),
        (1, 2, Ratio::new(1, 1)),
        (2, 0, Ratio::new(1, 1)),
        (2, 3, Ratio::new(2, 1)),
        (3, 4, Ratio::new(3, 1)),
        (4, 5, Ratio::new(1, 1)),
        (5, 6, Ratio::new(1, 1)),
        (6, 0, Ratio::new(2, 1)),
    ]);

    c.bench_function("parametric_solver", |b| {
        b.iter(|| {
            let mut solver = MaxParametricSolver::new(black_box(&graph), MyRatio);
            let mut dist: HashMap<u32, Ratio<i32>> = [
                (0, Ratio::new(0, 1)),
                (1, Ratio::new(0, 1)),
                (2, Ratio::new(0, 1)),
                (3, Ratio::new(0, 1)),
                (4, Ratio::new(0, 1)),
                (5, Ratio::new(0, 1)),
                (6, Ratio::new(0, 1)),
            ]
            .into();
            let mut ratio = Ratio::new(1_000_000, 1);
            solver.run(&mut dist, &mut ratio)
        })
    });
}

criterion_group!(
    benches,
    bench_neg_cycle_small,
    bench_neg_cycle_no_cycle,
    bench_neg_cycle_medium,
    bench_howard_ratio,
    bench_parametric_solver
);
criterion_main!(benches);
