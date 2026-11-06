//! Classic's mine odds. XP moves a mine under the first click to the first free cell, row by
//! row from the top left (never the last row or column), so a layout whose scan begins with
//! a run of k mines could also have been dealt in k other ways: it is 1 + k times as likely
//! as a uniform deal makes it. Early on this roughly doubles the top-left cell's odds.

use super::exact::{self, Error, Odds};
use super::{Fact, View};

/// Mines along the scan worth weighing: each further one is rarer by the field's density.
const TERMS: usize = 16;

/// The cells XP's scan visits for a first click on `first`, in order.
pub fn scan(v: &View, first: usize) -> impl Iterator<Item = usize> + '_ {
    (0..v.h.saturating_sub(1)).flat_map(move |y| (0..v.w - 1).map(move |x| y * v.w + x)).filter(move |&i| i != first)
}

/// [`exact::odds`] for a Classic field whose first click was `first`, weighted by XP's
/// relocation. Which cells are certain is unchanged: every layout keeps some weight.
pub fn odds(v: &View, facts: &[Fact], first: usize) -> Result<Odds, Error> {
    let base = exact::odds(v, facts)?;
    // The weight is 1 + Σ_k [the scan's first k cells are all mines], so
    // P(x) = (P(x) + Σ_k P(x and S_k)) / (1 + Σ_k P(S_k)).
    let mut num = base.p.clone();
    let mut den = 1.0;
    let mut facts = facts.to_vec();
    let mut given = base.clone();
    let mut run = 1.0;
    for s in scan(v, first).take(TERMS) {
        run *= given.p[s];
        if run < 1e-7 {
            break;
        }
        facts[s] = Fact::Mine;
        let Ok(next) = exact::odds(v, &facts) else { break };
        given = next;
        den += run;
        for (n, p) in num.iter_mut().zip(&given.p) {
            *n += run * p;
        }
    }
    let mut out = base;
    for (p, n) in out.p.iter_mut().zip(num) {
        *p = (n / den).clamp(0.0, 1.0);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Status};

    #[test]
    fn odds_match_every_layout_weighted_by_how_many_deals_give_it() {
        // By brute force: every layout that fits, weighted by 1 + the run of mines at the
        // start of the scan (each could have been the mine moved off the first click).
        let mut seed = crate::generate::Stream::new(5);
        let mut checked = 0;
        for round in 0..3000 {
            let (w, h) = (3 + round % 3, 3 + round % 2);
            let mines = 2 + seed.below(w * h / 3);
            let mut picks: Vec<usize> = (0..w * h).collect();
            for i in 0..mines {
                let j = i + seed.below(picks.len() - i);
                picks.swap(i, j);
            }
            let mut g = Game::with_mines(w, h, &picks[..mines]);
            let first = picks[mines + seed.below(w * h - mines)];
            g.open(first);
            if g.status != Status::Playing {
                continue;
            }
            let v = View::of(&g);
            let hidden: Vec<usize> = (0..v.len()).filter(|&i| v.open[i].is_none()).collect();
            if hidden.len() > 14 {
                continue;
            }
            let order: Vec<usize> = scan(&v, first).collect();
            let (mut total, mut mine) = (0.0, vec![0.0; v.len()]);
            for bits in 0u32..1 << hidden.len() {
                if bits.count_ones() as usize != v.mines {
                    continue;
                }
                let m = |i: usize| hidden.iter().position(|&c| c == i).is_some_and(|k| bits >> k & 1 == 1);
                if !(0..v.len()).all(|i| v.open[i].is_none_or(|n| v.nbrs(i).iter().filter(|&&j| m(j)).count() == n as usize)) {
                    continue;
                }
                let weight = 1.0 + order.iter().take_while(|&&s| m(s)).count() as f64;
                total += weight;
                for &i in &hidden {
                    if m(i) {
                        mine[i] += weight;
                    }
                }
            }
            let got = odds(&v, &vec![Fact::Unknown; v.len()], first).unwrap();
            for &i in &hidden {
                assert!((got.p[i] - mine[i] / total).abs() < 1e-5, "round {round} cell {i}: {} vs {}", got.p[i], mine[i] / total);
            }
            checked += 1;
        }
        assert!(checked > 1000, "{checked}");
    }

    #[test]
    fn early_on_the_top_left_hides_a_mine_more_often() {
        // A lone opening in the bottom right: away from it every cell is alike, but for XP's move.
        let g = {
            let mut g = Game::with_mines(9, 9, &[1, 12, 20, 33, 40, 45, 50, 57, 62, 66]);
            g.open(80);
            g
        };
        let v = View::of(&g);
        let facts = vec![Fact::Unknown; v.len()];
        let plain = exact::odds(&v, &facts).unwrap();
        let xp = odds(&v, &facts, 80).unwrap();
        assert!((plain.p[0] - plain.p[3]).abs() < 1e-9);
        assert!(xp.p[0] > 1.5 * plain.p[0], "{} vs {}", xp.p[0], plain.p[0]);
        assert!(xp.p[0] > xp.p[3]);
        assert_eq!(xp.safe, plain.safe);
    }
}
