use super::*;
use crate::game::{Game, Rng, Status};

/// A field with mines at `mines` and nothing open.
fn field(w: usize, h: usize, mines: &[usize]) -> Game {
    let mut g = Game::new(w, h, mines.len(), Rng::new(1));
    for c in &mut g.cells {
        c.mine = false;
    }
    for &i in mines {
        g.cells[i].mine = true;
    }
    for i in 0..w * h {
        g.cells[i].adj = g.neighbours(i).filter(|&j| g.cells[j].mine).count() as u8;
    }
    g.started = true;
    g.status = Status::Playing;
    g
}

/// Odds by trying every layout of the unresolved cells.
fn brute(v: &View) -> Option<Vec<f64>> {
    let hidden: Vec<usize> = (0..v.len()).filter(|&i| v.open[i].is_none()).collect();
    assert!(hidden.len() <= 20);
    let mut counts = vec![0u64; v.len()];
    let mut total = 0u64;
    for bits in 0u32..(1 << hidden.len()) {
        if bits.count_ones() as usize != v.mines {
            continue;
        }
        let mine = |i: usize| hidden.iter().position(|&h| h == i).is_some_and(|k| bits >> k & 1 == 1);
        let ok = (0..v.len()).all(|i| match v.open[i] {
            Some(n) => v.nbrs(i).iter().filter(|&&j| mine(j)).count() == n as usize,
            None => true,
        });
        if ok {
            total += 1;
            for (k, &i) in hidden.iter().enumerate() {
                if bits >> k & 1 == 1 {
                    counts[i] += 1;
                }
            }
        }
    }
    (total > 0).then(|| counts.iter().map(|&c| c as f64 / total as f64).collect())
}

#[test]
fn exact_odds_match_every_layout() {
    let mut rng = Rng::new(7);
    for round in 0..300 {
        let (w, h) = (4 + round % 3, 4 + round % 2);
        let mines = 2 + round % 5;
        let mut g = Game::new(w, h, mines, rng);
        rng = g.rng;
        // Open a few safe cells, as a player would.
        let safe: Vec<usize> = (0..w * h).filter(|&i| !g.cells[i].mine).collect();
        for k in 0..(1 + round % 4) {
            g.open(safe[(k * 7 + round) % safe.len()]);
            if g.status != Status::Playing {
                break;
            }
        }
        let v = View::of(&g);
        if (0..v.len()).filter(|&i| v.open[i].is_none()).count() > 20 {
            continue;
        }
        let want = brute(&v).expect("the real layout fits");
        let got = exact::odds(&v, &vec![Fact::Unknown; v.len()]).unwrap();
        for i in 0..v.len() {
            if v.open[i].is_some() {
                continue;
            }
            assert!((got.p[i] - want[i]).abs() < 1e-9, "round {round} cell {i}: {} vs {}", got.p[i], want[i]);
            assert_eq!(got.safe[i], want[i] == 0.0, "round {round} cell {i} safe");
            assert_eq!(got.mine[i], want[i] == 1.0, "round {round} cell {i} mine");
        }
    }
}

#[test]
fn every_proof_is_true() {
    let mut rng = Rng::new(11);
    for _ in 0..40 {
        let mut g = Game::new(30, 16, 99, rng);
        rng = g.rng;
        let mut facts = vec![Fact::Unknown; 30 * 16];
        g.open(0);
        for _ in 0..200 {
            if g.status != Status::Playing {
                break;
            }
            let v = View::of(&g);
            for strength in [Strength::Gentle, Strength::Full] {
                for p in deduce(&v, &facts, strength) {
                    assert_eq!(p.mine, g.cells[p.cell].mine, "{:?}", p);
                }
            }
            let proofs = deduce(&v, &facts, Strength::Full);
            if proofs.is_empty() {
                let i = play::guess(&v, &facts).unwrap();
                g.open(i);
                continue;
            }
            for p in proofs {
                if p.mine {
                    facts[p.cell] = Fact::Mine;
                } else {
                    g.open(p.cell);
                }
            }
        }
    }
}

#[test]
fn one_two_one_puts_mines_under_the_ones() {
    // Row 0 hidden; row 1 shows 1 2 1 against a wall of open cells below.
    //   ? ? ?
    //   1 2 1
    //   . . .
    let g = {
        let mut g = field(3, 3, &[0, 2]);
        for i in 3..9 {
            g.cells[i].cover = crate::game::Cover::Open;
        }
        g
    };
    let v = View::of(&g);
    let mut facts = vec![Fact::Unknown; 9];
    let proofs = deduce(&v, &facts, Strength::Gentle);
    let mut got: Vec<(usize, bool)> = proofs.iter().map(|p| (p.cell, p.mine)).collect();
    got.sort();
    assert_eq!(got, vec![(0, true), (2, true)]);
    assert!(proofs.iter().all(|p| p.tier == Tier::Pair));
    // With both mines known, the 2 is satisfied and the middle cell is safe.
    facts[0] = Fact::Mine;
    facts[2] = Fact::Mine;
    let next = deduce(&v, &facts, Strength::Gentle);
    assert_eq!(next.iter().map(|p| (p.cell, p.mine)).collect::<Vec<_>>(), vec![(1, false)]);
}

#[test]
fn the_bot_wins_beginner_often() {
    let mut rng = Rng::new(3);
    let mut wins = 0;
    for _ in 0..200 {
        let mut g = Game::new(9, 9, 10, rng);
        rng = g.rng;
        if play::play(&mut g, 0).won {
            wins += 1;
        }
    }
    assert!(wins > 150, "{wins}/200");
}
