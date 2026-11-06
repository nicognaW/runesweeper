//! The generator's seam: (mode, level, seed, first click) in, a field out.

use crate::game::{Cover, Mode, Status};
use crate::generate::{lay, max_guess};
use crate::solver::fifty::fifty_fifties;
use crate::solver::{Fact, Strength, View, deduce, play};

const BEGINNER: (usize, usize, usize) = (9, 9, 10);
const INTERMEDIATE: (usize, usize, usize) = (16, 16, 40);
const EXPERT: (usize, usize, usize) = (30, 16, 99);

fn mines(g: &crate::game::Game) -> Vec<usize> {
    g.mine_cells()
}

/// A first click spread over the field, corners and edges included.
fn first_click(seed: u64, (w, h, _): (usize, usize, usize)) -> usize {
    (seed as usize * 37 + seed as usize / 3) % (w * h)
}

#[test]
fn the_same_mode_level_seed_and_first_click_lay_the_same_field() {
    for seed in 0..20 {
        let first = first_click(seed, BEGINNER);
        let a = lay(Mode::NoGuess, BEGINNER, seed, first);
        let b = lay(Mode::NoGuess, BEGINNER, seed, first);
        assert_eq!(mines(&a), mines(&b), "seed {seed}");
        assert_eq!(mines(&a).len(), 10);
    }
    // Another seed, or another first click, lays another field.
    let a = mines(&lay(Mode::NoGuess, BEGINNER, 1, 40));
    assert_ne!(a, mines(&lay(Mode::NoGuess, BEGINNER, 2, 40)));
    assert_ne!(a, mines(&lay(Mode::NoGuess, BEGINNER, 1, 0)));
}

#[test]
fn a_field_is_the_same_whatever_came_before_it() {
    // Generation keeps no state between calls: laying other fields first changes nothing.
    let alone = mines(&lay(Mode::NoGuess, INTERMEDIATE, 77, 100));
    for seed in 0..5 {
        lay(Mode::NoGuess, INTERMEDIATE, seed, 3);
    }
    assert_eq!(alone, mines(&lay(Mode::NoGuess, INTERMEDIATE, 77, 100)));
}

#[test]
fn a_no_guess_field_has_nothing_open_and_its_mine_count() {
    let g = lay(Mode::NoGuess, EXPERT, 5, 200);
    assert_eq!((g.w, g.h, g.mines), (30, 16, 99));
    assert_eq!(mines(&g).len(), 99);
    assert!(g.cells.iter().all(|c| c.cover == Cover::Hidden));
    assert_eq!(g.status, Status::Ready);
}

/// The property: in No Guess the first click opens an area and the bot wins with no guess.
fn no_guess_fields_are_won_without_a_guess(level: (usize, usize, usize), seeds: u64) {
    for seed in 0..seeds {
        let first = first_click(seed, level);
        let mut g = lay(Mode::NoGuess, level, seed, first);
        let (w, _, _) = level;
        assert!(!g.cells[first].mine, "seed {seed}: the first click is safe");
        assert_eq!(g.cells[first].adj, 0, "seed {seed}: the first click opens an area ({}, {})", first % w, first / w);
        let played = play::play(&mut g, first);
        assert!(played.won && played.guesses == 0, "seed {seed}: {played:?}");
    }
}

#[test]
fn no_guess_beginner_is_won_without_a_guess() {
    no_guess_fields_are_won_without_a_guess(BEGINNER, 200);
}

#[test]
fn no_guess_intermediate_is_won_without_a_guess() {
    no_guess_fields_are_won_without_a_guess(INTERMEDIATE, 50);
}

#[test]
fn no_guess_expert_is_won_without_a_guess() {
    no_guess_fields_are_won_without_a_guess(EXPERT, 10);
}

#[test]
fn no_guess_custom_fields_are_won_without_a_guess() {
    // A small dense field and a long thin one.
    no_guess_fields_are_won_without_a_guess((9, 9, 20), 20);
    no_guess_fields_are_won_without_a_guess((30, 9, 40), 10);
}

#[test]
fn classic_is_the_xp_field_from_its_rand_stream() {
    // Classic ignores the first click when laying: XP moves a first-click mine at the click.
    let a = lay(Mode::Classic, EXPERT, 9, 0);
    let b = crate::game::Game::new(30, 16, 99, crate::game::Rng::new(9));
    assert_eq!(mines(&a), mines(&b));
    assert_eq!(mines(&lay(Mode::Classic, EXPERT, 9, 250)), mines(&b));
}

#[test]
fn a_stopped_search_gives_up_and_a_plain_one_never_does() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};
    // A field logic can almost never finish: without a stop it would search on and on.
    let dense = (30, 24, 600);
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let setter = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        flag.store(true, Ordering::Relaxed);
    });
    let t = Instant::now();
    crate::generate::stoppable(stop, || lay(Mode::NoGuess, dense, 1, 300));
    assert!(t.elapsed() < Duration::from_secs(5), "it stopped soon after asked: {:?}", t.elapsed());
    setter.join().unwrap();
    assert!(!crate::generate::stopped(), "outside a stoppable call nothing stops");
    // Not stopped, the same call lays the very field a plain one does.
    let quiet = Arc::new(AtomicBool::new(false));
    let a = crate::generate::stoppable(quiet, || lay(Mode::NoGuess, INTERMEDIATE, 9, 100));
    assert_eq!(mines(&a), mines(&lay(Mode::NoGuess, INTERMEDIATE, 9, 100)));
}

