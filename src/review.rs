//! Review (复盘): a finished game rebuilt from its record and walked move by move, with
//! markers where it could have gone better, each with the reason a hint would give, and the
//! speed figures competitive players track. Pure: no drawing, and the record is only read.
//!
//! A marker sits at a move: the field it shows is the moment just before that move.
//! - A needless guess: the move opened a cell nothing proved while some cell could be proven
//!   safe. It shows that proof.
//! - A missed deduction: more than [`PAUSE_MS`] passed before the move while some hidden cell
//!   could be proven: safe, or a mine not yet flagged. It shows that proof, a safe cell's
//!   when there is one.
//! - A forced guess: the move opened a cell when none could be proven safe (as History
//!   counts them, the first click excluded), and whether it was a true 50/50. It shows how a
//!   strong player would have chosen.
//! - A chord saving: the move starts a run of left clicks, each opening a cell around one
//!   number whose mines were all flagged, which one chord on it would have opened.
//!
//! "Proven" means perfect logic from the open numbers and the count of mines left, as for
//! History's forced guesses; the player's flags are never facts (ADR 0002).

use crate::game::{Cover, Game, Mode};
use crate::hint::{self, Hint, Step, Target, Technique, Verdict};
use crate::record::{self, Act, Layout, Record};
use crate::solver::{Fact, Strength, View, deduce, fifty, play};

/// A pause longer than this while a cell could be proven is a missed deduction.
pub const PAUSE_MS: u32 = 5000;

/// What a marker points out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// An unproven cell opened while a safe cell could be proven.
    Needless,
    /// A pause of `pause_ms` while a safe cell, or a mine not yet flagged, could be proven.
    Missed { pause_ms: u32 },
    /// A cell opened when none could be proven safe; `fifty` if it was half of a true 50/50.
    Forced { fifty: bool },
    /// The `cells` around `number`, opened one left click each, that one chord on it would
    /// have opened, saving `saved` clicks.
    Chord { number: usize, cells: Vec<usize>, saved: usize },
}

/// A moment worth a look. What a hint would have said there is worked out when it is
/// selected ([`Review::hint`]), so a review opens quickly however many there are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    /// The move it is about: the field shows the moment before it.
    pub at: usize,
    pub kind: Kind,
}

/// The speed figures: 3BV (and how much of it was solved), clicks and time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub three_bv: u32,
    pub solved: u32,
    /// Every click: left, right and chords.
    pub clicks: u32,
    /// From the first click to the last.
    pub seconds: f64,
}

impl Stats {
    /// 3BV/s: the 3BV solved per second.
    pub fn per_second(&self) -> f64 {
        if self.seconds > 0.0 { self.solved as f64 / self.seconds } else { 0.0 }
    }

    /// Efficiency (IOE): the 3BV solved per click.
    pub fn efficiency(&self) -> f64 {
        if self.clicks > 0 { self.solved as f64 / self.clicks as f64 } else { 0.0 }
    }
}

/// A game under review: its markers and figures, and the field at the move being looked at.
#[derive(Clone)]
pub struct Review {
    /// The record reviewed, as read (a copy: History's is never touched).
    pub record: Record,
    pub markers: Vec<Marker>,
    pub stats: Stats,
    /// Moves played on the field shown: 0 is before the first click.
    pub at: usize,
    /// The field after `at` moves.
    pub game: Game,
    /// The marker selected, whose field is shown, and its reason.
    pub marker: Option<usize>,
    pub reason: Option<Hint>,
    /// Max Guess: the true 50/50s the field was built with.
    pub announced: Vec<(usize, usize)>,
    base: Game,
}

impl Review {
    /// A review of `record` on `base`, its field as laid ([`Record::base`], or laid again
    /// off the UI thread); a Max Guess field's true 50/50s are found again from it.
    pub fn of(record: &Record, base: Game) -> Review {
        let announced = match record.layout {
            Layout::Seeded { first, .. } if record.mode == Mode::MaxGuess => play::walk(&base, first).fifty_fifties,
            _ => Vec::new(),
        };
        Review::on(record, base, announced)
    }

    /// A review of `record` on `base`, its field as laid, and the true 50/50s a Max Guess
    /// field was built with (the game just played has both, so nothing is laid again).
    pub fn on(record: &Record, base: Game, announced: Vec<(usize, usize)>) -> Review {
        let markers = markers(record, &base);
        let end = rebuild(&base, record, record.moves.len());
        let (three_bv, solved) = record::three_bv(&end);
        let ms = record.moves.last().map_or(0, |m| m.ms);
        let seconds = if ms > 0 { ms as f64 / 1000.0 } else { record.seconds as f64 };
        let stats = Stats { three_bv, solved, clicks: record.moves.len() as u32, seconds };
        Review { record: record.clone(), markers, stats, at: record.moves.len(), game: end, marker: None, reason: None, announced, base }
    }

