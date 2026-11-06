//! The window: menu bar, console (counters, face, field) and the dialogs.

use rune::{Cx, El, MotionIntent, col, effect, el, row, spacer, text};
use rune_kit::{Button, ButtonVariant, Dialog as KitDialog, Kit, Menu as KitMenu, MenuItem, PaletteSeed, TextInput};

use crate::app::{App, Dialog, Menu};
use crate::game::{Level, Status};
use crate::icons::{digit, face, led_text};
use crate::style::*;
use crate::{board, gfx};

fn kit(p: &Palette) -> Kit {
    let mut k = Kit::from_palette(PaletteSeed { bg: p.console_top, fg: p.text, accent: p.accent });
    k.radius.sm = 6.0;
    k
}

fn led(key: &str, v: i32, p: &Palette, color: bool, gold: bool) -> El {
    let (dw, dh) = (17.0, 30.0);
    // Digits change in the frame they change, as an LED does.
    let mut digits = row().gap(4.0).motion_intent(MotionIntent::STEP);
    for ch in led_text(v).chars() {
        let (on, off) = if gold { (c(0xffc531), ca(0xffc531, 0.09)) } else { (p.led_on, p.led_off) };
        digits = digits.child(digit(ch, dw, dh, on, off));
    }
    let (w, h) = (3.0 * dw + 8.0 + 16.0, dh + 12.0);
    el().key(key)
        .size(w, h)
        .radius(8.0)
        .fill(p.led_bg)
        .box_shadow(inset(1.5, 6.0, ca(0x000000, 0.9)))
        .box_shadow(drop(-1.0, 0.0, ca(0xffffff, 0.06)))
        .center()
        .child(effect(gfx::LED).effect_data([if gold { 0.0 } else if color { 7.0 } else { 4.0 }, 0.16, 5.0, 0.0]).absolute(6.0, 6.0).size(w - 12.0, h - 12.0).radius(4.0))
        .child(digits)
        .role("status")
        .label(format!("{key}: {v}"))
}

fn face_button(s: &App, p: &Palette) -> El {
    let sz = 44.0;
    let mood = s.mood();
    let ink = c(0x1a1206);
    el().key("face")
        .size(sz + 8.0, sz + 8.0)
        .radius((sz + 8.0) / 2.0)
        .fill(ca(0x000000, 0.35))
        .box_shadow(inset(1.0, 4.0, ca(0x000000, 0.8)))
        .center()
        .press_scale(0.9)
        .role("button")
        .label("New game")
        .focusable(true)
        .on_press(|s: &mut App| s.new_game())
        .child(
            el().size(sz, sz)
                .radius(sz / 2.0)
                .box_shadow(drop(2.0, 6.0, ca(0x000000, 0.6)))
                .child(effect(gfx::FACE).effect_data([if s.saved.color { 0.0 } else { 5.0 }, 0.0, 0.0, 0.0]).absolute(0.0, 0.0).size(sz, sz).radius(sz / 2.0))
                // The expression swaps in the frame it changes, as the original's does.
                .child(face(sz, mood, ink).key(format!("mood-{mood:?}")).absolute(0.0, 0.0).motion_intent(MotionIntent::CUT))
                // A win: one burst of stars off the face (rare, so it gets the delight budget).
                .children(s.moments.won.map(|t| effect(gfx::SPARKLES).key(format!("face-burst{}", s.field)).effect_data([t as f32 + 0.15, 8.0, 7.0, 0.0]).effect_bleed(36.0).absolute(0.0, 0.0).size(sz, sz).inert())),
        )
        .hover(p.tile_hover)
}

fn header(s: &App, p: &Palette, width: f32) -> El {
    row().key("header")
        .w(width)
        .h(HEADER_H)
        .px(10.0)
        .radius(12.0)
        .fill(ca(0x000000, 0.22))
        .box_shadow(inset(1.0, 5.0, ca(0x000000, 0.6)))
        .box_shadow(inset(-1.0, 0.0, ca(0xffffff, 0.05)))
        .center_y()
        .child(led("Mines", s.game.counter(), p, s.saved.color, false))
        .child(spacer())
        .child(face_button(s, p))
        .child(spacer())
        .child(led("Time", s.seconds as i32, p, s.saved.color, s.new_best && s.game.status == Status::Won))
}

