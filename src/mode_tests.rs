//! Modes through the App: the picker's mode row, the chip, saving, and a No Guess field laid
//! on the first click. Driven headlessly through Rune's `Runtime`, as the toolbar tests are.

use rune::{Key, Tx};

use crate::app::App;
use crate::game::{Cover, Level, Mode, Status};
use crate::generate;
use crate::storage::Saved;
use crate::strings::Lang;
use crate::toolbar_tests::{click, press, runtime, settle, shown, text_in};

fn no_guess(level: Level) -> Saved {
    Saved { mode: Mode::NoGuess, level, ..Saved::default() }
}

fn mines(a: &App) -> usize {
    a.game.cells.iter().filter(|c| c.mine).count()
}

/// A cell in the middle of the field, where a click would land.
fn middle(a: &App) -> usize {
    a.game.idx(a.game.w / 2, a.game.h / 2)
}

#[test]
fn the_chip_shows_mode_and_level() {
    let rt = runtime(Saved::default());
    assert_eq!(text_in(&rt, "level-chip"), "Classic Beginner");
    let rt = runtime(no_guess(Level::Expert));
    assert_eq!(text_in(&rt, "level-chip"), "No Guess Expert");
    let rt = runtime(Saved { lang: Some(Lang::ZhCn), ..no_guess(Level::Intermediate) });
    assert_eq!(text_in(&rt, "level-chip"), "无猜 中级");
    let rt = runtime(Saved { lang: Some(Lang::ZhCn), ..Saved::default() });
    assert_eq!(text_in(&rt, "level-chip"), "原版 初级");
}

#[test]
fn picking_a_mode_saves_it_and_starts_a_new_field() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "level-chip");
    settle(&mut rt);
    for k in ["mode-classic", "mode-no-guess"] {
        assert!(shown(&rt, k), "the picker's mode row holds {k}");
    }
    let field = rt.state.field;
    click(&mut rt, "mode-no-guess");
    settle(&mut rt);
    assert_eq!(rt.state.mode, Mode::NoGuess);
    assert_eq!(rt.state.saved.mode, Mode::NoGuess, "the choice is saved");
    assert!(rt.state.field > field, "a new field");
    assert_eq!(mines(&rt.state), 0, "no mines until the first click");
    assert_eq!(text_in(&rt, "level-chip"), "No Guess Beginner");
    // The mode applies to every level picked after it.
    click(&mut rt, "pick-expert");
    settle(&mut rt);
    assert_eq!((rt.state.mode, rt.state.level), (Mode::NoGuess, Level::Expert));
    assert_eq!(mines(&rt.state), 0);
    // And back.
    click(&mut rt, "level-chip");
    settle(&mut rt);
    click(&mut rt, "mode-classic");
    settle(&mut rt);
    assert_eq!(rt.state.mode, Mode::Classic);
    assert_eq!(mines(&rt.state), 99, "Classic lays at New Game");
}

#[test]
fn the_mode_survives_a_restart() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "level-chip");
    settle(&mut rt);
    click(&mut rt, "mode-no-guess");
    let file = rt.state.saved.write();
    let rt = runtime(Saved::parse(&file));
    assert_eq!(rt.state.mode, Mode::NoGuess);
    assert_eq!(text_in(&rt, "level-chip"), "No Guess Beginner");
    assert_eq!(Saved::parse("level=expert\n").mode, Mode::Classic, "Classic until another is chosen");
}

#[test]
fn the_first_click_lays_the_field_its_seed_names_and_opens_an_area() {
    for level in [Level::Beginner, Level::Intermediate, Level::Expert, Level::Custom] {
        let saved = Saved { custom: (20, 12, 50), ..no_guess(level) };
        let mut a = App::with(saved, 3);
        let i = middle(&a);
        let seed = a.seed;
        a.open_at(i);
        a.finish_laying();
        let want = generate::lay(Mode::NoGuess, a.dims(), seed, i);
        let got: Vec<bool> = a.game.cells.iter().map(|c| c.mine).collect();
        assert_eq!(got, want.cells.iter().map(|c| c.mine).collect::<Vec<_>>(), "{level:?}");
        assert_eq!(mines(&a), a.dims().2, "{level:?}");
        assert!(a.game.opened > 1, "{level:?}: the first click opens an area");
        assert_ne!(a.game.status, Status::Lost);
    }
}

