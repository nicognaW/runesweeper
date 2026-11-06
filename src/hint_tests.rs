//! The Hint through the App, driven headlessly through Rune's `Runtime` as the toolbar tests
//! are: the H key and the Hint button step through the four stages, any move starts afresh,
//! Apply plays the move, a wrong flag comes first, the callout never covers the field, and the
//! record counts the hints.

use rune::{Env, Key, Runtime};

use crate::app::App;
use crate::game::{Cover, Game, Level, Status};
use crate::hint::Verdict;
use crate::storage::Saved;
use crate::strings::Lang;
use crate::toolbar_tests::{click, press, runtime, settle, shown, text_in, words};
use crate::ui;

/// A field drawn as rows: `.` open, `#` hidden safe, `*` hidden mine, `F` flagged mine, `x`
/// flagged safe.
fn field(rows: &[&str]) -> Game {
    let w = rows[0].len();
    let at = |c: char| rows.iter().enumerate().flat_map(move |(y, r)| r.char_indices().filter(move |&(_, ch)| ch == c).map(move |(x, _)| y * w + x)).collect::<Vec<_>>();
    let mut g = Game::with_mines(w, rows.len(), &[at('*'), at('F')].concat());
    for i in at('.') {
        g.cells[i].cover = Cover::Open;
        g.opened += 1;
    }
    for i in [at('F'), at('x')].concat() {
        g.cells[i].cover = Cover::Flag;
        g.flags += 1;
    }
    g.started = true;
    g.status = Status::Playing;
    g
}

/// 1-1 from the left wall, on a field wide enough for the toolbar.
const ONE_ONE: [&str; 2] = ["*##*#....", "........."];

fn on(rt: &mut Runtime<App>, rows: &[&str]) {
    let g = field(rows);
    rt.update(move |s, _| s.game = g.clone());
    rt.frame(1.0 / 60.0);
}

fn stage(rt: &Runtime<App>) -> u8 {
    rt.state.hint.as_ref().map_or(0, |h| h.stage)
}

fn hint_key(rt: &mut Runtime<App>) {
    press(rt, Key::Text("h".into()));
}

#[test]
fn the_h_key_and_the_button_step_through_four_stages() {
    let mut rt = runtime(Saved::default());
    on(&mut rt, &ONE_ONE);
    assert!(!shown(&rt, "hint-callout"));
    hint_key(&mut rt);
    assert_eq!(stage(&rt), 1);
    assert!(text_in(&rt, "hint-callout").contains("1-1 pattern"), "the nudge names the technique: {}", text_in(&rt, "hint-callout"));
    assert!(text_in(&rt, "hint-callout").contains("Medium"), "and its grade");
    assert!(!shown(&rt, "hint-n9") && !shown(&rt, "hint-cell"), "nothing on the field yet");

    click(&mut rt, "hint-button");
    assert_eq!(stage(&rt), 2);
    assert!(shown(&rt, "hint-n9") && shown(&rt, "hint-n10"), "the two numbers are ringed");
    assert!(!shown(&rt, "hint-cell"));

    hint_key(&mut rt);
    assert_eq!(stage(&rt), 3);
    assert!(shown(&rt, "hint-cell"));
    assert_eq!(text_in(&rt, "hint-verdict"), "This cell is safe.");
    assert!(!shown(&rt, "hint-apply"), "Apply comes with the reason");

    hint_key(&mut rt);
    assert_eq!(stage(&rt), 4);
    assert!(text_in(&rt, "hint-reason").contains("so this cell must be safe"));
    assert!(shown(&rt, "hint-apply"));
    hint_key(&mut rt);
    assert_eq!(stage(&rt), 4, "the last stage stays");
    assert_eq!(rt.state.record.hints, 1, "one hint, however deep");
    assert_eq!(rt.state.game.cells[2].cover, Cover::Hidden, "a hint never plays the move unasked");
}

