//! A bot that plays a field to the end: every proven cell first, then the safest guess.

use super::{Fact, Strength, View, deduce, exact};
use crate::game::{Game, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Played {
    pub won: bool,
    /// Cells opened without proof, the first click excluded.
    pub guesses: u32,
}

/// Plays `game` from a first click on `first` until it is won or lost, with perfect logic.
pub fn play(game: &mut Game, first: usize) -> Played {
    play_at(game, first, Strength::Full)
}

/// [`play`], proving cells only as far as `strength` allows before it guesses.
pub fn play_at(game: &mut Game, first: usize, strength: Strength) -> Played {
    let n = game.w * game.h;
    let mut facts = vec![Fact::Unknown; n];
    let mut guesses = 0;
    game.open(first);
    while game.status == Status::Playing {
        let v = View::of(game);
        let proofs = deduce(&v, &facts, strength);
        if !proofs.is_empty() {
            for p in proofs {
                if p.mine {
                    facts[p.cell] = Fact::Mine;
                } else {
                    game.open(p.cell);
                }
            }
            continue;
        }
        let Some(cell) = guess(&v, &facts) else { break };
        guesses += 1;
        game.open(cell);
    }
    Played { won: game.status == Status::Won, guesses }
}

/// The unresolved cell least likely to be a mine; ties go to the cell with the fewest
/// neighbours (a corner is likeliest to open an area), then the first.
pub fn guess(v: &View, facts: &[Fact]) -> Option<usize> {
    let candidates = (0..v.len()).filter(|&i| v.open[i].is_none() && facts[i] == Fact::Unknown);
    match exact::odds(v, facts) {
        Ok(odds) => candidates.min_by(|&a, &b| {
            let (pa, pb) = (odds.p[a], odds.p[b]);
            if (pa - pb).abs() > 1e-9 {
                pa.total_cmp(&pb)
            } else {
                v.nbrs(a).len().cmp(&v.nbrs(b).len()).then(a.cmp(&b))
            }
        }),
        // Too many arrangements to count: fall back to the plain share of mines left.
        Err(_) => candidates.min_by_key(|&i| (v.nbrs(i).iter().filter(|&&j| v.open[j].is_some()).count(), i)),
    }
}

/// Whether the bot at `strength` finishes `game` from a first click on `first` without a
/// single guess. Stops at the first position with nothing proven.
pub fn clears(game: &Game, first: usize, strength: Strength) -> bool {
    let mut game = game.clone();
    let mut facts = vec![Fact::Unknown; game.w * game.h];
    game.open(first);
    while game.status == Status::Playing {
        let proofs = deduce(&View::of(&game), &facts, strength);
        if proofs.is_empty() {
            return false;
        }
        for p in proofs {
            if p.mine {
                facts[p.cell] = Fact::Mine;
            } else {
                game.open(p.cell);
            }
        }
    }
    game.status == Status::Won
}

/// Whether opening a cell now is a forced guess: no hidden cell can be proven safe from the
/// open numbers and the count of mines left. Mines proven along the way count as known; the
/// player's flags never do.
pub fn forced(game: &Game) -> bool {
    let v = View::of(game);
    let mut facts = vec![Fact::Unknown; v.len()];
    loop {
        let proofs = deduce(&v, &facts, Strength::Full);
        if proofs.is_empty() {
            return true;
        }
        if proofs.iter().any(|p| !p.mine) {
            return false;
        }
        for p in proofs {
            facts[p.cell] = Fact::Mine;
        }
    }
}
