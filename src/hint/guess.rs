//! The guess tier: when no cell can be proven, how a strong player chooses (ADR 0002). Each
//! reason is one a player can check by hand on the board; the hint gives the first that
//! holds, in the order of [`Guess`]. The exact odds never show: they only decide whether a
//! reason favours a cell, and which of the cells it covers to name (the least likely mine).
//! When no reason holds, the hint says a guess is needed and names no cell.

use super::{Field, Guess, Hint, Step, Target, Technique, Verdict};
use crate::solver::exact::{self, Odds};
use crate::solver::{Constraint, Fact, View, constraints, dead, fifty, relocation};

/// How much likelier to be a mine than the safest cell a cell may be and still be named by a
/// reason that only prefers it (arrangements, fewer mines, an opening), so that a hint never
/// talks a player into a clearly worse guess.
const SLACK: f64 = 0.05;
/// Arrangements few enough to count by hand.
const FEW_WAYS: usize = 6;
/// Numbers few enough to read together by hand.
const FEW_NUMBERS: usize = 4;
/// Cells and search steps a count of arrangements may take before it gives up.
const COUNT_CELLS: usize = 24;
const COUNT_STEPS: usize = 200_000;

/// The guess hint when no cell can be proven; None when one can (a deduction's to name), or
/// when there are too many arrangements to tell.
pub fn choose(v: &View, flags: &[bool], last: Option<usize>, field: &Field) -> Option<Hint> {
    let facts = vec![Fact::Unknown; v.len()];
    let odds = match field.classic_first {
        Some(first) => relocation::odds(v, &facts, first),
        None => exact::odds(v, &facts),
    }
    .ok()?;
    let hidden = |i: usize| v.open[i].is_none();
    if (0..v.len()).any(|i| hidden(i) && (odds.safe[i] || (odds.mine[i] && !flags[i]))) {
        return None;
    }
    // Cells a guess may name: unflagged, and not a mine already proven.
    let free = |i: usize| hidden(i) && !odds.mine[i] && !flags[i];
    let all: Vec<usize> = (0..v.len()).filter(|&i| free(i)).collect();
    if all.is_empty() {
        return None;
    }
    let safest = all.iter().map(|&i| odds.p[i]).fold(f64::INFINITY, f64::min);
    let g = Guessing { v, odds: &odds, last };
    let take = |guess, cell, numbers, steps| Some(Hint { technique: Technique::Guess(guess), numbers, target: Some(Target { cell, verdict: Verdict::Guess }), steps });
    let avoid = |guess, cell, numbers, steps| Some(Hint { technique: Technique::Guess(guess), numbers, target: Some(Target { cell, verdict: Verdict::Avoid }), steps });

    // 1. A true 50/50: take it now.
    let pairs = fifty::fifty_fifties(v, &facts);
    if let Some(cell) = g.pick(pairs.iter().flat_map(|&(a, b)| [a, b]).filter(|&c| free(c)), |_| 0) {
        let &(a, b) = pairs.iter().find(|&&(a, b)| a == cell || b == cell).expect("picked from a pair");
        let mut steps = vec![Step::FiftyFifty { cells: [a, b] }];
        if field.announced.contains(&(a, b)) {
            steps.push(Step::Announced { count: field.announced.len() });
        }
        return take(Guess::FiftyFifty, cell, g.around(&[a, b]), steps);
    }

    // 2. A pseudo-50/50: take it now too.
    let pseudo = fifty::pseudo(v, &facts);
    if let Some(cell) = g.pick(pseudo.iter().map(|&(x, _)| x).filter(|&c| free(c)), |_| 0) {
        let partner = pseudo.iter().find(|&&(x, _)| x == cell).expect("picked").1;
        return take(Guess::Pseudo, cell, g.around(&[cell, partner]), vec![Step::Pseudo { cell, partner }]);
    }

    // 3. A dead cell, when there is something else to guess: avoid it.
    let dead = dead::dead_cells(v, &facts);
    let is_dead = |c: usize| dead.iter().any(|&(x, _)| x == c);
    if all.iter().any(|&c| !is_dead(c))
        && let Some(cell) = g.pick(dead.iter().map(|&(x, _)| x).filter(|&c| free(c)), |_| 0)
    {
        let shows = dead.iter().find(|&&(x, _)| x == cell).expect("picked").1;
        let mut near: Vec<usize> = v.nbrs(cell).iter().copied().filter(|&j| hidden(j)).collect();
        near.push(cell);
        return avoid(Guess::Dead, cell, g.around(&near), vec![Step::Dead { cell, shows }]);
    }

    // 4 and 5. Counting the arrangements of a group of numbers.
    let counts = g.count();
    let near_best = |c: usize| free(c) && odds.p[c] <= safest + SLACK;
    // 4. Few arrangements, all with the same number of mines, so each as common.
    let few: Vec<(usize, usize)> = counts.iter().enumerate().filter(|(k, a)| a.same() && a.ways.len() <= FEW_WAYS && odds.numbers[*k].len() <= FEW_NUMBERS).flat_map(|(k, a)| a.cells.iter().copied().filter(|&c| near_best(c) && 2 * a.mine_in(c) < a.ways.len()).map(move |c| (c, k))).collect();
    if let Some(cell) = g.pick(few.iter().map(|&(c, _)| c), |_| 0) {
        let k = few.iter().find(|&&(c, _)| c == cell).expect("picked").1;
        let (a, numbers) = (&counts[k], odds.numbers[k].clone());
        let n = (numbers.len() == 1).then(|| v.open[numbers[0]].unwrap_or(0));
        return take(Guess::Ways, cell, numbers, vec![Step::Ways { n, ways: a.ways.len(), mine_in: a.mine_in(cell) }]);
    }
    // 5. Fewer mines: while the cells away from the numbers are plenty for the mines
    // left, each mine fewer among the numbers leaves more ways to place the rest. (A
    // group too big to count is taken to need none, which only makes this rarer.)
    let away = (0..v.len()).filter(|&i| odds.interior[i]).count();
    let least: usize = counts.iter().map(|a| a.least()).sum();
    if away > 0 && v.mines > least && away + 1 > 2 * (v.mines - least) {
        let fewer: Vec<(usize, usize)> = counts.iter().enumerate().filter(|(_, a)| a.counted && !a.same()).flat_map(|(k, a)| a.cells.iter().copied().filter(|&c| near_best(c) && a.safe_with_least(c)).map(move |c| (c, k))).collect();
        if let Some(cell) = g.pick(fewer.iter().map(|&(c, _)| c), |_| 0) {
            let k = fewer.iter().find(|&&(c, _)| c == cell).expect("picked").1;
            let least = counts[k].least();
            return take(Guess::Fewer, cell, odds.numbers[k].clone(), vec![Step::Fewer { least, away }, Step::FewerSafe { least }]);
        }
    }

    // 6. A cell on the edge, away from the numbers: apt to be empty and open an area.
    let edge = all.iter().copied().filter(|&c| odds.interior[c] && v.nbrs(c).len() < 8 && odds.p[c] <= safest + SLACK);
    if let Some(cell) = g.pick(edge, |c| v.nbrs(c).len()) {
        let hidden_cells = (0..v.len()).filter(|&i| hidden(i)).count();
        let mut steps = vec![Step::Opening { neighbours: v.nbrs(cell).len(), hidden: hidden_cells, mines: v.mines }];
        // In Classic, the corner XP's move would have been named but for that move: say why not.
        if let Some(corner) = field.classic_first.and_then(|first| relocation::scan(v, first).next())
            && corner != cell
            && free(corner)
            && odds.interior[corner]
            && v.nbrs(corner).len() <= v.nbrs(cell).len()
        {
            steps.push(Step::NotTopLeft { cell: corner });
        }
        return take(Guess::Opening, cell, Vec::new(), steps);
    }

    // 7. Classic early on: XP moved any first-click mine to the top left.
    if let Some(first) = field.classic_first
        && let Some(cell) = relocation::scan(v, first).next()
        && free(cell)
        && odds.interior[cell]
    {
        return avoid(Guess::TopLeft, cell, Vec::new(), vec![Step::TopLeft]);
    }

    Some(Hint { technique: Technique::Guess(Guess::Blind), numbers: Vec::new(), target: None, steps: Vec::new() })
}