#[test]
fn any_move_starts_the_next_hint_afresh() {
    let mut rt = runtime(Saved::default());
    on(&mut rt, &ONE_ONE);
    for _ in 0..3 {
        hint_key(&mut rt);
    }
    assert_eq!(stage(&rt), 3);
    // A flag elsewhere is a move.
    rt.update(|s, _| s.mark_at(8));
    rt.frame(1.0 / 60.0);
    assert!(!shown(&rt, "hint-callout") && !shown(&rt, "hint-cell"), "the move cleared it");
    hint_key(&mut rt);
    assert_eq!(stage(&rt), 1, "back to the nudge");
    assert_eq!(rt.state.record.hints, 2);
    // Escape puts it away without a move.
    press(&mut rt, Key::Escape);
    assert!(!shown(&rt, "hint-callout"));
}

#[test]
fn apply_plays_the_move_only_when_chosen() {
    let mut rt = runtime(Saved::default());
    on(&mut rt, &ONE_ONE);
    for _ in 0..4 {
        hint_key(&mut rt);
    }
    let cell = rt.state.hint.as_ref().unwrap().hint.target.unwrap().cell;
    assert_eq!(cell, 2);
    click(&mut rt, "hint-apply");
    assert_eq!(rt.state.game.cells[cell].cover, Cover::Open, "the safe cell opened");
    assert!(rt.state.hint.is_none(), "and the hint went with the move");
    assert_eq!(rt.state.record.moves.last().map(|m| m.cell), Some(cell), "recorded as a click");

    // A mine is flagged.
    let mut rt = runtime(Saved::default());
    on(&mut rt, &["*#*......", "........."]);
    for _ in 0..4 {
        hint_key(&mut rt);
    }
    let t = rt.state.hint.as_ref().unwrap().hint.target.unwrap();
    assert_eq!(t.verdict, Verdict::Mine);
    click(&mut rt, "hint-apply");
    assert_eq!(rt.state.game.cells[t.cell].cover, Cover::Flag);
}

#[test]
fn a_wrong_flag_is_named_first_and_apply_takes_it_off() {
    let mut rt = runtime(Saved::default());
    // The 1-1 proves the flagged cell safe; the Single on the right would be named otherwise.
    on(&mut rt, &["*#x*#...*", "........."]);
    hint_key(&mut rt);
    assert!(text_in(&rt, "hint-callout").contains("One of your flags can't be right."), "{}", text_in(&rt, "hint-callout"));
    for _ in 0..2 {
        hint_key(&mut rt);
    }
    assert_eq!(text_in(&rt, "hint-verdict"), "This flag is wrong: the cell is safe.");
    hint_key(&mut rt);
    assert!(text_in(&rt, "hint-reason").ends_with("The flag on it is wrong."), "{}", text_in(&rt, "hint-reason"));
    click(&mut rt, "hint-apply");
    assert_eq!(rt.state.game.cells[2].cover, Cover::Hidden, "the flag is off");
}

#[test]
fn after_flagging_the_mine_a_hint_named_the_next_names_another_cell() {
    let mut rt = runtime(Saved::default());
    on(&mut rt, &["*#*......", "........."]);
    for _ in 0..3 {
        hint_key(&mut rt);
    }
    let first = rt.state.hint.as_ref().unwrap().hint.target.unwrap();
    assert_eq!(first.verdict, Verdict::Mine);
    rt.update(move |s, _| s.mark_at(first.cell));
    rt.frame(1.0 / 60.0);
    for _ in 0..3 {
        hint_key(&mut rt);
    }
    let next = rt.state.hint.as_ref().unwrap().hint.target.unwrap();
    assert_ne!(next.cell, first.cell);
}