    /// How many moves the game has.
    pub fn moves(&self) -> usize {
        self.record.moves.len()
    }

    /// Shows the field after `at` moves, no marker selected.
    pub fn seek(&mut self, at: usize) {
        self.marker = None;
        self.reason = None;
        self.show(at.min(self.moves()));
    }

    /// `d` moves forward (or back).
    pub fn step(&mut self, d: isize) {
        self.seek(self.at.saturating_add_signed(d));
    }

    /// Selects marker `k`: shows its moment and works out its reason.
    pub fn select(&mut self, k: usize) {
        let Some(m) = self.markers.get(k) else { return };
        let at = m.at;
        self.show(at);
        self.reason = Some(self.hint(k));
        self.marker = Some(k);
    }

    /// Puts the selected marker's reason away, keeping its moment.
    pub fn unselect(&mut self) {
        self.marker = None;
        self.reason = None;
    }

    /// The selected marker and its reason.
    pub fn selected(&self) -> Option<(&Marker, &Hint)> {
        Some((self.markers.get(self.marker?)?, self.reason.as_ref()?))
    }

    /// What a hint would have said at marker `k`: for a needless guess or a missed
    /// deduction, the proof of a safe cell (of a mine to flag when no cell could be proven
    /// safe); for a forced guess, how a strong player chooses; for a chord saving, why the
    /// chord was safe.
    pub fn hint(&self, k: usize) -> Hint {
        let m = &self.markers[k];
        let g = rebuild(&self.base, &self.record, m.at);
        if let Kind::Chord { number, ref cells, .. } = m.kind {
            return chord_hint(&g, number, cells);
        }
        let moves = &self.record.moves;
        let last = m.at.checked_sub(1).map(|j| moves[j].cell);
        let p = Position::of(&g);
        if let Kind::Forced { .. } = m.kind {
            let classic_first = (self.record.mode == Mode::Classic).then(|| moves.iter().find(|m| m.act == Act::Open).map(|m| m.cell)).flatten();
            return hint::advise(&p.view, &p.flags(), last, &hint::Field { classic_first, announced: &self.announced });
        }
        let flags = if p.any_safe() { p.flags() } else { p.flagged_mines(&g) };
        hint::find(&p.view, &flags, last)
    }

    fn show(&mut self, at: usize) {
        if at != self.at {
            self.game = rebuild(&self.base, &self.record, at);
            self.at = at;
        }
    }
}

/// The field after the record's first `at` moves on `base`.
fn rebuild(base: &Game, record: &Record, at: usize) -> Game {
    let mut g = base.clone();
    for m in &record.moves[..at] {
        m.apply(&mut g);
    }
    g
}

/// What perfect logic proves on the field shown: each cell safe, a mine, or unknown.
fn proven(v: &View) -> Vec<Fact> {
    let mut facts = vec![Fact::Unknown; v.len()];
    loop {
        let proofs = deduce(v, &facts, Strength::Full);
        if proofs.is_empty() {
            return facts;
        }
        for p in proofs {
            facts[p.cell] = if p.mine { Fact::Mine } else { Fact::Safe };
        }
    }
}

/// A position as the player saw it, and what could be proven there.
struct Position {
    view: View,
    facts: Vec<Fact>,
}

impl Position {
    fn of(g: &Game) -> Position {
        let view = View::of(g);
        let facts = proven(&view);
        Position { view, facts }
    }

    fn any_safe(&self) -> bool {
        self.facts.contains(&Fact::Safe)
    }

    /// Whether some proven mine is still unflagged on `g`.
    fn any_unflagged_mine(&self, g: &Game) -> bool {
        self.facts.iter().zip(&g.cells).any(|(&f, c)| f == Fact::Mine && c.cover != Cover::Flag)
    }

    /// The proven mines the player has flagged on `g`, as flags: a hint then names a mine
    /// still to flag, never a player's wrong flag.
    fn flagged_mines(&self, g: &Game) -> Vec<bool> {
        self.facts.iter().zip(&g.cells).map(|(&f, c)| f == Fact::Mine && c.cover == Cover::Flag).collect()
    }

    /// The proven mines, as flags: a hint then names a safe cell, never a mine to flag or a
    /// player's wrong flag.
    fn flags(&self) -> Vec<bool> {
        self.facts.iter().map(|&f| f == Fact::Mine).collect()
    }
}

