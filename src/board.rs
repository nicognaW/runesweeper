//! The minefield: tiles, the numbers under them, the selector and the board's own effects.
//!
//! Each cell has at most one cover element for its whole life in a field (`t{field}-{i}`), so
//! dealing, lifting off and the win wave are one element changing its sequence, never a swap.

use rune::{Beat, Color, Cursor, El, MotionIntent, PointerButton, Sequence, Spring, Transition, effect, el, text};
use rune_motion::Easing;

use crate::app::App;
use crate::game::{Cover, Status};
use crate::gfx;
use crate::icons::{cross, flag, mine};
use crate::style::*;

/// Emil Kowalski's strong ease-out: for everything entering or leaving.
const OUT: Easing = Easing::cubic(0.23, 1.0, 0.32, 1.0);
/// A tile lifting off as its cell opens: a quick swell, then gone (240 ms).
const VANISH: &[Beat] = &[Beat::new(0.0), Beat::new(0.06).scale(1.06).easing(OUT), Beat::new(0.24).opacity(0.0).scale(0.85).easing(OUT)];
/// A mine coming up as the chain reaches it (220 ms).
const POP: &[Beat] = &[Beat::new(0.0).opacity(0.0).scale(0.6), Beat::new(0.22).easing(Easing::OVERSHOOT)];
/// A new field dealing its tiles in (220 ms each).
const DEAL: &[Beat] = &[Beat::new(0.0).opacity(0.0).scale(0.8), Beat::new(0.22).easing(OUT)];
/// The win: a swell that runs across the field (rare: the delight budget).
const WAVE: &[Beat] = &[Beat::new(0.0), Beat::new(0.14).scale(1.2).easing(OUT), Beat::new(0.36).easing(Easing::EASE_IN_OUT)];
/// Fades on frequent surfaces: 150 ms in and out.
const QUICK: rune_motion::SpringPair = rune_motion::SpringPair::new(Spring::new(0.15, 0.0), Spring::new(0.15, 0.0));
/// A mine going off: the flash behind it (300 ms).
const FLASH: &[Beat] = &[Beat::new(0.0).opacity(0.0).scale(0.6), Beat::new(0.06).scale(1.3).easing(OUT), Beat::new(0.3).opacity(0.0).scale(1.0).easing(OUT)];
/// The result banner: in quick, out a little slower once it's been read.
const BANNER: rune_motion::SpringPair = rune_motion::SpringPair::new(Spring::new(0.15, 0.0), Spring::new(0.25, 0.0));

pub fn size(s: &App) -> (f32, f32) {
    (s.game.w as f32 * PITCH - GAP + 2.0 * WELL, s.game.h as f32 * PITCH - GAP + 2.0 * WELL)
}

fn pos(s: &App, i: usize) -> (f32, f32) {
    let (x, y) = (i % s.game.w, i / s.game.w);
    (WELL + x as f32 * PITCH, WELL + y as f32 * PITCH)
}

/// Cells drawn sunken under the held pointer: the one pressed, or a chord's 3×3.
fn pressed(s: &App) -> Vec<usize> {
    let Some(i) = s.press.cell else { return Vec::new() };
    if s.game.over() || !(s.press.left || s.press.middle) {
        return Vec::new();
    }
    let mut v = vec![i];
    if s.press.chord() {
        v.extend(s.game.neighbours(i));
    }
    v.retain(|&j| matches!(s.game.cells[j].cover, Cover::Hidden | Cover::Question));
    v
}

fn with_seq(e: El, frames: &'static [Beat], delay: f64, reduced: bool) -> El {
    if reduced { e } else { e.sequence(Sequence::new(frames).delay(delay)) }
}

/// A raised tile, lit from the top left: its face shifts a little across the field.
fn raised(p: &Palette, sunk: bool, light: f32, gold: bool) -> El {
    let e = el().size(TILE, TILE).radius(5.0);
    if sunk {
        return e.fill(p.open).border(1.0, p.open_edge).box_shadow(inset(1.0, 3.0, ca(0x000000, 0.5)));
    }
    let (top, bottom) = if gold { (p.win_top, p.win_bottom) } else { (p.tile_top, p.tile_bottom) };
    let top = top.mix(Color::WHITE, 0.10 * light);
    let bottom = bottom.mix(Color::BLACK, 0.12 * (1.0 - light));
    e.linear_gradient(std::f32::consts::PI, [(0.0, top.mix(Color::WHITE, 0.12)), (0.42, top), (1.0, bottom)])
        .box_shadow(drop(1.5, 3.0, ca(0x000000, 0.55)))
        .box_shadow(inset(1.0, 0.0, p.tile_light))
        .box_shadow(inset(-1.0, 1.0, ca(0x000000, 0.25)))
        .hover(p.tile_hover)
}

