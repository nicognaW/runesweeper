//! Game records through the App: written on a win, a loss and every way of abandoning a game,
//! never for a field nobody clicked; forced guesses counted; the gold timer. Driven headlessly
//! through Rune's `Runtime`, as the toolbar tests are.

use rune::Key;

use crate::app::App;
use crate::game::{Cover, Game, Level, Mode, Status};
use crate::record::{Act, End, Layout, Record};
use crate::storage::Saved;
use crate::toolbar_tests::{click, press, runtime, settle, shown, text_in, words};

/// A Classic App on the field drawn as rows, `*` for a mine.
fn on(rows: &[&str]) -> App {
    let w = rows[0].len();
    let mines: Vec<usize> = rows.iter().enumerate().flat_map(|(y, r)| r.char_indices().filter(|c| c.1 == '*').map(move |(x, _)| y * w + x)).collect();
    let mut a = App::with(Saved::default(), 3);
    a.game = Game::with_mines(w, rows.len(), &mines);
    a
}

fn at(a: &App, x: usize, y: usize) -> usize {
    a.game.idx(x, y)
}

fn last(a: &App) -> &Record {
    a.history.last().expect("a record")
}

/// A safe cell with a number, on the App's own field.
fn a_number(a: &App) -> usize {
    (0..a.game.cells.len()).find(|&i| !a.game.cells[i].mine && a.game.cells[i].adj > 0).unwrap()
}

#[test]
fn a_win_and_a_loss_are_recorded() {
    let mut a = on(&["...*", "...."]);
    a.open_at(0);
    assert!(a.history.is_empty(), "nothing yet: the game is on");
    a.open_at(at(&a, 3, 1));
    assert_eq!(a.game.status, Status::Won);
    let r = last(&a);
    assert_eq!((r.end, r.mode, r.dims), (End::Won, Mode::Classic, (4, 2, 1)));
    assert_eq!((r.left, r.right, r.chords), (2, 0, 0));
    assert_eq!((r.three_bv, r.three_bv_solved), (2, 2));
    assert_eq!(r.layout, Layout::Mines(vec![3]));
    assert!(r.at > 0, "dated");

    let mut a = on(&["...*", "...."]);
    a.open_at(0);
    a.open_at(at(&a, 3, 0));
    assert_eq!(a.game.status, Status::Lost);
    assert_eq!(a.history.len(), 1, "one record per game");
    assert_eq!((last(&a).end, last(&a).three_bv_solved), (End::Lost, 1));
}

#[test]
fn every_way_of_leaving_a_clicked_game_records_it_as_abandoned() {
    let first_click = |rt: &mut rune::Runtime<App>| {
        rt.update(|s, _| {
            let i = a_number(s);
            s.open_at(i);
        });
    };
    let paths: [(&str, fn(&mut rune::Runtime<App>)); 6] = [
        ("F2", |rt| press(rt, Key::Function(2))),
        ("Cmd+N", |rt| press(rt, Key::Command('n'))),
        ("the face", |rt| click(rt, "face")),
        ("a level change", |rt| {
            click(rt, "level-chip");
            settle(rt);
            click(rt, "pick-expert");
        }),
        ("a mode change", |rt| {
            click(rt, "level-chip");
            settle(rt);
            click(rt, "mode-no-guess");
        }),
        ("quitting", |rt| press(rt, Key::Command('q'))),
    ];
    for (how, leave) in paths {
        let mut rt = runtime(Saved::default());
        first_click(&mut rt);
        leave(&mut rt);
        assert_eq!(rt.state.history.len(), 1, "{how}");
        let r = last(&rt.state);
        assert_eq!((r.end, r.mode, r.dims, r.left), (End::Abandoned, Mode::Classic, (9, 9, 10), 1), "{how}");
    }
    // A finished game is recorded once, not again when the next field comes.
    let mut rt = runtime(Saved::default());
    rt.update(|s, _| {
        let safe = a_number(s);
        s.open_at(safe);
        let mine = (0..s.game.cells.len()).find(|&i| s.game.cells[i].mine).unwrap();
        s.open_at(mine);
    });
    press(&mut rt, Key::Function(2));
    assert_eq!(rt.state.history.iter().map(|r| r.end).collect::<Vec<_>>(), [End::Lost]);
}

