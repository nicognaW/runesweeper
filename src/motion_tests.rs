//! The field's motion, driven headlessly through Rune's `Runtime` as the toolbar tests are: the
//! pointer lamp, numbers that read at once with light under them, waves that outlive the next
//! click, keys that never animate, and what Reduce Motion leaves out.

use rune::{Key, Runtime};

use crate::app::App;
use crate::board::field_key;
use crate::game::{Cover, Status};
use crate::storage::Saved;
use crate::style::{LAMP, PITCH};
use crate::toolbar_tests::{press, runtime, settle, shown, type_text};

/// Cell `i`'s centre in window points.
fn point(rt: &Runtime<App>, i: usize) -> (f32, f32) {
    let r = rt.presented("board").unwrap().0;
    let [x, y] = rt.state.cell_center(i);
    (r[0] + x, r[1] + y)
}

fn click_cell(rt: &mut Runtime<App>, i: usize) {
    let (x, y) = point(rt, i);
    rt.pointer_down(x, y);
    rt.frame(0.0);
    rt.pointer_up(x, y);
    rt.frame(0.0);
}

/// A Classic field one numbered cell from a win, with the selector on that cell.
fn one_from_a_win(rt: &mut Runtime<App>) -> usize {
    let last = rt.state.game.cells.iter().position(|c| !c.mine && c.adj > 0).unwrap();
    rt.update(|s, _| {
        s.game.start();
        let safe: Vec<usize> = (0..s.game.cells.len()).filter(|&i| i != last && !s.game.cells[i].mine).collect();
        for i in safe {
            s.game.cells[i].cover = Cover::Open;
            s.game.opened += 1;
        }
        s.hover_cell = Some(last);
    });
    settle(rt);
    last
}

#[test]
fn the_lamp_follows_the_pointer_without_rerunning_the_view() {
    let mut rt = runtime(Saved::default());
    settle(&mut rt);
    let (x, y) = point(&rt, 40);
    rt.pointer_move(x, y);
    rt.frame(1.0 / 60.0);
    let commits = rt.stats.commits;
    let before = rt.presented("lamp").unwrap().0[0];
    rt.pointer_move(x + 8.0, y);
    rt.frame(1.0 / 60.0);
    let after = rt.presented("lamp").unwrap().0[0];
    assert!(after > before && after < before + 8.0, "it springs toward the pointer: {before} → {after}");
    assert_eq!(rt.stats.commits, commits, "pointer motion is live-only");
    rt.pointer_move(x - PITCH * 3.0, y);
    rt.frame(1.0 / 60.0);
    let at = rt.presented("lamp").unwrap().0[0];
    assert!((at - (x - PITCH * 3.0 - LAMP / 2.0)).abs() < 0.1, "a jump snaps");
}

#[test]
fn opened_numbers_read_at_once_with_light_under_them_and_never_move() {
    let mut rt = runtime(Saved::default());
    settle(&mut rt);
    let blank = rt.state.game.cells.iter().position(|c| !c.mine && c.adj == 0).unwrap();
    click_cell(&mut rt, blank);
    let f = field_key(&rt.state);
    let i = rt.state.game.cells.iter().position(|c| c.cover == Cover::Open && c.adj > 0 && c.wave > 0).expect("a number the flood reached");
    let number = rt.presented(&format!("n{f}-{i}")).unwrap();
    assert_eq!(number.1, 1.0, "the number is opaque at once");
    let els = rt.elements();
    let root = els.iter().find(|e| e.key.as_deref() == Some(&format!("n{f}-{i}"))).unwrap();
    let parts: Vec<_> = els.iter().filter(|e| e.path.starts_with(&root.path)).collect();
    assert!(parts.iter().any(|e| e.text.is_some()), "its digit is there");
    assert!(parts.iter().all(|e| e.opacity == 1.0), "its digit and halo too: {:?}", parts.iter().map(|e| (&e.path, e.opacity)).collect::<Vec<_>>());
    assert!(rt.presented(&format!("t{f}-{i}")).is_none(), "its cover went in the same frame");
    assert!(shown(&rt, &format!("reveal{f}-{i}")), "light runs under it");
    rt.frame(0.18);
    assert_eq!(rt.presented(&format!("n{f}-{i}")).unwrap(), number, "the number never moves");
}

