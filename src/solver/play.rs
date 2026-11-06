//! A bot that plays a field to the end: every proven cell first, then the safest guess.

use super::{Fact, Strength, View, deduce, exact, fifty};
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

/// A perfect player's way through a field whose layout is known: what it had to guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walk {
    /// The true 50/50s guessed, in the order met, each as its pair of cells ascending.
    pub fifty_fifties: Vec<(usize, usize)>,
    /// Logic and true 50/50s alone finished the field. False when it stalled where no guess
    /// is a true 50/50.
    pub won: bool,
    /// Cells open when the walk ended.
    pub opened: usize,
}

/// Plays `game` from a first click on `first` as a perfect player who is always lucky: every
/// proof, and at each stall the first true 50/50, opening the cell the layout says is safe.
/// Stops at a stall with no true 50/50.
pub fn walk(game: &Game, first: usize) -> Walk {
    let mut game = game.clone();
    let mut facts = vec![Fact::Unknown; game.w * game.h];
    let mut fifty_fifties = Vec::new();
    game.open(first);
    while game.status == Status::Playing {
        let v = View::of(&game);
        let proofs = deduce(&v, &facts, Strength::Full);
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
        let Some(&(a, b)) = fifty::fifty_fifties(&v, &facts).first() else { break };
        let (mine, safe) = if game.cells[a].mine { (a, b) } else { (b, a) };
        fifty_fifties.push((a, b));
        facts[mine] = Fact::Mine;
        game.open(safe);
    }
    Walk { fifty_fifties, won: game.status == Status::Won, opened: game.opened }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Rng;

    #[test]
    fn a_walk_guesses_the_corner_fifty_fifty_and_wins_by_logic() {
        // The corner pair of the true 50/50 fixture: the mine is in the corner itself.
        let g = Game::with_mines(5, 4, &[0, 2, 7]);
        let w = walk(&g, 19);
        assert!(w.won);
        assert_eq!(w.fifty_fifties, vec![(0, 1)]);
    }

    #[test]
    fn every_fifty_fifty_a_walk_guesses_is_one_on_the_real_field() {
        // Classic Expert fields from the top-left corner: some stall on a lopsided guess, some
        // on true 50/50s. Each pair taken holds one mine, and moving it to the other cell would
        // change no number on the field.
        let (mut rng, mut pairs, mut stuck) = (Rng::new(3), 0, 0);
        for _ in 0..60 {
            let g = Game::new(30, 16, 99, rng);
            rng = g.rng;
            let mut first = g.clone();
            first.open(0);
            if first.status != Status::Playing {
                continue;
            }
            let w = walk(&g, 0);
            stuck += !w.won as u32;
            for &(a, b) in &w.fifty_fifties {
                pairs += 1;
                assert_ne!(first.cells[a].mine, first.cells[b].mine);
                let unseen = (0..g.cells.len()).filter(|&c| c != a && c != b && !first.cells[c].mine).all(|c| first.neighbours(c).any(|n| n == a) == first.neighbours(c).any(|n| n == b));
                assert!(unseen, "({a}, {b})");
            }
        }
        assert!(pairs > 0 && stuck > 0, "{pairs} pairs, {stuck} stuck");
    }
}
