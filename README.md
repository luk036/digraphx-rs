# 🔀 digraphx-rs

[![Crates.io](https://img.shields.io/crates/v/digraphx-rs.svg)](https://crates.io/crates/digraphx-rs)
[![Docs.rs](https://docs.rs/digraphx-rs/badge.svg)](https://docs.rs/digraphx-rs)
[![CI](https://github.com/luk036/digraphx-rs/workflows/CI/badge.svg)](https://github.com/luk036/digraphx-rs/actions)
[![codecov](https://codecov.io/gh/luk036/digraphx-rs/branch/main/graph/badge.svg?token=bamdGjpTmm)](https://codecov.io/gh/luk036/digraphx-rs)

Network optimization algorithms in Rust.

## Features

- **Bellman-Ford** - Shortest path algorithm with support for negative edge weights
- **Negative Cycle Detection** - Find and report cycles with negative total weight
- **Parametric Algorithms** - Maximum cycle ratio and related optimization problems
- **Howard's Algorithm** - Efficient negative cycle detection for large graphs

## 🛠️ Installation

### 📦 Cargo

- Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
- run `cargo install digraphx-rs`

Add this to your `Cargo.toml`:

```toml
[dependencies]
digraphx-rs = "0.1"
petgraph = "0.8"
```

## Quick Start

```rust
use digraphx_rs::bellman_ford;
use petgraph::Graph;

fn main() {
    // Create a directed graph with weighted edges
    let mut g = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.extend_with_edges([(a, b, 4.0), (b, c, 3.0), (a, c, 10.0)]);

    // Find shortest paths from node 'a'
    let paths = bellman_ford(&g, a).unwrap();
    println!("Distances: {:?}", paths.distances);
    // Output: Distances: [0.0, 4.0, 7.0]

    // Access predecessors to reconstruct paths
    println!("Predecessors: {:?}", paths.predecessors);
}
```

See [examples](examples/) directory for more detailed usage patterns.

## 📜 License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🤝 Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
