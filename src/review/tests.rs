//! Review annotations for scripted games: each marker at the move it belongs to, with the
//! reason a hint would give; and the speed figures against values worked by hand.

use super::*;
use crate::game::{Mode, Status};
use crate::hint::{Guess, Technique, Verdict};
use crate::record::{Act, End, Layout, Move, Record};

/// A Classic record on the field drawn as rows (`*` a mine), with `moves` as (ms, act, x, y).
fn scripted(rows: &[&str], end: End, moves: &[(u32, char, usize, usize)]) -> Record {
    let w = rows[0].len();
    let mines: Vec<usize> = rows.iter().enumerate().flat_map(|(y, r)| r.char_indices().filter(|c| c.1 == '*').map(move |(x, _)| y * w + x)).collect();
    let moves = moves
        .iter()
        .map(|&(ms, a, x, y)| {
            let act = match a {
                'o' => Act::Open,
                'c' => Act::Chord,
                _ => Act::Mark { marks: false },
            };
            Move { ms, act, cell: y * w + x }
        })
        .collect();
    Record { end, layout: Layout::Mines(mines.clone()), moves, ..Record::new(Mode::Classic, (w, rows.len(), mines.len())) }
}

/// Two true 50/50s, one at each end of the top row; between them, after the first click
/// opens the middle, (6,2), (7,2) and (9,2) are provably safe and (5,2), (8,2) mines.
const FIELD: [&str; 3] = ["*..........*", "............", "**...*..*.**"];

/// The first click opens the middle; then the left 50/50's safe half while safe cells could
/// be proven (a needless guess); a flag on (5,2), then (6,2) and (7,2) one at a time, which
/// one chord on the satisfied 1 at (6,1) would have opened; a 6.6 s pause before the provably
/// safe (9,2) (a missed deduction); and last the right 50/50, forced.
fn game() -> Record {
    scripted(&FIELD, End::Won, &[(0, 'o', 5, 0), (1500, 'o', 0, 1), (2500, 'f', 5, 2), (3000, 'o', 6, 2), (3400, 'o', 7, 2), (10_000, 'o', 9, 2), (11_000, 'o', 11, 1)])
}

fn kinds(r: &Review) -> Vec<(usize, Kind)> {
    r.markers.iter().map(|m| (m.at, m.kind.clone())).collect()
}

#[test]
fn a_scripted_game_gets_each_marker_at_its_move() {
    let r = Review::of(&game());
    assert_eq!(
        kinds(&r),
        [
            (1, Kind::Needless),
            (3, Kind::Chord { number: 18, cells: vec![30, 31], saved: 1 }),
            (5, Kind::Missed { pause_ms: 6600 }),
            (6, Kind::Forced { fifty: true }),
        ]
    );
}

#[test]
fn a_needless_guess_or_a_missed_deduction_shows_the_proof_of_a_safe_cell() {
    let r = Review::of(&game());
    let safe_then = [[30, 31, 33].as_slice(), &[33]];
    let shown: Vec<usize> = (0..r.markers.len()).filter(|&k| matches!(r.markers[k].kind, Kind::Needless | Kind::Missed { .. })).collect();
    assert_eq!(shown.len(), 2);
    for (k, safe) in shown.into_iter().zip(safe_then) {
        let (kind, h) = (&r.markers[k].kind, r.hint(k));
        let t = h.target.expect("a cell");
        assert_eq!(t.verdict, Verdict::Safe, "{kind:?}");
        assert!(safe.contains(&t.cell), "{kind:?} names {}", t.cell);
        assert!(!h.numbers.is_empty() && !h.steps.is_empty(), "with its numbers and reason");
    }
}

