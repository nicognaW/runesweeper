//! The Review on screen: opened from the result banner (or the tab it leaves on the console)
//! and from any History record; the timeline under the field, with its markers; the reason
//! callout a selected marker shows beside the field, worded as a hint is; and the rings on
//! the field at that moment. Nothing here writes to a record.

use rune::{El, Phase, col, el, row, spacer, text};

use crate::app::App;
use crate::callout::{self, line, ring};
use crate::generating::Generating;
use crate::hint::words;
use crate::record::Layout;
use crate::review::{Kind, Marker, Review};
use crate::strings::S;
use crate::style::*;
use crate::toolbar::well;

impl App {
    /// Reviews History's record `i` on the mines it keeps. One that kept only its seed and
    /// first click has its field laid again off the UI thread, and the review opens when it
    /// lands ([`App::tick`]); the window never waits for it.
    pub fn open_review(&mut self, i: usize) {
        let Some(r) = self.history.get(i) else { return };
        self.panel = None;
        match (r.base(), &r.layout) {
            (Some(base), _) => {
                self.review_laying = None;
                self.review = Some(Review::of(r, base));
            }
            (None, &Layout::Seeded { seed, first, .. }) => {
                self.review = None;
                self.review_laying = Some((i, Generating::start(self.lay, r.mode, r.dims, seed, first, self.waker.clone())));
            }
            (None, Layout::Mines(_)) => {}
        }
    }

    /// Reviews the game just played: its record is History's last, its field the one in
    /// play, so nothing is laid again.
    pub fn review_last(&mut self) {
        let Some(r) = self.history.last().filter(|_| self.game.over()) else { return };
        let base = match r.layout {
            Layout::Seeded { .. } => self.game.as_laid(),
            Layout::Mines(_) => r.base().expect("Classic keeps its mines"),
        };
        self.review = Some(Review::on(r, base, self.fifty_fifties.clone()));
        self.panel = None;
    }

    pub fn leave_review(&mut self) {
        self.review = None;
        self.review_laying = None;
    }

    /// `d` moves forward (or back) in the review.
    pub fn review_step(&mut self, d: isize) {
        if let Some(r) = &mut self.review {
            r.step(d);
        }
    }

    pub fn review_seek(&mut self, at: usize) {
        if let Some(r) = &mut self.review {
            r.seek(at);
        }
    }

    /// Shows marker `k`'s moment and its reason.
    pub fn review_select(&mut self, k: usize) {
        if let Some(r) = &mut self.review {
            r.select(k);
        }
    }

    /// Puts the reason away, keeping the moment.
    pub fn review_unselect(&mut self) {
        if let Some(r) = &mut self.review {
            r.unselect();
        }
    }

    /// Whether the game just played can be reviewed from the console.
    fn reviewable(&self) -> bool {
        self.review.is_none() && self.game.over() && self.game.started && !self.history.is_empty()
    }
}

/// The Review action: the result banner's, and the tab it leaves.
pub fn review_action(s: &App, p: &Palette, key: &str) -> Option<El> {
    s.reviewable().then(|| {
        well(el().key(key).h(24.0).px(9.0).radius(8.0))
            .center()
            .role("button")
            .label(s.tr(S::Review))
            .focusable(true)
            .press_scale(0.96)
            .hover(p.tile_hover)
            .on_press(|s: &mut App| s.review_last())
            .child(text(s.tr(S::Review)).font(SMALL).color(p.text))
    })
}

/// Once the banner has left, its Review action stays as a tab on the console's bottom rim,
/// covering no cell.
pub fn tab(s: &App, p: &Palette, console_w: f32, console_h: f32) -> Option<El> {
    if !s.banner_done {
        return None;
    }
    let pill = review_action(s, p, "review-tab")?.h(22.0).radius(11.0).fill(p.led_bg).border(1.0, ca(0xffffff, 0.08)).box_shadow(drop(2.0, 6.0, ca(0x000000, 0.5)));
    Some(row().absolute(0.0, console_h - 11.0).size(console_w, 22.0).center().child(pill))
}

/// What the timer showed at the review's moment: the time of the last move shown.
pub fn seconds(r: &Review) -> u32 {
    match r.at {
        0 => 0,
        at if at == r.moves() => r.record.seconds,
        at => (r.record.moves[at - 1].ms / 1000 + 1).min(999),
    }
}

/// Seconds since the first click at the review's moment, to a tenth.
fn moment(r: &Review) -> f64 {
    r.at.checked_sub(1).map_or(0.0, |k| r.record.moves[k].ms as f64 / 1000.0)
}

fn kind_name(k: &Kind) -> S {
    match k {
        Kind::Needless => S::MarkNeedless,
        Kind::Missed { .. } => S::MarkMissed,
        Kind::Forced { .. } => S::MarkForced,
        Kind::Chord { .. } => S::MarkChord,
    }
}

