//! Reasoning from what a player can see: which cells are proven safe or mined, by which
//! numbers, and how likely each cell is to hide a mine. No drawing, no game rules beyond
//! the numbers; the generators, the hint and the benchmark all build on it.

pub mod bench;
pub mod exact;
pub mod fifty;
pub mod play;

use crate::game::{Cover, Game};

/// The visible field: each cell's number once it is open.
#[derive(Clone, Debug)]
pub struct View {
    pub w: usize,
    pub h: usize,
    pub mines: usize,
    pub open: Vec<Option<u8>>,
    adj: Vec<Vec<usize>>,
}

impl View {
    pub fn new(w: usize, h: usize, mines: usize) -> View {
        let adj = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as isize, (i / w) as isize);
                let mut v = Vec::with_capacity(8);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if (dx, dy) != (0, 0) && nx >= 0 && ny >= 0 && nx < w as isize && ny < h as isize {
                            v.push(ny as usize * w + nx as usize);
                        }
                    }
                }
                v
            })
            .collect();
        View { w, h, mines, open: vec![None; w * h], adj }
    }

    /// What the player sees of a game. Flags are ignored: they are the player's guesses,
    /// not facts.
    pub fn of(game: &Game) -> View {
        let mut v = View::new(game.w, game.h, game.mines);
        for (i, c) in game.cells.iter().enumerate() {
            if c.cover == Cover::Open && !c.mine {
                v.open[i] = Some(c.adj);
            }
        }
        v
    }

    pub fn nbrs(&self, i: usize) -> &[usize] {
        &self.adj[i]
    }

    pub fn len(&self) -> usize {
        self.w * self.h
    }
}

/// What has been proven about a cell that is not open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Unknown,
    Safe,
    Mine,
}

/// How a cell was proven, from the cheapest kind of reasoning to the most expensive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// One number alone: satisfied, or needing all its hidden neighbours.
    Single,
    /// Two overlapping numbers (subset and difference, the 1-1 and 1-2 family).
    Pair,
    /// The count of mines left, near the end of a field.
    Count,
    /// Every arrangement of the numbers, and the count of mines left.
    Exact,
}

/// How hard the solver may think.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strength {
    /// One or two numbers at a time, and the mine count once few cells are left.
    Gentle,
    /// Perfect logic.
    Full,
}

/// Gentle uses the mine count only once this few cells are left unresolved.
pub const GENTLE_COUNT_CELLS: usize = 12;

/// A proven cell and the numbers that prove it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub cell: usize,
    pub mine: bool,
    pub tier: Tier,
    pub witnesses: Vec<usize>,
}

/// One open number's demand on the cells around it that are still unresolved.
#[derive(Clone, Debug)]
pub struct Constraint {
    /// The open number.
    pub at: usize,
    /// Unresolved neighbours, ascending.
    pub cells: Vec<usize>,
    /// Mines still needed among them.
    pub need: usize,
}

/// The constraints of every open number that still touches an unresolved cell.
pub fn constraints(v: &View, facts: &[Fact]) -> Vec<Constraint> {
    let mut out = Vec::new();
    for i in 0..v.len() {
        let Some(n) = v.open[i] else { continue };
        let mut cells = Vec::new();
        let mut mines = 0;
        for &j in v.nbrs(i) {
            if v.open[j].is_some() {
                continue;
            }
            match facts[j] {
                Fact::Unknown => cells.push(j),
                Fact::Mine => mines += 1,
                Fact::Safe => {}
            }
        }
        if !cells.is_empty() {
            out.push(Constraint { at: i, cells, need: (n as usize).saturating_sub(mines) });
        }
    }
    out
}

/// Cells that are neither open nor resolved.
pub fn unresolved(v: &View, facts: &[Fact]) -> usize {
    (0..v.len()).filter(|&i| v.open[i].is_none() && facts[i] == Fact::Unknown).count()
}

/// Proven mines that are not open (the field never opens a mine before the game ends).
pub fn proven_mines(facts: &[Fact]) -> usize {
    facts.iter().filter(|&&f| f == Fact::Mine).count()
}