struct Guessing<'a> {
    v: &'a View,
    odds: &'a Odds,
    last: Option<usize>,
}

impl Guessing<'_> {
    /// Of `cells`, the least likely mine; ties go by `then`, the nearest the last move, and
    /// the first.
    fn pick<K: Ord>(&self, cells: impl Iterator<Item = usize>, then: impl Fn(usize) -> K) -> Option<usize> {
        cells.min_by(|&a, &b| {
            let (pa, pb) = (self.odds.p[a], self.odds.p[b]);
            let by_odds = if (pa - pb).abs() > 1e-9 { pa.total_cmp(&pb) } else { std::cmp::Ordering::Equal };
            by_odds.then_with(|| then(a).cmp(&then(b))).then_with(|| self.near(a).cmp(&self.near(b))).then(a.cmp(&b))
        })
    }

    /// How far `cell` is from the last move (squared, in cells).
    fn near(&self, cell: usize) -> usize {
        let Some(l) = self.last else { return 0 };
        let w = self.v.w as isize;
        let (dx, dy) = ((cell as isize % w) - (l as isize % w), (cell as isize / w) - (l as isize / w));
        (dx * dx + dy * dy) as usize
    }

    /// The open numbers beside any of `cells`, ascending.
    fn around(&self, cells: &[usize]) -> Vec<usize> {
        let mut out: Vec<usize> = cells.iter().flat_map(|&c| self.v.nbrs(c).iter().copied()).filter(|&j| self.v.open[j].is_some()).collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Every component's arrangements whose count of mines fits the mines left. A component
    /// too big to count has none, and counts as holding anywhere from none to all mines.
    fn count(&self) -> Vec<Arrangements> {
        let (v, odds) = (self.v, self.odds);
        let cs = constraints(v, &vec![Fact::Unknown; v.len()]);
        let mut all: Vec<Arrangements> = (0..odds.numbers.len())
            .map(|k| {
                let cells: Vec<usize> = (0..v.len()).filter(|&i| odds.comp[i] == Some(k)).collect();
                let cons: Vec<&Constraint> = cs.iter().filter(|c| odds.numbers[k].contains(&c.at)).collect();
                match arrangements(&cells, &cons) {
                    Some(ways) => Arrangements { cells, ways, counted: true },
                    None => Arrangements { cells, ways: Vec::new(), counted: false },
                }
            })
            .collect();
        let away = (0..v.len()).filter(|&i| odds.interior[i]).count();
        // Keep the arrangements some choice of the others' and the cells away can complete.
        let counts: Vec<Vec<usize>> = all.iter().map(|a| a.counts()).collect();
        for (k, a) in all.iter_mut().enumerate() {
            let mut sums = vec![true];
            for c in counts.iter().enumerate().filter(|&(j, _)| j != k).map(|(_, c)| c) {
                let mut next = vec![false; sums.len() + c.iter().max().copied().unwrap_or(0)];
                for (s, _) in sums.iter().enumerate().filter(|&(_, &on)| on) {
                    for &m in c {
                        next[s + m] = true;
                    }
                }
                sums = next;
            }
            let fits = |m: usize| sums.iter().enumerate().any(|(s, &on)| on && v.mines >= m + s && v.mines - m - s <= away);
            a.ways.retain(|w| fits(w.count_ones() as usize));
        }
        all
    }
}

