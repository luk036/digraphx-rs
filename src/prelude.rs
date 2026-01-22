//! Prelude module for convenient imports

//! # Example
//!
//! ```rust
//! use digraphx_rs::prelude::*;
//! use petgraph::Graph;
//!
//! let mut g: Graph<(), f32> = Graph::new();
//! ```

pub use crate::bellman_ford;
pub use crate::bellman_ford_initialize_relax;
pub use crate::find_negative_cycle;
pub use crate::neg_cycle::NegCycleFinder;
pub use crate::parametric::{MaxParametricSolver, ParametricAPI};
pub use crate::Paths;