#[test]
fn every_new_field_draws_a_new_seed() {
    let mut rt = runtime(no_guess(Level::Beginner));
    let seed = rt.state.seed;
    press(&mut rt, Key::Function(2));
    assert_ne!(rt.state.seed, seed, "F2");
    let seed = rt.state.seed;
    click(&mut rt, "face");
    assert_ne!(rt.state.seed, seed, "the face");
}

#[test]
fn the_timer_starts_when_the_field_is_ready() {
    let mut a = App::with(no_guess(Level::Expert), 3);
    let mut tx = Tx::default();
    a.tick(10.0, &mut tx);
    let i = middle(&a);
    a.open_at(i);
    assert_eq!(a.started_at, None, "the clock waits while the field is laid");
    a.finish_laying();
    assert_eq!(a.started_at, None, "the clock waits for a poll");
    // The first poll after the field is ready starts it: the time spent laying never counts.
    a.tick(14.0, &mut tx);
    assert_eq!(a.started_at, Some(14.0));
    assert_eq!(a.seconds, 1);
    a.tick(14.5, &mut tx);
    assert_eq!(a.seconds, 1);
    a.tick(15.2, &mut tx);
    assert_eq!(a.seconds, 2);

    // Classic is unchanged: the clock starts at the click.
    let mut a = App::with(Saved::default(), 3);
    a.tick(10.0, &mut tx);
    let safe = (0..a.game.cells.len()).find(|&i| !a.game.cells[i].mine).unwrap();
    a.open_at(safe);
    assert_eq!(a.started_at, Some(10.0));
}

#[test]
fn flags_before_the_first_click_stay_and_the_counter_is_mines_less_flags() {
    let mut a = App::with(no_guess(Level::Beginner), 3);
    assert_eq!(a.game.counter(), 10);
    let corner = 0;
    a.game.mark(corner, a.saved.marks);
    assert_eq!(a.game.counter(), 9);
    // A press on the flag opens nothing and lays nothing.
    a.open_at(corner);
    assert_eq!((mines(&a), a.game.status), (0, Status::Ready));
    assert_eq!(a.started_at, None);
    let mut tx = Tx::default();
    a.tick(1.0, &mut tx);
    assert_eq!(a.started_at, None, "the clock waits for the field");
    a.open_at(middle(&a));
    a.finish_laying();
    assert_eq!(mines(&a), 10);
    assert_eq!(a.game.cells[corner].cover, Cover::Flag, "the flag stays");
    assert_eq!(a.game.counter(), 9);
}

#[test]
fn gentle_no_guess_sits_between_classic_and_no_guess_and_is_saved_and_shown() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "level-chip");
    settle(&mut rt);
    let x = |rt: &rune::Runtime<App>, k: &str| rt.find(k).unwrap_or_else(|| panic!("no {k}")).0;
    let (classic, gentle, full) = (x(&rt, "mode-classic"), x(&rt, "mode-gentle-no-guess"), x(&rt, "mode-no-guess"));
    assert!(classic < gentle && gentle < full, "Classic, Gentle No Guess, No Guess: {classic} {gentle} {full}");
    assert_eq!(text_in(&rt, "modes"), "Classic Gentle No Guess No Guess");
    click(&mut rt, "mode-gentle-no-guess");
    settle(&mut rt);
    assert_eq!(rt.state.mode, Mode::GentleNoGuess);
    assert_eq!(mines(&rt.state), 0, "no mines until the first click");
    assert_eq!(text_in(&rt, "level-chip"), "Gentle No Guess Beginner");
    let rt = runtime(Saved::parse(&rt.state.saved.write()));
    assert_eq!(rt.state.mode, Mode::GentleNoGuess, "the choice survives a restart");
    let rt = runtime(Saved { lang: Some(Lang::ZhCn), mode: Mode::GentleNoGuess, ..Saved::default() });
    assert_eq!(text_in(&rt, "level-chip"), "简单无猜 初级");
}

#[test]
fn a_gentle_first_click_lays_the_field_its_seed_names_and_opens_an_area() {
    let mut a = App::with(Saved { mode: Mode::GentleNoGuess, ..Saved::default() }, 3);
    let (i, seed) = (middle(&a), a.seed);
    a.open_at(i);
    a.finish_laying();
    let want = generate::lay(Mode::GentleNoGuess, a.dims(), seed, i);
    assert_eq!(a.game.cells.iter().map(|c| c.mine).collect::<Vec<_>>(), want.cells.iter().map(|c| c.mine).collect::<Vec<_>>());
    assert!(a.game.opened > 1, "the first click opens an area");
}