#[test]
fn a_forced_guess_says_whether_it_was_a_true_fifty_fifty() {
    let r = Review::of(&game());
    let forced = r.hint(r.markers.iter().position(|m| matches!(m.kind, Kind::Forced { .. })).unwrap());
    assert_eq!(forced.technique, Technique::Guess(Guess::FiftyFifty));
    assert_eq!(forced.target.map(|t| t.verdict), Some(Verdict::Guess));

    // A first click on a 2 that proves nothing: the next click is forced, but no 50/50.
    let r = Review::of(&scripted(&[".*....", ".*....", ".*...*"], End::Abandoned, &[(0, 'o', 0, 0), (900, 'o', 4, 0)]));
    assert_eq!(kinds(&r), [(1, Kind::Forced { fifty: false })]);
}

#[test]
fn clicks_that_open_nothing_and_short_pauses_are_not_marked() {
    // A click on an open cell, a flag, and quick play on proven cells.
    let r = Review::of(&scripted(&FIELD, End::Abandoned, &[(0, 'o', 5, 0), (400, 'o', 5, 0), (800, 'f', 5, 2), (5_700, 'o', 9, 2)]));
    assert_eq!(kinds(&r), [], "a pause of 4.9 s is no missed deduction");
    // A chord, as a chord, saves nothing.
    let r = Review::of(&scripted(&FIELD, End::Abandoned, &[(0, 'o', 5, 0), (800, 'f', 5, 2), (1200, 'c', 6, 1)]));
    assert_eq!(kinds(&r), []);
}

#[test]
fn the_field_at_a_marker_is_the_moment_before_its_move() {
    let mut r = Review::of(&game());
    assert_eq!((r.at, r.game.status), (r.moves(), Status::Won), "a review opens on the end of the game");
    r.select(2);
    let m = &r.markers[2];
    assert_eq!(r.at, m.at);
    assert!(r.selected().is_some_and(|(_, h)| *h == r.hint(2)), "with its reason");
    assert_eq!(r.game.cells[33].cover, crate::game::Cover::Hidden, "(9,2) is not open yet");
    assert_eq!(r.game.cells[31].cover, crate::game::Cover::Open, "(7,2) is");
    r.seek(0);
    assert!(r.game.cells.iter().all(|c| c.cover == crate::game::Cover::Hidden));
    assert_eq!(r.marker, None, "seeking leaves the marker");
    r.step(1);
    assert_eq!(r.at, 1);
    r.step(-5);
    assert_eq!(r.at, 0);
    r.step(99);
    assert_eq!(r.at, r.moves());
}

#[test]
fn speed_figures_match_values_worked_by_hand() {
    // FIELD: one opening and five numbers no opening reaches, 3BV 6, in 7 clicks and 11 s.
    let s = Review::of(&game()).stats;
    assert_eq!((s.three_bv, s.solved, s.clicks), (6, 6, 7));
    assert!((s.seconds - 11.0).abs() < 1e-9);
    assert!((s.per_second() - 6.0 / 11.0).abs() < 1e-9, "3BV/s {}", s.per_second());
    assert!((s.efficiency() - 6.0 / 7.0).abs() < 1e-9, "efficiency {}", s.efficiency());

    // Lost on the second click: only the opening is solved.
    let s = Review::of(&scripted(&FIELD, End::Lost, &[(0, 'o', 5, 0), (2000, 'o', 0, 0)])).stats;
    assert_eq!((s.three_bv, s.solved, s.clicks), (6, 1, 2));
    assert!((s.per_second() - 0.5).abs() < 1e-9 && (s.efficiency() - 0.5).abs() < 1e-9);

    // A strip cleared by one click on its opening: 3BV 1 in 1 click, at the timer's 1 s.
    let s = Review::of(&Record { seconds: 1, ..scripted(&["*........"; 1], End::Won, &[(0, 'o', 8, 0)]) }).stats;
    assert_eq!((s.three_bv, s.solved, s.clicks), (1, 1, 1));
    assert!((s.per_second() - 1.0).abs() < 1e-9 && (s.efficiency() - 1.0).abs() < 1e-9);
}

#[test]
fn reviewing_never_changes_the_record() {
    let r0 = game();
    let mut r = Review::of(&r0);
    for k in 0..r.markers.len() {
        r.select(k);
    }
    r.seek(0);
    assert_eq!(r.record, r0);
}
