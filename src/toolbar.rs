//! The slim toolbar over the console and the panels it opens: the Level picker (Custom's sizes
//! inline) and Settings. Panels are infrequent, so they slide in; a key closes one at once.

use rune::{Align, El, Icon, Side, Spring, Transition, col, el, icon, row, spacer, text};
use rune_kit::{Kit, Switch, TextInput};
use rune_motion::SpringPair;

use crate::app::{App, Panel};
use crate::game::Level;
use crate::icons::flag;
use crate::strings::{Lang, S};
use crate::style::*;

static SETTINGS: Icon = Icon {
    name: "runesweeper-settings",
    view: 16.0,
    body: r#"<path d="M2.5 4.5H13.5M2.5 11.5H13.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/><circle cx="10" cy="4.5" r="1.9" fill="currentColor"/><circle cx="6" cy="11.5" r="1.9" fill="currentColor"/>"#,
};

static CHEVRON: Icon = Icon {
    name: "runesweeper-chevron",
    view: 16.0,
    body: r#"<path d="M5 6.5L8 9.5L11 6.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

const LEVELS: [Level; 4] = [Level::Beginner, Level::Intermediate, Level::Expert, Level::Custom];

fn level_key(l: Level) -> &'static str {
    match l {
        Level::Beginner => "pick-beginner",
        Level::Intermediate => "pick-intermediate",
        Level::Expert => "pick-expert",
        Level::Custom => "pick-custom",
    }
}

fn sizes((w, h, m): (usize, usize, usize)) -> String {
    format!("{w} × {h} · {m}")
}

/// A recessed well, as the header's: the toolbar's controls and the panels' rows sit in these.
fn well(e: El) -> El {
    e.fill(ca(0x000000, 0.25)).box_shadow(inset(1.0, 4.0, ca(0x000000, 0.6))).box_shadow(inset(-1.0, 0.0, ca(0xffffff, 0.05)))
}

/// A square toolbar button holding `glyph`.
fn tool(key: &str, label: &str, glyph: El, p: &Palette) -> El {
    well(el().key(key).size(36.0, 30.0).radius(9.0)).center().role("button").label(label.to_string()).focusable(true).press_scale(0.94).hover(p.tile_hover).child(glyph)
}

/// The Level chip: the current Level, and the picker it opens.
fn chip(s: &App, p: &Palette) -> El {
    let name = s.tr(S::level(s.level));
    let mut label = row().gap(6.0).center_y().child(text(name).font(UI_BOLD).color(p.text));
    if s.level == Level::Custom {
        label = label.child(text(sizes(s.dims())).font(SMALL).color(p.muted));
    }
    well(row().key("level-chip").h(30.0).px(12.0).gap(6.0).radius(9.0))
        .center_y()
        .role("button")
        .label(name.to_string())
        .expanded(s.panel == Some(Panel::Picker))
        .focusable(true)
        .press_scale(0.97)
        .hover(p.tile_hover)
        .on_press(|s: &mut App| s.toggle_panel(Panel::Picker))
        .child(label)
        .child(icon(&CHEVRON, 14.0).color(p.muted))
}

/// The touch builds' flag toggle: its icon and state say what it does. Off, a dim flag in a
/// well; on, a lit flag on the flag's own red.
fn flag_toggle(s: &App, p: &Palette) -> El {
    let on = s.flag_mode;
    let (cloth, pole) = if on { (rune::Color::WHITE, rune::Color::WHITE) } else { (p.muted, p.muted) };
    let t = tool("flag-toggle", s.tr(S::FlagMode), flag(18.0, cloth, pole), p).role("switch").checked(on).on_press(|s: &mut App| s.flag_mode = !s.flag_mode);
    if on { t.fill(p.flag).box_shadow(drop(0.0, 12.0, p.flag.alpha(0.55))) } else { t }
}

/// The buttons on the toolbar's right, in order. Hint and History join them here.
fn tools(s: &App, p: &Palette) -> Vec<El> {
    let mut v = Vec::new();
    if s.touch {
        v.push(flag_toggle(s, p));
    }
    v.push(
        tool("settings-button", s.tr(S::Settings), icon(&SETTINGS, 16.0).color(p.text), p)
            .expanded(s.panel == Some(Panel::Settings))
            .on_press(|s: &mut App| s.toggle_panel(Panel::Settings)),
    );
    v
}

pub fn toolbar(s: &App, p: &Palette) -> El {
    row().key("toolbar").w_full().h(TOOLBAR_H).px(16.0).pt(8.0).gap(8.0).center_y().child(chip(s, p)).child(spacer()).children(tools(s, p))
}

/// A panel's surface, styled like the console it drops from, anchored under its button.
fn surface(key: &str, label: &str, anchor: &str, align: Align, body: El, p: &Palette) -> El {
    let slide = Transition::offset(0.0, -10.0).combined(Transition::opacity()).springs(SpringPair::new(Spring::new(0.22, 0.0), Spring::new(0.16, 0.0)));
    col()
        .key(key)
        .role("dialog")
        .label(label.to_string())
        .padding(8.0)
        .gap(6.0)
        .radius(16.0)
        .gradient(p.console_top, p.console_bottom)
        .border(1.0, ca(0xffffff, 0.07))
        .box_shadow(drop(16.0, 40.0, ca(0x000000, 0.65)))
        .box_shadow(inset(1.0, 0.0, ca(0xffffff, 0.08)))
        .child(body)
        .anchored(anchor, Side::Bottom, align, 8.0)
        .transition(slide)
}

