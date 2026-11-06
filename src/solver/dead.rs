//! Dead cells: cells that, if safe, can show only one number, and so tell a player nothing.
//! Opening one leaves the position as it was, a cell fewer: another guess follows at once.

use super::exact::{self, Error};
use super::{Fact, View};

/// The dead cells at this position, each with the one number it would show. A cell is dead
/// when it may be a mine or safe, every arrangement with it safe gives it the same number,
/// and knowing it safe proves no other cell safe. Empty when there are too many arrangements
/// to tell.
pub fn dead_cells(v: &View, facts: &[Fact]) -> Vec<(usize, u8)> {
    let Ok(odds) = exact::odds(v, facts) else { return Vec::new() };
    let hidden = |i: usize| v.open[i].is_none();
    let unresolved = |i: usize| hidden(i) && facts[i] == Fact::Unknown && !odds.safe[i] && !odds.mine[i];
    let mut out = Vec::new();
    for x in (0..v.len()).filter(|&x| unresolved(x)) {
        let around: Vec<usize> = v.nbrs(x).iter().copied().filter(|&j| hidden(j)).collect();
        let known = around.iter().filter(|&&j| facts[j] == Fact::Mine || odds.mine[j]).count();
        let open: Vec<usize> = around.iter().copied().filter(|&j| unresolved(j)).collect();
        // The likeliest numbers first, so a live cell shows two of them soon.
        let mean = known as f64 + open.iter().map(|&j| odds.p[j]).sum::<f64>();
        let mut tries: Vec<usize> = (known..=known + open.len()).collect();
        tries.sort_by(|&a, &b| (a as f64 - mean).abs().total_cmp(&(b as f64 - mean).abs()));
        let mut shown = None;
        let mut live = false;
        for k in tries {
            let mut opened = v.clone();
            opened.open[x] = Some(k as u8);
            match exact::odds(&opened, facts) {
                Err(Error::Inconsistent) => continue,
                Err(Error::TooBig) => live = true,
                Ok(given) => {
                    // A second number, or a cell proven safe by this one: progress.
                    live = shown.is_some() || (0..v.len()).any(|j| j != x && hidden(j) && given.safe[j] && !odds.safe[j]);
                    shown = Some(k as u8);
                }
            }
            if live {
                break;
            }
        }
        if let (false, Some(k)) = (live, shown) {
            out.push((x, k));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Status};

    /// Every layout of the hidden cells that fits the numbers and the mine count.
    fn layouts(v: &View) -> Vec<Vec<bool>> {
        let hidden: Vec<usize> = (0..v.len()).filter(|&i| v.open[i].is_none()).collect();
        let mut out = Vec::new();
        for bits in 0u32..(1 << hidden.len()) {
            if bits.count_ones() as usize != v.mines {
                continue;
            }
            let mut mine = vec![false; v.len()];
            for (k, &i) in hidden.iter().enumerate() {
                mine[i] = bits >> k & 1 == 1;
            }
            if (0..v.len()).all(|i| v.open[i].is_none_or(|n| v.nbrs(i).iter().filter(|&&j| mine[j]).count() == n as usize)) {
                out.push(mine);
            }
        }
        out
    }

    #[test]
    fn dead_cells_are_exactly_those_whose_opening_would_settle_nothing() {
        // The truth, by brute force: over the layouts with the cell safe, it shows one
        // number, and no other cell is safe in all of them that isn't safe in every layout.
        let mut seed = crate::generate::Stream::new(41);
        let (mut found, mut positions) = (0, 0);
        for round in 0..6000 {
            let (w, h) = (4 + round % 3, 3 + round % 3);
            let all: Vec<usize> = (0..w * h).collect();
            let mines = 2 + seed.below(w * h / 2);
            let mut picks = all.clone();
            for i in 0..mines {
                let j = i + seed.below(picks.len() - i);
                picks.swap(i, j);
            }
            let mut g = Game::with_mines(w, h, &picks[..mines]);
            let safe: Vec<usize> = all.iter().copied().filter(|&i| !g.cells[i].mine).collect();
            for _ in 0..1 + seed.below(3) {
                g.open(safe[seed.below(safe.len())]);
            }
            let v = View::of(&g);
            if g.status != Status::Playing || (0..v.len()).filter(|&i| v.open[i].is_none()).count() > 16 {
                continue;
            }
            positions += 1;
            let fits = layouts(&v);
            let hidden: Vec<usize> = (0..v.len()).filter(|&i| v.open[i].is_none()).collect();
            let mut want = Vec::new();
            for &x in &hidden {
                let clear: Vec<&Vec<bool>> = fits.iter().filter(|m| !m[x]).collect();
                if clear.is_empty() || clear.len() == fits.len() {
                    continue;
                }
                let shows = |m: &Vec<bool>| v.nbrs(x).iter().filter(|&&j| m[j]).count() as u8;
                if !clear.iter().all(|m| shows(m) == shows(clear[0])) {
                    continue;
                }
                let settles = hidden.iter().any(|&j| j != x && clear.iter().all(|m| !m[j]) && !fits.iter().all(|m| !m[j]));
                if !settles {
                    want.push((x, shows(clear[0])));
                }
            }
            assert_eq!(dead_cells(&v, &vec![Fact::Unknown; v.len()]), want, "round {round}");
            found += want.len();
        }
        assert!(positions > 2000 && found > 100, "enough dead cells to mean something: {found} in {positions}");
    }
}