/// A number with a soft halo of its own colour.
fn number(n: u8, p: &Palette) -> El {
    let ink = p.numbers[n as usize];
    el().size(TILE, TILE)
        .center()
        .child(el().absolute(0.0, 0.0).size(TILE, TILE).radial_gradient([0.5, 0.52], TILE * 0.5, [(0.0, ink.alpha(0.26)), (1.0, ink.alpha(0.0))]))
        .child(text(n.to_string()).font(NUMBER).color(ink))
}

pub fn view(s: &App, reduced: bool) -> El {
    let p = Palette::get(s.saved.color);
    let g = &s.game;
    let (bw, bh) = size(s);
    let sunk = pressed(s);
    let f = s.field;
    let lost = g.status == Status::Lost;
    let won = g.status == Status::Won;
    let hit = g.hit.unwrap_or(0);
    let (hx, hy) = (hit % g.w, hit / g.w);
    let dist = |i: usize| {
        let (x, y) = ((i % g.w) as f32 - hx as f32, (i / g.w) as f32 - hy as f32);
        (x * x + y * y).sqrt()
    };
    let (cx, cy) = (g.w as f32 / 2.0, g.h as f32 / 2.0);
    let deal = |i: usize| {
        let (x, y) = ((i % g.w) as f32 + 0.5 - cx, (i / g.w) as f32 + 0.5 - cy);
        ((x * x + y * y).sqrt() * 0.02).min(0.2) as f64
    };
    let ripple = |i: usize| (g.cells[i].wave as f64 * 0.02).min(0.3);
    let boom = |i: usize| if i == hit { 0.0 } else { (dist(i) as f64 * 0.03).min(0.3) + 0.05 };
    let wave = |i: usize| ((i % g.w + i / g.w) as f64 * 0.03 + 0.25).min(1.6);
    let light = |i: usize| 1.0 - ((i % g.w) as f32 / g.w as f32 * 0.6 + (i / g.w) as f32 / g.h as f32 * 0.4);

    let mut opens = Vec::new();
    let mut covers = Vec::new();
    for (i, cell_state) in g.cells.iter().enumerate() {
        let c_ = cell_state;
        let (x, y) = pos(s, i);
        let shown_mine = lost && c_.mine && c_.cover != Cover::Flag;
        let wrong_flag = lost && !c_.mine && c_.cover == Cover::Flag;
        let open = c_.cover == Cover::Open || shown_mine || wrong_flag;
        if open {
            let is_hit = lost && i == hit;
            let mut cell = el().key(format!("o{f}-{i}")).absolute(x, y).size(TILE, TILE).radius(4.0).center();
            cell = if is_hit {
                cell.radial_gradient([0.5, 0.5], TILE * 0.75, [(0.0, p.hit_core), (1.0, p.hit)])
            } else {
                cell.fill(p.open).border(1.0, p.open_edge).box_shadow(inset(1.0, 2.0, ca(0x000000, 0.35)))
            };
            let fresh = c_.turn == g.turn && c_.cover == Cover::Open;
            if c_.mine || wrong_flag {
                let d = boom(i);
                if c_.mine {
                    // The blast behind each mine as the chain reaches it.
                    let blast = el().key(format!("b{f}-{i}")).absolute(-6.0, -6.0).size(TILE + 12.0, TILE + 12.0).radius(TILE).radial_gradient(
                        [0.5, 0.5],
                        TILE * 0.8,
                        [(0.0, p.blast[0].alpha(0.95)), (0.35, p.blast[1].alpha(0.7)), (1.0, p.blast[2].alpha(0.0))],
                    );
                    cell = cell.child(with_seq(blast, FLASH, d, reduced).motion_intent(MotionIntent::KICK | MotionIntent::PULSE | MotionIntent::CUT).inert());
                }
                let ink = if is_hit { c(0x14060a) } else { p.mine_ink() };
                let m = mine(TILE * 0.78, ink);
                let m = if wrong_flag { el().size(TILE, TILE).center().child(m.opacity(0.55)).child(cross(TILE * 0.8, p.flag).absolute(TILE * 0.1, TILE * 0.1)) } else { m };
                cell = cell.child(with_seq(m.key(format!("m{f}-{i}")), POP, d, reduced).motion_intent(MotionIntent::BOUNCE));
            } else if c_.adj > 0 {
                let n = number(c_.adj, &p).key(format!("n{f}-{i}"));
                let n = if won && !fresh { with_seq(n, WAVE, wave(i), reduced) } else { n };
                cell = cell.child(n.motion_intent(MotionIntent::BOUNCE));
            }
            opens.push(cell);
            // The tile that was here lifts off, in a ripple out from the click (or the blast).
            if (fresh || shown_mine) && !reduced {
                let d = if shown_mine { (boom(i) - 0.1).max(0.0) } else { ripple(i) };
                covers.push(
                    with_seq(raised(&p, false, light(i), false).key(format!("t{f}-{i}")).absolute(x, y), VANISH, d, false)
                        .motion_intent(MotionIntent::BOUNCE)
                        .transition(Transition::none())
                        .inert(),
                );
            }
            continue;
        }
        let gold = won && c_.cover == Cover::Flag;
        let tile = raised(&p, sunk.contains(&i), light(i), gold).key(format!("t{f}-{i}")).absolute(x, y).center();
        let tile = match c_.cover {
            Cover::Flag => tile.child(
                flag(TILE * 0.92, p.flag, p.pole())
                    .key(format!("f{f}-{i}"))
                    .transition(Transition::opacity().springs(QUICK)),
            ),
            Cover::Question => tile.child(text("?").key(format!("q{f}-{i}")).font(NUMBER).color(p.text).transition(Transition::opacity().springs(QUICK))),
            _ => tile,
        };
        let tile = if won { with_seq(tile, WAVE, wave(i), reduced) } else { with_seq(tile, DEAL, deal(i), reduced || !s.deal) };
        covers.push(tile.motion_intent(MotionIntent::BOUNCE));
    }

    let won_t = s.moments.won.map(|t| t as f32).unwrap_or(-100.0);
    let mut board = el()
        .key("board")
        .size(bw, bh)
        .radius(12.0)
        .fill(p.well)
        .box_shadow(inset(2.0, 10.0, ca(0x000000, 0.8)))
        .box_shadow(inset(-1.0, 0.0, ca(0xffffff, 0.06)))
        .cursor(Cursor::Pointer)
        .pointer_buttons([PointerButton::Middle])
        .on_drag(|s: &mut App, d, tx| {
            let b = tx.button();
            s.on_board_drag(d, b, tx)
        })
        .on_secondary_press_local(|s: &mut App, e, tx| s.on_board_secondary(e.x, e.y, tx))
        .on_pointer_move_local(|s: &mut App, e, tx| {
            let c = s.cell_at(e.x, e.y);
            s.hover(c, tx)
        })
        .on_hover_change(|s: &mut App, inside, tx| {
            if !inside {
                s.hover(None, tx)
            }
        })
        .role("grid")
        .label("Minefield")
        // A field a key asked for arrives in one frame, by design.
        .motion_intent(if s.deal { MotionIntent::NONE } else { MotionIntent::CUT })
        .children(opens)
        .children(covers);
    // The selector sits on the cell under the pointer, without re-running the view.
    board = board.child(
        el().key("selector")
            .absolute(0.0, 0.0)
            .size(TILE, TILE)
            .radius(5.0)
            .border(1.5, p.accent.alpha(0.9))
            .live_offset(&s.hover_at)
            .live_opacity(s.hover_on.smooth(Spring::new(0.15, 0.0)))
            .inert()
            .child(effect(gfx::SELECT).effect_data([0.6, if s.saved.color { 0.0 } else { 1.0 }, 0.0, 0.0]).effect_bleed(12.0).absolute(0.0, 0.0).size(TILE, TILE).radius(5.0)),
    );
    board = board.child(
        effect(gfx::SPARKLES)
            .key("burst")
            .effect_data([won_t, if won { 30.0 } else { 0.0 }, 9.0, if s.saved.color { 0.0 } else { 1.0 }])
            .effect_bleed(40.0)
            .absolute(0.0, 0.0)
            .size(bw, bh)
            .radius(12.0)
            .inert(),
    );
    board.children(banner(s, &p, bw, bh, reduced))
}

