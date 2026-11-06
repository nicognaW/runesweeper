//! The window: toolbar, console (counters, face, field) and the panels.

use rune::{Cx, El, MotionIntent, col, effect, el, row, spacer, text};
use rune_kit::{Kit, PaletteSeed};

use crate::app::App;
use crate::game::Status;
use crate::icons::{digit, face, led_text};
use crate::strings::S;
use crate::style::*;
use crate::{board, gfx, toolbar};

fn kit(p: &Palette) -> Kit {
    let mut k = Kit::from_palette(PaletteSeed { bg: p.console_top, fg: p.text, accent: p.accent });
    k.radius.sm = 6.0;
    k
}

fn led(key: &str, label: String, v: i32, p: &Palette, color: bool, gold: bool) -> El {
    let (dw, dh) = (17.0, 30.0);
    // Digits change in the frame they change, as an LED does.
    let mut digits = row().gap(4.0).motion_intent(MotionIntent::STEP);
    for ch in led_text(v).chars() {
        let (on, off) = if gold { (p.win, p.win.alpha(0.09)) } else { (p.led_on, p.led_off) };
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
        .child(effect(gfx::LED).effect_data([if !color { 4.0 } else if gold { 0.0 } else { 7.0 }, 0.16, 5.0, 0.0]).absolute(6.0, 6.0).size(w - 12.0, h - 12.0).radius(4.0))
        .child(digits)
        .role("status")
        .label(label)
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
        .label(s.tr(S::NewGame))
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
                .children(s.moments.won.map(|t| effect(gfx::SPARKLES).key(format!("face-burst{}", s.field)).effect_data([t as f32 + 0.15, 8.0, 7.0, if s.saved.color { 0.0 } else { 1.0 }]).effect_bleed(36.0).absolute(0.0, 0.0).size(sz, sz).inert())),
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
        .child(led("Mines", s.trf(S::MinesLeft, &[&s.game.counter()]), s.game.counter(), p, s.saved.color, false))
        .child(spacer())
        .child(face_button(s, p))
        .child(spacer())
        .child(led("Time", s.trf(S::Time, &[&s.seconds]), s.seconds as i32, p, s.saved.color, s.new_best && s.game.status == Status::Won))
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
        // Gold on a win (chrome stays chrome with Color off), gunmetal on a loss.
        Status::Won if s.saved.color => 1.0,
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
        .child(board_el);
    if s.moments.won.is_some() {
        console = console.child(effect(gfx::AURA).key("aura").effect_data([won_t, if s.saved.color { 0.0 } else { 1.0 }, 0.0, 0.0]).effect_bleed(50.0).absolute(0.0, 0.0).size(console_w, console_h).radius(22.0));
    }
    if let Some((t, _)) = boom {
        console = console.child(effect(gfx::HEAT).key(format!("heat{}", s.field)).effect_data([t, if s.saved.color { 0.0 } else { 1.0 }, 0.0, 0.0]).effect_bleed(50.0).absolute(0.0, 0.0).size(console_w, console_h).radius(22.0));
    }
    // The shockwave reads a copy of the whole window, so it's mounted only around a blast
    // (at the root, outside any fading layer) and only for its 0.6 s, never left copying the
    // screen every frame.
    let origin = cx.layout("board").map(|r| [r[0], r[1]]).unwrap_or([0.0, 0.0]);
    let shock = boom.filter(|_| !s.shock_done).map(|(st, sp)| {
        effect(gfx::SHOCK).key("shock").effect_backdrop(true).effect_data([origin[0] + sp[0], origin[1] + sp[1], st, 16.0]).absolute(0.0, 0.0).w_full().h_full().inert()
    });
    let cheat = s.cheat.then(|| {
        let safe = s.pixel_safe;
        el().key("xyzzy").absolute(0.0, 0.0).size(2.0, 2.0).fill(if safe { rune::Color::WHITE } else { rune::Color::BLACK }).inert()
    });
    let body = col()
        .w_full()
        .h_full()
        .child(toolbar::toolbar(s, &p))
        .child(col().key("stage").grow(1.0).min_h(0.0).center().p_all(MARGIN).child(console));
    toolbar::panels(s, &k, &p, &mut layer);
    let base = el()
        .key("root")
        .w_full()
        .h_full()
        .fill(p.bg)
        .child(effect(gfx::BACKDROP).key("backdrop").effect_data([if s.saved.color { 0.0 } else { 1.0 }, 0.0, 0.0, 0.0]).absolute(0.0, 0.0).w_full().h_full())
        .child(body)
        .child(effect(gfx::GRAIN).key("grain").absolute(0.0, 0.0).w_full().h_full().inert())
        .children(shock)
        .children(cheat);
    rune_kit::root(base, layer)
}

trait Pad {
    fn p_all(self, v: f32) -> Self;
}

impl Pad for El {
    fn p_all(self, v: f32) -> Self {
        self.padding(v)
    }
}
