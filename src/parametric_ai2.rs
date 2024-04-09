use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::{Add, Sub, Mul, Div};
use std::collections::VecDeque;
use std::fmt::Debug;

pub trait ParametricAPI<V, R> {
    fn distance(&self, ratio: R, edge: &(V, V)) -> R;
    fn zero_cancel(&self, cycle: &Vec<(V, V)>) -> R;
}

pub struct MaxParametricSolver<V, R, P> {
    ncf: NegCycleFinder<V, R, P>,
    omega: P,
}

impl<V, R, P> MaxParametricSolver<V, R, P>
where
    V: Eq + Hash + Clone + Debug,
    R: PartialOrd + Default + Copy + Add<Output = R> + Sub<Output = R> + Mul<Output = R> + Div<Output = R> + Debug,
    P: ParametricAPI<V, R>,
{
    pub fn new(grph: &HashMap<V, HashMap<V, R>>, omega: P) -> Self {
        Self {
            ncf: NegCycleFinder::new(grph),
            omega,
        }
    }

    pub fn run(&mut self, dist: &mut HashMap<V, R>, ratio: R) -> (R, Vec<(V, V)>) {
        let mut r_min = ratio;
        let mut c_min = vec![];
        let mut cycle = vec![];
        loop {
            if let Some(ci) = self.ncf.find_neg_cycle(dist, |e| self.omega.distance(ratio, e)) {
                let ri = self.omega.zero_cancel(&ci);
                if r_min > ri {
                    r_min = ri;
                    c_min = ci.clone();
                }
            } else {
                break;
            }
            if r_min >= ratio {
                break;
            }
            cycle = c_min.clone();
            ratio = r_min;
        }
        (ratio, cycle)
    }
}

pub struct NegCycleFinder<V, R, P> {
    grph: HashMap<V, HashMap<V, R>>,
    omega: P,
}

impl<V, R, P> NegCycleFinder<V, R, P>
where
    V: Eq + Hash + Clone + Debug,
    R: PartialOrd + Default + Copy + Add<Output = R> + Sub<Output = R> + Mul<Output = R> + Div<Output = R> + Debug,
    P: ParametricAPI<V, R>,
{
    pub fn new(grph: &HashMap<V, HashMap<V, R>>) -> Self {
        Self {
            grph: grph.clone(),
            omega: Default::default(),
        }
    }

    pub fn find_neg_cycle<F>(&self, dist: &mut HashMap<V, R>, mut cost: F) -> Option<Vec<(V, V)>>
    where
        F: FnMut(&(V, V)) -> R,
    {
        let n = self.grph.len();
        let mut prev = vec![None; n];
        let mut last = vec![None; n];
        let mut to = vec![None; n];
        let mut label = vec![R::default(); n];
        let mut id = vec![0; n];
        let mut q = VecDeque::new();
        for s in self.grph.keys() {
            dist.insert(s.clone(), R::default());
            q.push_back(s.clone());
        }
        for k in 0..n {
            let mut upd = false;
            for i in 0..n {
                for (j, &w) in self.grph[&id[i]].iter() {
                    let j = self.get_id(&mut id, &mut to, j.clone());
                    let d = dist[&id[i]] + cost(&(id[i], j.clone()));
                    if dist[&j] > d {
                        dist.insert(j.clone(), d);
                        prev[j] = Some(id[i].clone());
                        last[j] = Some(j.clone());
                        upd = true;
                        if k == n - 1 {
                            let mut cycle = vec![];
                            let mut v = j.clone();
                            loop {
                                cycle.push((prev[v].unwrap(), v.clone()));
                                v = prev[v].unwrap();
                                if v == j {
                                    break;
                                }
                            }
                            cycle.reverse();
                            return Some(cycle);
                        }
                    }
                }
            }
            if !upd {
                break;
            }
        }
        None
    }

    fn get_id(&self, id: &mut Vec<usize>, to: &mut Vec<Option<usize>>, v: V) -> usize {
        let n = self.grph.len();
        let i = id[v.hash() % n];
        if let Some(j) = to[i] {
            if self.grph[&i].contains_key(&v) {
                return j;
            }
        }
        let j = id[v.hash() % n];
        to[j] = Some(v.hash() % n);
        id[v.hash() % n] = j;
        j
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_parametric_solver() {
        let mut grph = HashMap::new();
        grph.insert(0, [(1, 1), (2, 2)].iter().cloned().collect());
        grph.insert(1, [(2, 3)].iter().cloned().collect());
        grph.insert(2, [(0, -10)].iter().cloned().collect());
        let omega = Omega {};
        let mut solver = MaxParametricSolver::new(&grph, omega);
        let mut dist = HashMap::new();
        let (ratio, cycle) = solver.run(&mut dist, 1.0);
        assert_eq!(ratio, -0.5);
        assert_eq!(cycle, [(2, 0), (0, 1), (1, 2)]);
    }

    struct Omega {}

    impl ParametricAPI<usize, f64> for Omega {
        fn distance(&self, ratio: f64, edge: &(usize, usize)) -> f64 {
            let (u, v) = edge;
            if *u == 2 && *v == 0 {
                return -ratio;
            }
            let w = match (*u, *v) {
                (0, 1) => 1.0,
                (1, 2) => 3.0,
                (2, 0) => -10.0,
                _ => unreachable!(),
            };
            w - ratio
        }

