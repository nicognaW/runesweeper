//! The Hint on screen: the H key and the Hint button step through it (nudge, numbers, cell,
//! reason with Apply), the board rings the numbers and the cell, and the callout with the
//! words sits beside the console, or over the field clear of every cell it talks about. Review reuses the callout and rings
//! for its markers.

use rune::{El, Transition, col, el, row, spacer, text};

use crate::app::App;
use crate::game::{Cover, Mode};
use crate::record::Act;
use crate::hint::{self, Action, Hint, Verdict, words};
use crate::strings::S;
use crate::style::*;
use crate::toolbar::well;

/// A hint on screen and how far into it the player has stepped (1 to [`Hint::stages`]).
#[derive(Clone, Debug)]
pub struct Shown {
    pub hint: Hint,
    pub stage: u8,
}

impl App {
    /// The Hint action: a fresh hint at its nudge, or one stage deeper into the one shown.
    /// Each fresh hint counts in the game's record, which makes it an assisted game.
    pub fn press_hint(&mut self) {
        // A field being generated takes no input, nor one waiting out a review.
        if self.game.over() || self.generating.is_some() || self.review.is_some() {
            return;
        }
        if let Some(s) = &mut self.hint {
            s.stage = (s.stage + 1).min(s.hint.stages());
            return;
        }
        let last = self.record.moves.last().map(|m| m.cell);
        let field = hint::Field {
            classic_first: (self.mode == Mode::Classic).then(|| self.record.moves.iter().find(|m| m.act == Act::Open).map(|m| m.cell)).flatten(),
            announced: &self.fifty_fifties,
        };
        self.hint = Some(Shown { hint: hint::advise_game(&self.game, last, &field), stage: 1 });
        self.record.hints += 1;
        self.keep_draft();
    }

    /// Plays the move the hint's last stage names: open the safe cell or the guess, flag the
    /// mine, or take the wrong flag off. Nothing before that stage, nor for a cell to avoid.
    pub fn apply_hint(&mut self) {
        let Some(t) = self.hint.as_ref().filter(|s| s.stage >= s.hint.stages()).and_then(|s| s.hint.target) else { return };
        let Some(action) = t.verdict.action() else { return };
        match action {
            Action::Open => self.release_at(false, t.cell),
            // Right clicks until the cell shows what the hint says: a question mark goes to
            // hidden before a flag, and with marks on a flag comes off through one.
            Action::Flag => self.mark_until(t.cell, Cover::Flag),
            Action::Unflag => self.mark_until(t.cell, Cover::Hidden),
        }
        self.hint = None;
    }

    fn mark_until(&mut self, cell: usize, cover: Cover) {
        for _ in 0..3 {
            if self.game.cells[cell].cover == cover || self.game.over() {
                break;
            }
            self.mark_at(cell);
        }
    }

    pub fn close_hint(&mut self) {
        self.hint = None;
    }
}

/// The callout's width.
pub const WIDTH: f32 = 260.0;
/// Between the console and the callout.
pub const GAP: f32 = 12.0;
/// The callout's height until it has been laid out once.
const GUESS_H: f32 = 160.0;

/// Where the callout goes, and the console's scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    /// Beside the console in the layout: to its right, or below it.
    Beside { fit: f32, right: bool },
    /// Over the console at (`x`, `y`) from its drawn top left, clear of the cells the hint
    /// rings.
    Over { fit: f32, x: f32, y: f32 },
}

impl Place {
    pub fn fit(self) -> f32 {
        match self {
            Place::Beside { fit, .. } | Place::Over { fit, .. } => fit,
        }
    }
}

