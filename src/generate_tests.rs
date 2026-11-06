//! The generator's seam: (mode, level, seed, first click) in, a field out.

use crate::game::{Cover, Mode, Status};
use crate::generate::lay;
use crate::solver::play;

const BEGINNER: (usize, usize, usize) = (9, 9, 10);
const INTERMEDIATE: (usize, usize, usize) = (16, 16, 40);
const EXPERT: (usize, usize, usize) = (30, 16, 99);

fn mines(g: &crate::game::Game) -> Vec<usize> {
    (0..g.cells.len()).filter(|&i| g.cells[i].mine).collect()
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
