use crate::{
    app::{App, Dialog},
    game::{Cover, Status},
    storage::Saved,
    style::*,
    ui,
};
use rune::{Env, Key, Runtime};

fn runtime() -> Runtime<App> {
    let mut rt = Runtime::new(
        App::with(Saved::default(), 7),
        ui::view,
        window_size(9, 9),
        Env::new(true, 1.0),
    );
    rt.poll = Some(|s, t, tx| s.tick(t, tx));
    rt.on_key = Some(|s, e, tx| s.key(e, tx));
    rt.frame(0.0);
    for _ in 0..45 {
        rt.frame(1.0 / 60.0);
    }
    rt
}

fn point(rt: &Runtime<App>, i: usize) -> (f32, f32) {
    let r = rt.presented("board").unwrap().0;
    let [x, y] = rt.state.cell_center(i);
    (r[0] + x, r[1] + y)
}

fn click(rt: &mut Runtime<App>, i: usize) {
    let (x, y) = point(rt, i);
    rt.pointer_down(x, y);
    rt.frame(0.0);
    rt.pointer_up(x, y);
    rt.frame(0.0);
}

#[test]
fn pointer_lighting_retargets_without_rebuilding_the_view() {
    let mut rt = runtime();
    let (x, y) = point(&rt, 40);
    rt.pointer_move(x, y);
    rt.frame(1.0 / 60.0);
    let commits = rt.stats.commits;
    let before = rt.presented("pointer-light").unwrap().0[0];
    rt.pointer_move(x + 8.0, y);
    rt.frame(1.0 / 60.0);
    let after = rt.presented("pointer-light").unwrap().0[0];
    assert!(
        after > before && after < before + 8.0,
        "the lamp springs toward its target"
    );
    assert_eq!(rt.stats.commits, commits, "pointer motion is Live-only");
    rt.pointer_move(x - PITCH * 3.0, y);
    rt.frame(1.0 / 60.0);
    let lamp = rt.presented("pointer-light").unwrap().0;
    assert!((lamp[0] - (x - PITCH * 3.0 - 60.0)).abs() < 0.1, "large teleports snap");
}

#[test]
fn flood_numbers_are_visible_and_stationary_before_the_wave_reaches_them() {
    let mut rt = runtime();
    let blank = rt.state.game.cells.iter().position(|c| !c.mine && c.adj == 0).unwrap();
    click(&mut rt, blank);
    let i = rt
        .state
        .game
        .cells
        .iter()
        .position(|c| c.cover == Cover::Open && c.adj > 0 && c.wave > 0)
        .unwrap();
    let key = format!("n{}-{i}", rt.state.field);
    let before = rt.presented(&key).unwrap();
    assert_eq!(before.1, 1.0, "information is immediately opaque");
    let tile_key = format!("t{}-{i}", rt.state.field);
    let tile_before = rt.presented(&tile_key).unwrap();
    assert_eq!(tile_before.1, 1.0, "the recessed cell is opaque immediately");
    assert!(rt.presented(&format!("reveal-{}-{i}", rt.state.field)).is_some());
    rt.frame(0.18);
    assert_eq!(rt.presented(&key).unwrap(), before);
    assert_eq!(
        rt.presented(&tile_key).unwrap(),
        tile_before,
        "no swelling cover remains over the number"
    );
}

#[test]
fn a_second_reveal_does_not_cut_off_the_first_light_wave() {
    let mut rt = runtime();
    click(&mut rt, 0);
    rt.frame(0.05);
    let key = "t0-0/reveal-0-0";
    let first = rt
        .elements()
        .iter()
        .find(|e| e.key.as_deref() == Some("reveal-0-0"))
        .unwrap()
        .opacity;
    let next = rt
        .state
        .game
        .cells
        .iter()
        .position(|c| !c.mine && c.cover == Cover::Hidden)
        .unwrap();
    click(&mut rt, next);
    let light = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("reveal-0-0"));
    assert!(light.is_some(), "{key} keeps finishing after another reveal");
    assert!(first > 0.0);
}

#[test]
fn keyboard_reveal_is_instant_and_has_no_decorative_wave() {
    let mut rt = runtime();
    let blank = rt.state.game.cells.iter().position(|c| !c.mine && c.adj == 0).unwrap();
    rt.state.hover_cell = Some(blank);
    rt.key(Key::Text(" ".into()).into());
    rt.frame(0.0);
    assert!(rt.state.game.opened > 0);
    assert!(!rt.state.spatial);
    assert!(
        !rt.elements()
            .iter()
            .any(|e| e.key.as_deref().is_some_and(|k| k.starts_with("reveal-")))
    );
    let hidden = rt
        .state
        .game
        .cells
        .iter()
        .position(|c| c.cover == Cover::Hidden)
        .unwrap();
    let (x, y) = point(&rt, hidden);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(0.0);
    assert!(
        !rt.elements()
            .iter()
            .any(|e| e.key.as_deref().is_some_and(|k| k.starts_with("reveal-"))),
        "pointer flagging must not replay the previous keyboard reveal"
    );
}