#[test]
fn a_field_nobody_clicked_is_never_recorded() {
    let mut rt = runtime(Saved::default());
    press(&mut rt, Key::Function(2));
    click(&mut rt, "face");
    // Flags alone don't start a game.
    rt.update(|s, _| s.mark_at(0));
    press(&mut rt, Key::Function(2));
    let mut rt2 = runtime(Saved { mode: Mode::NoGuess, ..Saved::default() });
    rt2.update(|s, _| s.mark_at(0));
    // A press on the flag opens nothing and lays nothing.
    rt2.update(|s, _| s.open_at(0));
    press(&mut rt2, Key::Command('q'));
    assert!(rt.state.history.is_empty() && rt2.state.history.is_empty());
}

#[test]
fn the_move_list_rebuilds_the_same_final_field() {
    for (mode, level) in [(Mode::Classic, Level::Intermediate), (Mode::NoGuess, Level::Beginner)] {
        let mut a = App::with(Saved { mode, level, ..Saved::default() }, 11);
        // A flag before the first click, then a game with every kind of click in it.
        a.mark_at(0);
        a.mark_at(0);
        a.open_at(a.game.idx(a.game.w / 2, a.game.h / 2));
        a.finish_laying();
        let mut chorded = false;
        for _ in 0..40 {
            let g = &a.game;
            // Flag a mine next to an open number, then chord that number.
            let Some((n, m)) = (0..g.cells.len()).filter(|&i| g.cells[i].cover == Cover::Open && g.cells[i].adj > 0).find_map(|i| g.neighbours(i).find(|&j| g.cells[j].mine && g.cells[j].cover == Cover::Hidden).map(|m| (i, m))) else { break };
            a.mark_at(m);
            let flags = a.game.neighbours(n).filter(|&j| a.game.cells[j].cover == Cover::Flag).count();
            if flags == a.game.cells[n].adj as usize {
                a.chord_at(n);
                chorded = true;
            }
            if a.game.over() {
                break;
            }
        }
        assert!(chorded, "{mode:?}: the script chorded");
        let end = a.game.clone();
        a.new_game_now();
        let r = last(&a).clone();
        assert!(r.moves.iter().any(|m| m.act == Act::Chord) && r.moves.iter().any(|m| matches!(m.act, Act::Mark { .. })));
        let back = Record::parse(&r.write()).expect("the record reads back");
        let field = back.replay();
        let covers = |g: &Game| g.cells.iter().map(|c| (c.mine, c.cover)).collect::<Vec<_>>();
        assert_eq!(covers(&field), covers(&end), "{mode:?}");
        assert_eq!(field.status, end.status, "{mode:?}");
    }
}

#[test]
fn forced_guesses_are_counted_and_survived_ones_too() {
    // Two true 50/50s, one in each end column; the first click opens the middle.
    let rows = ["*.....*", "......."];
    let mut a = on(&rows);
    a.open_at(at(&a, 3, 0));
    assert_eq!((a.record.forced, a.record.survived), (0, 0), "the first click is never a guess");
    a.open_at(at(&a, 0, 1));
    a.open_at(at(&a, 6, 1));
    assert_eq!(a.game.status, Status::Won);
    assert_eq!((last(&a).forced, last(&a).survived), (2, 2));

    let mut a = on(&rows);
    a.open_at(at(&a, 3, 0));
    a.open_at(at(&a, 0, 1));
    a.open_at(at(&a, 6, 0));
    assert_eq!(a.game.status, Status::Lost);
    assert_eq!((last(&a).forced, last(&a).survived), (2, 1));

    // One mine between two numbers that both see it and its neighbour: the count of mines
    // left proves the far side safe, so only the last pair is a guess.
    let mut a = on(&["....*....", "........."]);
    a.open_at(0);
    a.open_at(at(&a, 8, 0));
    assert_eq!(a.record.forced, 0, "a provably safe cell was there");
    a.open_at(at(&a, 4, 1));
    assert_eq!((last(&a).forced, last(&a).survived), (1, 1));

    // Clicks that open nothing are no guess.
    let mut a = on(&[".*."]);
    a.open_at(0);
    a.open_at(0);
    a.mark_at(1);
    a.open_at(2);
    assert_eq!((last(&a).end, last(&a).forced), (End::Won, 0));
}