/// The arrangements of one group of numbers' hidden cells, each a bit per cell (in `cells`'
/// order) set where a mine is.
struct Arrangements {
    cells: Vec<usize>,
    ways: Vec<u32>,
    counted: bool,
}

impl Arrangements {
    /// The counts of mines its arrangements hold.
    fn counts(&self) -> Vec<usize> {
        if !self.counted {
            return (0..=self.cells.len()).collect();
        }
        let mut c: Vec<usize> = self.ways.iter().map(|w| w.count_ones() as usize).collect();
        c.sort_unstable();
        c.dedup();
        c
    }

    /// Whether every arrangement holds the same number of mines.
    fn same(&self) -> bool {
        !self.ways.is_empty() && self.counts().len() == 1
    }

    fn least(&self) -> usize {
        self.counts().first().copied().unwrap_or(0)
    }

    fn bit(&self, cell: usize) -> u32 {
        self.cells.iter().position(|&c| c == cell).map_or(0, |k| 1 << k)
    }

    fn mine_in(&self, cell: usize) -> usize {
        let b = self.bit(cell);
        self.ways.iter().filter(|&&w| w & b != 0).count()
    }

    /// Whether the cell is safe in every arrangement with the fewest mines.
    fn safe_with_least(&self, cell: usize) -> bool {
        let (b, least) = (self.bit(cell), self.least() as u32);
        b != 0 && self.ways.iter().filter(|w| w.count_ones() == least).all(|&w| w & b == 0)
    }
}

/// Every way to place mines in `cells` that satisfies `cons`; None past [`COUNT_CELLS`]
/// cells or [`COUNT_STEPS`] steps.
fn arrangements(cells: &[usize], cons: &[&Constraint]) -> Option<Vec<u32>> {
    if cells.len() > COUNT_CELLS {
        return None;
    }
    let masks: Vec<(u32, usize)> = cons.iter().map(|c| (c.cells.iter().filter_map(|x| cells.iter().position(|y| y == x)).fold(0u32, |m, k| m | 1 << k), c.need)).collect();
    fn go(k: usize, n: usize, mines: u32, masks: &[(u32, usize)], out: &mut Vec<u32>, steps: &mut usize) -> bool {
        *steps += 1;
        if *steps > COUNT_STEPS {
            return false;
        }
        // Cells from k on are still to place.
        let rest = if k >= 32 { 0 } else { !0u32 << k } & if n >= 32 { !0 } else { (1u32 << n) - 1 };
        for &(m, need) in masks {
            let have = (mines & m).count_ones() as usize;
            if have > need || have + ((rest & m).count_ones() as usize) < need {
                return true;
            }
        }
        if k == n {
            out.push(mines);
            return true;
        }
        go(k + 1, n, mines, masks, out, steps) && go(k + 1, n, mines | 1 << k, masks, out, steps)
    }
    let mut out = Vec::new();
    let mut steps = 0;
    go(0, cells.len(), 0, &masks, &mut out, &mut steps).then_some(out)
}

#[cfg(test)]
mod tests;
