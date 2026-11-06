//! The minefield: tiles, the numbers under them, the selector and the board's own effects.

use rune::{Beat, Cursor, El, PointerButton, Sequence, Spring, Transition, effect, el, text};

use crate::app::App;
use crate::game::{Cover, Status};
use crate::gfx;
use crate::icons::{cross, flag, mine};
use crate::style::*;

/// A tile lifting off as its cell opens.
const VANISH: &[Beat] = &[Beat::new(0.0), Beat::new(0.06).scale(1.07), Beat::new(0.24).opacity(0.0).scale(0.5)];
/// What was under it popping up.
const POP: &[Beat] = &[Beat::new(0.0).opacity(0.0).scale(0.35), Beat::new(0.17).scale(1.18), Beat::new(0.3)];
/// A new field dealing its tiles in.
const DEAL: &[Beat] = &[Beat::new(0.0).opacity(0.0).scale(0.4), Beat::new(0.26).scale(1.06), Beat::new(0.38)];

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

fn seq(frames: &'static [Beat], delay: f64, reduced: bool) -> Option<Sequence> {
    (!reduced).then(|| Sequence::new(frames).delay(delay))
}

fn with_seq(e: El, s: Option<Sequence>) -> El {
    match s {
        Some(s) => e.sequence(s),
        None => e,
    }
}

fn raised(p: &Palette, sunk: bool) -> El {
    let e = el().size(TILE, TILE).radius(5.0);
    if sunk {
        e.fill(p.open).border(1.0, p.open_edge).box_shadow(inset(1.0, 3.0, ca(0x000000, 0.5)))
    } else {
        e.gradient(p.tile_top, p.tile_bottom)
            .box_shadow(drop(1.5, 3.0, ca(0x000000, 0.55)))
            .box_shadow(inset(1.0, 0.0, p.tile_light))
            .box_shadow(inset(-1.0, 1.0, ca(0x000000, 0.25)))
            .hover(p.tile_hover)
    }
}

fn number(n: u8, p: &Palette) -> El {
    text(n.to_string()).font(NUMBER).color(p.numbers[n as usize])
}