/// The proofs of the cheapest tier that yields any, within `strength`. Empty means a guess
/// is needed (or the field is done).
pub fn deduce(v: &View, facts: &[Fact], strength: Strength) -> Vec<Proof> {
    let cs = constraints(v, facts);
    let found = single(&cs);
    if !found.is_empty() {
        return found;
    }
    let found = pair(&cs);
    if !found.is_empty() {
        return found;
    }
    match strength {
        Strength::Gentle if unresolved(v, facts) <= GENTLE_COUNT_CELLS => exact_proofs(v, facts, Tier::Count),
        Strength::Gentle => Vec::new(),
        Strength::Full => exact_proofs(v, facts, Tier::Exact),
    }
}

fn single(cs: &[Constraint]) -> Vec<Proof> {
    let mut out: Vec<Proof> = Vec::new();
    for c in cs {
        let mine = if c.need == 0 {
            false
        } else if c.need == c.cells.len() {
            true
        } else {
            continue;
        };
        for &cell in &c.cells {
            push(&mut out, Proof { cell, mine, tier: Tier::Single, witnesses: vec![c.at] });
        }
    }
    out
}

fn pair(cs: &[Constraint]) -> Vec<Proof> {
    let mut out = Vec::new();
    // Constraints touching each cell, to visit only overlapping pairs.
    let mut by_cell: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for (k, c) in cs.iter().enumerate() {
        for &cell in &c.cells {
            by_cell.entry(cell).or_default().push(k);
        }
    }
    let mut seen = std::collections::HashSet::new();
    for ks in by_cell.values() {
        for (x, &a) in ks.iter().enumerate() {
            for &b in &ks[x + 1..] {
                if seen.insert((a.min(b), a.max(b))) {
                    pair_one(&cs[a], &cs[b], &mut out);
                    pair_one(&cs[b], &cs[a], &mut out);
                }
            }
        }
    }
    out
}

/// What `a` tells about the cells only `b` touches, and about the shared cells.
fn pair_one(a: &Constraint, b: &Constraint, out: &mut Vec<Proof>) {
    let shared: Vec<usize> = a.cells.iter().copied().filter(|c| b.cells.binary_search(c).is_ok()).collect();
    let only_a = a.cells.len() - shared.len();
    let only_b: Vec<usize> = b.cells.iter().copied().filter(|c| a.cells.binary_search(c).is_err()).collect();
    // Bounds on the mines in the shared cells, from each number.
    let lo = a.need.saturating_sub(only_a).max(b.need.saturating_sub(only_b.len()));
    let hi = a.need.min(b.need).min(shared.len());
    if lo > hi {
        return; // inconsistent; not ours to report
    }
    let witnesses = vec![a.at.min(b.at), a.at.max(b.at)];
    // b's own cells hold between b.need - hi and b.need - lo mines (hi <= b.need).
    if !only_b.is_empty() {
        if b.need == lo {
            for &cell in &only_b {
                push(out, Proof { cell, mine: false, tier: Tier::Pair, witnesses: witnesses.clone() });
            }
        }
        if b.need - hi == only_b.len() {
            for &cell in &only_b {
                push(out, Proof { cell, mine: true, tier: Tier::Pair, witnesses: witnesses.clone() });
            }
        }
    }
    if lo == shared.len() && lo > 0 {
        for &cell in &shared {
            push(out, Proof { cell, mine: true, tier: Tier::Pair, witnesses: witnesses.clone() });
        }
    }
    if hi == 0 && !shared.is_empty() {
        for &cell in &shared {
            push(out, Proof { cell, mine: false, tier: Tier::Pair, witnesses: witnesses.clone() });
        }
    }
}

fn exact_proofs(v: &View, facts: &[Fact], tier: Tier) -> Vec<Proof> {
    let Ok(odds) = exact::odds(v, facts) else { return Vec::new() };
    let mut out = Vec::new();
    for i in 0..v.len() {
        if v.open[i].is_some() || facts[i] != Fact::Unknown {
            continue;
        }
        let mine = if odds.safe[i] {
            false
        } else if odds.mine[i] {
            true
        } else {
            continue;
        };
        let witnesses = if odds.interior[i] { Vec::new() } else { odds.component_numbers(v, i) };
        out.push(Proof { cell: i, mine, tier, witnesses });
    }
    out
}

fn push(out: &mut Vec<Proof>, p: Proof) {
    if !out.iter().any(|q| q.cell == p.cell) {
        out.push(p);
    }
}

#[cfg(test)]
mod tests;