fn menu_bar(s: &App, k: &Kit, p: &Palette, layer: &mut Vec<El>) -> El {
    let trigger = |label: &str, m: Menu| {
        Button::new(label)
            .variant(ButtonVariant::Ghost)
            .expanded(s.menu == Some(m))
            .on_press(move |s: &mut App| s.menu = if s.menu == Some(m) { None } else { Some(m) })
            .build(k)
    };
    let lvl = |l: Level| move |s: &mut App, tx: &mut rune::Tx| {
        s.menu = None;
        s.set_level(l, tx);
    };
    let game = KitMenu::new("game-menu")
        .trigger(trigger("Game", Menu::Game))
        .open(s.menu == Some(Menu::Game))
        .min_width(210.0)
        .item(MenuItem::new("New").shortcut("F2").on_select(|s: &mut App| {
            s.menu = None;
            s.new_game();
        }))
        .separator()
        .item(MenuItem::new("Beginner").checked(s.level == Level::Beginner).on_select_tx(lvl(Level::Beginner)))
        .item(MenuItem::new("Intermediate").checked(s.level == Level::Intermediate).on_select_tx(lvl(Level::Intermediate)))
        .item(MenuItem::new("Expert").checked(s.level == Level::Expert).on_select_tx(lvl(Level::Expert)))
        .item(MenuItem::new("Custom…").checked(s.level == Level::Custom).on_select(|s: &mut App| {
            s.menu = None;
            s.open_custom();
        }))
        .separator()
        .item(MenuItem::new("Marks (?)").checked(s.saved.marks).on_select(|s: &mut App| {
            s.menu = None;
            s.toggle_marks();
        }))
        .item(MenuItem::new("Color").checked(s.saved.color).on_select(|s: &mut App| {
            s.menu = None;
            s.toggle_color();
        }))
        .separator()
        .item(MenuItem::new("Best Times…").on_select(|s: &mut App| {
            s.menu = None;
            s.dialog = Some(Dialog::BestTimes);
        }))
        .separator()
        .item(MenuItem::new("Exit").on_select_tx(|s: &mut App, tx| {
            s.menu = None;
            tx.quit();
        }))
        .on_close(|s: &mut App| s.menu = None)
        .build(k, layer);
    let help = KitMenu::new("help-menu")
        .trigger(trigger("Help", Menu::Help))
        .open(s.menu == Some(Menu::Help))
        .min_width(190.0)
        .item(MenuItem::new("How to Play").on_select(|s: &mut App| {
            s.menu = None;
            s.dialog = Some(Dialog::HowTo);
        }))
        .separator()
        .item(MenuItem::new("About Runesweeper…").on_select(|s: &mut App| {
            s.menu = None;
            s.dialog = Some(Dialog::About);
        }))
        .on_close(|s: &mut App| s.menu = None)
        .build(k, layer);
    row().key("menubar")
        .w_full()
        .h(MENU_H)
        .px(8.0)
        .gap(2.0)
        .center_y()
        .fill(ca(0x000000, 0.25))
        .box_shadow(inset(-1.0, 0.0, ca(0xffffff, 0.06)))
        .child(game)
        .child(help)
        .child(spacer())
        .children(touch_first().then(|| {
            Button::new(if s.flag_mode { "Flagging" } else { "Flag" })
                .variant(if s.flag_mode { ButtonVariant::Contrast } else { ButtonVariant::Ghost })
                .help("Taps place flags (F on a keyboard)")
                .on_press(|s: &mut App| s.flag_mode = !s.flag_mode)
                .build(k)
        }))
        .child(text(s.level.name()).font(SMALL).color(p.muted).mr(8.0))
}

