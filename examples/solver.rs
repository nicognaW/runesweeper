//! How strong the solver bot is on Classic fields, what Gentle No Guess and No Guess
//! generation cost, and Max Guess fields as stress tests, per level.
//! `cargo run --release --example solver -- [games per level] [seed] [fields per mode and level] [Max Guess fields per level]`
//! (defaults: 1000, 1, 1000, 100)
//!
//! Classic: XP's `rand()` stream, seeded once, one field after another; first click on the
//! top-left cell with XP's first-click relocation. No Guess: each field's seed and first click
//! (anywhere on the field) drawn in turn from the seed; uniform fields with the 3×3 around the
//! click clear are redrawn until the full-strength bot clears one without guessing, and the
//! bot then plays it to check. Gentle No Guess is the same with the bot at Gentle strength
//! (one or two numbers at a time, the mine count near the end), which also does the check.
//! Max Guess: the same draws, each field searched for its game's budget of steps; the bot then
//! plays each once, and its win rate is set against 0.5^k, what perfect play wins on a field
//! with k true 50/50s. The same seed gives the same games and fields.
use std::time::Duration;

use runesweeper::{
    game::{Level, Mode},
    generate,
    solver::bench,
    strings::{Lang, S},
};

const LEVELS: [Level; 3] = [Level::Beginner, Level::Intermediate, Level::Expert];

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() {
    let mut args = std::env::args().skip(1);
    let usage = "usage: solver [games per level] [seed] [fields per mode and level] [Max Guess fields per level]";
    let games: u64 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1000);
    let seed: u32 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1);
    let fields: u64 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1000);
    let max_fields: u64 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(100);
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
    for mode in [Mode::GentleNoGuess, Mode::NoGuess] {
        println!();
        generation(mode, fields, seed as u64);
    }
    println!();
    stress(max_fields, seed as u64);
}

/// Max Guess fields as stress tests: the true 50/50s each forces, the bot's wins against the
/// 0.5^k perfect play expects, its time per game, and the search's time per field.
fn stress(fields: u64, seed: u64) {
    println!("Max Guess, first click anywhere with its 3×3 clear, {fields} fields per level, seed {seed}, the game's search budget");
    println!(
        "{:<13} {:>6} {:>8} {:>5} {:>7}  {:<17} {:>8} {:>9} {:>9} {:>9} {:>9}",
        "level", "steps", "50/50s", "most", "bot win", "95% Wilson", "0.5^k", "ms/game", "p50 ms", "p99 ms", "max ms"
    );
    for level in LEVELS {
        let dims = level.dims().unwrap();
        let t = bench::max_guess(dims, fields, seed, None);
        let (lo, hi) = bench::wilson(t.wins, t.fields);
        println!(
            "{:<13} {:>6} {:>8.2} {:>5} {:>6.2}%  {:>6.2}% – {:>6.2}% {:>7.2}% {:>9.3} {:>9.1} {:>9.1} {:>9.1}",
            S::level(level).get(Lang::En),
            generate::budget(dims),
            t.fifty_fifties_per_field(),
            t.most,
            100.0 * t.win_rate(),
            100.0 * lo,
            100.0 * hi,
            100.0 * t.expected_rate(),
            t.ms_per_game(),
            ms(t.percentile(0.5)),
            ms(t.percentile(0.99)),
            ms(t.percentile(1.0)),
        );
    }
}

/// Acceptance and generation time per level for `mode`, each field checked by the bot.
fn generation(mode: Mode, fields: u64, seed: u64) {
    let name = S::mode(mode).get(Lang::En);
    println!("{name}, first click anywhere with its 3×3 clear, {fields} fields per level, seed {seed}");
    println!("{:<13} {:>10}  {:<17} {:>9} {:>9} {:>9} {:>14}", "level", "accepted", "95% Wilson", "p50 ms", "p99 ms", "max ms", "no-guess wins");
    for level in LEVELS {
        let g = bench::generation(mode, level.dims().unwrap(), fields, seed);
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
