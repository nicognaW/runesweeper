//! Deductions from any number of numbers, and from the count of mines left: which cells a set
//! of numbers settles, the fewest numbers that settle a cell, and the reason as steps a player
//! can follow (a chain one or two numbers at a time, or a supposition that breaks a number).
//!
//! Up to [`EXHAUSTIVE`] numbers, every connected set of numbers is tried, smallest first, so
//! the hint reads the fewest numbers there are. Past that, a cell perfect logic proves keeps
//! only the numbers it can't do without: each is dropped in turn while the cell stays settled.

use std::collections::HashMap;

use super::{Step, Technique, digit, pair_one};
use crate::solver::{Constraint, Fact, View, exact};

/// Every connected set of up to this many numbers is tried.
pub(super) const EXHAUSTIVE: usize = 4;
/// Sets of numbers tried at most, so a hint stays quick.
const SETS: usize = 30_000;
/// Arrangements tried at most for one set of numbers.
const NODES: u64 = 1_000_000;

/// The mine count as one more fact: `left` mines to find among `hidden` cells, the cells the
/// numbers touch included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Count {
    pub left: usize,
    pub hidden: usize,
}

/// A cell some numbers settle, perhaps with the mine count.
#[derive(Clone, Debug)]
pub(super) struct Settles {
    /// The numbers, as the cells they still need mines from.
    pub cons: Vec<Constraint>,
    pub cell: usize,
    pub mine: bool,
    pub count: Option<Count>,
}

impl Settles {
    pub fn numbers(&self) -> Vec<usize> {
        self.cons.iter().map(|c| c.at).collect()
    }
}

/// What a set of numbers allows.
struct Settled {
    /// The hidden cells the numbers touch, ascending.
    cells: Vec<usize>,
    mine: Vec<bool>,
    safe: Vec<bool>,
    /// With the count: the hidden cells no number here touches, and whether they are all
    /// mines or all safe.
    others: usize,
    others_mine: bool,
    others_safe: bool,
    /// The fewest and most mines the numbers' cells hold, the count aside.
    min: usize,
    max: usize,
}

impl Settled {
    /// Whether `cell` (hidden) is a mine in every arrangement, safe in every one, or neither.
    fn verdict(&self, cell: usize) -> Option<bool> {
        let (mine, safe) = match self.cells.binary_search(&cell) {
            Ok(k) => (self.mine[k], self.safe[k]),
            Err(_) => (self.others_mine, self.others_safe),
        };
        if mine { Some(true) } else if safe { Some(false) } else { None }
    }
}

/// Sets of booleans by mine count: which totals two parts can make together.
fn sum(a: &[bool], b: &[bool]) -> Vec<bool> {
    let mut out = vec![false; a.len() + b.len() - 1];
    for (i, _) in a.iter().enumerate().filter(|p| *p.1) {
        for (j, _) in b.iter().enumerate().filter(|p| *p.1) {
            out[i + j] = true;
        }
    }
    out
}

