use petgraph::algo::find_negative_cycle;
use petgraph::prelude::*;
use petgraph::Graph;

fn main() {
    let graph_with_neg_cycle = Graph::<(), f32, Directed>::from_edges([
        (0, 1, 1.),
        (0, 2, 1.),
        (0, 3, 1.),
        (1, 3, 1.),
        (2, 1, 1.),
        (3, 2, -3.),
    ]);

    let path = find_negative_cycle(&graph_with_neg_cycle, NodeIndex::new(0));
    assert_eq!(
        path,
        Some([NodeIndex::new(1), NodeIndex::new(3), NodeIndex::new(2)].to_vec())
    );
}
