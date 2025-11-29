// use std::collections::HashMap;
// use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use petgraph::graph::{DiGraph, EdgeReference};
// use petgraph::prelude::*;
// use petgraph::visit::EdgeRef;
// use petgraph::visit::IntoNodeIdentifiers;
// use petgraph::Direction;

// use num::traits::Float;
use num::traits::Inv;
use num::traits::One;
use num::traits::Zero;

use crate::neg_cycle::NegCycleFinder;

/// The `ParametricAPI` trait defines two methods: `distance` and `zero_cancel`.
///
/// # Example
/// ```rust
/// use petgraph::graph::EdgeReference;
/// use digraphx_rs::parametric::ParametricAPI;
/// use num::rational::Ratio;
///
/// struct MyAPI;
///
/// impl<V> ParametricAPI<V, Ratio<i32>> for MyAPI
/// where
///     V: Eq + Clone + std::hash::Hash,
/// {
///     fn distance(&self, ratio: &Ratio<i32>, _edge: &EdgeReference<Ratio<i32>>) -> Ratio<i32> {
///         *ratio
///     }
///
///     fn zero_cancel(&self, _cycle: &[EdgeReference<Ratio<i32>>]) -> Ratio<i32> {
///         Ratio::new(0, 1)
///     }
/// }
/// ```
pub trait ParametricAPI<E, R>
where
    R: Copy + PartialOrd,
    E: Clone,
{
    fn distance(&self, ratio: &R, edge: &EdgeReference<R>) -> R;
    fn zero_cancel(&self, cycle: &[EdgeReference<R>]) -> R;
}

/// The `MaxParametricSolver` struct is a generic type that takes in parameters `V`, `R`, and `P` and
/// contains a `NegCycleFinder` and `omega` of type `P`.
///
/// Properties:
///
/// * `ncf`: NegCycleFinder is a struct that is used to find negative cycles in a graph. It takes three
///   type parameters: 'a, V, and R. 'a represents the lifetime of the struct, V represents the type of
///   the vertices in the graph, and R represents the type of the weights or
/// * `omega`: The `omega` property is of type `P`, which is a generic type parameter that implements
///   the `ParametricAPI` trait. This trait is not defined in the code snippet you provided, so it is
///   likely defined elsewhere in the codebase.
#[derive(Debug)]
pub struct MaxParametricSolver<'a, V, R, P>
where
    R: Copy
        + PartialOrd
        + Add<Output = R>
        + Sub<Output = R>
        + Mul<Output = R>
        + Div<Output = R>
        + Neg<Output = R>
        + Inv<Output = R>,
    V: Eq + Hash + Clone,
    P: ParametricAPI<V, R>,
{
    ncf: NegCycleFinder<'a, V, R>,
    omega: P,
}