fn field_row(label: &str, input: El, p: &Palette) -> El {
    row().gap(12.0).center_y().child(text(label.to_string()).font(UI).color(p.text).w(64.0)).child(input)
}

fn dialogs(s: &App, k: &Kit, p: &Palette, layer: &mut Vec<El>) {
    let Some(d) = s.dialog else { return };
    let close = |s: &mut App| s.dialog = None;
    let ok = |label: &str| Button::new(label).variant(ButtonVariant::Contrast);
    let buttons = |a: El, b: Option<El>| row().gap(8.0).mt(8.0).child(spacer()).children(b).child(a);
    match d {
        Dialog::Custom => {
            let input = |e: &rune::TextEdit, f: fn(&mut App) -> &mut rune::TextEdit| {
                TextInput::new(e).width(Some(110.0)).on_edit(f).on_submit(|s: &mut App, tx| s.apply_custom(tx)).build(k)
            };
            let body = col()
                .gap(10.0)
                .child(field_row("Height", input(&s.edit_h, |s| &mut s.edit_h), p))
                .child(field_row("Width", input(&s.edit_w, |s| &mut s.edit_w), p))
                .child(field_row("Mines", input(&s.edit_m, |s| &mut s.edit_m), p))
                .child(text("Height 9–24, width 9–30.").font(SMALL).color(p.muted))
                .child(buttons(
                    ok("OK").on_press_tx(|s: &mut App, tx| s.apply_custom(tx)).build(k),
                    Some(Button::new("Cancel").on_press(close).build(k)),
                ));
            KitDialog::new("custom", "Custom Field").open(true).fit().width(300.0).child(body).on_close(close).build(k, layer);
        }
        Dialog::BestTimes => {
            let mut body = col().gap(8.0);
            for (i, l) in [Level::Beginner, Level::Intermediate, Level::Expert].into_iter().enumerate() {
                let (t, name) = &s.saved.best[i];
                body = body.child(
                    row().gap(12.0)
                        .child(text(format!("{}:", l.name())).font(UI).color(p.muted).w(96.0))
                        .child(text(format!("{t} seconds")).font(UI_BOLD).color(p.text).w(96.0))
                        .child(text(name.clone()).font(UI).color(p.text).truncate()),
                );
            }
            body = body.child(buttons(
                ok("OK").on_press(close).build(k),
                Some(Button::new("Reset Scores").on_press(|s: &mut App| s.reset_scores()).build(k)),
            ));
            KitDialog::new("best", "Fastest Mine Sweepers").open(true).fit().width(380.0).child(body).on_close(close).build(k, layer);
        }
        Dialog::Record(li) => {
            let level = [Level::Beginner, Level::Intermediate, Level::Expert][li];
            let body = col()
                .gap(12.0)
                .child(text(format!("You have the fastest time for {} level. Please type your name.", level.name().to_lowercase())).font(UI).color(p.text).wrap())
                .child(TextInput::new(&s.edit_name).on_edit(|s: &mut App| &mut s.edit_name).on_submit(move |s: &mut App, _| s.save_record(li)).build(k))
                .child(buttons(ok("OK").on_press(move |s: &mut App| s.save_record(li)).build(k), None));
            KitDialog::new("record", "New Record").open(true).fit().width(340.0).child(body).on_close(move |s: &mut App| s.save_record(li)).build(k, layer);
        }
        Dialog::About => {
            let body = col()
                .gap(8.0)
                .child(text("Runesweeper 0.1").font(UI_BOLD).color(p.text))
                .child(text("The Minesweeper you know, drawn and lit on the GPU with Rune.").font(UI).color(p.muted).wrap())
                .child(text("Open source, MIT licensed.").font(SMALL).color(p.muted))
                .child(buttons(ok("OK").on_press(close).build(k), None));
            KitDialog::new("about", "About Runesweeper").open(true).fit().width(340.0).child(body).on_close(close).build(k, layer);
        }
        Dialog::HowTo => {
            let lines = [
                "Left click opens a square. A number counts the mines touching it.",
                "Right click flags a mine; again for ?, again to clear (Game ▸ Marks).",
                "With a number's flags placed, click both buttons (or the middle one) on it to open its other neighbours.",
                "Open every square without a mine to win. F2 or the face starts again.",
            ];
            let mut body = col().gap(8.0);
            for l in lines {
                body = body.child(text(l).font(UI).color(p.text).wrap());
            }
            body = body.child(buttons(ok("OK").on_press(close).build(k), None));
            KitDialog::new("howto", "How to Play").open(true).fit().width(380.0).child(body).on_close(close).build(k, layer);
        }
    }
}