/// The line that says how it went, over the bottom of the field.
fn banner(s: &App, p: &Palette, bw: f32, bh: f32, reduced: bool) -> Option<El> {
    let (title, sub, tint, delay) = match s.game.status {
        Status::Won => (
            format!("Cleared in {} second{}", s.seconds, if s.seconds == 1 { "" } else { "s" }),
            if s.new_best { format!("New best for {}", s.level.name()) } else { "F2 or the face for another".into() },
            p.win,
            0.3,
        ),
        Status::Lost => ("Boom.".to_string(), "F2 or the face to try again".to_string(), if s.saved.color { c(0xff5c6c) } else { p.text }, 0.35),
        _ => return None,
    };
    if s.banner_done {
        return None;
    }
    let w = (bw - 16.0).min(270.0);
    let banner = el()
            .key(format!("banner{}", s.field))
            // Over the field's bottom edge, so it covers as little of it as it can.
            .absolute((bw - w) / 2.0, bh - 26.0)
            .size(w, 50.0)
            .radius(14.0)
            .fill(ca(0x0b0e19, 0.72))
            .backdrop_blur(10.0)
            .border(1.0, tint.alpha(0.55))
            .box_shadow(drop(10.0, 30.0, ca(0x000000, 0.6)))
            .center()
            .child(
                rune::col()
                    .items_center()
                    .gap(2.0)
                    .child(text(title).font(UI_BOLD).color(tint))
                    .child(text(sub).font(SMALL).color(p.muted)),
            )
            .inert();
    Some(if reduced { banner } else { banner.enter_delay(delay).transition(Transition::opacity().springs(BANNER)) })
}

impl Palette {
    fn mine_ink(&self) -> Color {
        c(0xd6dbea).mix(self.text, 0.3)
    }
    fn pole(&self) -> Color {
        c(0xd6dbea)
    }
}
