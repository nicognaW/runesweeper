//! How strong the solver bot is on Classic fields, per level.
//! `cargo run --release --example solver -- [games per level] [seed]` (defaults: 1000, 1)
//!
//! Rules: Classic (XP's `rand()` stream, seeded once, one field after another), first click
//! on the top-left cell with XP's first-click relocation; the same seed gives the same games.
use runesweeper::{game::Level, solver::bench};

fn main() {
    let mut args = std::env::args().skip(1);
    let usage = "usage: solver [games per level] [seed]";
    let games: u64 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1000);
    let seed: u32 = args.next().map(|a| a.parse().expect(usage)).unwrap_or(1);
    println!("Classic, first click top left with XP relocation, {games} games per level, seed {seed}");
    println!("{:<13} {:>7}  {:<17} {:>10} {:>14} {:>9}", "level", "win", "95% Wilson", "guess/win", "no-guess wins", "ms/game");
    for level in [Level::Beginner, Level::Intermediate, Level::Expert] {
        let t = bench::classic(level.dims().unwrap(), games, seed);
        let (lo, hi) = bench::wilson(t.wins, t.games);
        println!(
            "{:<13} {:>6.2}%  {:>6.2}% – {:>6.2}% {:>10.2} {:>13.2}% {:>9.3}",
            level.name(),
            100.0 * t.win_rate(),
            100.0 * lo,
            100.0 * hi,
            t.guesses_per_win(),
            100.0 * t.clean_share(),
            t.ms_per_game(),
        );
    }
}