fn win(rt: &mut Runtime<App>, keyboard: bool) {
    // Prepare a field one numbered cell from a win, then finish through real input.
    rt.update(|s, tx| {
        s.reduced = true;
        let last = s.game.cells.iter().position(|c| !c.mine && c.adj > 0).unwrap();
        for i in 0..s.game.cells.len() {
            if !s.game.cells[i].mine {
                let _ = s.game.open(i);
            }
        }
        s.game.cells[last].cover = Cover::Hidden;
        s.game.opened -= 1;
        s.game.status = Status::Playing;
        s.started_at = Some(s.now - 20.0);
        s.seconds = 21;
        s.hover_cell = Some(last);
        tx.instant();
    });
    rt.frame(0.0);
    if keyboard {
        rt.key(Key::Text(" ".into()).into());
        rt.frame(0.0);
    } else {
        let last = rt.state.hover_cell.unwrap();
        click(rt, last);
    }
    assert_eq!(rt.state.game.status, Status::Won);
}

#[test]
fn a_reduced_keyboard_win_still_wakes_for_the_delayed_record() {
    let mut rt = runtime();
    win(&mut rt, true);
    assert!(rt.state.dialog.is_none());
    let (at, _) = rt.state.record_at.unwrap();
    assert!(rt.wake_deadline().is_some_and(|t| t <= at + 0.01));
    rt.frame(0.4);
    assert!(rt.state.dialog.is_none());
    rt.frame(0.5);
    assert_eq!(rt.state.dialog, Some(Dialog::Record(0)));
}

#[test]
fn resetting_during_the_victory_cancels_the_pending_record() {
    let mut rt = runtime();
    win(&mut rt, false);
    assert!(rt.state.dialog.is_none(), "still celebrating: {:?}", rt.state.dialog);
    assert!(rt.state.record_at.is_some());
    rt.key(Key::Function(2).into());
    rt.frame(0.0);
    assert_eq!(rt.state.game.status, Status::Ready);
    rt.frame(1.0);
    assert!(rt.state.record_at.is_none());
    assert!(rt.state.dialog.is_none());
    assert_eq!(rt.state.game.status, Status::Ready);
}

#[test]
fn the_record_waits_for_an_open_dialog_without_polling_in_a_loop() {
    let mut rt = runtime();
    win(&mut rt, true);
    rt.update(|s, _| s.dialog = Some(Dialog::About));
    rt.frame(1.0);
    assert_eq!(rt.state.dialog, Some(Dialog::About));
    assert!(rt.state.record_at.is_some());
    let won = rt.state.moments.won.unwrap();
    assert!(rt.wake_deadline().is_none_or(|t| t >= won + App::BANNER_FOR));
    rt.update(|s, _| s.dialog = None);
    rt.frame(0.0);
    assert_eq!(rt.state.dialog, Some(Dialog::Record(0)));
}

#[test]
fn results_do_not_move_or_cover_the_field() {
    let mut rt = runtime();
    let before = rt.presented("board").unwrap().0;
    win(&mut rt, false);
    let after = rt.presented("board").unwrap().0;
    assert_eq!(before, after);
    let status = rt.presented("result").unwrap().0;
    assert!(status[1] >= after[1] + after[3]);
}

#[test]
fn reduced_motion_omits_the_lamp_and_spatial_shaders() {
    for name in ["reduced-won", "reduced-lost"] {
        let mut rt = runtime();
        rt.update(|s, tx| {
            crate::SCENES.iter().find(|s| s.0 == name).unwrap().1(s);
            tx.instant();
        });
        rt.frame(0.0);
        for key in ["pointer-light", "shock", "burst", "aura", "heat0", "face-burst0"] {
            assert!(rt.presented(key).is_none(), "{name} should omit {key}");
        }
    }
}

#[test]
fn the_clock_stops_scheduling_at_999() {
    let mut rt = runtime();
    rt.update(|s, tx| {
        s.game.status = Status::Playing;
        s.started_at = Some(0.0);
        s.tick(999.0, tx);
    });
    rt.frame(999.0);
    assert_eq!(rt.state.seconds, 999);
    assert!(rt.wake_deadline().is_none());
}

#[test]
fn first_run_has_no_tutorial_or_extra_window_height() {
    let rt = runtime();
    let texts: Vec<_> = rt
        .elements()
        .iter()
        .filter_map(|e| e.text.as_deref())
        .map(str::to_string)
        .collect();
    assert!(
        !texts
            .iter()
            .any(|s| s.contains("Left click") || s.contains("How to") || s.contains("Right click"))
    );
    assert!(rt.presented("hint").is_none());
}
