//! Review through the App, driven headlessly through Rune's `Runtime` as the toolbar tests
//! are: opening a review from the result banner and from History in every mode, stepping
//! through it, selecting markers (the field at that moment, the reason in both languages),
//! leaving it, and never changing a record.

use rune::{Key, Runtime};

use crate::app::App;
use crate::game::{Cover, Level, Mode, Status};
use crate::record::{Act, End, Layout, Move, Record};
use crate::storage::Saved;
use crate::strings::Lang;
use crate::toolbar_tests::{click, press, runtime, settle, shown, text_in, words};

/// Ends a game in `mode` the quick way: a first click in the middle, then a mine.
fn lose(rt: &mut Runtime<App>) {
    rt.update(|s, _| {
        let first = s.game.idx(s.game.w / 2, s.game.h / 2);
        s.open_at(first);
        s.finish_laying();
        let mine = (0..s.game.cells.len()).find(|&i| s.game.cells[i].mine && s.game.cells[i].cover == Cover::Hidden).unwrap();
        s.open_at(mine);
    });
    rt.frame(1.0 / 60.0);
}

/// The review's field as the board draws it: open cells.
fn drawn_open(rt: &Runtime<App>) -> Vec<usize> {
    let f = crate::board::field_key(&rt.state);
    let g = &rt.state.review.as_ref().unwrap().game;
    (0..g.cells.len()).filter(|&i| shown(rt, &format!("o{f}-{i}"))).collect()
}

/// The cells the review's field has open, and once lost the mines and wrong flags laid bare.
fn open_cells(rt: &Runtime<App>) -> Vec<usize> {
    let g = &rt.state.review.as_ref().unwrap().game;
    let lost = g.status == Status::Lost;
    (0..g.cells.len()).filter(|&i| g.cells[i].cover == Cover::Open || (lost && g.cells[i].mine != (g.cells[i].cover == Cover::Flag))).collect()
}

#[test]
fn the_result_banner_opens_a_review_of_the_game_just_played_in_every_mode() {
    for mode in Mode::ALL {
        let mut rt = runtime(Saved { mode, level: Level::Beginner, ..Saved::default() });
        lose(&mut rt);
        assert_eq!(rt.state.game.status, Status::Lost, "{mode:?}");
        settle(&mut rt);
        assert!(!shown(&rt, "review"), "{mode:?}: nothing opens by itself");
        click(&mut rt, "banner-review");
        settle(&mut rt);
        assert!(shown(&rt, "review"), "{mode:?}: the timeline");
        let r = rt.state.review.as_ref().expect("a review");
        assert_eq!(&r.record, rt.state.history.last().unwrap(), "{mode:?}: of the game just played");
        assert_eq!(r.game.status, Status::Lost, "{mode:?}: on its last move");
        assert_eq!(open_cells(&rt), drawn_open(&rt), "{mode:?}: the board draws the review's field");
        click(&mut rt, "review-close");
        settle(&mut rt);
        assert!(rt.state.review.is_none() && !shown(&rt, "review"), "{mode:?}: left");
    }
}

#[test]
fn after_the_banner_leaves_a_review_tab_stays_on_the_console() {
    let mut rt = runtime(Saved::default());
    lose(&mut rt);
    rt.update(|s, _| s.banner_done = true);
    settle(&mut rt);
    assert!(!shown(&rt, "banner-review"));
    click(&mut rt, "review-tab");
    assert!(rt.state.review.is_some());
    // A new field takes it away.
    press(&mut rt, Key::Function(2));
    settle(&mut rt);
    assert!(rt.state.review.is_none() && !shown(&rt, "review-tab"));
}

#[test]
fn any_history_record_opens_in_review_in_every_mode() {
    let mut history = Vec::new();
    for mode in Mode::ALL {
        let mut rt = runtime(Saved { mode, level: Level::Beginner, ..Saved::default() });
        lose(&mut rt);
        history.push(rt.state.history.last().unwrap().clone());
    }
    let mut rt = runtime(Saved::default());
    rt.update(|s, _| s.history = history.clone());
    for (i, r) in history.iter().enumerate() {
        click(&mut rt, "history-button");
        settle(&mut rt);
        let m = r.mode;
        rt.update(move |s, _| s.history_filter = (m, Level::Beginner));
        settle(&mut rt);
        click(&mut rt, &format!("history-row-{i}"));
        settle(&mut rt);
        assert!(rt.state.panel.is_none(), "{m:?}: History makes way");
        let rv = rt.state.review.as_ref().expect("a review");
        assert_eq!(&rv.record, r, "{m:?}");
        assert_eq!(rv.game.status, Status::Lost, "{m:?}: the field rebuilt from the moves");
        assert_eq!(open_cells(&rt), drawn_open(&rt), "{m:?}");
        press(&mut rt, Key::Escape);
        settle(&mut rt);
        assert!(rt.state.review.is_none(), "{m:?}: Escape leaves");
    }
}

