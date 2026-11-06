//! The solver benchmark: seeded Classic games played by the bot, and the numbers that
//! describe how it did.
//!
//! The rule: Classic fields laid from the MSVC `rand()` stream seeded once with `seed`, one
//! field after another as the game deals them; the first click on the top-left cell, with
//! XP's first-click relocation. The bot plays every proof, then the cell least likely to be
//! a mine under uniform odds (it does not model the relocation's bias toward the top left).

use std::time::{Duration, Instant};

use super::play;
use crate::game::{Game, Rng};

/// How the bot did over a run of games.
#[derive(Clone, Copy, Debug, Default)]
pub struct Tally {
    pub games: u64,
    pub wins: u64,
    /// Guesses made in won games, the first click excluded.
    pub guesses_in_wins: u64,
    /// Games won without a single guess.
    pub clean_wins: u64,
    /// Time spent playing, laying the fields excluded.
    pub time: Duration,
}

impl Tally {
    pub fn win_rate(&self) -> f64 {
        self.wins as f64 / self.games.max(1) as f64
    }
    pub fn guesses_per_win(&self) -> f64 {
        self.guesses_in_wins as f64 / self.wins.max(1) as f64
    }
    /// The share of all games won with no guess.
    pub fn clean_share(&self) -> f64 {
        self.clean_wins as f64 / self.games.max(1) as f64
    }
    pub fn ms_per_game(&self) -> f64 {
        self.time.as_secs_f64() * 1000.0 / self.games.max(1) as f64
    }
}

/// Plays `games` Classic fields of `(columns, rows, mines)` from `seed`.
pub fn classic((w, h, mines): (usize, usize, usize), games: u64, seed: u32) -> Tally {
    let mut t = Tally::default();
    let mut rng = Rng::new(seed);
    for _ in 0..games {
        let mut g = Game::new(w, h, mines, rng);
        rng = g.rng;
        let start = Instant::now();
        let played = play::play(&mut g, 0);
        t.time += start.elapsed();
        t.games += 1;
        if played.won {
            t.wins += 1;
            t.guesses_in_wins += played.guesses as u64;
            if played.guesses == 0 {
                t.clean_wins += 1;
            }
        }
    }
    t
}

/// The 95% Wilson score interval for `wins` out of `games`, as fractions.
pub fn wilson(wins: u64, games: u64) -> (f64, f64) {
    if games == 0 {
        return (0.0, 1.0);
    }
    let (n, p, z) = (games as f64, wins as f64 / games as f64, 1.959_964);
    let z2 = z * z;
    let centre = (p + z2 / (2.0 * n)) / (1.0 + z2 / n);
    let half = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / (1.0 + z2 / n);
    ((centre - half).max(0.0), (centre + half).min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_interval_matches_a_worked_example() {
        // 40 wins in 100 games: the 95% Wilson interval is 30.94% to 49.80%.
        let (lo, hi) = wilson(40, 100);
        assert!((lo - 0.3094).abs() < 5e-5, "{lo}");
        assert!((hi - 0.4980).abs() < 5e-5, "{hi}");
        // It stays inside 0..1 at the extremes, where a plain ± interval would not.
        let (lo, hi) = wilson(0, 10);
        assert_eq!(lo, 0.0);
        assert!((hi - 0.2775).abs() < 5e-5, "{hi}");
    }

    /// The parts of a tally that do not depend on the machine.
    fn counts(t: &Tally) -> (u64, u64, u64, u64) {
        (t.games, t.wins, t.guesses_in_wins, t.clean_wins)
    }

    #[test]
    fn the_same_seed_plays_the_same_games() {
        let a = classic((9, 9, 10), 300, 42);
        let b = classic((9, 9, 10), 300, 42);
        assert_eq!(counts(&a), counts(&b));
        assert_eq!(a.games, 300);
        // Another seed lays other fields.
        let c = classic((9, 9, 10), 300, 43);
        assert_ne!(counts(&a), counts(&c));
    }
}
