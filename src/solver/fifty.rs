//! True 50/50s: two hidden cells holding exactly one mine between them, which no number, open
//! now or opened later, can ever tell apart. A perfect player must guess each one, and loses
//! nothing by guessing it at once.

use super::exact::{self, Error};
use super::{Fact, View};

/// The true 50/50s at this position, as pairs of cells in ascending order. A pair is one when
/// every arrangement the numbers and the mine count allow puts exactly one mine in it, and every
/// other cell that is or may yet be a number touches both cells or neither. Swapping the
/// pair's mine then changes nothing anyone can ever see, so each cell is a mine with equal
/// chance. Empty when there are too many arrangements to tell.
pub fn fifty_fifties(v: &View, facts: &[Fact]) -> Vec<(usize, usize)> {
    let Ok(odds) = exact::odds(v, facts) else { return Vec::new() };
    let hidden = |i: usize| v.open[i].is_none() && facts[i] == Fact::Unknown;
    // A cell that is a mine in every arrangement never shows a number.
    let dark = |i: usize| v.open[i].is_none() && (facts[i] == Fact::Mine || odds.mine[i]);
    let even: Vec<usize> = (0..v.len()).filter(|&i| hidden(i) && (odds.p[i] - 0.5).abs() < 1e-9).collect();
    let seen = |i: usize, other: usize| -> Vec<usize> { v.nbrs(i).iter().copied().filter(|&c| c != other && !dark(c)).collect() };
    let mut out = Vec::new();
    for (k, &a) in even.iter().enumerate() {
        for &b in &even[k + 1..] {
            if seen(a, b) == seen(b, a) && one_mine(v, facts, a, b) {
                out.push((a, b));
            }
        }
    }
    out
}

/// Whether every arrangement puts exactly one mine in `a` and `b`: none fits both a mine, nor
/// both clear.
fn one_mine(v: &View, facts: &[Fact], a: usize, b: usize) -> bool {
    let impossible = |f: Fact| {
        let mut facts = facts.to_vec();
        facts[a] = f;
        facts[b] = f;
        exact::odds(v, &facts).err() == Some(Error::Inconsistent)
    };
    impossible(Fact::Mine) && impossible(Fact::Safe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Status};

    /// `rows` drawn with `*` for a mine, opened from the cell marked `o`.
    fn opened(rows: &[&str]) -> Game {
        let (w, h) = (rows[0].len(), rows.len());
        let at = |ch: char| (0..w * h).filter(move |&i| rows[i / w].as_bytes()[i % w] as char == ch);
        let mut g = Game::with_mines(w, h, &at('*').collect::<Vec<_>>());
        g.open(at('o').next().unwrap());
        assert_eq!(g.status, Status::Playing);
        g
    }

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
    fn true_fifty_fifties_are_exactly_the_pairs_whose_swap_no_layout_can_show() {
        // The truth, by brute force: in every layout that fits, the pair holds one mine, and
        // moving it to the other cell changes no number that layout would show.
        let mut seed = crate::generate::Stream::new(11);
        let (mut found, mut positions) = (0, 0);
        for round in 0..10000 {
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
            for (k, &a) in hidden.iter().enumerate() {
                for &b in &hidden[k + 1..] {
                    let unseen = fits.iter().all(|m| {
                        m[a] != m[b] && (0..v.len()).filter(|&c| c != a && c != b && !m[c]).all(|c| v.nbrs(c).contains(&a) == v.nbrs(c).contains(&b))
                    });
                    if unseen {
                        want.push((a, b));
                    }
                }
            }
            assert_eq!(fifty_fifties(&v, &vec![Fact::Unknown; v.len()]), want, "round {round}");
            found += want.len();
        }
        assert!(positions > 3000 && found > 80, "the positions hold enough true 50/50s to mean something: {found} in {positions}");
    }

    #[test]
    fn a_corner_pair_sealed_by_mines_is_a_true_fifty_fifty() {
        // The corner mine could as well be its neighbour: the 1 and the 3 below see both, the
        // 2s on the right are filled by the two mines in column 2, and nothing else touches them.
        let g = opened(&["*.*..", "..*..", ".....", "....o"]);
        let v = View::of(&g);
        assert_eq!(fifty_fifties(&v, &vec![Fact::Unknown; v.len()]), vec![(0, 1)]);
    }

    #[test]
    fn a_pair_a_later_number_could_split_is_not() {
        // The corner pair still holds one mine, but (2,0) is safe: once opened, its number sees
        // the right-hand cell of the pair and not the corner.
        let g = opened(&["*....", "..*..", ".....", "....o"]);
        let v = View::of(&g);
        assert!(fifty_fifties(&v, &vec![Fact::Unknown; v.len()]).is_empty());
    }
}