/// The property: in Gentle No Guess the first click opens an area and the bot at Gentle
/// strength (one or two numbers at a time, the mine count near the end) wins with no guess.
/// Every such field is also one perfect logic finishes: Gentle fields are No Guess fields.
fn gentle_fields_are_won_by_pattern_play(level: (usize, usize, usize), seeds: u64) {
    for seed in 0..seeds {
        let first = first_click(seed, level);
        let mut g = lay(Mode::GentleNoGuess, level, seed, first);
        assert_eq!(mines(&g).len(), level.2, "seed {seed}");
        assert!(!g.cells[first].mine, "seed {seed}: the first click is safe");
        assert_eq!(g.cells[first].adj, 0, "seed {seed}: the first click opens an area");
        assert!(play::clears(&g, first, Strength::Full), "seed {seed}: perfect logic finishes it too");
        let played = play::play_at(&mut g, first, Strength::Gentle);
        assert!(played.won && played.guesses == 0, "seed {seed}: {played:?}");
    }
}

#[test]
fn gentle_beginner_is_won_by_pattern_play() {
    gentle_fields_are_won_by_pattern_play(BEGINNER, 200);
}

#[test]
fn gentle_intermediate_is_won_by_pattern_play() {
    gentle_fields_are_won_by_pattern_play(INTERMEDIATE, 30);
}

#[test]
fn gentle_expert_is_won_by_pattern_play() {
    gentle_fields_are_won_by_pattern_play(EXPERT, 10);
}

#[test]
fn gentle_custom_fields_are_won_by_pattern_play() {
    gentle_fields_are_won_by_pattern_play((9, 9, 20), 10);
    gentle_fields_are_won_by_pattern_play((30, 9, 40), 5);
}

#[test]
fn gentle_fields_are_the_same_from_the_same_seed_and_first_click() {
    for seed in 0..20 {
        let first = first_click(seed, BEGINNER);
        let a = mines(&lay(Mode::GentleNoGuess, BEGINNER, seed, first));
        assert_eq!(a, mines(&lay(Mode::GentleNoGuess, BEGINNER, seed, first)), "seed {seed}");
    }
    let a = mines(&lay(Mode::GentleNoGuess, BEGINNER, 1, 40));
    assert_ne!(a, mines(&lay(Mode::GentleNoGuess, BEGINNER, 2, 40)));
    assert_ne!(a, mines(&lay(Mode::GentleNoGuess, BEGINNER, 1, 0)));
}

#[test]
fn some_no_guess_fields_need_more_than_pattern_play() {
    // Gentle is a real restriction: over a few seeds, No Guess lays fields that only
    // perfect logic finishes.
    let harder = (0..40).any(|seed| {
        let first = first_click(seed, INTERMEDIATE);
        !play::clears(&lay(Mode::NoGuess, INTERMEDIATE, seed, first), first, Strength::Gentle)
    });
    assert!(harder);
}

/// The property: in Max Guess the first click opens an area, and the full-strength bot,
/// opening the safe cell whenever it must guess, stalls only on true 50/50s: two cells holding
/// one mine whose swap would change no number on the field. It finishes the field, and the
/// count reported before play is the number of stalls.
fn max_guess_stalls_are_true_fifty_fifties(level: (usize, usize, usize), seeds: u64) -> usize {
    let mut total = 0;
    for seed in 0..seeds {
        let first = first_click(seed, level);
        let laid = max_guess(level, seed, first);
        let mut g = laid.game.clone();
        assert_eq!(mines(&g).len(), level.2, "seed {seed}");
        assert_eq!(mines(&g), mines(&lay(Mode::MaxGuess, level, seed, first)), "seed {seed}: lay gives the same field");
        assert_eq!(g.cells[first].adj, 0, "seed {seed}: the first click opens an area");
        assert!(!g.cells[first].mine);
        let truth = g.clone();
        let mut facts = vec![Fact::Unknown; g.cells.len()];
        let mut stalls = 0;
        g.open(first);
        while g.status == Status::Playing {
            let v = View::of(&g);
            let proofs = deduce(&v, &facts, Strength::Full);
            if !proofs.is_empty() {
                for p in proofs {
                    assert_eq!(p.mine, truth.cells[p.cell].mine, "seed {seed}: proofs are true");
                    if p.mine {
                        facts[p.cell] = Fact::Mine;
                    } else {
                        g.open(p.cell);
                    }
                }
                continue;
            }
            stalls += 1;
            let pairs = fifty_fifties(&v, &facts);
            let Some(&(a, b)) = pairs.first() else { panic!("seed {seed}: stall {stalls} is not a true 50/50") };
            assert_ne!(truth.cells[a].mine, truth.cells[b].mine, "seed {seed}: one mine in ({a}, {b})");
            let swap_unseen = (0..g.cells.len())
                .filter(|&c| c != a && c != b && !truth.cells[c].mine)
                .all(|c| truth.neighbours(c).any(|n| n == a) == truth.neighbours(c).any(|n| n == b));
            assert!(swap_unseen, "seed {seed}: no number can ever tell ({a}, {b}) apart");
            let (mine, safe) = if truth.cells[a].mine { (a, b) } else { (b, a) };
            facts[mine] = Fact::Mine;
            g.open(safe);
        }
        assert_eq!(g.status, Status::Won, "seed {seed}");
        assert_eq!(stalls, laid.fifty_fifties.len(), "seed {seed}: the reported count");
        total += stalls;
    }
    total
}