/// Every arrangement of mines the numbers `cs` allow (and the count, if given), summed up.
/// Cells touched by the same numbers form a box, and only how many mines a box holds matters;
/// boxes linked by numbers are enumerated together, and the groups combined by mine count.
fn settle(cs: &[Constraint], count: Option<Count>) -> Result<Settled, exact::Error> {
    let mut cells: Vec<usize> = cs.iter().flat_map(|c| c.cells.iter().copied()).collect();
    cells.sort_unstable();
    cells.dedup();
    let mut boxes: Vec<(Vec<usize>, Vec<usize>)> = Vec::new(); // (cell positions in `cells`, numbers)
    for (k, cell) in cells.iter().enumerate() {
        let on: Vec<usize> = (0..cs.len()).filter(|&j| cs[j].cells.binary_search(cell).is_ok()).collect();
        match boxes.iter_mut().find(|b| b.1 == on) {
            Some(b) => b.0.push(k),
            None => boxes.push((vec![k], on)),
        }
    }
    // Groups of numbers linked through boxes.
    let mut group: Vec<usize> = (0..cs.len()).collect();
    fn root(g: &mut [usize], i: usize) -> usize {
        if g[i] == i { i } else { let r = root(g, g[i]); g[i] = r; r }
    }
    for (_, on) in &boxes {
        for w in on.windows(2) {
            let (a, b) = (root(&mut group, w[0]), root(&mut group, w[1]));
            group[a] = b;
        }
    }
    let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
    for (b, (_, on)) in boxes.iter().enumerate() {
        let r = root(&mut group, on[0]);
        match groups.iter_mut().find(|g| g.0 == r) {
            Some(g) => g.1.push(b),
            None => groups.push((r, vec![b])),
        }
    }
    let groups: Vec<Vec<usize>> = groups.into_iter().map(|g| g.1).collect();

    // Per group, by mine count: each box's fewest and most mines.
    let mut nodes = 0;
    let mut tables = Vec::new();
    for g in &groups {
        let mut table: Vec<Option<Vec<(usize, usize)>>> = vec![None; g.iter().map(|&b| boxes[b].0.len()).sum::<usize>() + 1];
        let mut room = vec![0; cs.len()];
        for &b in g {
            for &c in &boxes[b].1 {
                room[c] += boxes[b].0.len();
            }
        }
        let need: Vec<usize> = cs.iter().map(|c| c.need).collect();
        let mut placed = vec![0; cs.len()];
        let mut held = vec![0; g.len()];
        enumerate(&boxes, g, 0, &need, &mut placed, &mut room, &mut held, &mut table, &mut nodes)?;
        if table.iter().all(Option::is_none) {
            return Err(exact::Error::Inconsistent);
        }
        tables.push(table);
    }
    let totals: Vec<Vec<bool>> = tables.iter().map(|t| t.iter().map(Option::is_some).collect()).collect();
    let all = totals.iter().fold(vec![true], |acc, t| sum(&acc, t));
    let others = count.map_or(0, |c| c.hidden.saturating_sub(cells.len()));
    let (lo, hi) = count.map_or((0, usize::MAX), |c| (c.left.saturating_sub(others), c.left));
    let fits = |g: usize| lo <= g && g <= hi;
    let feasible: Vec<usize> = (0..all.len()).filter(|&g| all[g] && fits(g)).collect();
    if feasible.is_empty() {
        return Err(exact::Error::Inconsistent);
    }
    let (mut mine, mut safe) = (vec![false; cells.len()], vec![false; cells.len()]);
    for (i, g) in groups.iter().enumerate() {
        let rest = totals.iter().enumerate().filter(|&(j, _)| j != i).fold(vec![true], |acc, (_, t)| sum(&acc, t));
        let mut range: Vec<(usize, usize)> = vec![(usize::MAX, 0); g.len()];
        for (m, row) in tables[i].iter().enumerate() {
            let Some(row) = row else { continue };
            if count.is_some() && !(0..rest.len()).any(|s| rest[s] && fits(m + s)) {
                continue;
            }
            for (r, &(a, b)) in range.iter_mut().zip(row) {
                *r = (r.0.min(a), r.1.max(b));
            }
        }
        for (k, &b) in g.iter().enumerate() {
            let size = boxes[b].0.len();
            for &p in &boxes[b].0 {
                mine[p] = range[k].0 == size;
                safe[p] = range[k].1 == 0;
            }
        }
    }
    let left = count.map_or(0, |c| c.left);
    Ok(Settled {
        cells,
        mine,
        safe,
        others,
        others_safe: count.is_some() && others > 0 && feasible.iter().all(|&g| g == left),
        others_mine: count.is_some() && others > 0 && feasible.iter().all(|&g| g + others == left),
        min: (0..all.len()).find(|&g| all[g]).unwrap_or(0),
        max: (0..all.len()).rev().find(|&g| all[g]).unwrap_or(0),
    })
}

/// Tries every count of mines for box `g[i]` and on, keeping each number within reach of
/// what it needs, and records each full arrangement in `table` by its total.
#[allow(clippy::too_many_arguments)]
fn enumerate(
    boxes: &[(Vec<usize>, Vec<usize>)],
    g: &[usize],
    i: usize,
    need: &[usize],
    placed: &mut [usize],
    room: &mut [usize],
    held: &mut [usize],
    table: &mut [Option<Vec<(usize, usize)>>],
    nodes: &mut u64,
) -> Result<(), exact::Error> {
    *nodes += 1;
    if *nodes > NODES {
        return Err(exact::Error::TooBig);
    }
    if i == g.len() {
        let total = held.iter().sum::<usize>();
        let row = table[total].get_or_insert_with(|| held.iter().map(|&h| (h, h)).collect());
        for (r, &h) in row.iter_mut().zip(held.iter()) {
            *r = (r.0.min(h), r.1.max(h));
        }
        return Ok(());
    }
    let (cells, on) = &boxes[g[i]];
    let size = cells.len();
    for &c in on {
        room[c] -= size;
    }
    for k in 0..=size {
        if on.iter().all(|&c| placed[c] + k <= need[c] && placed[c] + k + room[c] >= need[c]) {
            for &c in on {
                placed[c] += k;
            }
            held[i] = k;
            let r = enumerate(boxes, g, i + 1, need, placed, room, held, table, nodes);
            for &c in on {
                placed[c] -= k;
            }
            if r.is_err() {
                for &c in on {
                    room[c] += size;
                }
                return r;
            }
        }
    }
    for &c in on {
        room[c] += size;
    }
    Ok(())
}