#[test]
fn a_second_click_does_not_cut_the_first_wave_short() {
    let mut rt = runtime(Saved::default());
    settle(&mut rt);
    let first = rt.state.game.cells.iter().position(|c| !c.mine && c.adj > 0).unwrap();
    click_cell(&mut rt, first);
    let f = field_key(&rt.state);
    let next = rt.state.game.cells.iter().position(|c| !c.mine && c.cover == Cover::Hidden).unwrap();
    click_cell(&mut rt, next);
    assert!(shown(&rt, &format!("reveal{f}-{first}")), "the first wave keeps running");
    assert!(shown(&rt, &format!("reveal{f}-{next}")));
}

#[test]
fn keys_open_and_flag_at_once_with_no_wave() {
    let mut rt = runtime(Saved::default());
    settle(&mut rt);
    let blank = rt.state.game.cells.iter().position(|c| !c.mine && c.adj == 0).unwrap();
    rt.update(|s, _| s.hover_cell = Some(blank));
    type_text(&mut rt, " ");
    assert!(rt.state.game.opened > 0);
    assert!(!rt.elements().iter().any(|e| e.key.as_deref().is_some_and(|k| k.starts_with("reveal"))), "keys never animate");
    // A flag placed by the pointer after it never replays the keyboard's opens.
    let hidden = rt.state.game.cells.iter().position(|c| c.cover == Cover::Hidden).unwrap();
    let (x, y) = point(&rt, hidden);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(0.0);
    assert!(!rt.elements().iter().any(|e| e.key.as_deref().is_some_and(|k| k.starts_with("reveal"))));
}

#[test]
fn a_keyboard_win_still_celebrates() {
    let mut rt = runtime(Saved::default());
    settle(&mut rt);
    one_from_a_win(&mut rt);
    type_text(&mut rt, " ");
    assert_eq!(rt.state.game.status, Status::Won);
    let f = rt.state.field;
    for key in ["burst", "aura", &format!("face-burst{f}")] {
        assert!(shown(&rt, key), "{key}: keys skip the reveal's light, not the win");
    }
}

#[test]
fn reduce_motion_leaves_out_the_lamp_and_the_spatial_effects() {
    for scene in ["won", "lost"] {
        let mut rt = runtime(Saved::default());
        rt.update(|s, _| {
            crate::SCENES.iter().find(|s| s.0 == scene).unwrap().1(s);
            s.reduced = true;
        });
        rt.frame(0.0);
        let f = rt.state.field;
        assert!(shown(&rt, "board"));
        for key in ["lamp", "shock", "burst", "aura", &format!("heat{f}"), &format!("face-burst{f}")] {
            assert!(!shown(&rt, key), "{scene}: {key} is left out");
        }
        let f = field_key(&rt.state);
        assert!(!rt.elements().iter().any(|e| e.key.as_deref().is_some_and(|k| k.starts_with(&format!("b{f}-")))), "{scene}: no blast");
    }
}

#[test]
fn a_new_field_from_a_key_puts_the_lamp_out_at_once() {
    let mut rt = runtime(Saved::default());
    settle(&mut rt);
    let (x, y) = point(&rt, 40);
    rt.pointer_move(x, y);
    settle(&mut rt);
    assert!(rt.state.lamp.on.get() > 0.5);
    press(&mut rt, Key::Function(2));
    assert_eq!(rt.state.lamp.on.get(), 0.0);
    assert!(rt.state.lit_turns.is_empty());
}