pub fn view(s: &App) -> El {
    let p = Palette::get(s.saved.color);
    let g = &s.game;
    let (bw, bh) = size(s);
    let sunk = pressed(s);
    let f = s.field;
    let lost = g.status == Status::Lost;
    let hit = g.hit.unwrap_or(0);
    let (hx, hy) = (hit % g.w, hit / g.w);
    let dist = |i: usize| {
        let (x, y) = ((i % g.w) as f32 - hx as f32, (i / g.w) as f32 - hy as f32);
        (x * x + y * y).sqrt()
    };
    let (cx, cy) = (g.w as f32 / 2.0, g.h as f32 / 2.0);
    let deal_delay = |i: usize| {
        let (x, y) = ((i % g.w) as f32 + 0.5 - cx, (i / g.w) as f32 + 0.5 - cy);
        ((x * x + y * y).sqrt() * 0.025) as f64
    };
    let ripple = |i: usize| (g.cells[i].wave as f64 * 0.022).min(0.9);
    let boom = |i: usize| (dist(i) as f64 * 0.04).min(1.3) + 0.12;

    let mut opens = Vec::new();
    let mut covers = Vec::new();
    for (i, c) in g.cells.iter().enumerate() {
        let (x, y) = pos(s, i);
        let shown_mine = lost && c.mine && c.cover != Cover::Flag;
        let wrong_flag = lost && !c.mine && c.cover == Cover::Flag;
        if c.cover == Cover::Open || shown_mine || wrong_flag {
            let is_hit = lost && i == hit;
            let mut cell = el().key(format!("o{f}-{i}")).absolute(x, y).size(TILE, TILE).radius(4.0).center();
            cell = if is_hit {
                cell.radial_gradient([0.5, 0.5], TILE * 0.75, [(0.0, c_hex(0xff6b6b)), (1.0, p.hit)])
            } else {
                cell.fill(p.open).border(1.0, p.open_edge)
            };
            let fresh = c.turn == g.turn && c.cover == Cover::Open;
            let content = if c.mine || wrong_flag {
                let m = mine(TILE * 0.78, if is_hit { c_hex(0x14060a) } else { p.mine_ink() });
                let m = if wrong_flag { el().size(TILE, TILE).center().child(m.opacity(0.55)).child(cross(TILE * 0.8, p.flag).absolute(TILE * 0.1, TILE * 0.1)) } else { m };
                let delay = if is_hit { 0.0 } else { boom(i) };
                Some(with_seq(m.key(format!("m{f}-{i}")), seq(POP, delay, s.reduced)))
            } else if c.adj > 0 {
                let d = if fresh { ripple(i) + 0.05 } else { 0.0 };
                let n = number(c.adj, &p).key(format!("n{f}-{i}-{}", c.turn));
                Some(if fresh { with_seq(n, seq(POP, d, s.reduced)) } else { n })
            } else {
                None
            };
            opens.push(cell.children(content));
            // The tile that was here lifts off, a ripple out from the click.
            if (fresh || shown_mine) && !s.reduced {
                let d = if shown_mine && !(i == hit) { boom(i) - 0.1 } else if i == hit { 0.0 } else { ripple(i) };
                covers.push(with_seq(raised(&p, false).key(format!("g{f}-{i}-{}", c.turn)).absolute(x, y), seq(VANISH, d.max(0.0), false)).transition(Transition::none()));
            }
            continue;
        }
        let tile = raised(&p, sunk.contains(&i)).key(format!("t{f}-{i}")).absolute(x, y).center();
        let tile = match c.cover {
            Cover::Flag => tile.child(
                flag(TILE * 0.92, p.flag, p.pole())
                    .key(format!("f{f}-{i}"))
                    .transition(Transition::scale(0.3).combined(Transition::offset(0.0, -6.0))),
            ),
            Cover::Question => tile.child(text("?").key(format!("q{f}-{i}")).font(NUMBER).color(p.text).transition(Transition::scale(0.5))),
            _ => tile,
        };
        covers.push(with_seq(tile, seq(DEAL, deal_delay(i), s.reduced)));
    }

    let (rt, rp) = s.moments.opened.map(|(t, at)| (t as f32, at)).unwrap_or((-100.0, [0.0, 0.0]));
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
        .on_secondary_press_local(|s: &mut App, e, _| s.on_board_secondary(e.x, e.y))
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
        .children(opens)
        .child(effect(gfx::RIPPLE).key("ripple").effect_data([rp[0], rp[1], rt, if s.saved.color { 1.0 } else { 4.0 }]).absolute(0.0, 0.0).size(bw, bh).radius(12.0))
        .children(covers);
    // The selector glides from cell to cell on a spring without re-running the view.
    let spring = Spring::new(0.22, 0.12);
    board = board.child(
        el().key("selector")
            .absolute(0.0, 0.0)
            .size(TILE, TILE)
            .radius(5.0)
            .border(1.5, p.accent.alpha(0.9))
            .live_offset(s.hover_at.smooth(spring))
            .live_opacity(s.hover_on.smooth(Spring::new(0.18, 0.0)))
            .inert()
            .child(effect(gfx::SELECT).effect_data([0.6, 0.0, 0.0, 0.0]).effect_bleed(12.0).absolute(0.0, 0.0).size(TILE, TILE).radius(5.0)),
    );
    if lost {
        let (x, y) = pos(s, hit);
        board = board.child(effect(gfx::FIRE).key(format!("fire{f}")).effect_bleed(70.0).absolute(x + 2.0, y + 4.0).size(TILE - 4.0, TILE - 6.0).radius(4.0).inert());
    }
    board = board.child(
        effect(gfx::SPARKLES)
            .key("burst")
            .effect_data([won_t, if s.moments.won.is_some() { 26.0 } else { 0.0 }, 9.0, 0.0])
            .effect_bleed(40.0)
            .absolute(0.0, 0.0)
            .size(bw, bh)
            .radius(12.0)
            .inert(),
    );
    board
}

fn c_hex(h: u32) -> rune::Color {
    c(h)
}

impl Palette {
    fn mine_ink(&self) -> rune::Color {
        c(0xd6dbea).mix(self.text, 0.3)
    }
    fn pole(&self) -> rune::Color {
        c(0xd6dbea)
    }
}
