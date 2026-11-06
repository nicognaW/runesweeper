//! Exact mine odds: every arrangement the numbers allow, weighted by how many ways the
//! cells away from the numbers can hold the rest of the mines.
//!
//! Hidden cells touched by exactly the same numbers form a box; their order inside the box
//! doesn't matter, only how many mines it holds. Boxes linked by numbers form independent
//! components, each enumerated alone and combined by mine count.

use super::{Fact, View, constraints};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// No arrangement fits the numbers and the mine count.
    Inconsistent,
    /// A component has too many arrangements to enumerate.
    TooBig,
}

/// Enumeration nodes allowed per call before giving up.
pub const NODE_LIMIT: u64 = 4_000_000;

#[derive(Clone, Debug)]
pub struct Odds {
    /// Chance each cell is a mine (0 for open cells, 1 for proven mines).
    pub p: Vec<f64>,
    /// Safe in every arrangement.
    pub safe: Vec<bool>,
    /// A mine in every arrangement.
    pub mine: Vec<bool>,
    /// Unresolved and touching no open number.
    pub interior: Vec<bool>,
    /// The component of each frontier cell.
    pub comp: Vec<Option<usize>>,
    /// The open numbers of each component.
    pub numbers: Vec<Vec<usize>>,
}

impl Odds {
    /// The numbers whose arrangements decide cell `i`.
    pub fn component_numbers(&self, _v: &View, i: usize) -> Vec<usize> {
        self.comp[i].map(|c| self.numbers[c].clone()).unwrap_or_default()
    }
}

struct Box_ {
    cells: Vec<usize>,
    cons: Vec<usize>,
}

/// Per component, by mine count m: total weight, and per box the weight-summed mines, the
/// weight with at least one mine, and the weight with at least one safe cell.
struct Dist {
    w: Vec<f64>,
    s: Vec<Vec<f64>>,
    some: Vec<Vec<f64>>,
    gap: Vec<Vec<f64>>,
}