fn hits(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// Where a `callout` (width, height) goes in a `stage` (window less the toolbar) holding a
/// `console`, whose ringed cells are at `marks` ((x0, y0, x1, y1) in the console's own
/// points). In order: beside the console if it fits there at the console's usual scale; over
/// the console where it covers none of the marks; else beside it, the console shrunk to make
/// room. It never covers a cell the hint talks about.
pub fn place(stage: Option<[f32; 2]>, console: (f32, f32), callout: Option<[f32; 2]>, marks: &[[f32; 4]]) -> Place {
    let Some([sw, sh]) = stage else { return Place::Beside { fit: 1.0, right: false } };
    let (cw, ch) = console;
    let (w, h) = (sw - 2.0 * MARGIN, sh - 2.0 * MARGIN);
    let alone = (w / cw).min(h / ch).clamp(0.3, 1.0);
    let Some([kw, kh]) = callout else { return Place::Beside { fit: alone, right: false } };
    let kh = if kh > 0.0 { kh } else { GUESS_H };
    let right = ((w - kw - GAP) / cw).min(h / ch);
    let below = (w / cw).min((h - kh - GAP) / ch);
    if right >= alone {
        return Place::Beside { fit: alone, right: true };
    }
    if below >= alone {
        return Place::Beside { fit: alone, right: false };
    }
    // Over the console, inside the stage less a small margin, from its drawn top left.
    let (dw, dh) = (cw * alone, ch * alone);
    let (mx, my) = ((sw - dw) / 2.0 - 8.0, (sh - dh) / 2.0 - 8.0);
    let marks: Vec<[f32; 4]> = marks.iter().map(|m| [m[0] * alone - 4.0, m[1] * alone - 4.0, m[2] * alone + 4.0, m[3] * alone + 4.0]).collect();
    // Of the spots clear of the marks, the one farthest from them, so the cells around what
    // the hint reads stay in view too.
    let xs = [((dw - kw) / 2.0).clamp(-mx, dw + mx - kw), -mx, dw + mx - kw];
    let ys = [dh + my - kh, -my];
    let centre = |r: [f32; 4]| [(r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0];
    let n = marks.len().max(1) as f32;
    let at = marks.iter().map(|&m| centre(m)).fold([0.0, 0.0], |a, c| [a[0] + c[0] / n, a[1] + c[1] / n]);
    let clear = ys.iter().flat_map(|&y| xs.iter().map(move |&x| [x, y, x + kw, y + kh])).filter(|&r| !marks.iter().any(|&m| hits(r, m)));
    if let Some(r) = clear.max_by(|&a, &b| {
        let d = |r: [f32; 4]| (centre(r)[0] - at[0]).powi(2) + (centre(r)[1] - at[1]).powi(2);
        d(a).total_cmp(&d(b))
    }) {
        return Place::Over { fit: alone, x: r[0], y: r[1] };
    }
    if right >= below { Place::Beside { fit: right.clamp(0.1, 1.0), right: true } } else { Place::Beside { fit: below.clamp(0.1, 1.0), right: false } }
}

/// The rings' rects ((x0, y0, x1, y1), in the board's own points), whatever the stage, so
/// the callout doesn't move as the hint goes deeper.
pub fn marks(s: &App) -> Vec<[f32; 4]> {
    let Some(shown) = &s.hint else { return Vec::new() };
    let h = &shown.hint;
    h.numbers
        .iter()
        .copied()
        .chain(h.target.map(|t| t.cell))
        .chain(h.partner())
        .map(|i| cell_rect(s.game.w, i))
        .collect()
}

/// The size of the callout keyed `key` as last laid out (its height follows its text).
pub fn size(cx: &rune::Cx<App>, key: &str) -> [f32; 2] {
    cx.layout_size(key).unwrap_or([WIDTH, GUESS_H])
}

/// A cell's rect ((x0, y0, x1, y1), in the board's own points) on a field `w` cells wide,
/// with room for its ring.
pub(crate) fn cell_rect(w: usize, i: usize) -> [f32; 4] {
    let (x, y) = (WELL + (i % w) as f32 * PITCH, WELL + (i / w) as f32 * PITCH);
    [x - 2.0, y - 2.0, x + TILE + 2.0, y + TILE + 2.0]
}

/// A ring on cell `i` of a field `w` cells wide.
pub(crate) fn ring(key: String, w: usize, i: usize, color: rune::Color) -> El {
    let [x, y, ..] = cell_rect(w, i);
    el().key(key).absolute(x, y).size(TILE + 4.0, TILE + 4.0).radius(7.0).border(2.0, color).fill(color.alpha(0.12)).transition(Transition::opacity()).inert()
}

/// Rings on the numbers (from stage 2) and the cell (from stage 3), in the board's own
/// coordinates.
pub fn rings(s: &App, p: &Palette) -> Vec<El> {
    let Some(shown) = &s.hint else { return Vec::new() };
    let h = &shown.hint;
    let ring = |key: String, i: usize, color: rune::Color| ring(key, s.game.w, i, color);
    let mut v = Vec::new();
    if shown.stage >= 2 {
        v.extend(h.numbers.iter().map(|&n| ring(format!("hint-n{n}"), n, p.accent)));
    }
    if let Some(t) = h.target.filter(|_| shown.stage >= 3) {
        v.push(ring("hint-cell".into(), t.cell, verdict_color(t.verdict, p)));
        if let Some(other) = h.partner() {
            v.push(ring("hint-partner".into(), other, p.muted));
        }
    }
    v
}

pub(crate) fn verdict_color(v: Verdict, p: &Palette) -> rune::Color {
    match v {
        Verdict::Safe => p.numbers[2],
        Verdict::Mine | Verdict::WrongFlag | Verdict::Avoid => p.flag,
        Verdict::Guess => p.accent,
    }
}

pub(crate) fn line(t: impl Into<String>, font: rune::TextStyle, color: rune::Color) -> El {
    text(t.into()).font(font).color(color).wrap().w_full()
}

/// The technique's name and its grade.
pub(crate) fn technique(h: &Hint, lang: crate::strings::Lang, grade_key: &str, p: &Palette) -> El {
    let mut head = row().w_full().gap(8.0).center_y().child(text(words::name(h.technique, lang)).font(UI_BOLD).color(p.text));
    if let Some(g) = h.grade() {
        head = head.child(el().key(grade_key).px(6.0).h(18.0).radius(5.0).center().fill(p.accent.alpha(0.18)).child(text(words::grade(g, lang)).font(SMALL).color(p.accent)));
    }
    head
}

/// A callout's close button.
pub(crate) fn close(key: &str, label: &str, f: fn(&mut App), p: &Palette) -> El {
    el().key(key)
        .size(22.0, 22.0)
        .radius(6.0)
        .center()
        .role("button")
        .label(label.to_string())
        .focusable(true)
        .hover(p.tile_hover)
        .on_press(f)
        .child(text("×").font(UI_BOLD).color(p.muted))
}

/// A callout's surface, styled like the console.
pub(crate) fn surface(key: &str, label: &str, width: f32, body: El, p: &Palette) -> El {
    col()
        .key(key)
        .role("status")
        .label(label.to_string())
        .w(width)
        .padding(12.0)
        .radius(14.0)
        .gradient(p.console_top, p.console_bottom)
        .border(1.0, p.accent.alpha(0.35))
        .box_shadow(drop(12.0, 30.0, ca(0x000000, 0.6)))
        .child(body)
        .transition(Transition::opacity())
}

/// The callout: the technique and its grade, then each stage's words, Apply at the end.
pub fn view(s: &App, p: &Palette, width: f32) -> Option<El> {
    let shown = s.hint.as_ref()?;
    let (h, lang) = (&shown.hint, s.lang);
    let head = technique(h, lang, "hint-grade", p).child(spacer()).child(close("hint-close", s.tr(S::HintClose), |s: &mut App| s.close_hint(), p));
    // Each stage keeps the nudge and adds its line; the reason, which says it all, stands alone.
    let mut body = col().w_full().gap(6.0).child(head);
    if shown.stage < 4 {
        body = body.child(line(words::nudge(h, lang), UI, p.text).key("hint-nudge"));
    }
    if shown.stage == 2 && !h.numbers.is_empty() {
        body = body.child(line(words::look(h, lang), UI, p.muted).key("hint-look"));
    }
    if let (3, Some(v), Some(t)) = (shown.stage, words::verdict(h, lang), h.target) {
        body = body.child(line(v, UI_BOLD, verdict_color(t.verdict, p)).key("hint-verdict"));
    }
    if shown.stage >= 4 {
        let mut reason = col().key("hint-reason").w_full().gap(4.0);
        for (i, r) in words::reason(h, lang).into_iter().enumerate() {
            reason = reason.child(line(r, SMALL, p.text).key(format!("hint-step{i}")));
        }
        body = body.child(reason);
        if h.target.and_then(|t| t.verdict.action()).is_some() {
            let apply = well(el().key("hint-apply").h(30.0).px(14.0).radius(8.0))
                .center()
                .role("button")
                .label(s.tr(S::HintApply))
                .focusable(true)
                .press_scale(0.96)
                .hover(p.tile_hover)
                .on_press(|s: &mut App| s.apply_hint())
                .child(text(s.tr(S::HintApply)).font(UI_BOLD).color(p.text));
            body = body.child(row().w_full().child(spacer()).child(apply));
        }
    }
    Some(surface("hint-callout", s.tr(S::Hint), width, body, p))
}
