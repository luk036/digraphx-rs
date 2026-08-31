//! Min-cost flow via cycle-cancellation descent.
//!
//! Uses Bellman-Ford to find negative-cost cycles in the residual graph.
//! Ported from `digraphx.mcf` (Python).
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashMap;
//! use digraphx_rs::mcf::{Edge, cycle_canceling_mcf};
//!
//! let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
//! g.insert(0, [(1, Edge::new(1, 5)), (2, Edge::new(5, 5))].into());
//! g.insert(1, [(2, Edge::new(1, 5)), (3, Edge::new(2, 5))].into());
//! g.insert(2, [(3, Edge::new(1, 5))].into());
//! g.insert(3, HashMap::new());
//!
//! let demands = [(0, -2), (3, 2)].into();
//! let result = cycle_canceling_mcf(&g, &demands);
//! assert!(result.is_some());
//! let (cost, _flow) = result.unwrap();
//! assert_eq!(cost, 6);
//! ```

use std::collections::{HashMap, VecDeque};

/// Edge in the original cost / capacity graph.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub weight: i64,
    pub capacity: i64,
}

impl Edge {
    #[inline]
    pub fn new(weight: i64, capacity: i64) -> Self {
        Self { weight, capacity }
    }
}

/// Edge in the residual graph.
#[derive(Debug, Clone)]
pub struct ResidualEdge {
    pub cost: i64,
    pub capacity: i64,
    pub orig: (usize, usize),
    pub forward: bool,
}

type FlowMap = HashMap<usize, HashMap<usize, i64>>;

/// Current flow value on edge (u, v), defaulting to 0 when absent.
#[inline]
fn flow_value(flow: &FlowMap, u: usize, v: usize) -> i64 {
    flow.get(&u).and_then(|r| r.get(&v)).copied().unwrap_or(0)
}

/// Find an initial feasible flow by greedily routing supply → demand via BFS.
fn find_feasible_flow(
    g: &HashMap<usize, HashMap<usize, Edge>>,
    demands: &HashMap<usize, i64>,
) -> Option<HashMap<usize, HashMap<usize, i64>>> {
    let mut nodes: Vec<usize> = g.keys().copied().collect();
    nodes.sort_unstable();

    let mut flow: HashMap<usize, HashMap<usize, i64>> = HashMap::new();
    for (&u, nbrs) in g {
        let row = flow.entry(u).or_default();
        for &v in nbrs.keys() {
            row.insert(v, 0);
        }
    }

    let mut remaining: HashMap<usize, i64> = demands.clone();

    let supply_nodes: Vec<usize> = remaining
        .iter()
        .filter(|(_, &d)| d < 0)
        .map(|(&n, _)| n)
        .collect();
    let demand_nodes: Vec<usize> = remaining
        .iter()
        .filter(|(_, &d)| d > 0)
        .map(|(&n, _)| n)
        .collect();

    if supply_nodes.is_empty() || demand_nodes.is_empty() {
        return Some(flow);
    }

    for &src in &supply_nodes {
        let mut supply_amount = -remaining[&src];
        while supply_amount > 0 {
            let path = bfs_path(g, &flow, src, &demand_nodes, &remaining)?;
            let dst = *path.last().unwrap();

            let mut bottleneck = i64::MAX;
            for window in path.windows(2) {
                let u = window[0];
                let v = window[1];
                let cap = g[&u][&v].capacity;
                let existing = flow_value(&flow, u, v);
                bottleneck = bottleneck.min(cap - existing);
            }
            bottleneck = bottleneck.min(remaining[&dst]).min(supply_amount);
            if bottleneck <= 0 {
                return None;
            }

            for window in path.windows(2) {
                let u = window[0];
                let v = window[1];
                *flow.entry(u).or_default().entry(v).or_insert(0) += bottleneck;
            }

            *remaining.get_mut(&src).unwrap() += bottleneck;
            *remaining.get_mut(&dst).unwrap() -= bottleneck;
            supply_amount -= bottleneck;
        }
    }

    Some(flow)
}