impl<'a, V, R, P> MaxParametricSolver<'a, V, R, P>
where
    R: Copy
        + PartialOrd
        + Zero
        + One
        + Add<Output = R>
        + Sub<Output = R>
        + Mul<Output = R>
        + Div<Output = R>
        + Neg<Output = R>
        + Inv<Output = R>,
    V: Eq + Hash + Clone,
    P: ParametricAPI<V, R>,
{
    /// The function creates a new instance of a struct with a given directed graph and a value.
    ///
    /// Arguments:
    ///
    /// * `grph`: The `grph` parameter is a reference to a directed graph (`DiGraph`) with vertices of
    ///   type `V` and edges of type `R`.
    /// * `omega`: The `omega` parameter is of type `P`. It represents some value or parameter that is
    ///   used in the implementation of the `new` function. The specific meaning or purpose of `omega`
    ///   would depend on the context and the code that uses this function.
    ///
    /// Returns:
    ///
    /// The `new` function is returning an instance of the struct that it is defined in.
    ///
    /// # Example
    /// ```rust
    /// use petgraph::prelude::*;
    /// use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
    /// use petgraph::graph::{EdgeReference, DiGraph};
    /// use num::rational::Ratio;
    /// use num::traits::{Zero, One, Inv};
    /// use std::ops::{Add, Sub, Mul, Div, Neg};
    ///
    /// #[derive(Debug)]
    /// struct MyOmega;
    ///
    /// impl<V> ParametricAPI<V, Ratio<i32>> for MyOmega
    /// where
    ///     V: Eq + Clone,
    /// {
    ///     fn distance(&self, ratio: &Ratio<i32>, edge: &EdgeReference<Ratio<i32>>) -> Ratio<i32> {
    ///         *edge.weight() - *ratio
    ///     }
    ///
    ///     fn zero_cancel(&self, cycle: &[EdgeReference<Ratio<i32>>]) -> Ratio<i32> {
    ///         let total_weight: Ratio<i32> = cycle.iter().map(|e| *e.weight()).sum();
    ///         total_weight / Ratio::from_integer(cycle.len() as i32)
    ///     }
    /// }
    ///
    /// let digraph = DiGraph::<(), Ratio<i32>>::new();
    /// let solver = MaxParametricSolver::new(&digraph, MyOmega);
    /// ```
    pub fn new(grph: &'a DiGraph<V, R>, omega: P) -> Self {
        Self {
            ncf: NegCycleFinder::new(grph),
            omega,
        }
    }

    /// The function `run` finds the minimum ratio and corresponding cycle in a given graph.
    ///
    /// Arguments:
    ///
    /// * `dist`: `dist` is a mutable reference to a slice of type `R`. It represents a distance matrix
    ///   or array, where `R` is the type of the elements in the matrix.
    /// * `ratio`: The `ratio` parameter is a mutable reference to a value of type `R`. It represents
    ///   the current ratio value that is being used in the algorithm. The algorithm will update this
    ///   value if it finds a smaller ratio during its execution.
    ///
    /// Returns:
    ///
    /// a vector of `EdgeReference<R>`.
    ///
    /// # Example
    /// ```rust
    /// use petgraph::prelude::*;
    /// use digraphx_rs::parametric::{MaxParametricSolver, ParametricAPI};
    /// use petgraph::graph::{EdgeReference, DiGraph};
    /// use num::rational::Ratio;
    /// use num::traits::{Zero, One, Inv};
    /// use std::ops::{Add, Sub, Mul, Div, Neg};
    ///
    /// #[derive(Debug)]
    /// struct MyOmega;
    ///
    /// impl<V> ParametricAPI<V, Ratio<i32>> for MyOmega
    /// where
    ///     V: Eq + Clone,
    /// {
    ///     fn distance(&self, ratio: &Ratio<i32>, edge: &EdgeReference<Ratio<i32>>) -> Ratio<i32> {
    ///         *edge.weight() - *ratio
    ///     }
    ///
    ///     fn zero_cancel(&self, cycle: &[EdgeReference<Ratio<i32>>]) -> Ratio<i32> {
    ///         let total_weight: Ratio<i32> = cycle.iter().map(|e| *e.weight()).sum();
    ///         total_weight / Ratio::from_integer(cycle.len() as i32)
    ///     }
    /// }
    ///
    /// let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
    ///     (0, 1, Ratio::new(1, 1)),
    ///     (1, 2, Ratio::new(1, 1)),
    ///     (2, 0, Ratio::new(1, 1)),
    /// ]);
    /// let mut solver = MaxParametricSolver::new(&digraph, MyOmega);
    /// let mut dist = [Ratio::new(0, 1), Ratio::new(0, 1), Ratio::new(0, 1)];
    /// let mut ratio = Ratio::new(100, 1);
    /// let cycle = solver.run(&mut dist, &mut ratio);
    /// // The function returns a cycle if found
    /// ```
    pub fn run(&mut self, dist: &mut [R], ratio: &mut R) -> Vec<EdgeReference<'a, R>> {
        let mut cycle = Vec::<EdgeReference<R>>::new();
        loop {
            for d in dist.iter_mut() {
                *d = R::zero();
            }
            if let Some(ci) = self.ncf.howard(dist, |e| self.omega.distance(ratio, &e)) {
                let ri = self.omega.zero_cancel(&ci);
                if *ratio > ri {
                    *ratio = ri;
                    cycle = ci;
                    continue;
                }
            }
            break;
        }
        cycle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use num::rational::Ratio;

    #[derive(Debug)]
    struct MyRatio {}

    impl<V, R> ParametricAPI<V, R> for MyRatio
    where
        R: Copy
            + PartialOrd
            + Add<Output = R>
            + Sub<Output = R>
            + Mul<Output = R>
            + Div<Output = R>
            + Neg<Output = R>
            + Inv<Output = R>
            + Zero
            + One
            + From<i32>,
        V: Eq + Hash + Clone,
    {
        fn distance(&self, ratio: &R, edge: &EdgeReference<R>) -> R {
            *edge.weight() - *ratio
        }

        fn zero_cancel(&self, cycle: &[EdgeReference<R>]) -> R {
            let mut total_weight = R::zero();
            for edge in cycle {
                total_weight = total_weight + *edge.weight();
            }
            total_weight / R::from(cycle.len() as i32)
        }
    }

    #[test]
    fn test_parametric_solver_simple() {
        let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
            (0, 1, Ratio::new(1, 1)),
            (1, 2, Ratio::new(1, 1)),
            (2, 0, Ratio::new(1, 1)),
        ]);

        let mut solver = MaxParametricSolver::new(&digraph, MyRatio {});
        let mut dist = [Ratio::new(0, 1), Ratio::new(0, 1), Ratio::new(0, 1)];
        let mut ratio = Ratio::new(1_000_000, 1);
        solver.run(&mut dist, &mut ratio);

        assert_eq!(ratio, Ratio::new(1, 1));
    }

    #[test]
    fn test_parametric_solver_negative_cycle() {
        let digraph = DiGraph::<(), Ratio<i32>>::from_edges([
            (0, 1, Ratio::new(1, 1)),
            (1, 2, Ratio::new(-5, 1)),
            (2, 0, Ratio::new(1, 1)),
        ]);

        let mut solver = MaxParametricSolver::new(&digraph, MyRatio {});
        let mut dist = [Ratio::new(0, 1), Ratio::new(0, 1), Ratio::new(0, 1)];
        let mut ratio = Ratio::new(1_000_000, 1);
        solver.run(&mut dist, &mut ratio);

        assert_eq!(ratio, Ratio::new(-1, 1));
    }

    #[test]
    fn test_parametric_solver_no_cycle() {
        let digraph = DiGraph::<(), Ratio<i32>>::from_edges([(0, 1, Ratio::new(1, 1))]);

        let mut solver = MaxParametricSolver::new(&digraph, MyRatio {});
        let mut dist = [Ratio::new(0, 1), Ratio::new(0, 1)];
        let mut ratio = Ratio::new(1_000_000, 1);
        let cycle = solver.run(&mut dist, &mut ratio);

        assert_eq!(ratio, Ratio::new(1_000_000, 1));
        assert!(cycle.is_empty());
    }
}