#[test]
fn the_timer_turns_gold_when_a_win_beats_the_fastest_win() {
    // A first click opens all but a 50/50 in the last column; its bottom cell wins.
    let win = |rows: &[&str], history: Vec<Record>, seconds: u32| {
        let mut a = on(rows);
        a.history = history;
        a.open_at(0);
        a.seconds = seconds;
        let i = at(&a, a.game.w - 1, 1);
        a.open_at(i);
        assert_eq!(a.game.status, Status::Won);
        a
    };
    let rows = ["...*", "...."];
    let mut a = win(&rows, Vec::new(), 30);
    assert!(!a.new_best, "the first win has no fastest win to beat");
    for (seconds, gold) in [(31, false), (30, false), (12, true), (20, false)] {
        a = win(&rows, a.history, seconds);
        assert_eq!(a.new_best, gold, "{seconds} s");
    }
    // Another level has its own fastest win, so its first is plain and its next beats it.
    let a = win(&["....*", "....."], a.history, 40);
    assert!(!a.new_best, "another level's first win");
    let mut a = win(&["....*", "....."], a.history, 39);
    assert!(a.new_best, "another level's own fastest beaten");
    a.new_game_now();
    assert!(!a.new_best, "a new field starts plain");
}

/// A finished game, for History to list.
fn game(at: u64, mode: Mode, dims: (usize, usize, usize), end: End, seconds: u32) -> Record {
    Record { at: 1_700_000_000_000 + at * 60_000, end, seconds, three_bv: 20, three_bv_solved: 20, left: 25, ..Record::new(mode, dims) }
}

fn sample_history() -> Vec<Record> {
    vec![
        game(0, Mode::Classic, (9, 9, 10), End::Won, 41),
        game(1, Mode::Classic, (9, 9, 10), End::Lost, 5),
        game(2, Mode::Classic, (9, 9, 10), End::Won, 33),
        game(3, Mode::NoGuess, (9, 9, 10), End::Won, 19),
        game(4, Mode::Classic, (30, 16, 99), End::Abandoned, 120),
        game(5, Mode::Classic, (9, 9, 10), End::Abandoned, 2),
        game(6, Mode::Classic, (20, 12, 50), End::Won, 90),
        game(7, Mode::Classic, (10, 10, 12), End::Won, 70),
    ]
}

/// The History list's rows, top to bottom, by their place in History.
fn rows(rt: &rune::Runtime<App>) -> Vec<usize> {
    let mut found: Vec<(f32, usize)> = rt
        .elements()
        .into_iter()
        .filter(|e| !e.exiting)
        .filter_map(|e| Some((e.key.as_deref()?.strip_prefix("history-row-")?.parse().ok()?, e)))
        .map(|(i, e)| (rt.find(e.key.as_deref().unwrap()).unwrap().1, i))
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, i)| i).collect()
}