/// BFS from `src` to any demand node with remaining capacity.
fn bfs_path(
    g: &HashMap<usize, HashMap<usize, Edge>>,
    flow: &HashMap<usize, HashMap<usize, i64>>,
    src: usize,
    demand_nodes: &[usize],
    remaining: &HashMap<usize, i64>,
) -> Option<Vec<usize>> {
    let demand_set: std::collections::HashSet<usize> = demand_nodes.iter().copied().collect();
    let mut visited = std::collections::HashSet::new();
    visited.insert(src);
    let mut parent: HashMap<usize, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back(src);

    while let Some(u) = queue.pop_front() {
        if demand_set.contains(&u) && remaining.get(&u).copied().unwrap_or(0) > 0 {
            let mut path = vec![u];
            let mut cur = u;
            while cur != src {
                cur = parent[&cur];
                path.push(cur);
            }
            path.reverse();
            return Some(path);
        }

        if let Some(nbrs) = g.get(&u) {
            for (&v, edge) in nbrs {
                if visited.contains(&v) {
                    continue;
                }
                let existing = flow_value(flow, u, v);
                if existing < edge.capacity {
                    visited.insert(v);
                    parent.insert(v, u);
                    queue.push_back(v);
                }
            }
        }
    }

    None
}

/// Build the residual edges for one original edge (u, v) with the given flow.
///
/// Returns at most two candidate residual edges: the forward edge (if the
/// capacity is not exhausted) and the backward edge (if flow is positive).
#[inline]
fn residual_edges_for(u: usize, v: usize, cap: i64, wgt: i64, f: i64) -> Vec<ResidualEdge> {
    let mut edges = Vec::with_capacity(2);
    if f < cap {
        edges.push(ResidualEdge {
            cost: wgt,
            capacity: cap - f,
            orig: (u, v),
            forward: true,
        });
    }
    if f > 0 {
        edges.push(ResidualEdge {
            cost: -wgt,
            capacity: f,
            orig: (u, v),
            forward: false,
        });
    }
    edges
}

/// Build the full residual graph from current flow.
fn build_residual(
    g: &HashMap<usize, HashMap<usize, Edge>>,
    flow: &FlowMap,
) -> HashMap<usize, HashMap<usize, ResidualEdge>> {
    let mut residual: HashMap<usize, HashMap<usize, ResidualEdge>> = HashMap::new();

    for (&u, nbrs) in g {
        for (&v, data) in nbrs {
            let f = flow_value(flow, u, v);
            for edge in residual_edges_for(u, v, data.capacity, data.weight, f) {
                let src = if edge.forward { u } else { v };
                let prev = residual
                    .entry(src)
                    .or_default()
                    .get(&if edge.forward { v } else { u });
                if prev.is_none() || edge.cost < prev.unwrap().cost {
                    residual
                        .entry(src)
                        .or_default()
                        .insert(if edge.forward { v } else { u }, edge);
                }
            }
        }
    }

    residual
}

/// Update a single residual edge after a flow change.
fn update_residual_edge(
    residual: &mut HashMap<usize, HashMap<usize, ResidualEdge>>,
    g: &HashMap<usize, HashMap<usize, Edge>>,
    flow: &FlowMap,
    u: usize,
    v: usize,
) {
    if let Some(row) = residual.get_mut(&u) {
        row.remove(&v);
        if row.is_empty() {
            residual.remove(&u);
        }
    }
    if let Some(row) = residual.get_mut(&v) {
        row.remove(&u);
        if row.is_empty() {
            residual.remove(&v);
        }
    }

    if let Some(data) = g.get(&u).and_then(|nbrs| nbrs.get(&v)) {
        let f = flow_value(flow, u, v);
        for edge in residual_edges_for(u, v, data.capacity, data.weight, f) {
            let src = if edge.forward { u } else { v };
            let dst = if edge.forward { v } else { u };
            residual.entry(src).or_default().insert(dst, edge);
        }
    }
}