fn kind_color(k: &Kind, p: &Palette) -> rune::Color {
    match k {
        Kind::Needless => p.flag,
        Kind::Missed { .. } => p.accent,
        Kind::Forced { .. } => p.win,
        Kind::Chord { .. } => p.numbers[2],
    }
}

/// The timeline's height, until it has been laid out once.
pub const BAR_H: f32 = 112.0;
/// Between the console and the timeline.
pub const BAR_GAP: f32 = 12.0;
const BUTTON: f32 = 30.0;

fn button(key: &str, label: &str, glyph: &str, f: fn(&mut App), p: &Palette) -> El {
    well(el().key(key).size(BUTTON, 26.0).radius(8.0))
        .center()
        .role("button")
        .label(label.to_string())
        .focusable(true)
        .press_scale(0.94)
        .hover(p.tile_hover)
        .on_press(f)
        .child(text(glyph.to_string()).font(UI_BOLD).color(p.text))
}

/// Where the moment `at` of `moves` sits along a track `w` wide.
fn x_of(at: usize, moves: usize, w: f32) -> f32 {
    7.0 + (w - 14.0) * if moves == 0 { 0.0 } else { at as f32 / moves as f32 }
}

/// The track: a marker per moment worth a look, the moments so far lit, and a playhead. A
/// drag along it scrubs through the moves.
fn track(s: &App, r: &Review, w: f32, p: &Palette) -> El {
    let n = r.moves();
    let mut t = well(el().key("review-track").size(w, 26.0).radius(8.0))
        .role("slider")
        .label(s.tr(S::ReviewTimeline))
        .on_drag(move |s: &mut App, d, _tx| {
            if matches!(d.phase, Phase::Down | Phase::Move) {
                let at = (((d.x - 7.0) / (w - 14.0)).clamp(0.0, 1.0) * n as f32).round() as usize;
                s.review_seek(at);
            }
        })
        .child(el().absolute(7.0, 12.0).size(w - 14.0, 2.0).radius(1.0).fill(ca(0xffffff, 0.1)).inert())
        .child(el().absolute(7.0, 12.0).size(x_of(r.at, n, w) - 7.0, 2.0).radius(1.0).fill(p.accent.alpha(0.7)).inert());
    for (k, m) in r.markers.iter().enumerate() {
        let on = r.marker == Some(k);
        let color = kind_color(&m.kind, p);
        let dot = el()
            .key(format!("review-marker-{k}"))
            .absolute(x_of(m.at, n, w) - 7.0, 6.0)
            .size(14.0, 14.0)
            .radius(7.0)
            .fill(color)
            .border(if on { 2.0 } else { 1.0 }, if on { p.text } else { ca(0x000000, 0.5) })
            .role("button")
            .label(s.tr(kind_name(&m.kind)))
            .checked(on)
            .focusable(true)
            .press_scale(0.9)
            .on_press(move |s: &mut App| s.review_select(k));
        t = t.child(dot);
    }
    t.child(el().key("review-head").absolute(x_of(r.at, n, w) - 1.0, 3.0).size(2.0, 20.0).radius(1.0).fill(p.text).inert())
}

/// The timeline under the field: the speed figures, the moves with their markers, and where
/// the field is.
pub fn bar(s: &App, p: &Palette, width: f32) -> Option<El> {
    let r = s.review.as_ref()?;
    let st = r.stats;
    let bv = if st.solved == st.three_bv { st.three_bv.to_string() } else { format!("{}/{}", st.solved, st.three_bv) };
    let stats = s.trf(S::ReviewStats, &[&bv, &format!("{:.2}", st.per_second()), &format!("{:.2}", st.efficiency())]);
    let at = format!("{} · {}", s.trf(S::ReviewMove, &[&r.at, &r.moves()]), s.trf(S::Seconds, &[&format!("{:.1}", moment(r))]));
    let head = row()
        .w_full()
        .gap(8.0)
        .center_y()
        .child(text(s.tr(S::Review)).font(UI_BOLD).color(p.text))
        .child(text(at).key("review-move").font(SMALL).color(p.muted))
        .child(spacer())
        .child(callout::close("review-close", s.tr(S::ReviewLeave), |s: &mut App| s.leave_review(), p));
    let track_w = width - 24.0 - 2.0 * BUTTON - 12.0;
    let moves = row()
        .w_full()
        .gap(6.0)
        .center_y()
        .child(button("review-prev", s.tr(S::ReviewPrev), "‹", |s: &mut App| s.review_step(-1), p))
        .child(track(s, r, track_w, p))
        .child(button("review-next", s.tr(S::ReviewNext), "›", |s: &mut App| s.review_step(1), p));
    let mut foot = row().w_full().gap(8.0).center_y().child(text(stats).key("review-stats").font(SMALL).color(p.text));
    if r.markers.is_empty() {
        foot = foot.child(spacer()).child(text(s.tr(S::ReviewClean)).key("review-clean").font(SMALL).color(p.muted));
    }
    Some(
        col()
            .key("review")
            .role("region")
            .label(s.tr(S::Review))
            .w(width)
            .padding(12.0)
            .gap(8.0)
            .radius(16.0)
            .gradient(p.console_top, p.console_bottom)
            .border(1.0, ca(0xffffff, 0.07))
            .box_shadow(drop(16.0, 40.0, ca(0x000000, 0.65)))
            .child(head)
            .child(moves)
            .child(foot),
    )
}

