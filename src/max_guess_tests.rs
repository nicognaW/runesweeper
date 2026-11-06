//! Max Guess through the App: the picker, the count of true 50/50s once the field is laid,
//! and the result banner's survived and forced. Driven headlessly through Rune's `Runtime`.

use rune::Runtime;

use crate::app::App;
use crate::game::{Cover, Level, Mode, Status};
use crate::generate;
use crate::storage::Saved;
use crate::strings::Lang;
use crate::toolbar_tests::{click, runtime, settle, shown, text_in};

fn max_guess(level: Level) -> Saved {
    Saved { mode: Mode::MaxGuess, level, ..Saved::default() }
}

/// The first click in the middle of the field, landed: the field and its true 50/50s, as the
/// generator reports them for the same mode, level, seed and click.
fn laid(rt: &mut Runtime<App>) -> generate::MaxGuess {
    let (first, seed, dims) = { let a = &rt.state; (a.game.idx(a.game.w / 2, a.game.h / 2), a.seed, a.dims()) };
    rt.update(|s, _| {
        s.open_at(first);
        s.finish_laying();
    });
    settle(rt);
    generate::max_guess(dims, seed, first)
}

/// The banner's text, once the game has ended.
fn banner(rt: &Runtime<App>) -> String {
    text_in(rt, &format!("banner{}", rt.state.field))
}

#[test]
fn max_guess_follows_no_guess_in_the_picker_and_is_saved_and_shown() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "level-chip");
    settle(&mut rt);
    let at = |k: &str| {
        let (x, y) = rt.find(k).unwrap_or_else(|| panic!("no {k}"));
        (y.round() as i32, x.round() as i32)
    };
    assert!(at("mode-no-guess") < at("mode-max-guess"), "Max Guess comes last, after No Guess");
    // Every mode's name lies inside its segment, and every segment inside the picker.
    let els = rt.elements();
    let rect = |k: &str| els.iter().find(|e| e.key.as_deref() == Some(k) && !e.exiting).unwrap_or_else(|| panic!("no {k}"));
    let inside = |r: [f32; 4], o: [f32; 4]| r[0] >= o[0] - 0.5 && r[0] + r[2] <= o[0] + o[2] + 0.5;
    let panel = rect("picker").target;
    for m in Mode::ALL {
        let seg = rect(&format!("mode-{}", m.tag()));
        assert!(inside(seg.target, panel), "{m:?} sticks out of the picker: {:?} in {panel:?}", seg.target);
        for e in els.iter().filter(|e| e.path.starts_with(&seg.path) && e.text.is_some()) {
            assert!(inside(e.target, seg.target), "{:?} sticks out of its segment: {:?} in {:?}", e.text, e.target, seg.target);
        }
    }
    click(&mut rt, "mode-max-guess");
    settle(&mut rt);
    assert_eq!((rt.state.mode, rt.state.saved.mode), (Mode::MaxGuess, Mode::MaxGuess));
    assert_eq!(text_in(&rt, "level-chip"), "Max Guess Beginner");
    assert!(rt.state.game.cells.iter().all(|c| !c.mine), "no mines until the first click");
    let rt = runtime(Saved::parse(&rt.state.saved.write()));
    assert_eq!(rt.state.mode, Mode::MaxGuess, "the choice survives a restart");
    let rt = runtime(Saved { lang: Some(Lang::ZhCn), ..max_guess(Level::Intermediate) });
    assert_eq!(text_in(&rt, "level-chip"), "死猜挑战 中级");
}

#[test]
fn the_count_of_true_fifty_fifties_shows_once_the_field_is_laid() {
    let mut rt = runtime(max_guess(Level::Beginner));
    assert!(!shown(&rt, "fifty-count"), "no field, no count");
    let want = laid(&mut rt);
    assert!(rt.state.game.status == Status::Playing && rt.state.game.opened > 1, "the first click opened an area");
    let k = want.fifty_fifties.len();
    assert!(k > 0);
    assert_eq!(text_in(&rt, "fifty-count"), format!("Forced 50/50s: {k}"));
    rt.update(|s, _| s.set_lang(Lang::ZhCn));
    rt.frame(1.0 / 60.0);
    assert_eq!(text_in(&rt, "fifty-count"), format!("死猜：{k} 处"));
    // Other modes have none to show.
    let mut rt = runtime(Saved { mode: Mode::NoGuess, ..Saved::default() });
    laid_any(&mut rt);
    assert!(!shown(&rt, "fifty-count"));
}

fn laid_any(rt: &mut Runtime<App>) {
    let first = rt.state.game.idx(4, 4);
    rt.update(|s, _| {
        s.open_at(first);
        s.finish_laying();
    });
    settle(rt);
}

#[test]
fn a_win_says_every_fifty_fifty_was_survived() {
    let mut rt = runtime(max_guess(Level::Beginner));
    let want = laid(&mut rt);
    let k = want.fifty_fifties.len();
    let safe: Vec<usize> = (0..want.game.cells.len()).filter(|&i| !want.game.cells[i].mine).collect();
    rt.update(|s, _| {
        for &i in &safe {
            s.open_at(i);
        }
    });
    rt.frame(1.0 / 60.0);
    assert_eq!(rt.state.game.status, Status::Won);
    assert!(banner(&rt).contains(&format!("Forced 50/50s survived: {k} of {k}")), "{}", banner(&rt));
}

#[test]
fn a_loss_on_a_fifty_fifty_says_it_was_forced_and_how_many_were_survived() {
    let mut rt = runtime(max_guess(Level::Beginner));
    let want = laid(&mut rt);
    let k = want.fifty_fifties.len();
    assert!(k >= 2, "this field has {k}");
    // Get through the first 50/50 (open its safe cell), then lose on the second's mine.
    let mine = |&(a, _b): &(usize, usize)| if want.game.cells[a].mine { a } else { _b };
    let other = |p: &(usize, usize)| if mine(p) == p.0 { p.1 } else { p.0 };
    let (first, second) = (want.fifty_fifties[0], want.fifty_fifties[1]);
    rt.update(|s, _| {
        s.open_at(other(&first));
        s.open_at(mine(&second));
    });
    rt.frame(1.0 / 60.0);
    assert_eq!(rt.state.game.status, Status::Lost);
    assert_eq!(banner(&rt), format!("Boom. A forced 50/50 · survived 1 of {k}"));
    rt.update(|s, _| s.set_lang(Lang::ZhCn));
    rt.frame(1.0 / 60.0);
    assert_eq!(banner(&rt), format!("失败 死猜，并非失误 · 闯过 1 / {k} 处"));
}

#[test]
fn a_loss_elsewhere_is_not_called_forced() {
    let mut rt = runtime(max_guess(Level::Beginner));
    let want = laid(&mut rt);
    let k = want.fifty_fifties.len();
    let in_pair = |i: usize| want.fifty_fifties.iter().any(|&(a, b)| a == i || b == i);
    let m = (0..want.game.cells.len()).find(|&i| want.game.cells[i].mine && !in_pair(i) && rt.state.game.cells[i].cover == Cover::Hidden).expect("a mine outside every pair");
    rt.update(|s, _| s.open_at(m));
    rt.frame(1.0 / 60.0);
    assert_eq!(rt.state.game.status, Status::Lost);
    assert_eq!(banner(&rt), format!("Boom. Forced 50/50s survived: 0 of {k}"));
}