/// The app icon: a raised console tile with the face over a corner of the field.
fn icon(p: &Palette) -> El {
    let n = 1024.0;
    let mut tiles = el().absolute(150.0, 360.0).size(724.0, 514.0);
    for y in 0..3 {
        for x in 0..4 {
            let open = (x + y) % 3 == 0;
            let t = el().absolute(x as f32 * 184.0, y as f32 * 176.0).size(172.0, 164.0).radius(30.0);
            let t = if open {
                t.fill(p.open).child(text(["1", "2", "3"][y]).font(rune::TextStyle::new(110.0, 700.0)).color(p.numbers[y + 1])).center()
            } else {
                t.gradient(p.tile_top, p.tile_bottom).box_shadow(inset(6.0, 0.0, p.tile_light)).box_shadow(drop(8.0, 16.0, ca(0, 0.6)))
            };
            tiles = tiles.child(t);
        }
    }
    el().key("icon")
        .size(n, n)
        .child(
            el().absolute(64.0, 64.0)
                .size(896.0, 896.0)
                .radius(200.0)
                .gradient(p.console_top, p.console_bottom)
                .clip_rounded(200.0)
                .child(effect(gfx::BACKDROP).absolute(0.0, 0.0).size(896.0, 896.0))
                .child(effect(gfx::RIM).effect_data([1.0, 22.0, 0.0, 0.0]).absolute(0.0, 0.0).size(896.0, 896.0).radius(200.0))
                .child(tiles)
                .child(
                    el().absolute(298.0, 70.0)
                        .size(300.0, 300.0)
                        .child(effect(gfx::FACE).effect_data([0.0, 0.0, 0.0, 0.0]).absolute(0.0, 0.0).size(300.0, 300.0).radius(150.0))
                        .child(face(300.0, crate::icons::Mood::Cool, c(0x1a1206)).absolute(0.0, 0.0)),
                ),
        )
}

pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let p = Palette::get(s.saved.color);
    if s.icon {
        return icon(&p);
    }
    let k = kit(&p);
    let mut layer = Vec::new();
    let (bw, bh) = board::size(s);
    let inner_w = bw.max(9.0 * PITCH + 2.0 * WELL);
    let console_w = inner_w + 2.0 * PAD;
    let console_h = bh + HEADER_H + 12.0 + 2.0 * PAD;
    let rim = match s.game.status {
        Status::Won => 1.0,
        Status::Lost => 3.0,
        _ => 0.0,
    };
    let won_t = s.moments.won.map(|t| t as f32).unwrap_or(-100.0);
    let boom = s.moments.boom.map(|(t, at)| (t as f32, at));
    let reduced = cx.system_reduced_motion() || s.reduced;
    let board_el = board::view(s, reduced);
    // Small windows and phones: draw the console scaled to fit (layout keeps its size).
    let fit = cx.layout_size("stage").map_or(1.0, |[w, h]| ((w - 2.0 * MARGIN) / console_w).min((h - 2.0 * MARGIN) / console_h).clamp(0.3, 1.0));
    let mut console = col()
        .key("console")
        .scale(fit)
        .size(console_w, console_h)
        .padding(PAD)
        .gap(12.0)
        .items_center()
        .radius(22.0)
        .gradient(p.console_top, p.console_bottom)
        .box_shadow(drop(24.0, 60.0, ca(0x000000, 0.7)))
        .box_shadow(inset(1.0, 0.0, ca(0xffffff, 0.08)))
        .child(effect(gfx::RIM).key("rim").effect_data([rim, 3.5, 0.0, 0.0]).absolute(0.0, 0.0).size(console_w, console_h).radius(22.0))
        .child(header(s, &p, inner_w))
        .child(board_el)
        // Out of the layout, so the console doesn't move when it goes.
        .children(hint(s, &p).map(|h| {
            col().key("hint")
                .absolute(0.0, console_h + 12.0)
                .w(console_w)
                .items_center()
                .child(h)
                .transition(rune::Transition::opacity().springs(rune_motion::SpringPair::new(rune::Spring::new(0.2, 0.0), rune::Spring::new(0.2, 0.0))))
        }));
    if s.moments.won.is_some() {
        console = console.child(effect(gfx::AURA).key("aura").effect_data([won_t, 0.0, 0.0, 0.0]).effect_bleed(50.0).absolute(0.0, 0.0).size(console_w, console_h).radius(22.0));
    }
    if let Some((t, _)) = boom {
        console = console.child(effect(gfx::HEAT).key(format!("heat{}", s.field)).effect_data([t, 0.0, 0.0, 0.0]).effect_bleed(50.0).absolute(0.0, 0.0).size(console_w, console_h).radius(22.0));
    }
    // The shockwave reads the whole window; it stays mounted (idle, it draws nothing).
    let origin = cx.layout("board").map(|r| [r[0], r[1]]).unwrap_or([0.0, 0.0]);
    let (st, sp) = boom.unwrap_or((-100.0, [0.0, 0.0]));
    let shock = effect(gfx::SHOCK).key("shock").effect_backdrop(true).effect_data([origin[0] + sp[0], origin[1] + sp[1], st, 16.0]).absolute(0.0, 0.0).w_full().h_full().inert();
    let cheat = s.cheat.then(|| {
        let safe = s.hover_cell.is_some_and(|i| !s.game.cells[i].mine);
        el().key("xyzzy").absolute(0.0, 0.0).size(2.0, 2.0).fill(if safe { rune::Color::WHITE } else { rune::Color::BLACK }).inert()
    });
    let body = col()
        .w_full()
        .h_full()
        .child(menu_bar(s, &k, &p, &mut layer))
        .child(col().key("stage").grow(1.0).min_h(0.0).center().p_all(MARGIN).child(console));
    dialogs(s, &k, &p, &mut layer);
    let base = el()
        .key("root")
        .w_full()
        .h_full()
        .fill(p.bg)
        .child(effect(gfx::BACKDROP).key("backdrop").effect_data([if s.saved.color { 0.0 } else { 1.0 }, 0.0, 0.0, 0.0]).absolute(0.0, 0.0).w_full().h_full())
        .child(body)
        .child(effect(gfx::GRAIN).key("grain").absolute(0.0, 0.0).w_full().h_full().inert())
        .child(shock)
        .children(cheat);
    rune_kit::root(base, layer)
}

/// Builds with no right button to lean on (the web, which phones use) show the flag toggle.
fn touch_first() -> bool {
    cfg!(target_arch = "wasm32")
}

/// The first time, a line of how to play under the console, until the first square opens.
fn hint(s: &App, p: &Palette) -> Option<El> {
    if s.saved.played || s.icon {
        return None;
    }
    let (a, b) = if touch_first() {
        ("Tap opens  ·  Flag, then tap, marks a mine", "Tap a number whose flags are placed to open around it")
    } else {
        ("Left click opens  ·  Right click flags", "Both buttons on a number open around it")
    };
    Some(col().items_center().gap(3.0).child(text(a).font(SMALL).color(p.text.alpha(0.8))).child(text(b).font(SMALL).color(p.muted)))
}

trait Pad {
    fn p_all(self, v: f32) -> Self;
}

impl Pad for El {
    fn p_all(self, v: f32) -> Self {
        self.padding(v)
    }
}
