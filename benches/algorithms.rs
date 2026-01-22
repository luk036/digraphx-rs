//! Benchmark suite for digraphx-rs using criterion

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use digraphx_rs::{
    bellman_ford, find_negative_cycle,
    neg_cycle::NegCycleFinder,
    parametric::{MaxParametricSolver, ParametricAPI},
};
use num::rational::Ratio;
use petgraph::{
    graph::{DiGraph, EdgeReference},
    Graph,
};

fn bench_bellman_ford_small(c: &mut Criterion) {
    let mut g = Graph::new();
    let nodes: Vec<_> = (0..10).map(|_| g.add_node(())).collect();

    for i in 0..9 {
        g.add_edge(nodes[i], nodes[i + 1], 1.0);
    }

    c.bench_function("bellman_ford_small", |b| {
        b.iter(|| bellman_ford(black_box(&g), nodes[0]))
    });
}

fn bench_bellman_ford_medium(c: &mut Criterion) {
    let mut g = Graph::new();
    let nodes: Vec<_> = (0..100).map(|_| g.add_node(())).collect();

    for i in 0..99 {
        g.add_edge(nodes[i], nodes[(i + 1) % 100], 1.0);
    }

    for i in 0..100 {
        for j in (i + 2)..100 {
            g.add_edge(nodes[i], nodes[j], 2.0);
        }
    }

    c.bench_function("bellman_ford_medium", |b| {
        b.iter(|| bellman_ford(black_box(&g), nodes[0]))
    });
}

fn bench_find_negative_cycle_small(c: &mut Criterion) {
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let d = g.add_node(());

    g.extend_with_edges([(a, b, 1.0), (b, d, 1.0), (d, a, -3.0)]);

    c.bench_function("find_negative_cycle_small", |b| {
        b.iter(|| find_negative_cycle(black_box(&g), a))
    });
}

fn bench_neg_cycle_finder_howard(c: &mut Criterion) {
    let digraph = DiGraph::<(), Ratio<i32>>::from_edges(
        (0u32..100)
            .flat_map(|i| (0u32..100).map(move |j| (i, j, Ratio::new((i + j) as i32 % 10, 1))))
            .take(500),
    );

    let _ncf = NegCycleFinder::new(&digraph);
    let dist = vec![Ratio::new(0, 1); digraph.node_count()];

    c.bench_function("neg_cycle_finder_howard", |b| {
        b.iter(|| {
            let mut ncf = NegCycleFinder::new(black_box(&digraph));
            let mut dist = dist.clone();
            ncf.howard(black_box(&mut dist), |e| *e.weight())
        })
    });
}

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

fn bench_parametric_solver(c: &mut Criterion) {
    let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
        (0, 1, Ratio::new(1, 1)),
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
            let mut solver = MaxParametricSolver::new(black_box(&digraph), MyRatio {});
            let mut dist = [
                Ratio::new(0, 1),
                Ratio::new(0, 1),
                Ratio::new(0, 1),
                Ratio::new(0, 1),
                Ratio::new(0, 1),
                Ratio::new(0, 1),
                Ratio::new(0, 1),
            ];
            let mut ratio = Ratio::new(1_000_000, 1);
            solver.run(black_box(&mut dist), black_box(&mut ratio))
        })
    });
}

criterion_group!(
    benches,
    bench_bellman_ford_small,
    bench_bellman_ford_medium,
    bench_find_negative_cycle_small,
    bench_neg_cycle_finder_howard,
    bench_parametric_solver
);
criterion_main!(benches);