/// The cells a move would open on `g`.
fn opens(g: &Game, act: Act, cell: usize) -> Vec<usize> {
    let c = &g.cells;
    match act {
        Act::Open if matches!(c[cell].cover, Cover::Hidden | Cover::Question) => vec![cell],
        Act::Chord if c[cell].cover == Cover::Open && c[cell].adj > 0 => {
            let flags = g.neighbours(cell).filter(|&j| c[j].cover == Cover::Flag).count();
            if flags == c[cell].adj as usize { g.neighbours(cell).filter(|&j| matches!(c[j].cover, Cover::Hidden | Cover::Question)).collect() } else { Vec::new() }
        }
        _ => Vec::new(),
    }
}

/// An open number around `cell` whose mines are all flagged, rightly, and the hidden cells
/// around it a chord would open.
fn satisfied_around(g: &Game, cell: usize) -> Vec<(usize, Vec<usize>)> {
    let c = &g.cells;
    g.neighbours(cell)
        .filter(|&n| c[n].cover == Cover::Open && !c[n].mine && c[n].adj > 0)
        .filter(|&n| g.neighbours(n).all(|j| (c[j].cover == Cover::Flag) == c[j].mine))
        .map(|n| (n, g.neighbours(n).filter(|&j| matches!(c[j].cover, Cover::Hidden | Cover::Question)).collect::<Vec<_>>()))
        .filter(|(_, hidden)| hidden.len() >= 2)
        .collect()
}

/// The longest run of left clicks from move `k` that each open one of the hidden cells around
/// a satisfied number: (number, cells opened in order).
fn chord_run(g: &Game, record: &Record, k: usize) -> Option<(usize, Vec<usize>)> {
    let mut best: Option<(usize, Vec<usize>)> = None;
    for (n, hidden) in satisfied_around(g, record.moves[k].cell) {
        let mut g = g.clone();
        let mut run = Vec::new();
        for m in &record.moves[k..] {
            let open = matches!(g.cells[m.cell].cover, Cover::Hidden | Cover::Question);
            if m.act != Act::Open || !hidden.contains(&m.cell) || !open || g.over() {
                break;
            }
            run.push(m.cell);
            m.apply(&mut g);
        }
        if run.len() >= 2 && best.as_ref().is_none_or(|b| run.len() > b.1.len()) {
            best = Some((n, run));
        }
    }
    best
}

fn markers(record: &Record, base: &Game) -> Vec<Marker> {
    let moves = &record.moves;
    let mut out = Vec::new();
    let mut g = base.clone();
    // The position is worked out once per field shown: flags and clicks that open nothing
    // leave it as it was.
    let mut seen: Option<(usize, Position)> = None;
    let mut chord_until = 0;
    for (k, m) in moves.iter().enumerate() {
        if g.over() {
            break;
        }
        let pause = k.checked_sub(1).map_or(0, |j| m.ms.saturating_sub(moves[j].ms));
        let targets = opens(&g, m.act, m.cell);
        let looks = g.started && (!targets.is_empty() || pause > PAUSE_MS);
        if looks {
            if seen.as_ref().is_none_or(|(opened, _)| *opened != g.opened) {
                seen = Some((g.opened, Position::of(&g)));
            }
            let p = &seen.as_ref().expect("worked out").1;
            let unproven = targets.iter().any(|&t| p.facts[t] != Fact::Safe);
            let needless = unproven && p.any_safe();
            if needless {
                out.push(Marker { at: k, kind: Kind::Needless });
            } else if pause > PAUSE_MS && (p.any_safe() || p.any_unflagged_mine(&g)) {
                out.push(Marker { at: k, kind: Kind::Missed { pause_ms: pause } });
            }
            if unproven && !p.any_safe() {
                let pairs = fifty::fifty_fifties(&p.view, &vec![Fact::Unknown; p.view.len()]);
                let fifty = targets.iter().any(|&t| pairs.iter().any(|&(a, b)| a == t || b == t));
                out.push(Marker { at: k, kind: Kind::Forced { fifty } });
            }
        }
        if k >= chord_until
            && m.act == Act::Open
            && g.started
            && !targets.is_empty()
            && let Some((number, cells)) = chord_run(&g, record, k)
        {
            chord_until = k + cells.len();
            out.push(Marker { at: k, kind: Kind::Chord { number, saved: cells.len() - 1, cells } });
        }
        m.apply(&mut g);
    }
    out
}

/// The reason a chord works: the number has all its mines flagged, so the rest are safe.
fn chord_hint(g: &Game, number: usize, cells: &[usize]) -> Hint {
    let hidden = g.neighbours(number).filter(|&j| matches!(g.cells[j].cover, Cover::Hidden | Cover::Question)).collect();
    Hint {
        technique: Technique::Satisfied,
        numbers: vec![number],
        target: Some(Target { cell: cells[0], verdict: Verdict::Safe }),
        steps: vec![Step::Satisfied { at: number, n: g.cells[number].adj, cells: hidden }],
    }
}

#[cfg(test)]
mod tests;
