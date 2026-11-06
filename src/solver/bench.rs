//! The solver benchmark: seeded Classic games played by the bot, seeded Gentle No Guess and
//! No Guess fields generated, and the numbers that describe how each went.
//!
//! Classic's rule: Classic fields laid from the MSVC `rand()` stream seeded once with `seed`, one
//! field after another as the game deals them; the first click on the top-left cell, with
//! XP's first-click relocation. The bot plays every proof, then the cell least likely to be
//! a mine under uniform odds (it does not model the relocation's bias toward the top left).

use std::time::{Duration, Instant};

use super::{Strength, play};
use crate::game::{Game, Mode, Rng};
use crate::generate::{self, Stream};

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

/// How No Guess generation went over a run of fields.
#[derive(Clone, Debug, Default)]
pub struct Generation {
    pub fields: u64,
    /// Uniform fields drawn, accepted or not.
    pub attempts: u64,
    /// Fields the bot then won without a guess (all of them, if generation is right).
    pub clean_wins: u64,
    /// Time to generate each field, ascending.
    pub times: Vec<Duration>,
}

impl Generation {
    /// The share of uniform fields the bot could clear.
    pub fn acceptance(&self) -> f64 {
        self.fields as f64 / self.attempts.max(1) as f64
    }
    /// The generation time `q` of the way up (0.5 the median), by the nearest rank.
    pub fn percentile(&self, q: f64) -> Duration {
        nearest_rank(&self.times, q)
    }
}

/// Generates `fields` No Guess fields of `(columns, rows, mines)`: each one's seed and first
/// click drawn in turn from `seed`, the click anywhere on the field. Each field is then played
/// by the bot, to check it is won with no guess.
pub fn no_guess(dims: (usize, usize, usize), fields: u64, seed: u64) -> Generation {
    generation(Mode::NoGuess, dims, fields, seed)
}

/// [`no_guess`] for any Mode laid on the first click. The bot checks each field at the
/// strength the Mode promises: Gentle for Gentle No Guess, perfect logic otherwise.
pub fn generation(mode: Mode, (w, h, mines): (usize, usize, usize), fields: u64, seed: u64) -> Generation {
    let strength = if mode == Mode::GentleNoGuess { Strength::Gentle } else { Strength::Full };
    let mut t = Generation::default();
    let mut draw = Stream::new(seed);
    for _ in 0..fields {
        let (field_seed, first) = (draw.next_u64(), draw.below(w * h));
        let start = Instant::now();
        let (mut g, attempts) = generate::lay_counting(mode, (w, h, mines), field_seed, first);
        t.times.push(start.elapsed());
        t.fields += 1;
        t.attempts += attempts as u64;
        let played = play::play_at(&mut g, first, strength);
        if played.won && played.guesses == 0 {
            t.clean_wins += 1;
        }
    }
    t.times.sort();
    t
}

/// The time `q` of the way up ascending `times` (0.5 the median), by the nearest rank.
fn nearest_rank(times: &[Duration], q: f64) -> Duration {
    let n = times.len();
    if n == 0 {
        return Duration::ZERO;
    }
    times[((q * n as f64).ceil() as usize).clamp(1, n) - 1]
}

/// Max Guess fields as solver stress tests: how many true 50/50s generation forced, how the
/// bot did against the odds perfect play has on them, and what generation cost.
#[derive(Clone, Debug, Default)]
pub struct Stress {
    pub fields: u64,
    /// True 50/50s over all fields, and the most on one field.
    pub fifty_fifties: u64,
    pub most: usize,
    /// Fields the bot won, and the wins perfect play expects: the sum over fields of 0.5^k.
    pub wins: u64,
    pub expected_wins: f64,
    /// Fields generation left with no true 50/50 at all.
    pub none: u64,
    /// Search steps taken, all fields.
    pub steps: u64,
    /// Time the bot spent playing the fields.
    pub play_time: Duration,
    /// Time to generate each field, ascending.
    pub times: Vec<Duration>,
}

impl Stress {
    pub fn fifty_fifties_per_field(&self) -> f64 {
        self.fifty_fifties as f64 / self.fields.max(1) as f64
    }
    pub fn win_rate(&self) -> f64 {
        self.wins as f64 / self.fields.max(1) as f64
    }
    /// Perfect play's win rate: the mean of 0.5^k.
    pub fn expected_rate(&self) -> f64 {
        self.expected_wins / self.fields.max(1) as f64
    }
    pub fn ms_per_game(&self) -> f64 {
        self.play_time.as_secs_f64() * 1000.0 / self.fields.max(1) as f64
    }
    pub fn percentile(&self, q: f64) -> Duration {
        nearest_rank(&self.times, q)
    }
}

/// Generates `fields` Max Guess fields of `(columns, rows, mines)` with `budget` search steps
/// each (the game's own budget if `None`), each one's seed and first click drawn in turn from
/// `seed` as for [`generation`]. The bot then plays each field once.
pub fn max_guess((w, h, mines): (usize, usize, usize), fields: u64, seed: u64, budget: Option<u32>) -> Stress {
    let mut t = Stress::default();
    let mut draw = Stream::new(seed);
    let budget = budget.unwrap_or(generate::budget((w, h, mines)));
    for _ in 0..fields {
        let (field_seed, first) = (draw.next_u64(), draw.below(w * h));
        let start = Instant::now();
        let laid = generate::searched((w, h, mines), field_seed, first, budget);
        t.times.push(start.elapsed());
        let k = laid.fifty_fifties.len();
        t.fields += 1;
        t.fifty_fifties += k as u64;
        t.most = t.most.max(k);
        t.none += (k == 0) as u64;
        t.expected_wins += 0.5f64.powi(k as i32);
        t.steps += laid.steps as u64;
        let mut g = laid.game;
        let start = Instant::now();
        let played = play::play(&mut g, first);
        t.play_time += start.elapsed();
        t.wins += played.won as u64;
    }
    t.times.sort();
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
    fn no_guess_fields_are_counted_and_every_one_is_won_without_a_guess() {
        let a = no_guess((9, 9, 10), 40, 5);
        assert_eq!((a.fields, a.times.len()), (40, 40));
        assert_eq!(a.clean_wins, 40, "the bot wins every field with no guess");
        assert!(a.attempts >= 40 && a.acceptance() <= 1.0 && a.acceptance() > 0.0);
        assert!(a.percentile(0.5) <= a.percentile(0.99));
        // The same seed draws the same fields.
        let b = no_guess((9, 9, 10), 40, 5);
        assert_eq!((a.attempts, a.clean_wins), (b.attempts, b.clean_wins));
    }

    #[test]
    fn gentle_fields_are_counted_and_every_one_is_won_by_pattern_play() {
        let a = generation(Mode::GentleNoGuess, (16, 16, 40), 20, 5);
        assert_eq!((a.fields, a.times.len()), (20, 20));
        assert_eq!(a.clean_wins, 20, "the Gentle bot wins every field with no guess");
        // The same draws: no uniform field passes at Gentle strength that fails at full strength.
        assert!(a.attempts >= no_guess((16, 16, 40), 20, 5).attempts);
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
