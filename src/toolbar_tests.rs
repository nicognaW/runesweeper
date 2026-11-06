//! The toolbar and its panels, driven headlessly through Rune's `Runtime`: pointer and key
//! input in, the element tree and app state out.

use rune::{Env, Key, KeyEvent, Mods, Runtime};

use crate::app::App;
use crate::game::Level;
use crate::storage::Saved;
use crate::ui;

fn runtime(saved: Saved) -> Runtime<App> {
    let (w, h) = crate::style::window_size(30, 24);
    let mut rt = Runtime::new(App::with(saved, 7), ui::view, (w, h), Env { system_dark: true, scale: 1.0 });
    rt.on_key = Some(|s, e, tx| s.key(e, tx));
    rt.frame(0.0);
    rt
}

/// Lets springs run out, so exits finish and entrances land.
fn settle(rt: &mut Runtime<App>) {
    for _ in 0..90 {
        rt.frame(1.0 / 60.0);
    }
}

fn click(rt: &mut Runtime<App>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no element keyed {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}

fn press(rt: &mut Runtime<App>, key: Key) {
    let cmd = matches!(key, Key::Command(_));
    rt.key(KeyEvent { key, mods: Mods { cmd, ..Mods::default() } });
    rt.frame(1.0 / 60.0);
}

fn type_text(rt: &mut Runtime<App>, s: &str) {
    rt.key(KeyEvent { key: Key::Text(s.into()), mods: Mods::default() });
    rt.frame(1.0 / 60.0);
}

fn shown(rt: &Runtime<App>, key: &str) -> bool {
    rt.find(key).is_some()
}

/// Every string the screen shows or names, text and accessible labels alike.
fn words(rt: &Runtime<App>) -> Vec<String> {
    rt.elements().into_iter().filter(|e| !e.exiting).flat_map(|e| [e.text, e.label, e.help].into_iter().flatten()).collect()
}

/// The text inside the element keyed `key`.
fn text_in(rt: &Runtime<App>, key: &str) -> String {
    let els = rt.elements();
    let Some(root) = els.iter().find(|e| e.key.as_deref() == Some(key) && !e.exiting) else { return String::new() };
    els.iter().filter(|e| e.path.starts_with(&root.path) && !e.exiting).filter_map(|e| e.text.clone()).collect::<Vec<_>>().join(" ")
}

#[test]
fn no_xp_menu_bar_dialogs_or_help_text_are_drawn() {
    for touch in [false, true] {
        let mut rt = runtime(Saved::default());
        rt.update(|s, _| s.touch = touch);
        rt.frame(1.0 / 60.0);
        assert_eq!(shown(&rt, "flag-toggle"), touch, "the flag toggle shows on touch builds only");
        assert!(!shown(&rt, "menubar"), "no menu bar");
        assert!(shown(&rt, "toolbar"), "a toolbar instead");
        for w in words(&rt) {
            let l = w.to_lowercase();
            // The XP menus' titles.
            assert!(!["game", "help"].contains(&l.as_str()), "{w:?} on screen (touch: {touch})");
            for banned in ["how to play", "about", "best times", "fastest", "click", "tap", "your name", "taps place flags"] {
                assert!(!l.contains(banned), "{w:?} on screen (touch: {touch})");
            }
        }
    }
}

#[test]
fn the_level_chip_names_the_level_and_opens_the_picker() {
    let mut rt = runtime(Saved::default());
    assert_eq!(text_in(&rt, "level-chip"), "Beginner");
    assert!(!shown(&rt, "picker"));
    click(&mut rt, "level-chip");
    settle(&mut rt);
    assert!(shown(&rt, "picker"), "the chip opens the picker");
    for k in ["pick-beginner", "pick-intermediate", "pick-expert", "pick-custom", "custom-w", "custom-h", "custom-m"] {
        assert!(shown(&rt, k), "the picker holds {k}");
    }
}

#[test]
fn picking_a_level_starts_a_new_field_at_once() {
    let mut rt = runtime(Saved::default());
    let field = rt.state.field;
    click(&mut rt, "level-chip");
    settle(&mut rt);
    click(&mut rt, "pick-expert");
    settle(&mut rt);
    assert_eq!(rt.state.level, Level::Expert);
    assert_eq!((rt.state.game.w, rt.state.game.h, rt.state.game.mines), (30, 16, 99));
    assert!(rt.state.field > field, "a new field");
    assert!(!shown(&rt, "picker"), "picking closes the picker");
    assert_eq!(text_in(&rt, "level-chip"), "Expert");
}

#[test]
fn custom_sizes_are_typed_inline_and_clamped_as_xp_does() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "level-chip");
    settle(&mut rt);
    for (k, v) in [("custom-w", "40"), ("custom-h", "5"), ("custom-m", "999")] {
        click(&mut rt, k);
        press(&mut rt, Key::Command('a'));
        type_text(&mut rt, v);
    }
    press(&mut rt, Key::Enter);
    settle(&mut rt);
    assert_eq!(rt.state.level, Level::Custom);
    // Width 9–30, height 9–24, mines up to (w−1)(h−1).
    assert_eq!((rt.state.game.w, rt.state.game.h, rt.state.game.mines), (30, 9, 29 * 8));
    assert_eq!(rt.state.saved.custom, (30, 9, 232));
    assert!(!shown(&rt, "picker"));
    assert!(text_in(&rt, "level-chip").starts_with("Custom"));
}