/// One row of a panel: a well that lights its edge when it is the current choice.
fn choice(key: &str, on: bool, p: &Palette) -> El {
    let r = well(row().key(key).w_full().h(40.0).px(12.0).gap(8.0).radius(10.0)).center_y().focusable(true).hover(p.tile_hover);
    if on { r.border(1.0, p.accent.alpha(0.8)) } else { r }
}

fn picker(s: &App, k: &Kit, p: &Palette) -> El {
    let mut body = col().w(270.0).gap(6.0);
    for l in LEVELS {
        let detail = l.dims().map(sizes);
        let name = s.tr(S::level(l));
        let mut r = choice(level_key(l), s.level == l, p)
            .role("button")
            .label(name.to_string())
            .on_press_tx(move |s: &mut App, tx| s.pick_level(l, tx))
            .child(text(name).font(UI_BOLD).color(p.text))
            .child(spacer());
        if let Some(d) = detail {
            r = r.child(text(d).font(SMALL).color(p.muted));
        }
        body = body.child(r);
    }
    // Custom's sizes, typed in place: Enter in any of them starts the field.
    let input = |key: &str, label: &str, e: &rune::TextEdit, f: fn(&mut App) -> &mut rune::TextEdit| {
        col().gap(3.0).grow(1.0).child(text(label.to_string()).font(SMALL).color(p.muted)).child(
            TextInput::new(e).label(label).width(None).on_edit(f).on_submit(|s: &mut App, tx| s.pick_level(Level::Custom, tx)).build(k).key(key),
        )
    };
    body = body.child(
        row()
            .key("custom-sizes")
            .gap(8.0)
            .px(4.0)
            .pb(2.0)
            .child(input("custom-w", s.tr(S::Width), &s.edit_w, |s| &mut s.edit_w))
            .child(input("custom-h", s.tr(S::Height), &s.edit_h, |s| &mut s.edit_h))
            .child(input("custom-m", s.tr(S::Mines), &s.edit_m, |s| &mut s.edit_m)),
    );
    surface("picker", s.tr(S::LevelPicker), "level-chip", Align::Start, body, p)
}

fn setting(key: &str, label: &str, on: bool, toggle: fn(&mut App), k: &Kit, p: &Palette) -> El {
    choice(key, false, p)
        .role("switch")
        .label(label.to_string())
        .checked(on)
        .on_press(toggle)
        .child(text(label.to_string()).font(UI_BOLD).color(p.text))
        .child(spacer())
        .child(Switch::new(on).hide_label().build(k).inert())
}

/// The Language row: each language by its own name, the current one lit. A press switches
/// every string at once.
fn language(s: &App, p: &Palette) -> El {
    let mut options = row().key("language").role("radiogroup").label(s.tr(S::Language)).gap(4.0);
    for l in Lang::ALL {
        let on = s.lang == l;
        let mut o = el()
            .key(format!("lang-{}", l.tag()))
            .h(28.0)
            .px(10.0)
            .radius(7.0)
            .center()
            .role("radio")
            .label(l.endonym())
            .checked(on)
            .focusable(true)
            .press_scale(0.96)
            .on_press(move |s: &mut App| s.set_lang(l))
            .child(text(l.endonym()).font(UI_BOLD).color(if on { p.text } else { p.muted }));
        o = if on { o.fill(p.accent.alpha(0.22)).border(1.0, p.accent.alpha(0.8)) } else { o.hover(p.tile_hover) };
        options = options.child(o);
    }
    well(row().w_full().h(40.0).pl(12.0).pr(6.0).gap(8.0).radius(10.0))
        .center_y()
        .child(text(s.tr(S::Language)).font(UI_BOLD).color(p.text))
        .child(spacer())
        .child(options)
}

fn settings(s: &App, k: &Kit, p: &Palette) -> El {
    let body = col()
        .w(270.0)
        .gap(6.0)
        .child(setting("marks-switch", s.tr(S::Marks), s.saved.marks, App::toggle_marks, k, p))
        .child(setting("color-switch", s.tr(S::Color), s.saved.color, App::toggle_color, k, p))
        .child(language(s, p));
    surface("settings", s.tr(S::Settings), "settings-button", Align::End, body, p)
}

/// The open panel, over a transparent catcher that closes it on a press anywhere else.
pub fn panels(s: &App, k: &Kit, p: &Palette, layer: &mut Vec<El>) {
    let Some(open) = s.panel else { return };
    layer.push(rune_kit::scrim("panel-scrim", |s: &mut App| s.close_panel()));
    layer.push(match open {
        Panel::Picker => picker(s, k, p),
        Panel::Settings => settings(s, k, p),
    });
}
