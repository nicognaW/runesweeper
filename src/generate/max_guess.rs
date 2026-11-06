//! Max Guess: a local search, with the solver in the loop, for a field that forces as many
//! true 50/50s as it can on a perfect player and is finished by logic between them.
//!
//! It starts from the uniform field No Guess would draw first (the same stream, the 3×3 around
//! the first click clear) and moves one mine at a time. Each move is scored by walking the
//! field as a perfect, lucky player ([`play::walk`]): fields the walk finishes beat fields it
//! stalls on a lopsided guess; among those, more true 50/50s win, then more ground opened.
//! A move is kept when it scores no worse. Each pair's mine is finally put on a coin flip. The search is anytime: the best finished field
//! after [`budget`] steps is used, and it keeps going past the budget only until it holds one.
//! Steps, not time, so the field depends only on (dims, seed, first click) (ADR 0001), and a
//! caller can run the steps in slices.

use super::{Dims, Stream};
use crate::game::Game;
use crate::solver::play::{self, Walk};

/// A Max Guess field and what a perfect player must guess on it.
#[derive(Clone)]
pub struct MaxGuess {
    pub game: Game,
    /// The true 50/50s along the field's real layout, in the order a perfect player meets
    /// them, each as its pair of cells ascending.
    pub fifty_fifties: Vec<(usize, usize)>,
    /// Search steps taken.
    pub steps: u32,
}

/// The Max Guess field for `dims` from `seed`, for a first click on `first`.
pub fn max_guess(dims: Dims, seed: u64, first: usize) -> MaxGuess {
    searched(dims, seed, first, budget(dims))
}

/// [`max_guess`] with another budget, for measuring what more steps buy.
pub fn searched(dims: Dims, seed: u64, first: usize, budget: u32) -> MaxGuess {
    let mut s = Search::with_budget(dims, seed, first, budget);
    while !s.done() && !super::stopped() {
        s.step();
    }
    s.finish()
}

/// Search steps for a field of `dims`: 2,000 for Beginner, 1,500 for Intermediate, 1,000 for
/// Expert, and Custom sizes in proportion to their cells between and beyond (300 at least).
/// Measured with `examples/solver.rs`: more steps keep adding true 50/50s (Expert averages 4.5
/// at 300 steps, 9.6 at 1,000, 17 at 2,000), while a step costs more on a bigger field (about
/// 0.04 ms on Beginner, 0.24 ms on Intermediate, 0.85 ms on Expert, native release). These
/// keep Expert under a second natively.
pub fn budget((w, h, _): Dims) -> u32 {
    let cells = (w * h) as i64;
    let steps = if cells <= 256 { 2000 - (cells - 81) * 500 / 175 } else { 1500 - (cells - 256) * 500 / 224 };
    steps.clamp(300, 2000) as u32
}

/// How a walk ranks: finished first, then true 50/50s, then ground opened.
fn score(w: &Walk) -> (bool, usize, usize) {
    (w.won, w.fifty_fifties.len(), w.opened)
}

/// The search in progress. [`Search::step`] tries one move; [`Search::done`] says when to stop.
pub struct Search {
    w: usize,
    h: usize,
    first: usize,
    rng: Stream,
    /// Whether each cell may hold a mine (not within the 3×3 around the first click).
    free: Vec<bool>,
    /// The cells that may hold a mine.
    open_ground: Vec<usize>,
    /// The current field's mines, and whether each cell holds one.
    at: Vec<usize>,
    mine: Vec<bool>,
    score: (bool, usize, usize),
    best: Option<(Vec<usize>, Walk)>,
    steps: u32,
    budget: u32,
}

impl Search {
    pub fn new(dims: Dims, seed: u64, first: usize) -> Search {
        Search::with_budget(dims, seed, first, budget(dims))
    }