#[test]
fn the_custom_row_starts_the_custom_field_it_shows() {
    let saved = Saved { custom: (20, 12, 50), ..Saved::default() };
    let mut rt = runtime(saved);
    click(&mut rt, "level-chip");
    settle(&mut rt);
    assert_eq!(text_in(&rt, "custom-w"), "20");
    click(&mut rt, "pick-custom");
    settle(&mut rt);
    assert_eq!((rt.state.game.w, rt.state.game.h, rt.state.game.mines), (20, 12, 50));
}

#[test]
fn settings_toggle_marks_and_color() {
    let mut rt = runtime(Saved::default());
    assert!(rt.state.saved.marks && rt.state.saved.color);
    click(&mut rt, "settings-button");
    settle(&mut rt);
    assert!(shown(&rt, "settings"), "the button opens Settings");
    click(&mut rt, "marks-switch");
    assert!(!rt.state.saved.marks);
    click(&mut rt, "color-switch");
    assert!(!rt.state.saved.color);
    click(&mut rt, "marks-switch");
    assert!(rt.state.saved.marks);
    assert!(shown(&rt, "settings"), "Settings stays open while toggling");
}

#[test]
fn escape_closes_a_panel_in_one_frame() {
    let mut rt = runtime(Saved::default());
    for (button, panel) in [("settings-button", "settings"), ("level-chip", "picker")] {
        click(&mut rt, button);
        settle(&mut rt);
        assert!(shown(&rt, panel));
        press(&mut rt, Key::Escape);
        assert_eq!(rt.state.panel, None);
        let left = rt.presented(panel).map_or(0.0, |(_, opacity)| opacity);
        assert_eq!(left, 0.0, "{panel} is gone the frame Escape is pressed, with no exit to watch");
    }
}

#[test]
fn a_press_outside_or_on_its_button_again_closes_a_panel() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "settings-button");
    settle(&mut rt);
    click(&mut rt, "settings-button");
    settle(&mut rt);
    assert!(!shown(&rt, "settings"), "its button closes it");
    click(&mut rt, "level-chip");
    settle(&mut rt);
    click(&mut rt, "board");
    settle(&mut rt);
    assert!(!shown(&rt, "picker"), "a press outside closes it");
}

#[test]
fn f2_and_the_face_still_start_a_new_field() {
    let mut rt = runtime(Saved::default());
    let field = rt.state.field;
    press(&mut rt, Key::Function(2));
    assert_eq!(rt.state.field, field + 1, "F2");
    click(&mut rt, "face");
    assert_eq!(rt.state.field, field + 2, "the face");
}

#[test]
fn the_flag_toggle_shows_its_state_by_look_alone() {
    let mut rt = runtime(Saved::default());
    rt.update(|s, _| s.touch = true);
    settle(&mut rt);
    assert!(text_in(&rt, "flag-toggle").is_empty(), "no words on the toggle");
    let fill = |rt: &Runtime<App>| rt.elements().into_iter().find(|e| e.key.as_deref() == Some("flag-toggle")).unwrap().fill;
    let off = fill(&rt);
    click(&mut rt, "flag-toggle");
    settle(&mut rt);
    assert!(rt.state.flag_mode);
    let on = fill(&rt);
    let d = (on.r - off.r).abs() + (on.g - off.g).abs() + (on.b - off.b).abs() + (on.a - off.a).abs();
    assert!(d > 0.3, "on and off look clearly different: {off:?} vs {on:?}");
    click(&mut rt, "flag-toggle");
    assert!(!rt.state.flag_mode);
}