/// The scripted game of the review's own tests: a needless guess (move 1), a chord saving
/// (move 3), a missed deduction (move 5) and a forced true 50/50 (move 6).
fn scripted() -> Record {
    let rows = ["*..........*", "............", "**...*..*.**"];
    let w = rows[0].len();
    let mines = crate::test_fields::drawn(&rows, '*');
    let mv = |ms, act, x: usize, y: usize| Move { ms, act, cell: y * w + x };
    let moves = vec![
        mv(0, Act::Open, 5, 0),
        mv(1500, Act::Open, 0, 1),
        mv(2500, Act::Mark { marks: false }, 5, 2),
        mv(3000, Act::Open, 6, 2),
        mv(3400, Act::Open, 7, 2),
        mv(10_000, Act::Open, 9, 2),
        mv(11_000, Act::Open, 11, 1),
    ];
    Record { at: 1_700_000_000_000, end: End::Won, seconds: 12, three_bv: 6, three_bv_solved: 6, left: 6, right: 1, layout: Layout::Mines(mines.clone()), moves, ..Record::new(Mode::Classic, (w, 3, mines.len())) }
}

/// A runtime with the scripted game as History's only record, opened in review.
fn reviewing(lang: Lang) -> Runtime<App> {
    let mut rt = runtime(Saved { lang: Some(lang), ..Saved::default() });
    rt.update(|s, _| s.history = vec![scripted()]);
    click(&mut rt, "history-button");
    settle(&mut rt);
    rt.update(|s, _| s.history_filter = (Mode::Classic, Level::Custom));
    settle(&mut rt);
    click(&mut rt, "history-row-0");
    settle(&mut rt);
    rt
}

fn at(rt: &Runtime<App>) -> usize {
    rt.state.review.as_ref().unwrap().at
}

#[test]
fn stepping_moves_through_the_game_a_move_at_a_time() {
    let mut rt = reviewing(Lang::En);
    assert_eq!(text_in(&rt, "review-move"), "Move 7 of 7 · 11.0 s");
    click(&mut rt, "review-prev");
    assert_eq!(at(&rt), 6);
    assert_eq!(text_in(&rt, "review-move"), "Move 6 of 7 · 10.0 s");
    assert_eq!(rt.state.review.as_ref().unwrap().game.cells[23].cover, Cover::Hidden, "the last click undone");
    press(&mut rt, Key::Left);
    press(&mut rt, Key::Left);
    assert_eq!(at(&rt), 4);
    press(&mut rt, Key::Right);
    assert_eq!(at(&rt), 5);
    click(&mut rt, "review-next");
    click(&mut rt, "review-next");
    assert_eq!(at(&rt), 7, "the last move is as far as it goes");
    for _ in 0..9 {
        press(&mut rt, Key::Left);
    }
    assert_eq!(at(&rt), 0);
    assert_eq!(text_in(&rt, "review-move"), "Move 0 of 7 · 0.0 s");
    assert!(drawn_open(&rt).is_empty(), "before the first click");
    // The figures.
    assert_eq!(text_in(&rt, "review-stats"), "3BV 6 · 3BV/s 0.55 · Efficiency 0.86");
}

#[test]
fn selecting_a_marker_shows_the_field_at_that_moment_and_the_reason() {
    let mut rt = reviewing(Lang::En);
    let expect = [
        (1, "Needless guess", "You opened a cell nothing proved"),
        (3, "A chord saves clicks", "1 click saved"),
        (5, "Missed deduction", "You paused 6.6 s here"),
        (6, "Forced guess", "a true 50/50"),
    ];
    for (k, (move_at, title, says)) in expect.into_iter().enumerate() {
        click(&mut rt, &format!("review-marker-{k}"));
        settle(&mut rt);
        assert_eq!(at(&rt), move_at, "{title}");
        assert_eq!(open_cells(&rt), drawn_open(&rt), "{title}: the field at that moment");
        let callout = text_in(&rt, "review-callout");
        assert!(callout.starts_with(title), "{callout}");
        assert!(callout.contains(says), "{callout}");
        assert!(shown(&rt, "review-cell"), "{title}: the cell ringed");
        let numbers = rt.state.review.as_ref().unwrap().selected().unwrap().1.numbers.clone();
        assert!(!numbers.is_empty() && numbers.iter().all(|n| shown(&rt, &format!("review-n{n}"))), "{title}: the numbers ringed");
    }
    // The needless guess shows a hint's reason, with the hint's wording.
    click(&mut rt, "review-marker-0");
    assert!(text_in(&rt, "review-callout").contains("so this cell must be safe"), "{}", text_in(&rt, "review-callout"));
    // Closing the callout keeps the moment.
    click(&mut rt, "review-callout-close");
    settle(&mut rt);
    assert!(!shown(&rt, "review-callout") && at(&rt) == 1);
}