/// Numbers sharing a hidden cell, by index.
fn links(cs: &[Constraint]) -> Vec<Vec<usize>> {
    let mut by_cell: HashMap<usize, Vec<usize>> = HashMap::new();
    for (k, c) in cs.iter().enumerate() {
        for &cell in &c.cells {
            by_cell.entry(cell).or_default().push(k);
        }
    }
    let mut adj = vec![Vec::new(); cs.len()];
    for ks in by_cell.values() {
        for &a in ks {
            for &b in ks {
                if a != b && !adj[a].contains(&b) {
                    adj[a].push(b);
                }
            }
        }
    }
    for a in &mut adj {
        a.sort_unstable();
    }
    adj
}

/// Visits every connected set of `k` numbers once (Wernicke's ESU), until `visit` says stop.
fn connected(adj: &[Vec<usize>], k: usize, visit: &mut dyn FnMut(&[usize]) -> bool) {
    fn extend(adj: &[Vec<usize>], sub: &mut Vec<usize>, mut ext: Vec<usize>, root: usize, k: usize, visit: &mut dyn FnMut(&[usize]) -> bool) -> bool {
        if sub.len() == k {
            return visit(sub);
        }
        while let Some(w) = ext.pop() {
            let mut next = ext.clone();
            for &u in &adj[w] {
                if u > root && !sub.contains(&u) && !next.contains(&u) && !sub.iter().any(|&s| adj[s].binary_search(&u).is_ok()) {
                    next.push(u);
                }
            }
            sub.push(w);
            let go = extend(adj, sub, next, root, k, visit);
            sub.pop();
            if !go {
                return false;
            }
        }
        true
    }
    for root in 0..adj.len() {
        let ext = adj[root].iter().copied().filter(|&u| u > root).collect();
        if !extend(adj, &mut vec![root], ext, root, k, visit) {
            return;
        }
    }
}

/// The cells worth naming that the fewest numbers settle, trying every connected set of 1, 2,
/// ... `upto` numbers and stopping at the first size that settles any. Numbers showing 0 are
/// left out: in play they never border a hidden cell.
pub(super) fn fewest(cs: &[Constraint], upto: usize, nameable: &dyn Fn(usize, bool) -> bool) -> Vec<Settles> {
    let cs: Vec<Constraint> = cs.iter().filter(|c| c.need > 0).cloned().collect();
    let adj = links(&cs);
    let mut tried = 0;
    for k in 1..=upto {
        let mut out = Vec::new();
        connected(&adj, k, &mut |set| {
            tried += 1;
            let mut set = set.to_vec();
            set.sort_unstable();
            let cons: Vec<Constraint> = set.iter().map(|&i| cs[i].clone()).collect();
            if let Ok(s) = settle(&cons, None) {
                for &cell in &s.cells {
                    if let Some(mine) = s.verdict(cell).filter(|&m| nameable(cell, m)) {
                        out.push(Settles { cons: cons.clone(), cell, mine, count: None });
                    }
                }
            }
            tried < SETS
        });
        if !out.is_empty() || tried >= SETS {
            return out;
        }
    }
    Vec::new()
}