#[test]
fn max_guess_beginner_stalls_only_on_true_fifty_fifties() {
    let total = max_guess_stalls_are_true_fifty_fifties(BEGINNER, 12);
    assert!(total >= 30, "the search finds them: {total} over 12 fields");
}

#[test]
fn max_guess_intermediate_stalls_only_on_true_fifty_fifties() {
    let total = max_guess_stalls_are_true_fifty_fifties(INTERMEDIATE, 2);
    assert!(total >= 10, "the search finds them: {total} over 2 fields");
}

#[test]
fn max_guess_expert_stalls_only_on_true_fifty_fifties() {
    let total = max_guess_stalls_are_true_fifty_fifties(EXPERT, 1);
    assert!(total >= 6, "the search finds them: {total}");
}

#[test]
fn max_guess_custom_fields_stall_only_on_true_fifty_fifties() {
    // A small dense field and a long thin one.
    max_guess_stalls_are_true_fifty_fifties((9, 9, 20), 3);
    max_guess_stalls_are_true_fifty_fifties((30, 9, 40), 1);
}

#[test]
fn the_search_budget_is_fixed_per_level_and_scales_with_custom_sizes() {
    use crate::generate::budget;
    assert_eq!([budget(BEGINNER), budget(INTERMEDIATE), budget(EXPERT)], [2000, 1500, 1000]);
    assert!(budget((20, 20, 80)) < budget(INTERMEDIATE) && budget((20, 20, 80)) > budget(EXPERT));
    assert!((300..budget(EXPERT)).contains(&budget((30, 24, 667))), "the largest Custom field gets fewer, dearer steps");
}

#[test]
fn max_guess_fields_are_the_same_from_the_same_seed_and_first_click() {
    for seed in 0..4 {
        let first = first_click(seed, BEGINNER);
        let a = max_guess(BEGINNER, seed, first);
        let b = max_guess(BEGINNER, seed, first);
        assert_eq!((mines(&a.game), &a.fifty_fifties, a.steps), (mines(&b.game), &b.fifty_fifties, b.steps), "seed {seed}");
    }
    let a = mines(&lay(Mode::MaxGuess, BEGINNER, 1, 40));
    assert_ne!(a, mines(&lay(Mode::MaxGuess, BEGINNER, 2, 40)));
    assert_ne!(a, mines(&lay(Mode::MaxGuess, BEGINNER, 1, 0)));
}

#[test]
fn a_max_guess_field_is_a_fixed_field_whatever_the_machine() {
    // The search counts steps, never time, and draws from SplitMix64: this field is the same
    // on every platform, native and wasm.
    let laid = max_guess(BEGINNER, 7, 40);
    assert_eq!(mines(&laid.game), MAX_GUESS_BEGINNER_7_40);
    assert_eq!(laid.steps, 2000);
}

#[test]
fn either_cell_of_a_max_guess_fifty_fifty_is_the_mine_as_often() {
    // Nothing about the search may favour one side: a player who always guessed the cell with
    // fewer neighbours (the corner or the edge), or the first, would win more than half.
    let (mut pairs, mut first_cell, mut fewer) = (0, 0, 0);
    for seed in 0..100 {
        let first = first_click(seed, BEGINNER);
        let laid = crate::generate::searched(BEGINNER, seed, first, 200);
        for &(a, b) in &laid.fifty_fifties {
            pairs += 1;
            first_cell += laid.game.cells[a].mine as u32;
            let n = |i: usize| laid.game.neighbours(i).count();
            fewer += laid.game.cells[if n(a) <= n(b) { a } else { b }].mine as u32;
        }
    }
    // Within three standard deviations of half.
    let band = |k: u32| (k as f64 / pairs as f64 - 0.5).abs() < 1.5 / (pairs as f64).sqrt();
    assert!(pairs > 100, "{pairs} pairs");
    assert!(band(first_cell), "the first cell is the mine in {first_cell} of {pairs}");
    assert!(band(fewer), "the cell with fewer neighbours is the mine in {fewer} of {pairs}");
}

/// `max_guess(BEGINNER, 7, 40)`'s mines, recorded once.
const MAX_GUESS_BEGINNER_7_40: &[usize] = &[0, 1, 2, 12, 18, 19, 20, 21, 22, 70];