pub fn odds(v: &View, facts: &[Fact]) -> Result<Odds, Error> {
    let n = v.len();
    // Open numbers already satisfied by proven facts must be exactly satisfied.
    for i in 0..n {
        let Some(k) = v.open[i] else { continue };
        let mines = v.nbrs(i).iter().filter(|&&j| v.open[j].is_none() && facts[j] == Fact::Mine).count();
        let unknown = v.nbrs(i).iter().filter(|&&j| v.open[j].is_none() && facts[j] == Fact::Unknown).count();
        if mines > k as usize || mines + unknown < k as usize {
            return Err(Error::Inconsistent);
        }
    }
    let known = facts.iter().enumerate().filter(|&(i, &f)| f == Fact::Mine && v.open[i].is_none()).count();
    if known > v.mines {
        return Err(Error::Inconsistent);
    }
    let left = v.mines - known;
    let cs = constraints(v, facts);

    // Boxes: frontier cells keyed by the constraints that touch them.
    let mut touch: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (k, c) in cs.iter().enumerate() {
        for &cell in &c.cells {
            touch[cell].push(k);
        }
    }
    let mut boxes: Vec<Box_> = Vec::new();
    let mut key: std::collections::HashMap<&[usize], usize> = std::collections::HashMap::new();
    for cell in 0..n {
        if touch[cell].is_empty() {
            continue;
        }
        let b = *key.entry(&touch[cell][..]).or_insert_with(|| {
            boxes.push(Box_ { cells: Vec::new(), cons: touch[cell].clone() });
            boxes.len() - 1
        });
        boxes[b].cells.push(cell);
    }
    let mut con_boxes: Vec<Vec<usize>> = vec![Vec::new(); cs.len()];
    for (b, bx) in boxes.iter().enumerate() {
        for &c in &bx.cons {
            con_boxes[c].push(b);
        }
    }

    // Components, each with its boxes in breadth-first order so numbers close early.
    let mut comp_of_box = vec![usize::MAX; boxes.len()];
    let mut comps: Vec<Vec<usize>> = Vec::new();
    for start in 0..boxes.len() {
        if comp_of_box[start] != usize::MAX {
            continue;
        }
        let id = comps.len();
        let mut order = vec![start];
        comp_of_box[start] = id;
        let mut q = 0;
        while q < order.len() {
            let b = order[q];
            q += 1;
            for &c in &boxes[b].cons {
                for &nb in &con_boxes[c] {
                    if comp_of_box[nb] == usize::MAX {
                        comp_of_box[nb] = id;
                        order.push(nb);
                    }
                }
            }
        }
        comps.push(order);
    }

    let mut nodes = 0u64;
    let dists: Vec<Dist> = comps.iter().map(|order| enumerate(order, &boxes, &cs, &mut nodes)).collect::<Result<_, _>>()?;

    let unknown: Vec<usize> = (0..n).filter(|&i| v.open[i].is_none() && facts[i] == Fact::Unknown).collect();
    let interior: Vec<usize> = unknown.iter().copied().filter(|&i| touch[i].is_empty()).collect();
    let ni = interior.len();

    // Ways to put r mines in the interior, scaled so the largest feasible one is 1.
    let max_front: usize = dists.iter().map(|d| d.w.len() - 1).sum();
    let r_lo = left.saturating_sub(max_front);
    let r_hi = left.min(ni);
    if r_lo > r_hi {
        return Err(Error::Inconsistent);
    }
    let lnf = ln_factorials(ni + 1);
    let ln_c = |n: usize, k: usize| lnf[n] - lnf[k] - lnf[n - k];
    let scale = (r_lo..=r_hi).map(|r| ln_c(ni, r)).fold(f64::NEG_INFINITY, f64::max);
    let interior_ways = |r: isize, n: usize| -> f64 {
        if r < 0 || r as usize > n {
            return 0.0;
        }
        (ln_c(n, r as usize) - scale).exp()
    };
    let f = |m: usize| interior_ways(left as isize - m as isize, ni);

    let all = convolve(dists.iter().map(|d| &d.w[..]));
    let total: f64 = all.iter().enumerate().map(|(m, &w)| w * f(m)).sum();
    if !(total > 0.0) {
        return Err(Error::Inconsistent);
    }

    let mut out = Odds {
        p: vec![0.0; n],
        safe: vec![false; n],
        mine: vec![false; n],
        interior: vec![false; n],
        comp: vec![None; n],
        numbers: comps.iter().map(|order| {
            let mut v: Vec<usize> = order.iter().flat_map(|&b| boxes[b].cons.iter().map(|&c| cs[c].at)).collect();
            v.sort_unstable();
            v.dedup();
            v
        }).collect(),
    };
    for i in 0..n {
        if v.open[i].is_none() && facts[i] == Fact::Mine {
            out.p[i] = 1.0;
            out.mine[i] = true;
        } else if v.open[i].is_none() && facts[i] == Fact::Safe {
            out.safe[i] = true;
        } else if v.open[i].is_some() {
            out.safe[i] = true;
        }
    }

    for (c, order) in comps.iter().enumerate() {
        let rest = convolve(dists.iter().enumerate().filter(|&(k, _)| k != c).map(|(_, d)| &d.w[..]));
        let d = &dists[c];
        // Weight of everything outside this component, for each count inside it.
        let g: Vec<f64> = (0..d.w.len()).map(|mc| rest.iter().enumerate().map(|(mr, &w)| w * f(mc + mr)).sum()).collect();
        for (j, &b) in order.iter().enumerate() {
            let size = boxes[b].cells.len() as f64;
            let p: f64 = (0..d.w.len()).map(|m| d.s[j][m] * g[m]).sum::<f64>() / size / total;
            let some: f64 = (0..d.w.len()).map(|m| d.some[j][m] * g[m]).sum();
            let gap: f64 = (0..d.w.len()).map(|m| d.gap[j][m] * g[m]).sum();
            for &cell in &boxes[b].cells {
                out.p[cell] = p.clamp(0.0, 1.0);
                out.safe[cell] = some == 0.0;
                out.mine[cell] = gap == 0.0;
                out.comp[cell] = Some(c);
            }
        }
    }

    if ni > 0 {
        let mined: f64 = all.iter().enumerate().map(|(m, &w)| w * interior_ways(left as isize - m as isize - 1, ni - 1)).sum();
        let clear: f64 = all.iter().enumerate().map(|(m, &w)| w * interior_ways(left as isize - m as isize, ni - 1)).sum();
        // Both are scaled like `total`; their ratio is the chance an interior cell is a mine.
        let p = mined / (mined + clear);
        for &cell in &interior {
            out.p[cell] = p.clamp(0.0, 1.0);
            out.safe[cell] = mined == 0.0;
            out.mine[cell] = clear == 0.0;
            out.interior[cell] = true;
        }
    }
    Ok(out)
}