/// When no few numbers settle anything: for each group of linked numbers, the cell perfect
/// logic proves nearest the last move, with only the numbers it needs; failing that, the
/// nearest cell the mine count settles, with the mines the player has rightly flagged taken
/// as found.
pub(super) fn deep(v: &View, cs: &[Constraint], flags: &[bool], nameable: &dyn Fn(usize, bool) -> bool, near: &dyn Fn(usize) -> usize) -> Vec<Settles> {
    let cs: Vec<Constraint> = cs.iter().filter(|c| c.need > 0).cloned().collect();
    let adj = links(&cs);
    let mut seen = vec![false; cs.len()];
    let mut out = Vec::new();
    for start in 0..cs.len() {
        if seen[start] {
            continue;
        }
        let mut group = vec![start];
        seen[start] = true;
        let mut k = 0;
        while k < group.len() {
            for &u in &adj[group[k]] {
                if !seen[u] {
                    seen[u] = true;
                    group.push(u);
                }
            }
            k += 1;
        }
        group.sort_unstable();
        let cons: Vec<Constraint> = group.iter().map(|&i| cs[i].clone()).collect();
        let Ok(s) = settle(&cons, None) else { continue };
        let Some((cell, mine)) = s.cells.iter().filter_map(|&i| s.verdict(i).filter(|&m| nameable(i, m)).map(|m| (i, m))).min_by_key(|&(i, _)| (near(i), i)) else { continue };
        out.extend(fewer(v, cons, cell, mine, None));
    }
    if !out.is_empty() {
        return out;
    }
    let Ok(odds) = exact::odds(v, &vec![Fact::Unknown; v.len()]) else { return out };
    let known: Vec<usize> = (0..v.len()).filter(|&i| flags[i] && odds.mine[i]).collect();
    let cons: Vec<Constraint> = cs
        .iter()
        .filter_map(|c| {
            let cells: Vec<usize> = c.cells.iter().copied().filter(|x| !known.contains(x)).collect();
            let need = c.need.saturating_sub(c.cells.len() - cells.len());
            (!cells.is_empty()).then_some(Constraint { at: c.at, cells, need })
        })
        .collect();
    let hidden = (0..v.len()).filter(|&i| v.open[i].is_none() && !known.contains(&i)).count();
    let count = Count { left: v.mines - known.len(), hidden };
    let proven = |i: usize| v.open[i].is_none() && (odds.safe[i] || odds.mine[i]) && nameable(i, odds.mine[i]);
    let Some(cell) = (0..v.len()).filter(|&i| proven(i)).min_by_key(|&i| (near(i), i)) else { return out };
    out.extend(fewer(v, cons, cell, odds.mine[cell], Some(count)));
    out
}

/// `cons` settling `cell`, less every number it can do without, the farthest tried first.
fn fewer(v: &View, mut cons: Vec<Constraint>, cell: usize, mine: bool, count: Option<Count>) -> Option<Settles> {
    let holds = |cons: &[Constraint]| settle(cons, count).ok().and_then(|s| s.verdict(cell)) == Some(mine);
    if !holds(&cons) {
        return None;
    }
    let far = |at: usize| {
        let (x, y) = ((at % v.w) as isize - (cell % v.w) as isize, (at / v.w) as isize - (cell / v.w) as isize);
        x * x + y * y
    };
    let mut order: Vec<usize> = cons.iter().map(|c| c.at).collect();
    order.sort_by_key(|&at| (std::cmp::Reverse(far(at)), at));
    for at in order {
        let less: Vec<Constraint> = cons.iter().filter(|c| c.at != at).cloned().collect();
        if holds(&less) {
            cons = less;
        }
    }
    Some(Settles { cons, cell, mine, count })
}

/// One derivation in a chain: its lines, the cells whose facts it reads, the cells it settles.
struct Unit {
    steps: Vec<Step>,
    reads: Vec<usize>,
    gives: Vec<usize>,
}

/// A number with what is settled taken out: its unsettled cells, the mines it still needs
/// (negative if it has too many), and the settled cells it touches.
fn now(c: &Constraint, facts: &HashMap<usize, bool>) -> (Vec<usize>, isize, Vec<usize>) {
    let (read, open): (Vec<usize>, Vec<usize>) = c.cells.iter().partition(|x| facts.contains_key(x));
    let mines = read.iter().filter(|x| facts[x]).count();
    (open, c.need as isize - mines as isize, read)
}

/// Settles cells of `cons` one number or two at a time from `facts`, until `cell` is settled.
/// Returns the derivations in order, and the number that breaks if the facts can't all hold.
fn derive(v: &View, cons: &[Constraint], facts: &mut HashMap<usize, bool>, cell: usize) -> (Vec<Unit>, Option<Unit>) {
    let mut units = Vec::new();
    'more: while !facts.contains_key(&cell) {
        for c in cons {
            let (open, need, read) = now(c, facts);
            if need < 0 || need as usize > open.len() {
                return (units, Some(Unit { steps: vec![Step::Broken { at: c.at, n: digit(v, c.at), over: need < 0 }], reads: read, gives: Vec::new() }));
            }
            if !open.is_empty() && (need == 0 || need as usize == open.len()) {
                let mine = need > 0;
                for &x in &open {
                    facts.insert(x, mine);
                }
                units.push(Unit { steps: vec![Step::Now { at: c.at, n: digit(v, c.at), need: need as usize, cells: open.clone(), mine }], reads: read, gives: open });
                continue 'more;
            }
        }
        for a in cons {
            for b in cons.iter().filter(|b| b.at != a.at) {
                let ((oa, na, ra), (ob, nb, rb)) = (now(a, facts), now(b, facts));
                let ka = a.need - na as usize;
                let kb = b.need - nb as usize;
                let (ca, cb) = (Constraint { at: a.at, cells: oa, need: na as usize }, Constraint { at: b.at, cells: ob, need: nb as usize });
                let Some((_, cells, mine, two)) = pair_one(v, &ca, &cb) else { continue };
                let mut steps = Vec::new();
                for (c, k) in [(a, ka), (b, kb)] {
                    if k > 0 {
                        steps.push(Step::Reduced { at: c.at, n: digit(v, c.at), known: k });
                    }
                }
                steps.extend(two);
                for &x in &cells {
                    facts.insert(x, mine);
                }
                units.push(Unit { steps, reads: [ra, rb].concat(), gives: cells });
                continue 'more;
            }
        }
        break;
    }
    (units, None)
}

