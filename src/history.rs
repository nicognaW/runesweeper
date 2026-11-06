//! The History panel: game records newest first, filtered by Mode and Level, with the fastest
//! win for the pair on top.

use rune::{Align, El, Icon, ScrollSpec, col, el, row, spacer, text};

use crate::app::App;
use crate::game::{Level, Mode};
use crate::record::{self, End, Record};
use crate::strings::S;
use crate::style::*;
use crate::toolbar::{LEVELS, choice, level_key, sizes, surface, well};

pub static HISTORY: Icon = Icon {
    name: "runesweeper-history",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="5.6" stroke="currentColor" stroke-width="1.4" fill="none"/><path d="M8 4.8V8L10.2 9.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" fill="none"/>"#,
};

/// The most rows the list draws; older games stay in History, past the end of the list.
const MAX_ROWS: usize = 200;
const ROW_H: f32 = 44.0;

/// The records the filter shows, newest first.
pub fn shown(s: &App) -> Vec<(usize, &Record)> {
    let (mode, level) = s.history_filter;
    s.history.iter().enumerate().rev().filter(|(_, r)| r.mode == mode && r.level() == level).collect()
}

/// The fastest win for the filter's Mode and Level; Custom has one per size, smallest first.
pub fn bests(s: &App) -> Vec<&Record> {
    let (mode, level) = s.history_filter;
    let mut dims: Vec<_> = s.history.iter().filter(|r| r.mode == mode && r.level() == level).map(|r| r.dims).collect();
    dims.sort_by_key(|&(w, h, m)| (w * h, m, w));
    dims.dedup();
    dims.into_iter().filter_map(|d| record::fastest(&s.history, mode, d)).collect()
}

/// A record's date and time, `YYYY-MM-DD HH:MM`, `offset` minutes from UTC.
pub fn date(ms: u64, offset: i64) -> String {
    let mins = (ms / 60_000) as i64 + offset;
    let (days, m) = (mins.div_euclid(1440), mins.rem_euclid(1440));
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + (month <= 2) as i64;
    format!("{year:04}-{month:02}-{day:02} {:02}:{:02}", m / 60, m % 60)
}

/// One segmented filter row: a segment per option, the current one lit.
fn segments<T: Copy + PartialEq + 'static>(key: &str, label: &str, options: Vec<(T, String, String)>, on: T, pick: fn(&mut App, T), p: &Palette) -> El {
    let mut r = row().key(key).role("radiogroup").label(label.to_string()).w_full().h(36.0).px(4.0).gap(4.0).center_y();
    for (v, k, name) in options {
        let lit = v == on;
        let mut o = el()
            .key(k)
            .h(28.0)
            .grow(1.0)
            .radius(7.0)
            .center()
            .role("radio")
            .label(name.clone())
            .checked(lit)
            .focusable(true)
            .press_scale(0.96)
            .on_press(move |s: &mut App| pick(s, v))
            .child(text(name).font(SMALL).color(if lit { p.text } else { p.muted }));
        o = if lit { o.fill(p.accent.alpha(0.22)).border(1.0, p.accent.alpha(0.8)) } else { o.hover(p.tile_hover) };
        r = r.child(o);
    }
    well(r.radius(10.0))
}

fn seconds(s: &App, r: &Record) -> String {
    s.trf(S::Seconds, &[&r.seconds])
}

/// The line under a record: 3BV and clicks, forced guesses if any, Custom's size.
fn detail(s: &App, r: &Record) -> String {
    let mut parts = vec![s.trf(S::Bv3Clicks, &[&r.three_bv_solved, &r.three_bv, &(r.left + r.right + r.chords)])];
    if r.forced > 0 {
        parts.push(s.trf(S::ForcedGuesses, &[&r.forced, &r.survived]));
    }
    if r.assisted() {
        parts.push(if r.hints == 1 { s.tr(S::Assisted1).to_string() } else { s.trf(S::Assisted, &[&r.hints]) });
    }
    if r.level() == Level::Custom {
        parts.push(sizes(r.dims));
    }
    parts.join(" · ")
}

