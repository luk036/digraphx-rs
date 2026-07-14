//! Benchmark suite for digraphx-rs using criterion
//!
//! All benchmarks use the container-of-containers graph API
//! (`HashMap<N, HashMap<N, W>>`) — no petgraph dependency.

use std::collections::HashMap;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use digraphx_rs::graph_from_edges;
use digraphx_rs::mcf::{cycle_canceling_mcf, Edge};
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
            black_box(
                ncf.howard(&mut dist, |w| *w)
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
        })
    });
}

fn bench_neg_cycle_no_cycle(c: &mut Criterion) {
    let graph = graph_from_edges(&[(0, 1, 4.0f64), (1, 2, 3.0), (0, 2, 10.0)]);

    c.bench_function("neg_cycle_no_cycle", |b| {
        b.iter(|| {
            let mut ncf = NegCycleFinder::new(black_box(&graph));
            let mut dist: HashMap<i32, f64> = HashMap::new();
            black_box(
                ncf.howard(&mut dist, |w| *w)
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
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
            black_box(
                ncf.howard(&mut dist, |w| *w)
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
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
            black_box(
                ncf.howard(&mut dist, |w| *w)
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
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

// ---------------------------------------------------------------------------
// Min-cost flow (cycle-cancelling MCF) benchmarks
// ---------------------------------------------------------------------------

/// Build a chain graph: 0 → 1 → 2 → … → (n-1)
fn build_chain(n: usize) -> HashMap<usize, HashMap<usize, Edge>> {
    let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
    for i in 0..n - 1 {
        g.entry(i).or_default().insert(i + 1, Edge::new(1, 100));
    }
    g.entry(n - 1).or_default();
    g
}

fn bench_mcf_chain_small(c: &mut Criterion) {
    let g = build_chain(10);
    let demands = [(0, -5), (9, 5)].into();

    c.bench_function("mcf_chain_10", |b| {
        b.iter(|| cycle_canceling_mcf(black_box(&g), black_box(&demands)))
    });
}

fn bench_mcf_chain_medium(c: &mut Criterion) {
    let g = build_chain(100);
    let demands = [(0, -50), (99, 50)].into();

    c.bench_function("mcf_chain_100", |b| {
        b.iter(|| cycle_canceling_mcf(black_box(&g), black_box(&demands)))
    });
}

fn bench_mcf_negative_cycle(c: &mut Criterion) {
    // A graph with a negative-cost cycle (like the spare TSV residual)
    let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
    g.insert(0, [(1, Edge::new(10, 10)), (2, Edge::new(1, 10))].into());
    g.insert(1, [(2, Edge::new(1, 10)), (3, Edge::new(1, 10))].into());
    g.insert(2, [(3, Edge::new(1, 10))].into());
    g.insert(3, [(1, Edge::new(-8, 10))].into());
    let demands = [(0, -5), (3, 5)].into();

    c.bench_function("mcf_neg_cycle", |b| {
        b.iter(|| cycle_canceling_mcf(black_box(&g), black_box(&demands)))
    });
}

fn bench_mcf_spare_tsv_scale(c: &mut Criterion) {
    let (g, demands) = digraphx_rs::mcf::build_spare_tsv_graph();
    // Verify cost matches Python reference (1108)
    let (cost, _) = cycle_canceling_mcf(&g, &demands).unwrap();
    assert_eq!(cost, 1108);

    c.bench_function("mcf_spare_tsv_195", |b| {
        b.iter(|| cycle_canceling_mcf(black_box(&g), black_box(&demands)))
    });
}

fn bench_mcf_grid_like(c: &mut Criterion) {
    // A denser graph: 50 nodes, each connected to next 3, supply/demand spread
    let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
    for i in 0..50usize {
        for j in 1..=3 {
            if i + j < 50 {
                g.entry(i)
                    .or_default()
                    .insert(i + j, Edge::new(j as i64, 50));
            }
        }
    }
    for i in 0..50 {
        g.entry(i).or_default();
    }
    let mut demands: HashMap<usize, i64> = HashMap::new();
    demands.insert(0, -25);
    demands.insert(49, 25);

    c.bench_function("mcf_grid_50", |b| {
        b.iter(|| cycle_canceling_mcf(black_box(&g), black_box(&demands)))
    });
}

criterion_group!(
    benches,
    bench_neg_cycle_small,
    bench_neg_cycle_no_cycle,
    bench_neg_cycle_medium,
    bench_howard_ratio,
    bench_parametric_solver,
    bench_mcf_chain_small,
    bench_mcf_chain_medium,
    bench_mcf_negative_cycle,
    bench_mcf_grid_like,
    bench_mcf_spare_tsv_scale,
);
criterion_main!(benches);