#[test]
fn a_review_speaks_zh_cn_and_never_gives_odds() {
    for lang in Lang::ALL {
        let mut rt = reviewing(lang);
        let n = rt.state.review.as_ref().unwrap().markers.len();
        for k in 0..n {
            click(&mut rt, &format!("review-marker-{k}"));
            settle(&mut rt);
            for w in words(&rt) {
                let l = w.to_lowercase();
                for banned in ["%", "probab", "chance", "odds", "likely", "概率", "几率", "可能性"] {
                    assert!(!l.contains(banned), "{w:?}");
                }
                if lang == Lang::ZhCn {
                    let rest = w.replace("3BV/s", "").replace("3BV", "").replace("MiSans", "").replace(Lang::En.endonym(), "");
                    assert!(!rest.chars().any(|c| c.is_ascii_alphabetic()), "{w:?} is still English");
                }
            }
        }
    }
    let mut rt = reviewing(Lang::ZhCn);
    click(&mut rt, "review-marker-0");
    settle(&mut rt);
    let callout = text_in(&rt, "review-callout");
    assert!(callout.starts_with("不必要的猜雷") && callout.contains("所以这一格必定安全"), "{callout}");
    click(&mut rt, "review-marker-3");
    assert!(text_in(&rt, "review-callout").contains("死猜"), "{}", text_in(&rt, "review-callout"));
    assert_eq!(text_in(&rt, "review-stats"), "3BV 6 · 3BV/s 0.55 · 效率 0.86");
}

#[test]
fn reviewing_never_changes_a_record_nor_the_game_in_play() {
    let mut rt = runtime(Saved::default());
    // A game in play, its first click made.
    rt.update(|s, _| {
        let i = (0..s.game.cells.len()).find(|&i| !s.game.cells[i].mine && s.game.cells[i].adj > 0).unwrap();
        s.open_at(i);
        s.history = vec![scripted()];
    });
    let (history, live, game) = (rt.state.history.clone(), rt.state.record.clone(), rt.state.game.cells.clone());
    click(&mut rt, "history-button");
    settle(&mut rt);
    rt.update(|s, _| s.history_filter = (Mode::Classic, Level::Custom));
    settle(&mut rt);
    click(&mut rt, "history-row-0");
    settle(&mut rt);
    let n = rt.state.review.as_ref().unwrap().markers.len();
    for k in 0..n {
        click(&mut rt, &format!("review-marker-{k}"));
    }
    press(&mut rt, Key::Left);
    // Play keys and the Hint button do nothing to the game in play while reviewing.
    press(&mut rt, Key::Text("h".into()));
    press(&mut rt, Key::Text(" ".into()));
    press(&mut rt, Key::Text("f".into()));
    click(&mut rt, "hint-button");
    let (x, y) = rt.find("board").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    click(&mut rt, "review-close");
    settle(&mut rt);
    assert_eq!(rt.state.history, history, "History is as it was");
    assert_eq!(rt.state.record, live, "and so is the record of the game in play");
    assert!(rt.state.game.cells == game && rt.state.hint.is_none(), "and its field");
}

#[test]
fn forced_guess_markers_are_the_forced_guesses_history_counts() {
    use crate::solver::{Fact, Strength, View, deduce};
    // Bot-played Intermediate games: every proof first, flagging mines, else the first
    // hidden cell.
    for seed in 1..6 {
        let mut a = App::with(Saved { level: Level::Intermediate, ..Saved::default() }, seed);
        a.open_at(a.game.idx(8, 8));
        while !a.game.over() {
            let v = View::of(&a.game);
            let proofs = deduce(&v, &vec![Fact::Unknown; v.len()], Strength::Full);
            if let Some(p) = proofs.iter().find(|p| !p.mine) {
                a.open_at(p.cell);
            } else if let Some(p) = proofs.iter().find(|p| a.game.cells[p.cell].cover == Cover::Hidden) {
                a.mark_at(p.cell);
                let open = |i: usize| a.game.cells[i].cover == Cover::Open;
                if let Some(n) = a.game.neighbours(p.cell).find(|&n| open(n) && a.game.cells[n].adj > 0) {
                    a.chord_at(n);
                }
            } else {
                let i = (0..a.game.cells.len()).find(|&i| a.game.cells[i].cover == Cover::Hidden).unwrap();
                a.open_at(i);
            }
        }
        let r = a.history.last().unwrap().clone();
        let review = crate::review::Review::of(&r, r.base().expect("its mines"));
        let forced = review.markers.iter().filter(|m| matches!(m.kind, crate::review::Kind::Forced { .. })).count();
        assert_eq!(forced as u32, r.forced, "seed {seed}");
        assert_eq!(review.stats.three_bv, r.three_bv, "seed {seed}");
        assert_eq!(review.stats.solved, r.three_bv_solved, "seed {seed}");
    }
}