/// The derivations the cells in `need` rest on, in order.
fn rests_on(units: Vec<Unit>, mut need: Vec<usize>) -> Vec<Step> {
    let mut kept = Vec::new();
    for u in units.into_iter().rev() {
        if u.gives.iter().any(|g| need.contains(g)) {
            need.extend(&u.reads);
            kept.push(u.steps);
        }
    }
    kept.into_iter().rev().flatten().collect()
}

/// The open numbers lines name, in reading order.
fn read_order(steps: &[Step], all: &[usize]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for s in steps {
        let ats = match *s {
            Step::Now { at, .. } | Step::Reduced { at, .. } | Step::Rest { at, .. } | Step::Broken { at, .. } => vec![at],
            Step::Shared { at, other, .. } => vec![at, other],
            _ => Vec::new(),
        };
        for at in ats {
            if !out.contains(&at) {
                out.push(at);
            }
        }
    }
    for &a in all {
        if !out.contains(&a) {
            out.push(a);
        }
    }
    out
}

/// The reason `s` holds, as the technique, the numbers in reading order and the lines: a
/// chain if one or two numbers at a time get there, else a supposition that breaks a number,
/// or the count of mines left.
pub(super) fn explain(v: &View, s: &Settles) -> (Technique, Vec<usize>, Vec<Step>) {
    let all = s.numbers();
    if let Some(count) = s.count {
        let steps = counted(s, count);
        let technique = if matches!(steps.first(), Some(Step::Suppose { .. })) { Technique::Combinations } else { Technique::Count };
        return (technique, all, steps);
    }
    let mut facts = HashMap::new();
    let (units, _) = derive(v, &s.cons, &mut facts, s.cell);
    if facts.get(&s.cell) == Some(&s.mine) {
        let steps = rests_on(units, vec![s.cell]);
        return (Technique::Chain, read_order(&steps, &all), steps);
    }
    let mut facts = HashMap::from([(s.cell, !s.mine)]);
    let (units, broken) = derive(v, &s.cons, &mut facts, usize::MAX);
    let mut steps = vec![Step::Suppose { cell: s.cell, mine: !s.mine }];
    match broken {
        Some(b) => {
            steps.extend(rests_on(units, b.reads));
            steps.extend(b.steps);
        }
        None => steps.push(Step::NoRoom { count: false }),
    }
    steps.push(Step::So { mine: s.mine });
    (Technique::Suppose, read_order(&steps, &all), steps)
}

/// The lines of a reason that rests on the count of mines left.
fn counted(s: &Settles, count: Count) -> Vec<Step> {
    let numbers = s.cons.len();
    let Ok(all) = settle(&s.cons, Some(count)) else { return vec![Step::NoRoom { count: true }] };
    if all.cells.binary_search(&s.cell).is_err() {
        return vec![if s.mine {
            Step::CountRest { left: count.left, held: all.max, numbers }
        } else {
            Step::CountAll { left: count.left, numbers }
        }];
    }
    // Inside the numbers: suppose the other way, and the count no longer fits.
    let mut fixed = s.cons.clone();
    fixed.push(Constraint { at: usize::MAX, cells: vec![s.cell], need: !s.mine as usize });
    let line = match settle(&fixed, None) {
        Ok(f) if f.min > count.left => Step::CountOver { need: f.min, left: count.left },
        Ok(f) if f.max + all.others < count.left => Step::CountUnder { most: f.max + all.others, left: count.left },
        Ok(_) => Step::NoRoom { count: true },
        Err(_) => Step::NoRoom { count: false },
    };
    vec![Step::Suppose { cell: s.cell, mine: !s.mine }, line, Step::So { mine: s.mine }]
}