fn best_row(s: &App, r: &Record, p: &Palette) -> El {
    let (w, h, m) = r.dims;
    let mut line = row().w_full().gap(8.0).center_y().child(text(s.tr(S::FastestWin)).font(UI_BOLD).color(p.win)).child(text(seconds(s, r)).font(UI_BOLD).color(p.text));
    if r.level() == Level::Custom {
        line = line.child(text(sizes(r.dims)).font(SMALL).color(p.muted));
    }
    choice(&format!("history-best-{w}x{h}x{m}"), false, p).border(1.0, p.win.alpha(0.6)).child(line.child(spacer()).child(text(date(r.at, s.utc_offset)).font(SMALL).color(p.muted)))
}

fn record_row(s: &App, i: usize, r: &Record, p: &Palette) -> El {
    let (end, tint) = match r.end {
        End::Won => (S::Won, p.win),
        End::Lost => (S::LostGame, if s.saved.color { c(0xff5c6c) } else { p.text }),
        End::Abandoned => (S::Abandoned, p.muted),
    };
    col()
        .key(format!("history-row-{i}"))
        .w_full()
        .h(ROW_H)
        .px(10.0)
        .gap(2.0)
        .justify_center()
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(text(s.tr(end)).font(UI_BOLD).color(tint))
                .child(text(seconds(s, r)).font(UI).color(p.text))
                .child(spacer())
                .child(text(date(r.at, s.utc_offset)).font(SMALL).color(p.muted)),
        )
        .child(text(detail(s, r)).font(SMALL).color(p.muted))
}

pub fn panel(s: &App, p: &Palette) -> El {
    let (mode, level) = s.history_filter;
    let modes = Mode::ALL.into_iter().map(|m| (m, format!("history-mode-{}", m.tag()), s.tr(S::mode(m)).to_string())).collect();
    let levels = LEVELS.into_iter().map(|l| (l, format!("history-level-{}", level_key(l).trim_start_matches("pick-")), s.tr(S::level(l)).to_string())).collect();
    let mut body = col()
        .w(270.0)
        .gap(6.0)
        .child(segments("history-modes", s.tr(S::ModeRow), modes, mode, |s, m| s.history_filter.0 = m, p))
        .child(segments("history-levels", s.tr(S::LevelRow), levels, level, |s, l| s.history_filter.1 = l, p))
        .children(bests(s).into_iter().map(|r| best_row(s, r, p)));
    let rows: Vec<El> = shown(s).into_iter().take(MAX_ROWS).map(|(i, r)| record_row(s, i, r, p)).collect();
    body = if rows.is_empty() {
        body.child(el().key("history-empty").w_full().h(ROW_H).center().child(text(s.tr(S::NoGames)).font(UI).color(p.muted)))
    } else {
        let h = (rows.len() as f32 * ROW_H).min(4.5 * ROW_H);
        body.child(well(col().key("history-list").w_full().h(h).radius(10.0).scroll(ScrollSpec { stick_bottom: false }).scroll_edge_fade(16.0).children(rows)))
    };
    surface("history", s.tr(S::History), "history-button", Align::End, body, p)
}

#[cfg(test)]
mod tests {
    use super::date;

    #[test]
    fn dates_read_as_the_calendar_does() {
        assert_eq!(date(0, 0), "1970-01-01 00:00");
        assert_eq!(date(1_700_000_000_000, 0), "2023-11-14 22:13");
        assert_eq!(date(1_700_000_000_000, 8 * 60), "2023-11-15 06:13", "Beijing time");
        assert_eq!(date(951_782_400_000, 0), "2000-02-29 00:00", "a leap day");
        assert_eq!(date(0, -60), "1969-12-31 23:00");
    }
}
