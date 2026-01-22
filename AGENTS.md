# AGENTS.md

This file guides AI agents working on digraphx-rs, a Rust library for network optimization algorithms.

## Build, Test, and Lint Commands

### Building
- `cargo build` - Debug build
- `cargo build --release` - Release build
- `cargo build --release && cargo run --release` - Build and run release version

### Testing
- `cargo test --all-features --workspace` - Run all tests
- `cargo test <test_name>` - Run a specific test function
- `cargo test <module>::tests` - Run tests in a specific module

Example: `cargo test bellman_ford_simple`

### Linting and Formatting
- `cargo clippy --all-targets --all-features --workspace` - Run Clippy linter
- `cargo fmt --all` - Format all code
- `cargo fmt --all -- --check` - Check formatting without modifying

### Documentation
- `cargo doc --no-deps --document-private-items --all-features --workspace --examples` - Generate docs with warnings as errors

## Code Style Guidelines

### Naming Conventions
- **Structs**: PascalCase (e.g., `NegCycleFinder`, `MaxParametricSolver`, `Paths`)
- **Traits**: PascalCase (e.g., `ParametricAPI`)
- **Functions & Methods**: snake_case (e.g., `bellman_ford`, `find_negative_cycle`, `new`, `run`)
- **Constants**: UPPER_SNAKE_CASE
- **Type Parameters**: Single uppercase letters (e.g., `V`, `R`, `P`, `G`, `E`)
- **Variables**: snake_case

### Import Organization
Imports should be organized in this order, with blank lines between groups:
1. `std` imports
2. External crate imports (e.g., `petgraph`, `num`)
3. Local crate imports (`crate::...`)

```rust
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;

use petgraph::graph::{DiGraph, EdgeReference};
use petgraph::prelude::*;

use crate::neg_cycle::NegCycleFinder;
```

### Documentation Style
- Module-level documentation uses `//!`
- Function/struct documentation uses `///`
- Include examples in docstrings
- Link to external references (Wikipedia, algorithm papers) where appropriate

```rust
//! Bellman-Ford algorithms.

/// Compute shortest paths from node `source` to all other.
///
/// Using the [Bellman–Ford algorithm][bf]; negative edge costs are
/// permitted, but the graph must not have a cycle of negative weights.
///
/// [bf]: https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm
```

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `Option<T>` for optional values
- Leverage petgraph error types (e.g., `NegativeCycle`)
- Never panic on expected errors - always return `Result` or `Option`

```rust
pub fn bellman_ford<G>(
    g: G,
    source: G::NodeId,
) -> Result<Paths<G::NodeId, G::EdgeWeight>, NegativeCycle>

pub fn find_negative_cycle<G>(g: G, source: G::NodeId) -> Option<Vec<G::NodeId>>
```

### Type System Patterns
- Use generics extensively with clear trait bounds
- Specify trait bounds in `where` clauses for readability
- Leverage petgraph traits (`FloatMeasure`, `Visitable`, `NodeCount`, etc.)
- Use numeric traits from `num` crate (`Zero`, `One`, `Inv`, etc.)
- Apply lifetime annotations for references (`'a`, etc.)

```rust
impl<'a, V, R, P> MaxParametricSolver<'a, V, R, P>
where
    R: Copy + PartialOrd + Add<Output = R> + Sub<Output = R>,
    V: Eq + Hash + Clone,
    P: ParametricAPI<V, R>,
```

### Testing Organization
- Unit tests are placed inline in `#[cfg(test)]` modules
- Test functions are prefixed with `test_`
- Use `assert!` and `assert_eq!` macros
- Cover edge cases and typical usage patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bellman_ford_simple() {
        let mut g = Graph::new();
        let a = g.add_node(());
        let b = g.add_node(());
        g.extend_with_edges(&[(a, b, 1.0)]);
        let path = bellman_ford(&g, a).unwrap();
        assert_eq!(path.distances, vec![0.0, 1.0]);
    }
}
```

### Project Structure
- `src/lib.rs` - Main library entry point, module declarations
- `src/neg_cycle.rs` - Negative cycle detection algorithms
- `src/parametric.rs` - Parametric algorithms
- `src/main.rs` - Example usage/demonstration
- No separate `tests/` directory - all tests are inline

### Dependencies
- **petgraph**: Graph library (version 0.8.2 with serde-1 feature)
- **num**: Numeric traits (version 0.4.3)
- **num-traits**: Numeric trait definitions (version 0.2.19)

### General Guidelines
- Follow Rust 2021 edition conventions
- Prefer `#[inline(always)]` for performance-critical small functions
- Use `#[derive(Debug)]` for structs that benefit from debug output
- Keep functions focused and single-purpose
- Document complex algorithms with references to original papers