/// What the marker says happened there, in one line.
fn summary(s: &App, m: &Marker) -> String {
    match m.kind {
        Kind::Needless => s.tr(S::SumNeedless).to_string(),
        Kind::Missed { pause_ms } => s.trf(S::SumMissed, &[&s.trf(S::Seconds, &[&format!("{:.1}", pause_ms as f64 / 1000.0)])]),
        Kind::Forced { fifty: true } => s.tr(S::SumForcedFifty).to_string(),
        Kind::Forced { fifty: false } => s.tr(S::SumForced).to_string(),
        Kind::Chord { .. } => String::new(),
    }
}

/// The selected marker's callout: what happened, then the reason as a hint gives it, in the
/// hint's words.
pub fn view(s: &App, p: &Palette, width: f32) -> Option<El> {
    let r = s.review.as_ref()?;
    let (m, hint) = r.selected()?;
    let lang = s.lang;
    let head = row()
        .w_full()
        .gap(8.0)
        .center_y()
        .child(text(s.tr(kind_name(&m.kind))).font(UI_BOLD).color(kind_color(&m.kind, p)))
        .child(spacer())
        .child(callout::close("review-callout-close", s.tr(S::HintClose), |s: &mut App| s.review_unselect(), p));
    let mut body = col().w_full().gap(6.0).child(head);
    let mut reason = col().key("review-reason").w_full().gap(4.0);
    if let Kind::Chord { number, saved, .. } = m.kind {
        let n = r.game.cells[number].adj;
        let mines = if n == 1 { s.tr(S::CountMine1).to_string() } else { s.trf(S::CountMines, &[&n]) };
        let words = if saved == 1 { s.trf(S::ChordReason1, &[&n, &mines]) } else { s.trf(S::ChordReason, &[&n, &mines, &saved]) };
        reason = reason.child(line(words, SMALL, p.text).key("review-step0"));
    } else {
        body = body.child(line(summary(s, m), UI, p.text).key("review-summary")).child(callout::technique(hint, lang, "review-grade", p));
        for (i, l) in words::reason(hint, lang).into_iter().enumerate() {
            reason = reason.child(line(l, SMALL, p.text).key(format!("review-step{i}")));
        }
    }
    Some(callout::surface("review-callout", s.tr(S::Review), width, body.child(reason), p))
}

/// The cells the selected marker talks about: its numbers, its cell (and a 50/50's other
/// half), the cells a chord would have opened, and the cell the move clicked.
fn ringed(r: &Review) -> Vec<(String, usize, Ring)> {
    let Some((m, hint)) = r.selected() else { return Vec::new() };
    let mut v: Vec<(String, usize, Ring)> = hint.numbers.iter().map(|&n| (format!("review-n{n}"), n, Ring::Number)).collect();
    if let Some(t) = hint.target {
        v.push(("review-cell".into(), t.cell, Ring::Cell(t.verdict)));
    }
    if let Some(o) = hint.partner() {
        v.push(("review-partner".into(), o, Ring::Other));
    }
    let clicked = r.record.moves[m.at].cell;
    match &m.kind {
        Kind::Chord { cells, .. } => v.extend(cells.iter().skip(1).map(|&c| (format!("review-run{c}"), c, Ring::Cell(crate::hint::Verdict::Safe)))),
        _ if hint.target.is_some_and(|t| t.cell == clicked) => {}
        _ => v.push(("review-move-cell".into(), clicked, Ring::Move)),
    }
    v
}

#[derive(Clone, Copy)]
enum Ring {
    Number,
    Cell(crate::hint::Verdict),
    Other,
    Move,
}

/// The rings on the review's field, in the board's own coordinates.
pub fn rings(s: &App, p: &Palette) -> Vec<El> {
    let Some(r) = &s.review else { return Vec::new() };
    ringed(r)
        .into_iter()
        .map(|(key, i, kind)| {
            let color = match kind {
                Ring::Number => p.accent,
                Ring::Cell(v) => callout::verdict_color(v, p),
                Ring::Other => p.muted,
                Ring::Move => p.text,
            };
            ring(key, r.game.w, i, color)
        })
        .collect()
}

/// The rings' rects, for placing the callout clear of them.
pub fn marks(s: &App) -> Vec<[f32; 4]> {
    let Some(r) = &s.review else { return Vec::new() };
    ringed(r).into_iter().map(|(_, i, _)| callout::cell_rect(r.game.w, i)).collect()
}
