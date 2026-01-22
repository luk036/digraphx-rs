# Graph Representations

This document describes the different graph representations used in digraphx-rs.

## Petgraph-based Representations (Primary)

The primary graph representation uses [`petgraph`](https://docs.rs/petgraph/0.8/petgraph/) library:

### `DiGraph<V, E>`

Directed graph with node type `V` and edge weight type `E`.

**Used in:**
- `src/neg_cycle.rs` - `NegCycleFinder`
- `src/parametric.rs` - `MaxParametricSolver`
- `src/main.rs` - Example usage

**Advantages:**
- Efficient graph operations
- Rich set of algorithms from petgraph ecosystem
- Well-tested and battle-hardened
- Automatic indexing via `NodeIndex`

**Example:**
```rust
use petgraph::graph::DiGraph;

let mut g = DiGraph::new();
let a = g.add_node(());
let b = g.add_node(());
g.add_edge(a, b, 5);
```

### `Graph<V, E, Ty>`

More general graph that can be directed or undirected based on `Ty`.

**Used in:**
- `src/lib.rs` - Bellman-Ford algorithms (public API)

**Example:**
```rust
use petgraph::Graph;

let mut g = Graph::new();
let a = g.add_node(());
let b = g.add_node(());
g.extend_with_edges([(a, b, 4.0)]);
```

## HashMap-based Representation (Alternative)

Simple adjacency list representation using nested HashMaps.

### `HashMap<Node, HashMap<Node, Edge>>`

**Used in:**
- `src/parametric_ai2.rs` - Alternative implementation
- `experiments/neg_cycle_ai.rs` - AI exploration artifacts
- `experiments/parametric_ai.rs` - AI exploration artifacts

**Advantages:**
- Simple to understand
- Easy to serialize
- No external dependencies

**Disadvantages:**
- Less efficient operations
- No automatic indexing
- Manual index management required
- No access to petgraph's rich algorithm set

**Example:**
```rust
use std::collections::HashMap;

let mut graph: HashMap<usize, HashMap<usize, i32>> = HashMap::new();
graph.insert(0, HashMap::new());
graph.get_mut(&0).unwrap().insert(1, 5);
```

## Recommendation

**Use petgraph-based representations (`DiGraph`, `Graph`) for:**

- Production code
- Performance-critical operations
- Large graphs (>1000 nodes)
- Need for petgraph's algorithm ecosystem

**Use HashMap-based representations only for:**

- Learning/experimental code
- Simple scripts
- When you explicitly need adjacency list semantics
- Migration from existing codebases

## Future Considerations

The HashMap-based representations in `src/parametric_ai2.rs` and AI exploration files should either:
1. Be consolidated with petgraph-based implementations, or
2. Be marked as experimental/deprecated with clear migration path

Choose one representation pattern and document it consistently throughout the codebase.
