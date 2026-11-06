//! App state and everything a click, key or tick does to it.

use std::time::Duration;

use rune::{Key, KeyEvent, Live, TextEdit, Tx};

use crate::game::{Cover, Game, Level, Outcome, Status, clamp_custom};
use crate::icons::Mood;
use crate::storage::{self, Saved};
use crate::style::{PITCH, WELL, window_size};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Menu {
    Game,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dialog {
    Custom,
    BestTimes,
    /// A new best time for this level index: ask for a name.
    Record(usize),
    About,
    HowTo,
}

/// The board's pointer state, as the original tracks its buttons.
#[derive(Clone, Copy, Default, Debug)]
pub struct Press {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    /// The cell under the held pointer (None when outside the field).
    pub cell: Option<usize>,
}

impl Press {
    pub fn chord(&self) -> bool {
        self.middle || (self.left && self.right)
    }
}

/// What the effects need to know about the last few moves.
#[derive(Clone, Copy, Default, Debug)]
pub struct Moments {
    /// The deal of a new field (time).
    pub dealt: f64,
    /// The last opening: time and field point.
    pub opened: Option<(f64, [f32; 2])>,
    /// The explosion: time and field point.
    pub boom: Option<(f64, [f32; 2])>,
    pub won: Option<f64>,
}

pub struct App {
    pub game: Game,
    pub level: Level,
    pub saved: Saved,
    /// Bumped by every new field, so each deal mounts fresh tiles.
    pub field: u32,
    /// The app clock, from poll.
    pub now: f64,
    pub started_at: Option<f64>,
    pub seconds: u32,
    pub press: Press,
    pub face_down: bool,
    pub menu: Option<Menu>,
    pub dialog: Option<Dialog>,
    pub moments: Moments,
    pub reduced: bool,
    /// The hover selector's position on the field, and whether it shows.
    pub hover_at: Live<[f32; 2]>,
    pub hover_on: Live<f32>,
    pub hover_cell: Option<usize>,
    /// XYZZY, then Shift+Enter: the corner pixel tells mine from safe under the pointer.
    pub xyzzy: usize,
    pub cheat: bool,
    /// Where the face looks: toward the pointer, in face points.
    pub gaze: Live<[f32; 2]>,
    /// The console is knocked sideways for a moment by a blast.
    pub shake: bool,
    /// The last input (app clock): effects drift to a stop when nobody is playing.
    pub last_input: f64,
    /// The last win beat the level's best time.
    pub new_best: bool,
    /// Draw the app icon instead of the game (the `icon` scene).
    pub icon: bool,
    pub edit_w: TextEdit,
    pub edit_h: TextEdit,
    pub edit_m: TextEdit,
    pub edit_name: TextEdit,
}

impl App {
    pub fn new() -> App {
        let saved = storage::load();
        Self::with(saved, storage::seed())
    }

    pub fn with(saved: Saved, seed: u64) -> App {
        let level = saved.level;
        let (w, h, m) = level.dims().unwrap_or(saved.custom);
        App {
            game: Game::new(w, h, m, seed),
            level,
            saved,
            field: 0,
            now: 0.0,
            started_at: None,
            seconds: 0,
            press: Press::default(),
            face_down: false,
            menu: None,
            dialog: None,
            moments: Moments::default(),
            reduced: false,
            hover_at: Live::new([0.0, 0.0]),
            hover_on: Live::new(0.0),
            hover_cell: None,
            xyzzy: 0,
            cheat: false,
            icon: false,
            gaze: Live::new([0.0, 0.0]),
            shake: false,
            last_input: 0.0,
            new_best: false,
            edit_w: TextEdit::new(""),
            edit_h: TextEdit::new(""),
            edit_m: TextEdit::new(""),
            edit_name: TextEdit::new(""),
        }
    }

    pub fn dims(&self) -> (usize, usize, usize) {
        self.level.dims().unwrap_or(self.saved.custom)
    }

    pub fn new_game(&mut self) {
        let (w, h, m) = self.dims();
        self.game = Game::new(w, h, m, storage::seed() ^ self.field as u64);
        self.field += 1;
        self.started_at = None;
        self.seconds = 0;
        self.press = Press::default();
        self.shake = false;
        self.new_best = false;
        self.touch();
        self.moments = Moments { dealt: self.now, ..Moments::default() };
    }

    pub fn set_level(&mut self, level: Level, tx: &mut Tx) {
        self.level = level;
        self.saved.level = level;
        storage::save(&self.saved);
        self.new_game();
        tx.resize_window(window_size(self.game.w, self.game.h));
    }

    pub fn mood(&self) -> Mood {
        match self.game.status {
            Status::Won => Mood::Cool,
            Status::Lost => Mood::Dead,
            _ if self.press.left || self.press.middle => Mood::Oh,
            _ => Mood::Smile,
        }
    }

    /// The cell at a point in the board's own coordinates.
    pub fn cell_at(&self, x: f32, y: f32) -> Option<usize> {
        let (fx, fy) = (x - WELL, y - WELL);
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (cx, cy) = ((fx / PITCH) as usize, (fy / PITCH) as usize);
        (cx < self.game.w && cy < self.game.h).then(|| self.game.idx(cx, cy))
    }

    pub fn cell_center(&self, i: usize) -> [f32; 2] {
        let (x, y) = (i % self.game.w, i / self.game.w);
        [WELL + x as f32 * PITCH + PITCH / 2.0 - 1.0, WELL + y as f32 * PITCH + PITCH / 2.0 - 1.0]
    }

    fn start_clock(&mut self) {
        if self.started_at.is_none() && self.game.status == Status::Playing {
            self.started_at = Some(self.now);
            self.seconds = 1;
        }
    }

    /// A release on the board.
    fn release(&mut self, chord: bool, cell: Option<usize>) {
        let Some(i) = cell else { return };
        self.release_at(chord, i);
        // The selector hides over what just opened.
        let shown = !self.game.over() && self.game.cells[i].cover != Cover::Open;
        self.hover_on.set(if shown { 1.0 } else { 0.0 });
    }

    fn release_at(&mut self, chord: bool, i: usize) {
        let out = if chord { self.game.chord(i) } else { self.game.open(i) };
        self.start_clock();
        let at = self.cell_center(i);
        match out {
            Outcome::Nothing => {}
            Outcome::Opened(_) => self.moments.opened = Some((self.now, at)),
            Outcome::Lost(m) => {
                self.moments.boom = Some((self.now, self.cell_center(m)));
                self.started_at = None;
                self.shake = true;
            }
            Outcome::Won => {
                self.moments.opened = Some((self.now, at));
                self.moments.won = Some(self.now);
                self.started_at = None;
                self.record();
            }
        }
    }

    fn record(&mut self) {
        let Some(li) = self.level.index() else { return };
        if self.seconds < self.saved.best[li].0 {
            self.new_best = true;
            self.edit_name = TextEdit::new("Anonymous");
            self.dialog = Some(Dialog::Record(li));
        }
    }

    pub fn save_record(&mut self, li: usize) {
        let name = self.edit_name.text.trim().to_string();
        self.saved.best[li] = (self.seconds, if name.is_empty() { "Anonymous".into() } else { name });
        storage::save(&self.saved);
        self.dialog = Some(Dialog::BestTimes);
    }

    pub fn reset_scores(&mut self) {
        self.saved.best = Saved::default().best;
        storage::save(&self.saved);
    }

    pub fn toggle_marks(&mut self) {
        self.saved.marks = !self.saved.marks;
        if !self.saved.marks {
            self.game.clear_questions();
        }
        storage::save(&self.saved);
    }

    pub fn toggle_color(&mut self) {
        self.saved.color = !self.saved.color;
        storage::save(&self.saved);
    }

    pub fn open_custom(&mut self) {
        let (w, h, m) = self.saved.custom;
        self.edit_h = TextEdit::new(&h.to_string());
        self.edit_w = TextEdit::new(&w.to_string());
        self.edit_m = TextEdit::new(&m.to_string());
        self.dialog = Some(Dialog::Custom);
    }

    pub fn apply_custom(&mut self, tx: &mut Tx) {
        let n = |e: &TextEdit, d: usize| e.text.trim().parse::<usize>().unwrap_or(d);
        let (w0, h0, m0) = self.saved.custom;
        self.saved.custom = clamp_custom(n(&self.edit_w, w0), n(&self.edit_h, h0), n(&self.edit_m, m0));
        self.dialog = None;
        self.set_level(Level::Custom, tx);
    }

    pub fn on_board_drag(&mut self, d: &rune::Drag, button: rune::PointerButton, tx: &mut Tx) {
        let cell = self.cell_at(d.x, d.y);
        self.touch();
        match d.phase {
            rune::Phase::Down => {
                if self.game.over() {
                    return;
                }
                match button {
                    rune::PointerButton::Middle => self.press.middle = true,
                    _ => self.press.left = true,
                }
                self.press.cell = cell;
            }
            rune::Phase::Move => {
                if self.press.cell != cell {
                    self.press.cell = cell;
                } else {
                    tx.live_only();
                }
                self.hover(cell, tx);
            }
            rune::Phase::Up => {
                let chord = self.press.chord();
                let any = self.press.left || self.press.middle;
                self.press = Press { right: self.press.right && !chord, ..Press::default() };
                if any && !d.cancelled() {
                    self.release(chord, cell);
                    if self.shake {
                        tx.wake_after(Duration::from_millis(50));
                    }
                }
            }
        }
    }

    pub fn on_board_secondary(&mut self, x: f32, y: f32, _tx: &mut Tx) {
        self.touch();
        if self.game.over() {
            return;
        }
        if self.press.left {
            // Both buttons: the release will chord.
            self.press.right = true;
            return;
        }
        if let Some(i) = self.cell_at(x, y) {
            self.game.mark(i, self.saved.marks);
        }
    }

    /// Moves the selector (without re-running the view unless the cheat needs the cell).
    pub fn hover(&mut self, cell: Option<usize>, tx: &mut Tx) {
        self.touch();
        match cell {
            Some(i) => {
                let (x, y) = (i % self.game.w, i / self.game.w);
                // The face sits centred above the field: look from there toward the cell.
                let bw = self.game.w as f32 * PITCH - crate::style::GAP + 2.0 * WELL;
                let c = self.cell_center(i);
                let (dx, dy) = (c[0] - bw / 2.0, c[1] + 12.0 + crate::style::HEADER_H / 2.0);
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                let k = (len / 120.0).min(1.0) * 2.4;
                self.gaze.set([dx / len * k, dy / len * k]);
                self.hover_at.set([WELL + x as f32 * PITCH, WELL + y as f32 * PITCH]);
                self.hover_on.set(if self.game.cells[i].cover == Cover::Open || self.game.over() { 0.0 } else { 1.0 });
            }
            None => {
                self.hover_on.set(0.0);
                self.gaze.set([0.0, 0.0]);
            }
        }
        if self.cheat && self.hover_cell != cell {
            self.hover_cell = cell;
        } else {
            self.hover_cell = cell;
            tx.live_only();
        }
    }

    /// Marks input; the next tick stamps it with the clock (handlers may run on a stale `now`).
    pub fn touch(&mut self) {
        self.last_input = f64::MAX;
    }

    pub fn tick(&mut self, now: f64, tx: &mut Tx) -> bool {
        self.now = now;
        if self.last_input == f64::MAX {
            self.last_input = now;
        }
        if self.shake && self.moments.boom.is_some_and(|(t, _)| now - t > 0.04) {
            self.shake = false;
            return true;
        }
        let Some(t0) = self.started_at else { return false };
        if self.game.status != Status::Playing {
            return false;
        }
        let secs = ((now - t0) as u32 + 1).min(999);
        let next = 1.0 - (now - t0).fract();
        tx.wake_after(Duration::from_secs_f64(next.max(0.01)));
        if secs != self.seconds {
            self.seconds = secs;
            return true;
        }
        false
    }

    pub fn key(&mut self, e: &KeyEvent, tx: &mut Tx) -> bool {
        self.touch();
        match &e.key {
            Key::Function(2) => {
                self.new_game();
                true
            }
            Key::Escape if self.menu.is_some() => {
                self.menu = None;
                true
            }
            Key::Text(t) if !e.mods.cmd && !e.mods.ctrl => {
                let word = "xyzzy";
                let c = t.to_ascii_lowercase();
                if word[self.xyzzy..].starts_with(&c) {
                    self.xyzzy += c.len();
                } else {
                    self.xyzzy = if word.starts_with(&c) { c.len() } else { 0 };
                }
                false
            }
            Key::Enter if e.mods.shift && self.xyzzy >= 5 => {
                self.cheat = !self.cheat;
                self.xyzzy = 0;
                true
            }
            Key::Command('n') => {
                self.new_game();
                true
            }
            Key::Command('q') | Key::Command('w') => {
                tx.quit();
                true
            }
            _ => false,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_testing_maps_points_to_cells() {
        let a = App::with(Saved::default(), 3);
        assert_eq!(a.cell_at(WELL + 1.0, WELL + 1.0), Some(0));
        assert_eq!(a.cell_at(WELL + PITCH * 8.5, WELL + PITCH * 8.5), Some(80));
        assert_eq!(a.cell_at(WELL + PITCH * 9.5, WELL), None);
        assert_eq!(a.cell_at(1.0, 1.0), None);
    }
}