#[test]
fn hint_text_never_gives_odds_in_either_language() {
    for lang in Lang::ALL {
        let mut rt = runtime(Saved { lang: Some(lang), ..Saved::default() });
        for rows in [&ONE_ONE[..], &["*#x*#...*", "........."], &["*#.......", "........."], &["##*......", "*........", "*........"]] {
            on(&mut rt, rows);
            for _ in 0..4 {
                hint_key(&mut rt);
                for w in words(&rt) {
                    let l = w.to_lowercase();
                    for banned in ["%", "probab", "chance", "odds", "likely", "概率", "几率", "可能性"] {
                        assert!(!l.contains(banned), "{w:?}");
                    }
                    if lang == Lang::ZhCn {
                        let rest = w.replace("F2", "").replace("MiSans", "").replace(Lang::En.endonym(), "");
                        assert!(!rest.chars().any(|c| c.is_ascii_alphabetic()), "{w:?} is still English");
                    }
                }
            }
            press(&mut rt, Key::Escape);
        }
    }
}

/// Whether two `[x, y, w, h]` rects share any area.
fn overlap(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}

#[test]
fn the_callout_never_covers_the_cells_it_talks_about() {
    let levels = [(Level::Beginner, (9, 9)), (Level::Expert, (30, 16))];
    for (level, (w, h)) in levels {
        let (ww, wh) = crate::style::window_size(w, h);
        for window in [(ww, wh), (300.0, 300.0), (ww * 2.0, wh), (ww, wh * 2.0), (420.0, 900.0)] {
            // A mine in each corner, alone under its numbers: the Single there is the hint.
            for corner in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
                let mut rt = Runtime::new(App::with(Saved { level, ..Saved::default() }, 7), ui::view, window, Env { system_dark: true, scale: 1.0 });
                rt.on_key = Some(|s, e, tx| s.key(e, tx));
                rt.frame(0.0);
                let rows: Vec<String> = (0..h).map(|y| (0..w).map(|x| if (x, y) == corner { '*' } else { '.' }).collect()).collect();
                let rows: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
                on(&mut rt, &rows);
                for _ in 0..4 {
                    hint_key(&mut rt);
                }
                for _ in 0..40 {
                    rt.frame(1.0 / 60.0);
                }
                let (callout, _) = rt.presented("hint-callout").expect("the callout");
                let cell = rt.presented("hint-cell").expect("the cell").0;
                assert!(!overlap(callout, cell), "{level:?} {window:?} {corner:?}: callout {callout:?} over the cell {cell:?}");
                assert!(callout[0] >= 0.0 && callout[0] + callout[2] <= window.0 + 0.5, "{level:?} {window:?} {corner:?}: callout {callout:?} off the window");
                let n = rt.state.hint.as_ref().unwrap().hint.numbers.clone();
                for i in n {
                    assert!(!overlap(callout, rt.presented(&format!("hint-n{i}")).unwrap().0), "{level:?} {window:?} {corner:?}");
                }
            }
        }
    }
}

#[test]
fn records_count_hints_and_mark_the_game_assisted() {
    let mut rt = runtime(Saved::default());
    on(&mut rt, &ONE_ONE);
    hint_key(&mut rt);
    hint_key(&mut rt);
    press(&mut rt, Key::Escape);
    hint_key(&mut rt);
    // Leaving the game records it.
    press(&mut rt, Key::Function(2));
    let r = rt.state.history.last().expect("a record").clone();
    assert_eq!(r.hints, 2);
    assert!(r.assisted());
    assert!(crate::record::Record::parse(&r.write()).is_some_and(|back| back.hints == 2), "kept in the line");
    // History says so.
    click(&mut rt, "history-button");
    rt.update(|s, _| s.history_filter = (crate::game::Mode::Classic, Level::Custom));
    settle(&mut rt);
    assert!(text_in(&rt, "history-list").contains("Assisted, 2 hints"), "{}", text_in(&rt, "history-list"));
    // A game without hints is not assisted.
    assert!(!crate::record::Record::new(crate::game::Mode::Classic, (9, 9, 10)).assisted());
}