fn enumerate(order: &[usize], boxes: &[Box_], cs: &[super::Constraint], nodes: &mut u64) -> Result<Dist, Error> {
    let max: usize = order.iter().map(|&b| boxes[b].cells.len()).sum();
    let nb = order.len();
    let mut d = Dist { w: vec![0.0; max + 1], s: vec![vec![0.0; max + 1]; nb], some: vec![vec![0.0; max + 1]; nb], gap: vec![vec![0.0; max + 1]; nb] };
    // Remaining need and remaining unassigned capacity of each constraint touched here.
    let mut need: std::collections::HashMap<usize, (isize, isize)> = std::collections::HashMap::new();
    for &b in order {
        for &c in &boxes[b].cons {
            let e = need.entry(c).or_insert((cs[c].need as isize, 0));
            e.1 += boxes[b].cells.len() as isize;
        }
    }
    let cons: Vec<Vec<usize>> = order.iter().map(|&b| boxes[b].cons.clone()).collect();
    let sizes: Vec<usize> = order.iter().map(|&b| boxes[b].cells.len()).collect();
    // Dense constraint state for speed.
    let ids: Vec<usize> = need.keys().copied().collect();
    let local: std::collections::HashMap<usize, usize> = ids.iter().enumerate().map(|(k, &c)| (c, k)).collect();
    let mut st: Vec<(isize, isize)> = ids.iter().map(|c| need[c]).collect();
    let cons: Vec<Vec<usize>> = cons.iter().map(|v| v.iter().map(|c| local[c]).collect()).collect();
    let mut ks = vec![0usize; nb];
    rec(0, 1.0, 0, &sizes, &cons, &mut st, &mut ks, &mut d, nodes)?;
    Ok(d)
}

#[allow(clippy::too_many_arguments)]
fn rec(j: usize, w: f64, m: usize, sizes: &[usize], cons: &[Vec<usize>], st: &mut [(isize, isize)], ks: &mut [usize], d: &mut Dist, nodes: &mut u64) -> Result<(), Error> {
    *nodes += 1;
    if *nodes > NODE_LIMIT {
        return Err(Error::TooBig);
    }
    if j == sizes.len() {
        d.w[m] += w;
        for (b, &k) in ks.iter().enumerate() {
            d.s[b][m] += w * k as f64;
            if k > 0 {
                d.some[b][m] += w;
            }
            if k < sizes[b] {
                d.gap[b][m] += w;
            }
        }
        return Ok(());
    }
    let size = sizes[j] as isize;
    // The range of mines this box may take, from every number touching it.
    let mut lo = 0isize;
    let mut hi = size;
    for &c in &cons[j] {
        let (need, cap) = st[c];
        hi = hi.min(need);
        lo = lo.max(need - (cap - size));
    }
    if lo > hi {
        return Ok(());
    }
    for c in &cons[j] {
        st[*c].1 -= size;
    }
    for k in lo..=hi {
        for c in &cons[j] {
            st[*c].0 -= k;
        }
        ks[j] = k as usize;
        rec(j + 1, w * binom(sizes[j], k as usize), m + k as usize, sizes, cons, st, ks, d, nodes)?;
        for c in &cons[j] {
            st[*c].0 += k;
        }
    }
    for c in &cons[j] {
        st[*c].1 += size;
    }
    Ok(())
}

fn binom(n: usize, k: usize) -> f64 {
    let mut r = 1.0;
    for i in 0..k {
        r = r * (n - i) as f64 / (i + 1) as f64;
    }
    r
}

fn ln_factorials(n: usize) -> Vec<f64> {
    let mut v = vec![0.0; n + 1];
    for i in 1..=n {
        v[i] = v[i - 1] + (i as f64).ln();
    }
    v
}

fn convolve<'a>(parts: impl Iterator<Item = &'a [f64]>) -> Vec<f64> {
    let mut acc = vec![1.0];
    for p in parts {
        let mut next = vec![0.0; acc.len() + p.len() - 1];
        for (a, &x) in acc.iter().enumerate() {
            if x == 0.0 {
                continue;
            }
            for (b, &y) in p.iter().enumerate() {
                next[a + b] += x * y;
            }
        }
        acc = next;
    }
    acc
}