    fn with_budget((w, h, mines): Dims, seed: u64, first: usize, budget: u32) -> Search {
        let mut rng = Stream::new(seed);
        let (fx, fy) = ((first % w) as isize, (first / w) as isize);
        let free: Vec<bool> = (0..w * h).map(|i| ((i % w) as isize - fx).abs() > 1 || ((i / w) as isize - fy).abs() > 1).collect();
        let open_ground: Vec<usize> = (0..w * h).filter(|&i| free[i]).collect();
        let at = rng.pick(&open_ground, mines.min(open_ground.len()));
        let mut mine = vec![false; w * h];
        for &i in &at {
            mine[i] = true;
        }
        let mut s = Search { w, h, first, rng, free, open_ground, at, mine, score: (false, 0, 0), best: None, steps: 0, budget };
        let walk = s.walk();
        s.score = score(&walk);
        s.keep(walk);
        s
    }

    /// Steps taken so far.
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// The budget is spent and a finished field is in hand.
    pub fn done(&self) -> bool {
        self.steps >= self.budget && self.best.is_some()
    }

    /// Moves one mine and walks the field; keeps the move if it scores no worse.
    pub fn step(&mut self) {
        self.steps += 1;
        let Some((k, to)) = self.propose() else { return };
        let from = self.at[k];
        self.swap(k, from, to);
        let walk = self.walk();
        let s = score(&walk);
        if s >= self.score {
            self.score = s;
            self.keep(walk);
        } else {
            self.swap(k, to, from);
        }
    }

    /// The best finished field, once [`Search::done`]. Stopped early with none, the current
    /// field, unfinished (a cancelled search's, which is thrown away).
    ///
    /// The search may favour one cell of a pair (an edge cell, say) for the mine, and a player
    /// who learned that would beat the odds. So each pair's mine goes to either cell on a fair
    /// coin: no number anywhere else changes, so the walk is the same either way.
    pub fn finish(mut self) -> MaxGuess {
        let Some((mut at, walk)) = self.best.take() else {
            let walk = self.walk();
            return MaxGuess { game: Game::with_mines(self.w, self.h, &self.at), fifty_fifties: walk.fifty_fifties, steps: self.steps };
        };
        let unflipped = at.clone();
        for &(a, b) in &walk.fifty_fifties {
            if self.rng.below(2) == 1 {
                let k = at.iter().position(|&i| i == a || i == b).expect("one of the pair is a mine");
                at[k] = if at[k] == a { b } else { a };
            }
        }
        let game = Game::with_mines(self.w, self.h, &at);
        let again = play::walk(&game, self.first);
        let (game, walk) = if again.won && again.fifty_fifties == walk.fifty_fifties { (game, again) } else { (Game::with_mines(self.w, self.h, &unflipped), walk) };
        MaxGuess { game, fifty_fifties: walk.fifty_fifties, steps: self.steps }
    }

    fn walk(&self) -> Walk {
        play::walk(&Game::with_mines(self.w, self.h, &self.at), self.first)
    }

    /// Keeps the current field if the walk finished it with more true 50/50s than the best.
    fn keep(&mut self, walk: Walk) {
        if walk.won && self.best.as_ref().is_none_or(|(_, b)| walk.fifty_fifties.len() > b.fifty_fifties.len()) {
            self.best = Some((self.at.clone(), walk));
        }
    }

    fn swap(&mut self, k: usize, from: usize, to: usize) {
        self.mine[from] = false;
        self.mine[to] = true;
        self.at[k] = to;
    }

    /// A mine to move (its index in `at`) and the empty cell it moves to: usually one within
    /// two cells, sometimes anywhere.
    fn propose(&mut self) -> Option<(usize, usize)> {
        if self.at.is_empty() || self.at.len() == self.open_ground.len() {
            return None;
        }
        let k = self.rng.below(self.at.len());
        let (x, y) = ((self.at[k] % self.w) as isize, (self.at[k] / self.w) as isize);
        if self.rng.below(4) != 0 {
            for _ in 0..8 {
                let (nx, ny) = (x + self.rng.below(5) as isize - 2, y + self.rng.below(5) as isize - 2);
                if nx < 0 || ny < 0 || nx >= self.w as isize || ny >= self.h as isize {
                    continue;
                }
                let to = ny as usize * self.w + nx as usize;
                if self.free[to] && !self.mine[to] {
                    return Some((k, to));
                }
            }
        }
        loop {
            let to = self.open_ground[self.rng.below(self.open_ground.len())];
            if !self.mine[to] {
                return Some((k, to));
            }
        }
    }
}