/// Find all negative-cost cycles in the residual graph using Bellman-Ford.
///
/// Runs |V| passes; in the V-th pass, every node whose distance still
/// improves is part of a negative cycle.  Returns each cycle as a list
/// of residual edges.
fn find_all_neg_cycles_bf(
    residual: &HashMap<usize, HashMap<usize, ResidualEdge>>,
) -> Vec<Vec<ResidualEdge>> {
    let mut all_nodes: Vec<usize> = residual.keys().copied().collect();
    for row in residual.values() {
        for &v in row.keys() {
            if !all_nodes.contains(&v) {
                all_nodes.push(v);
            }
        }
    }
    all_nodes.sort_unstable();

    let n = all_nodes.len();
    if n == 0 {
        return vec![];
    }

    // Map node → index for fast distance lookups
    let node_to_idx: HashMap<usize, usize> =
        all_nodes.iter().enumerate().map(|(i, &n)| (n, i)).collect();

    // Bellman-Ford pass with edge tracking.
    // pred: node → (prev_node, edge)
    let mut pred: HashMap<usize, (usize, &ResidualEdge)> = HashMap::new();
    let mut dist = vec![0i64; n];
    let mut updated_in_last = vec![false; n];

    for pass in 0..n {
        let mut changed = false;
        updated_in_last.fill(false);

        for (&u, nbrs) in residual {
            let ui = node_to_idx[&u];
            let du = dist[ui];
            for (&v, edge) in nbrs {
                let vi = node_to_idx[&v];
                let nd = du + edge.cost;
                if nd < dist[vi] {
                    dist[vi] = nd;
                    pred.insert(v, (u, edge));
                    changed = true;
                    if pass == n - 1 {
                        updated_in_last[vi] = true;
                    }
                }
            }
        }

        if !changed && pass < n - 1 {
            return vec![];
        }
    }

    if !updated_in_last.iter().any(|&x| x) {
        return vec![];
    }

    let mut cycles: Vec<Vec<ResidualEdge>> = Vec::new();
    let mut yielded_orig: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    let mut visited_trace: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for &start_node in &all_nodes {
        let start_idx = node_to_idx[&start_node];
        if !updated_in_last[start_idx] {
            continue;
        }
        if visited_trace.contains(&start_node) {
            continue;
        }

        let mut trace_visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut u = start_node;
        while !trace_visited.contains(&u) {
            trace_visited.insert(u);
            match pred.get(&u) {
                Some(&(prev, _)) => u = prev,
                None => break,
            }
        }
        let cycle_start = u;

        let mut cycle_edges: Vec<ResidualEdge> = Vec::new();
        u = cycle_start;
        while let Some(&(prev, edge_ref)) = pred.get(&u) {
            let edge = edge_ref.clone();
            if !yielded_orig.contains(&edge.orig) {
                cycle_edges.push(edge);
            }
            u = prev;
            if u == cycle_start {
                break;
            }
        }
        cycle_edges.reverse();

        if !cycle_edges.is_empty() {
            let total_cost: i64 = cycle_edges.iter().map(|e| e.cost).sum();
            if total_cost < 0 {
                for e in &cycle_edges {
                    yielded_orig.insert(e.orig);
                    visited_trace.insert(e.orig.0);
                }
                cycles.push(cycle_edges);
            }
        }
    }

    cycles
}

/// Solve min-cost flow using cycle-cancellation descent.
///
/// Uses Bellman-Ford for negative-cycle detection.
///
/// # Arguments
///
/// * `g` — Graph as `{u: {v: Edge {weight, capacity}}}`.  All nodes must
///   appear as keys (even with an empty neighbour map).
/// * `demands` — `{node: demand}` where negative = supply, positive = demand.
///   Nodes not listed have demand 0.
///
/// # Returns
///
/// `Some((total_cost, flow_dict))` or `None` if the problem is infeasible.
pub fn cycle_canceling_mcf(
    g: &HashMap<usize, HashMap<usize, Edge>>,
    demands: &HashMap<usize, i64>,
) -> Option<(i64, FlowMap)> {
    let mut flow = find_feasible_flow(g, demands)?;

    let mut residual: HashMap<usize, HashMap<usize, ResidualEdge>> = HashMap::new();

    loop {
        if residual.is_empty() {
            residual = build_residual(g, &flow);
            if residual.is_empty() {
                break;
            }
        }

        let cycles = find_all_neg_cycles_bf(&residual);
        if cycles.is_empty() {
            break;
        }

        let mut cancelled = false;
        for cycle_edges in &cycles {
            let bottleneck: i64 = cycle_edges.iter().map(|e| e.capacity).min().unwrap_or(0);
            if bottleneck <= 0 {
                continue;
            }

            for edge in cycle_edges {
                let (u_orig, v_orig) = edge.orig;
                if edge.forward {
                    *flow.entry(u_orig).or_default().entry(v_orig).or_insert(0) += bottleneck;
                } else {
                    *flow.entry(u_orig).or_default().entry(v_orig).or_insert(0) -= bottleneck;
                }
            }

            let mut seen = std::collections::HashSet::new();
            for edge in cycle_edges {
                let (uo, vo) = edge.orig;
                if seen.insert((uo, vo)) {
                    update_residual_edge(&mut residual, g, &flow, uo, vo);
                }
            }

            cancelled = true;
            break; // Process one cycle at a time (restart B-F)
        }

        if !cancelled {
            break;
        }
    }

    let mut total_cost: i64 = 0;
    for (u, nbrs) in g.iter() {
        for (v, data) in nbrs.iter() {
            let f = flow_value(&flow, *u, *v);
            total_cost += f * data.weight;
        }
    }

    Some((total_cost, flow))
}