#[test]
fn the_history_button_opens_records_for_the_current_mode_and_level_newest_first() {
    let mut rt = runtime(Saved::default());
    rt.update(|s, _| s.history = sample_history());
    assert!(!shown(&rt, "history"));
    click(&mut rt, "history-button");
    settle(&mut rt);
    assert!(shown(&rt, "history"), "the button opens History");
    assert_eq!(rows(&rt), [5, 2, 1, 0], "Classic Beginner, newest first");
    // The fastest win sits on top.
    let best = text_in(&rt, "history-best-9x9x10");
    assert!(best.contains("Fastest win") && best.contains("33 s"), "{best}");
    assert!(text_in(&rt, "history-row-2").contains("3BV 20/20 · 25 clicks"), "{}", text_in(&rt, "history-row-2"));
    click(&mut rt, "history-button");
    settle(&mut rt);
    assert!(!shown(&rt, "history"), "and closes it");
}

#[test]
fn every_record_is_reachable_in_history_however_many_there_are() {
    let n = 1500;
    let mut rt = runtime(Saved::default());
    rt.update(|s, _| s.history = (0..n).map(|k| game(k as u64, Mode::Classic, (9, 9, 10), End::Lost, 5)).collect());
    click(&mut rt, "history-button");
    settle(&mut rt);
    let first = rows(&rt);
    assert_eq!(first.first(), Some(&(n - 1)), "newest first");
    assert!(first.len() < 60, "only the rows near the view are drawn, not {}", first.len());
    // Scrolled to the end, the oldest game is there.
    let (x, y) = rt.find("history-list").expect("the list");
    for _ in 0..200 {
        rt.wheel(x, y, -2000.0);
        rt.frame(1.0 / 60.0);
    }
    settle(&mut rt);
    let last = rows(&rt);
    assert_eq!(last.last(), Some(&0), "the oldest game, past {n} others");
    assert!(last.len() < 60, "{} rows drawn", last.len());
    click(&mut rt, "history-row-0");
    settle(&mut rt);
    assert!(rt.state.review.is_some(), "and it opens in Review");
}

#[test]
fn history_filters_by_mode_and_level() {
    let mut rt = runtime(Saved::default());
    rt.update(|s, _| s.history = sample_history());
    click(&mut rt, "history-button");
    settle(&mut rt);
    click(&mut rt, "history-mode-no-guess");
    assert_eq!(rows(&rt), [3]);
    assert!(text_in(&rt, "history-best-9x9x10").contains("19 s"), "No Guess's own fastest win");
    click(&mut rt, "history-mode-classic");
    click(&mut rt, "history-level-expert");
    assert_eq!(rows(&rt), [4]);
    assert!(!shown(&rt, "history-best-30x16x99"), "no win, no fastest");
    // Custom: every size, each with its own fastest win.
    click(&mut rt, "history-level-custom");
    assert_eq!(rows(&rt), [7, 6]);
    assert!(text_in(&rt, "history-best-10x10x12").contains("70 s"));
    assert!(text_in(&rt, "history-best-20x12x50").contains("90 s"));
    click(&mut rt, "history-level-intermediate");
    assert!(rows(&rt).is_empty() && shown(&rt, "history-empty"));
    assert!(rt.state.panel.is_some(), "filtering keeps History open");
}

#[test]
fn a_game_just_played_shows_in_history_and_history_speaks_zh_cn() {
    let mut rt = runtime(Saved { lang: Some(crate::strings::Lang::ZhCn), ..Saved::default() });
    rt.update(|s, _| {
        let i = a_number(s);
        s.open_at(i);
    });
    press(&mut rt, Key::Function(2));
    click(&mut rt, "history-button");
    settle(&mut rt);
    assert_eq!(rows(&rt), [0], "the abandoned game");
    assert!(text_in(&rt, "history-row-0").contains("放弃"), "{}", text_in(&rt, "history-row-0"));
    for w in words(&rt) {
        let rest = w.replace("3BV", "").replace("English", "");
        assert!(!rest.chars().any(|c| c.is_ascii_alphabetic()), "{w:?} is still English");
    }
}
