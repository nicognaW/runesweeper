//! How strong the solver bot is on Classic fields, and what No Guess generation costs, per
//! level. `cargo run --release --example solver -- [games per level] [seed] [No Guess fields]`
//! (defaults: 1000, 1, 1000)
//!
//! Classic: XP's `rand()` stream, seeded once, one field after another; first click on the
//! top-left cell with XP's first-click relocation. No Guess: each field's seed and first click
//! (anywhere on the field) drawn in turn from the seed; uniform fields with the 3×3 around the
//! click clear are redrawn until the full-strength bot clears one without guessing, and the
//! bot then plays it to check. The same seed gives the same games and fields.
use std::time::Duration;

use runesweeper::{
    game::Level,
    solver::bench,
    strings::{Lang, S},
};

const LEVELS: [Level; 3] = [Level::Beginner, Level::Intermediate, Level::Expert];

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() {
    let mut args = std::env::args().skip(1);
    let usage = "usage: solver [games per level] [seed] [No Guess fields per level]";
    let games: u64 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1000);
    let seed: u32 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1);
    let fields: u64 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1000);
    println!("Classic, first click top left with XP relocation, {games} games per level, seed {seed}");
    println!("{:<13} {:>7}  {:<17} {:>10} {:>14} {:>9}", "level", "win", "95% Wilson", "guess/win", "no-guess wins", "ms/game");
    for level in LEVELS {
        let t = bench::classic(level.dims().unwrap(), games, seed);
        let (lo, hi) = bench::wilson(t.wins, t.games);
        println!(
            "{:<13} {:>6.2}%  {:>6.2}% – {:>6.2}% {:>10.2} {:>13.2}% {:>9.3}",
            S::level(level).get(Lang::En),
            100.0 * t.win_rate(),
            100.0 * lo,
            100.0 * hi,
            t.guesses_per_win(),
            100.0 * t.clean_share(),
            t.ms_per_game(),
        );
    }
    println!();
    println!("No Guess, first click anywhere with its 3×3 clear, {fields} fields per level, seed {seed}");
    println!("{:<13} {:>10}  {:<17} {:>9} {:>9} {:>9} {:>14}", "level", "accepted", "95% Wilson", "p50 ms", "p99 ms", "max ms", "no-guess wins");
    for level in LEVELS {
        let g = bench::no_guess(level.dims().unwrap(), fields, seed as u64);
        let (lo, hi) = bench::wilson(g.fields, g.attempts);
        println!(
            "{:<13} {:>9.2}%  {:>6.2}% – {:>6.2}% {:>9.2} {:>9.2} {:>9.2} {:>13.2}%",
            S::level(level).get(Lang::En),
            100.0 * g.acceptance(),
            100.0 * lo,
            100.0 * hi,
            ms(g.percentile(0.5)),
            ms(g.percentile(0.99)),
            ms(g.percentile(1.0)),
            100.0 * g.clean_wins as f64 / g.fields.max(1) as f64,
        );
    }
}