/// Build the spareTSV experiment graph (195 nodes, ~1684 edges).
///
/// Reproduces the graph from `spareTSV/experi-descent.py` with:
///   N=155 primal, M=40 spare, capacity=4, VdC bases (2,3), eta=1.6.
pub fn build_spare_tsv_graph() -> (HashMap<usize, HashMap<usize, Edge>>, HashMap<usize, i64>) {
    let (n, m) = (155usize, 40usize);
    let t = n + m;
    let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
    for i in 0..=t {
        g.insert(i, HashMap::new());
    }

    // Van der Corput positions (base 2 for x, base 3 for y)
    fn vdc(mut n: u32, base: u32) -> f64 {
        let mut v = 0.0;
        let mut denom = 1.0;
        while n > 0 {
            denom *= base as f64;
            let remainder = (n % base) as f64;
            n /= base;
            v += remainder / denom;
        }
        v
    }
    let pos: Vec<(f64, f64)> = (0..t)
        .map(|i| (vdc(i as u32, 2), vdc(i as u32, 3)))
        .collect();

    let n_int = (t as f64).sqrt() as usize;
    let eta = 1.6 / ((n_int - 1) as f64);

    for i in 0..t {
        for j in (i + 1)..t {
            let dx = pos[i].0 - pos[j].0;
            let dy = pos[i].1 - pos[j].1;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= eta {
                let w = (d * 100.0).trunc() as i64;
                g.get_mut(&i).unwrap().insert(j, Edge::new(w, 4));
                g.get_mut(&j).unwrap().insert(i, Edge::new(w, 4));
            }
        }
    }
    for i in n..t {
        g.get_mut(&i).unwrap().insert(t, Edge::new(0, 4));
    }

    let mut demands: HashMap<usize, i64> = HashMap::new();
    for i in 0..n {
        demands.insert(i, -1);
    }
    demands.insert(t, n as i64);
    (g, demands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_chain() {
        // 0 --1/5--> 1 --1/5--> 2
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(1, 5))].into());
        g.insert(1, [(2, Edge::new(1, 5))].into());
        g.insert(2, HashMap::new());

        let demands = [(0, -2), (2, 2)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, _flow) = result.unwrap();
        assert_eq!(cost, 4); // 2 × (1+1)
    }

    #[test]
    fn test_two_paths() {
        // Two parallel paths: 0→1→3 (cost 1+2=3) and 0→2→3 (cost 5+1=6)
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(1, 5)), (2, Edge::new(5, 5))].into());
        g.insert(1, [(3, Edge::new(2, 5))].into());
        g.insert(2, [(3, Edge::new(1, 5))].into());
        g.insert(3, HashMap::new());

        let demands = [(0, -2), (3, 2)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, flow) = result.unwrap();
        // Optimal: 2 units via 0→1→3 = 2 × (1+2) = 6
        assert_eq!(cost, 6);
        assert_eq!(flow[&0][&1], 2);
        assert_eq!(flow[&0][&2], 0);
    }

    #[test]
    fn test_negative_cycle_cancellation() {
        // Graph where a negative cycle reduces cost
        // 0 --10/3--> 1 --1/3--> 2 --1/5--> 3
        // |                     ↑
        // +-------1/5-----------+
        // 3 --(-8)/3--> 1  (cheap back-edge creating negative cycle)
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(10, 3)), (2, Edge::new(1, 5))].into());
        g.insert(1, [(2, Edge::new(1, 3))].into());
        g.insert(2, [(3, Edge::new(1, 5))].into());
        g.insert(3, [(1, Edge::new(-8, 3))].into());

        let demands = [(0, -3), (3, 3)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, _flow) = result.unwrap();
        // Python reference gives cost = -6
        // The cycle 1→2→3→1 has cost 1+1+(-8) = -6
        // Send 2 units around it to reduce cost
        assert_eq!(cost, -6);
    }

    #[test]
    fn test_cross_validate_with_python() {
        // Same test case as in the Python reference.
        // Python output: cost=6, flow = 0->1:2, 1->3:2
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(1, 5)), (2, Edge::new(5, 5))].into());
        g.insert(1, [(2, Edge::new(1, 5)), (3, Edge::new(2, 5))].into());
        g.insert(2, [(3, Edge::new(1, 5))].into());
        g.insert(3, HashMap::new());

        let demands = [(0, -2), (3, 2)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, _flow) = result.unwrap();
        // Total cost must match Python reference (6)
        assert_eq!(cost, 6);
    }

    #[test]
    fn test_spare_tsv_fixture() {
        // SpareTSV test fixture (12 nodes: 9 primal + 3 spare + sink).
        // Reproduced from spareTSV/tests/conftest.py:
        //   T=12, vdc base=(2,3), mu=0.12, eta=1.6, seed=5
        // Python reference: cost = 264
        let x_vals = [
            0.0, 0.5, 0.25, 0.75, 0.125, 0.625, 0.375, 0.875, 0.0625, 0.5625, 0.3125, 0.8125,
        ];
        let y_vals = [
            0.0,
            0.3333333333333333,
            0.6666666666666666,
            0.1111111111111111,
            0.4444444444444444,
            0.7777777777777777,
            0.2222222222222222,
            0.5555555555555556,
            0.8888888888888888,
            0.037037037037037035,
            0.37037037037037035,
            0.7037037037037037,
        ];
        let pos: Vec<(f64, f64)> = x_vals
            .iter()
            .zip(y_vals.iter())
            .map(|(&x, &y)| (x, y))
            .collect();

        let t = 12usize;
        let eta = 0.8f64;

        // Build graph: 12 nodes + sink (node 12)
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        for i in 0..=t {
            g.insert(i, HashMap::new());
        }

        // Add geometric edges (undirected pair -> both directions)
        for i in 0..t {
            for j in (i + 1)..t {
                let dx = pos[i].0 - pos[j].0;
                let dy = pos[i].1 - pos[j].1;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= eta {
                    let w = (dist * 100.0).trunc() as i64;
                    g.get_mut(&i).unwrap().insert(j, Edge::new(w, 4));
                    g.get_mut(&j).unwrap().insert(i, Edge::new(w, 4));
                }
            }
        }

        // Sink edges from spare nodes (9, 10, 11) to sink (12)
        for i in 9..t {
            g.get_mut(&i).unwrap().insert(t, Edge::new(0, 4));
        }

        // Demands: primal nodes (0-8) demand -1, sink (12) demand +9
        let mut demands: HashMap<usize, i64> = HashMap::new();
        for i in 0..9 {
            demands.insert(i, -1);
        }
        demands.insert(t, 9);

        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, flow) = result.unwrap();
        assert_eq!(cost, 264);
        // Verify flow assignment matches Python reference exactly
        assert_eq!(flow[&0][&9], 1);
        assert_eq!(flow[&1][&10], 1);
        assert_eq!(flow[&2][&10], 1);
        assert_eq!(flow[&3][&9], 1);
        assert_eq!(flow[&4][&10], 1);
        assert_eq!(flow[&5][&11], 1);
        assert_eq!(flow[&6][&9], 1);
        assert_eq!(flow[&7][&11], 1);
        assert_eq!(flow[&8][&10], 1);
        assert_eq!(flow[&9][&12], 3);
        assert_eq!(flow[&10][&12], 4);
        assert_eq!(flow[&11][&12], 2);
    }

    #[test]
    fn test_infeasible() {
        // Not enough capacity
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(1, 1))].into());
        g.insert(1, HashMap::new());

        let demands = [(0, -2), (1, 2)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_flow_needed() {
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(1, 5))].into());
        g.insert(1, HashMap::new());

        let demands = HashMap::new();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, _flow) = result.unwrap();
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_supply_equals_demand_single_edge() {
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(1, Edge::new(3, 10))].into());
        g.insert(1, HashMap::new());

        let demands = [(0, -5), (1, 5)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, flow) = result.unwrap();
        assert_eq!(cost, 15); // 5 × 3
        assert_eq!(flow[&0][&1], 5);
    }

    #[test]
    fn test_empty_graph() {
        let g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        let demands = HashMap::new();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, _flow) = result.unwrap();
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_multiple_supply_demand() {
        // 0 --1/5--> 2
        // 1 --2/5--> 2
        // 2 --3/5--> 3
        let mut g: HashMap<usize, HashMap<usize, Edge>> = HashMap::new();
        g.insert(0, [(2, Edge::new(1, 5))].into());
        g.insert(1, [(2, Edge::new(2, 5))].into());
        g.insert(2, [(3, Edge::new(3, 5))].into());
        g.insert(3, HashMap::new());

        let demands = [(0, -3), (1, -2), (3, 5)].into();
        let result = cycle_canceling_mcf(&g, &demands);
        assert!(result.is_some());
        let (cost, _flow) = result.unwrap();
        // Optimal: 3 × (1+3) + 2 × (2+3) = 12 + 10 = 22
        assert_eq!(cost, 22);
    }
}
