//! App state and everything a click, key or tick does to it.

use std::time::Duration;

use rune::{Key, KeyEvent, Live, Modifier, TextEdit, Tx, Waker};

use crate::callout::Shown;
use crate::game::{Cover, Game, Level, Mode, Outcome, Rng, Status, clamp_custom};
use crate::generate::{self, Stream};
use crate::generating::{self, Generating, Lay};
use crate::icons::Mood;
use crate::record::{self, Act, End, Layout, Move, Record};
use crate::review::Review;
use crate::solver::play;
use crate::storage::{self, Saved};
use crate::strings::{Lang, S};
use crate::style::{PITCH, WELL, window_size};

/// The panels the toolbar opens, one at a time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Panel {
    /// The Level picker, with Custom's sizes inline.
    Picker,
    /// Game records, filtered by Mode and Level.
    History,
    Settings,
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
    /// The explosion: time and field point.
    pub boom: Option<(f64, [f32; 2])>,
    pub won: Option<f64>,
}

pub struct App {
    pub game: Game,
    pub mode: Mode,
    pub level: Level,
    /// Classic's `rand()` stream, one for the whole session as Windows seeds once.
    pub rng: Rng,
    /// Where the new modes' seeds come from.
    pub seeds: Stream,
    /// This field's seed (the new modes): with the mode, level and first click it names the
    /// field.
    pub seed: u64,
    /// The new modes' field has no mines yet: the first click lays them.
    pub unlaid: bool,
    /// The field was just laid: the clock starts at the next poll, so laying never counts.
    pub clock_waits: bool,
    /// The field the first click asked for, being generated off the UI thread. The face
    /// thinks and the field takes no input until it lands.
    pub generating: Option<Generating>,
    /// Max Guess: the true 50/50s the laid field forces on a perfect player, each as its pair
    /// of cells (empty in the other modes, and before the field is laid).
    pub fifty_fifties: Vec<(usize, usize)>,
    /// When that generation started, on the poll clock (stamped by the first poll after the
    /// click, as the click's own clock may be stale).
    pub generating_since: Option<f64>,
    /// When the band of light over the hidden tiles started, 600 ms into a generation that is
    /// still running (on the poll clock).
    pub sweep_from: Option<f64>,
    /// History's record `.0`, whose review waits for its field, laid again off the UI thread:
    /// the record keeps only its seed and first click. The face thinks meanwhile.
    pub review_laying: Option<(usize, Generating)>,
    /// What lays the new modes' fields: [`generate::lay`] (tests slow it down).
    pub lay: Lay,
    /// Wakes the window when a field lands (from `AppSpec::setup`; none headless).
    pub waker: Option<Waker>,
    pub saved: Saved,
    /// The language every string is in: the one picked in Settings, else the system's.
    pub lang: Lang,
    /// Bumped by every new field, so each deal mounts fresh tiles.
    pub field: u32,
    /// The app clock, from poll.
    pub now: f64,
    pub started_at: Option<f64>,
    pub seconds: u32,
    pub press: Press,
    pub face_down: bool,
    /// The open panel, if any.
    pub panel: Option<Panel>,
    pub moments: Moments,
    pub reduced: bool,
    /// The hover selector's position on the field, and whether it shows.
    pub hover_at: Live<[f32; 2]>,
    pub hover_on: Live<f32>,
    pub hover_cell: Option<usize>,
    /// XYZZY, then Shift: the corner pixel tells mine from safe under the pointer.
    pub xyzzy: usize,
    pub cheat: bool,
    /// The corner pixel: the last square the pointer was over is safe. It keeps its colour
    /// off the field, as Windows' does.
    pub pixel_safe: bool,
    /// Touch screens have no right button: while on, a tap flags (a tap on a number chords).
    pub flag_mode: bool,
    /// The toolbar offers the flag toggle (the web build, which phones use).
    pub touch: bool,
    /// Whether this field deals its tiles in (not when a key asked for it).
    pub deal: bool,
    /// The last win beat the fastest win for its mode and level, or was the first.
    pub new_best: bool,
    /// Every game record, oldest first.
    pub history: Vec<Record>,
    /// The game in play's record, click by click. It joins History when the game is won,
    /// lost, or left after its first click.
    pub record: Record,
    /// When the field's first click came, on the app clock.
    record_t0: Option<f64>,
    /// The Mode and Level History shows; the current ones each time it opens.
    pub history_filter: (Mode, Level),
    /// Minutes from UTC to the player's clock, for History's dates.
    pub utc_offset: i64,
    /// The Hint on screen and how far into it the player has stepped. Any move clears it.
    pub hint: Option<Shown>,
    /// A game being reviewed: its field is drawn instead of the game in play, which waits,
    /// untouched, and takes no input until the review is left.
    pub review: Option<Review>,
    /// The result banner has had its moment (it leaves on a timer, so its wait draws nothing).
    pub banner_done: bool,
    /// The shockwave has run; its full-window backdrop copy can go.
    pub shock_done: bool,
    /// Draw the app icon instead of the game (the `icon` scene).
    pub icon: bool,
    /// Custom's sizes as typed in the picker.
    pub edit_w: TextEdit,
    pub edit_h: TextEdit,
    pub edit_m: TextEdit,
}

impl App {
    pub fn new() -> App {
        let saved = storage::load();
        let lang = saved.lang.unwrap_or_else(crate::locale::system);
        let mut a = Self::in_lang(saved, storage::seed(), lang);
        a.history = storage::load_history();
        // A game the last run left in play (the window or tab closed on it) was abandoned.
        if let Some(r) = storage::take_draft() {
            storage::append_record(&r);
            a.history.push(r);
        }
        a
    }

    /// The app in the saved language, English if none was picked (headless runs and tests
    /// don't depend on the machine's).
    pub fn with(saved: Saved, seed: u32) -> App {
        let lang = saved.lang.unwrap_or_default();
        Self::in_lang(saved, seed, lang)
    }

    fn in_lang(saved: Saved, seed: u32, lang: Lang) -> App {
        shape_in(lang);
        let level = saved.level;
        let mut a = App {
            lang,
            game: Game::unlaid(9, 9, 10),
            mode: saved.mode,
            level,
            rng: Rng::new(seed),
            seeds: Stream::new(seed as u64),
            seed: 0,
            unlaid: false,
            clock_waits: false,
            generating: None,
            review_laying: None,
            fifty_fifties: Vec::new(),
            generating_since: None,
            sweep_from: None,
            lay: generate::lay,
            waker: None,
            saved,
            field: 0,
            now: 0.0,
            started_at: None,
            seconds: 0,
            press: Press::default(),
            face_down: false,
            panel: None,
            moments: Moments::default(),
            reduced: false,
            hover_at: Live::new([0.0, 0.0]),
            hover_on: Live::new(0.0),
            hover_cell: None,
            xyzzy: 0,
            cheat: false,
            pixel_safe: false,
            icon: false,
            deal: true,
            flag_mode: false,
            touch: cfg!(target_arch = "wasm32"),
            new_best: false,
            history: Vec::new(),
            record: Record::new(Mode::Classic, (0, 0, 0)),
            record_t0: None,
            history_filter: (Mode::Classic, level),
            utc_offset: storage::utc_offset_minutes(),
            hint: None,
            review: None,
            banner_done: false,
            shock_done: false,
            edit_w: TextEdit::new(""),
            edit_h: TextEdit::new(""),
            edit_m: TextEdit::new(""),
        };
        a.lay_new_field();
        a
    }

    /// `k` in the game's language.
    pub fn tr(&self, k: S) -> &'static str {
        k.get(self.lang)
    }

    /// `k` in the game's language, its `{}` slots filled by `args` in order.
    pub fn trf(&self, k: S, args: &[&dyn std::fmt::Display]) -> String {
        k.fill(self.lang, args)
    }

    /// Switches every string to `lang` at once, and saves it.
    pub fn set_lang(&mut self, lang: Lang) {
        self.lang = lang;
        self.saved.lang = Some(lang);
        storage::save(&self.saved);
        shape_in(lang);
    }

    pub fn dims(&self) -> (usize, usize, usize) {
        self.level.dims().unwrap_or(self.saved.custom)
    }

    pub fn new_game(&mut self) {
        self.deal = true;
        self.fresh_field();
    }

    /// A new field from the keyboard (F2, Cmd/Ctrl+N): it's there at once.
    pub fn new_game_now(&mut self) {
        self.deal = false;
        self.fresh_field();
    }

    /// Classic deals its field now; the new modes draw a seed and wait for the first click.
    fn lay_new_field(&mut self) {
        let (w, h, m) = self.dims();
        self.record = Record::new(self.mode, (w, h, m));
        self.record_t0 = None;
        self.clock_waits = false;
        // A field still generating is cancelled: it never lands.
        self.generating = None;
        self.fifty_fifties.clear();
        self.generating_since = None;
        self.sweep_from = None;
        self.unlaid = self.mode.lays_on_first_click();
        if self.unlaid {
            self.seed = self.seeds.next_u64();
            self.game = Game::unlaid(w, h, m);
            generating::warm();
        } else {
            // One stream for the whole session, as Windows seeds once at startup.
            self.game = Game::new(w, h, m, self.rng);
            self.rng = self.game.rng;
        }
    }

    /// The new modes' first click: starts generating the field its mode, level, seed and
    /// click name, off the UI thread.
    fn lay_at(&mut self, first: usize) {
        self.generating = Some(Generating::start(self.lay, self.mode, self.dims(), self.seed, first, self.waker.clone()));
        self.generating_since = None;
        self.sweep_from = None;
    }

    /// The generated field lands: any marks already placed are kept, and the first click
    /// opens it. The clock starts at the next poll.
    fn land(&mut self, mut g: Game, first: usize) {
        self.generating = None;
        self.generating_since = None;
        self.sweep_from = None;
        for (c, old) in g.cells.iter_mut().zip(&self.game.cells) {
            c.cover = old.cover;
        }
        g.flags = self.game.flags;
        // Found from the layout alone, as a perfect player meets them: the web's worker
        // sends back only the mines.
        if self.mode == Mode::MaxGuess {
            self.fifty_fifties = play::walk(&g, first).fifty_fifties;
        }
        self.game = g;
        // The mines as laid, so a review of it never lays the field again.
        let laid = (0..self.game.cells.len()).filter(|&i| self.game.cells[i].mine).collect();
        self.record.layout = Layout::Seeded { seed: self.seed, first, laid: Some(laid) };
        // The record's clock, like the timer, starts with the field: generating never counts.
        self.record_t0 = Some(self.now);
        self.unlaid = false;
        self.clock_waits = true;
        self.play(false, first);
        if self.game.cells[first].cover == Cover::Open {
            self.hover_on.set(0.0);
        }
    }

    /// The true 50/50s got through: the pair's safe cell is open.
    pub fn fifty_fifties_survived(&self) -> usize {
        let c = &self.game.cells;
        self.fifty_fifties.iter().filter(|&&(a, b)| (c[a].cover == Cover::Open && !c[a].mine) || (c[b].cover == Cover::Open && !c[b].mine)).count()
    }

    /// The mine stepped on was one of a true 50/50's: no reading of the field could have
    /// avoided it.
    pub fn lost_on_fifty_fifty(&self) -> bool {
        self.game.status == Status::Lost && self.game.hit.is_some_and(|m| self.fifty_fifties.iter().any(|&(a, b)| a == m || b == m))
    }

    /// Waits for a field being generated and lands it (scenes and tests).
    pub fn finish_laying(&mut self) {
        if let Some(job) = self.generating.take() {
            let first = job.first;
            let g = job.wait();
            self.land(g, first);
        }
    }

    /// How long a generation runs before the sweep shows it is still working.
    pub const SWEEP_AFTER: f64 = 0.6;

    fn fresh_field(&mut self) {
        self.abandon();
        self.hint = None;
        self.review = None;
        self.review_laying = None;
        self.lay_new_field();
        self.field += 1;
        self.started_at = None;
        self.seconds = 0;
        self.press = Press::default();
        self.new_best = false;
        self.banner_done = false;
        self.shock_done = false;
        self.moments = Moments::default();
    }

    /// A Mode from the picker: saved, and a new field at once. The picker stays open, so a
    /// Level can be picked next.
    pub fn pick_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.saved.mode = mode;
        storage::save(&self.saved);
        self.new_game();
    }

    pub fn set_level(&mut self, level: Level, tx: &mut Tx) {
        self.level = level;
        self.saved.level = level;
        storage::save(&self.saved);
        self.new_game();
        tx.resize_window(window_size(self.game.w, self.game.h));
    }

    /// The field the board draws: a review's, else the game in play.
    pub fn shown(&self) -> &Game {
        self.review.as_ref().map_or(&self.game, |r| &r.game)
    }

    pub fn mood(&self) -> Mood {
        if (self.generating.is_some() && self.review.is_none()) || self.review_laying.is_some() {
            return Mood::Think;
        }
        match self.shown().status {
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
        if self.started_at.is_none() && self.game.status == Status::Playing && !self.clock_waits {
            self.started_at = Some(self.now);
            self.seconds = 1;
        }
    }

    /// A left release on cell `i`.
    pub fn open_at(&mut self, i: usize) {
        self.release(false, Some(i));
    }

    /// A chord on cell `i`.
    pub fn chord_at(&mut self, i: usize) {
        self.release(true, Some(i));
    }

    /// A right click on cell `i`.
    pub fn mark_at(&mut self, i: usize) {
        if self.game.over() || self.waits() {
            return;
        }
        self.note(Act::Mark { marks: self.saved.marks }, i);
        self.game.mark(i, self.saved.marks);
        self.keep_draft();
    }

    /// Adds a click to the game in play's record.
    fn note(&mut self, act: Act, cell: usize) {
        // Any move starts the next hint afresh.
        self.hint = None;
        let t0 = *self.record_t0.get_or_insert(self.now);
        if self.record.moves.is_empty() {
            self.record.at = storage::unix_ms();
        }
        let ms = ((self.now - t0).max(0.0) * 1000.0).round() as u32;
        self.record.moves.push(Move { ms, act, cell });
        match act {
            Act::Open => self.record.left += 1,
            Act::Chord => self.record.chords += 1,
            Act::Mark { .. } => self.record.right += 1,
        }
    }

    /// The game in play's record, as it would be kept if it ended now with `end`.
    fn record_now(&self, end: End) -> Record {
        let (three_bv, solved) = record::three_bv(&self.game);
        Record { end, dims: (self.game.w, self.game.h, self.game.mines), seconds: self.seconds, three_bv, three_bv_solved: solved, ..self.record.clone() }
    }

    /// Keeps the game in play, so closing the window or tab still records it.
    pub(crate) fn keep_draft(&self) {
        if self.game.started && !self.game.over() {
            storage::save_draft(Some(&self.record_now(End::Abandoned)));
        }
    }

    /// The game ended: its record joins History. A field nobody clicked has none. A win that
    /// beats the fastest win for its mode and level turns the timer gold; the first has none to beat.
    fn finish(&mut self, end: End) {
        if !self.game.started {
            return;
        }
        let r = self.record_now(end);
        if end == End::Won {
            self.new_best = record::fastest(&self.history, r.mode, r.dims).is_some_and(|best| r.seconds < best.seconds);
        }
        storage::append_record(&r);
        storage::save_draft(None);
        self.history.push(r);
    }

    /// The game in play is left for another field, or the app quits.
    fn abandon(&mut self) {
        if self.game.started && !self.game.over() {
            self.finish(End::Abandoned);
        }
    }

    /// A field just laid starts its clock at the next poll, and a generation just started
    /// stamps its start there: ask for one now.
    fn wake_clock(&self, tx: &mut Tx) {
        if self.clock_waits || (self.generating.is_some() && self.generating_since.is_none()) {
            tx.wake_after(Duration::ZERO);
        }
    }

    /// A field being generated takes no input, nor one under a review or waiting for one.
    pub(crate) fn waits(&self) -> bool {
        self.generating.is_some() || self.review.is_some() || self.review_laying.is_some()
    }

    /// A release on the board.
    fn release(&mut self, chord: bool, cell: Option<usize>) {
        let Some(i) = cell else { return };
        if self.waits() {
            return;
        }
        if self.flag_mode && !chord && self.game.cells[i].cover != Cover::Open {
            self.mark_at(i);
            return;
        }
        let chord = chord || (self.flag_mode && self.game.cells[i].cover == Cover::Open);
        self.release_at(chord, i);
        // The selector hides over what just opened.
        let shown = !self.game.over() && self.game.cells[i].cover != Cover::Open;
        self.hover_on.set(if shown { 1.0 } else { 0.0 });
    }

    pub(crate) fn release_at(&mut self, chord: bool, i: usize) {
        if self.game.over() {
            return;
        }
        self.note(if chord { Act::Chord } else { Act::Open }, i);
        if self.unlaid {
            // Only opening a cell lays the field; nothing else starts it.
            if chord || !matches!(self.game.cells[i].cover, Cover::Hidden | Cover::Question) {
                return;
            }
            self.lay_at(i);
            return;
        }
        if !self.game.started && !chord {
            // Classic keeps its field as dealt: replaying the first click moves a mine again.
            self.record.layout = Layout::Mines((0..self.game.cells.len()).filter(|&j| self.game.cells[j].mine).collect());
        }
        self.play(chord, i);
    }

    /// Opens or chords on `i` of a laid field.
    fn play(&mut self, chord: bool, i: usize) {
        // A forced guess: this click can open a cell, and none could be proven safe.
        let cover = self.game.cells[i].cover;
        let opens = if chord { cover == Cover::Open } else { matches!(cover, Cover::Hidden | Cover::Question) };
        let forced = opens && self.game.started && play::forced(&self.game);
        self.game.start();
        let out = if chord { self.game.chord(i) } else { self.game.open(i) };
        if forced && out != Outcome::Nothing {
            self.record.forced += 1;
            self.record.survived += !matches!(out, Outcome::Lost(_)) as u32;
        }
        self.start_clock();
        match out {
            Outcome::Nothing | Outcome::Opened(_) => self.keep_draft(),
            Outcome::Lost(m) => {
                self.moments.boom = Some((self.now, self.cell_center(m)));
                self.started_at = None;
                self.finish(End::Lost);
            }
            Outcome::Won => {
                self.moments.won = Some(self.now);
                self.started_at = None;
                self.finish(End::Won);
            }
        }
    }

    pub fn toggle_marks(&mut self) {
        // Only the next right click changes: question marks already placed stay.
        self.saved.marks = !self.saved.marks;
        storage::save(&self.saved);
    }

    pub fn toggle_color(&mut self) {
        self.saved.color = !self.saved.color;
        storage::save(&self.saved);
    }

    /// Opens `panel`, or closes it if it is the one open. The picker's Custom sizes start from
    /// the saved Custom field.
    pub fn toggle_panel(&mut self, panel: Panel) {
        if self.panel == Some(panel) {
            self.panel = None;
            return;
        }
        if panel == Panel::Picker {
            let (w, h, m) = self.saved.custom;
            self.edit_w = TextEdit::new(&w.to_string());
            self.edit_h = TextEdit::new(&h.to_string());
            self.edit_m = TextEdit::new(&m.to_string());
        }
        if panel == Panel::History {
            self.history_filter = (self.mode, self.level);
        }
        self.panel = Some(panel);
    }

    pub fn close_panel(&mut self) {
        self.panel = None;
    }

    /// A Level from the picker: a new field at once. Custom takes the sizes typed inline.
    pub fn pick_level(&mut self, level: Level, tx: &mut Tx) {
        if level == Level::Custom {
            let n = |e: &TextEdit, d: usize| e.text.trim().parse::<usize>().unwrap_or(d);
            let (w0, h0, m0) = self.saved.custom;
            self.saved.custom = clamp_custom(n(&self.edit_w, w0), n(&self.edit_h, h0), n(&self.edit_m, m0));
        }
        self.panel = None;
        self.set_level(level, tx);
    }

    pub fn on_board_drag(&mut self, d: &rune::Drag, button: rune::PointerButton, tx: &mut Tx) {
        let cell = self.cell_at(d.x, d.y);
        match d.phase {
            rune::Phase::Down => {
                if self.game.over() || self.generating.is_some() {
                    return;
                }
                match button {
                    rune::PointerButton::Middle => self.press.middle = true,
                    // Shift with the left button is a chord, as both buttons are.
                    _ if tx.mods().shift => self.press.middle = true,
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
                    self.wake_clock(tx);
                }
            }
        }
    }

    pub fn on_board_secondary(&mut self, x: f32, y: f32, _tx: &mut Tx) {
        if self.game.over() || self.generating.is_some() {
            return;
        }
        if self.press.left {
            // Both buttons: the release will chord.
            self.press.right = true;
            return;
        }
        if let Some(i) = self.cell_at(x, y) {
            self.mark_at(i);
        }
    }

    /// Moves the selector (without re-running the view unless the cheat needs the cell).
    pub fn hover(&mut self, cell: Option<usize>, tx: &mut Tx) {
        match cell {
            Some(i) => {
                let (x, y) = (i % self.game.w, i / self.game.w);
                self.hover_at.set([WELL + x as f32 * PITCH, WELL + y as f32 * PITCH]);
                self.hover_on.set(if self.game.cells[i].cover == Cover::Open || self.game.over() { 0.0 } else { 1.0 });
            }
            None => {
                self.hover_on.set(0.0);
            }
        }
        if let Some(i) = cell {
            self.pixel_safe = !self.game.cells[i].mine;
        }
        if self.cheat && self.hover_cell != cell {
            self.hover_cell = cell;
        } else {
            self.hover_cell = cell;
            tx.live_only();
        }
    }

    /// When the result banner leaves after the game ends.
    pub const BANNER_FOR: f64 = 3.0;

    pub fn tick(&mut self, now: f64, tx: &mut Tx) -> bool {
        self.now = now;
        if self.generating.is_some() && self.poll_generation(tx) {
            return true;
        }
        if self.review_laying.is_some() && self.poll_review_laying(tx) {
            return true;
        }
        if let Some((t, _)) = self.moments.boom.filter(|_| !self.shock_done) {
            if now - t >= crate::gfx::SHOCK_FOR {
                self.shock_done = true;
                return true;
            }
            tx.wake_after(Duration::from_secs_f64((t + crate::gfx::SHOCK_FOR - now).max(0.01)));
        }
        let ended = self.moments.won.or(self.moments.boom.map(|(t, _)| t));
        if let Some(t) = ended.filter(|_| !self.banner_done) {
            if now - t >= Self::BANNER_FOR {
                self.banner_done = true;
                return true;
            }
            tx.wake_after(Duration::from_secs_f64((t + Self::BANNER_FOR - now).max(0.01)));
        }
        if self.clock_waits {
            self.clock_waits = false;
            self.start_clock();
            if self.started_at.is_some() {
                tx.wake_after(Duration::from_secs(1));
                return true;
            }
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

    /// A generation under way: lands its field once ready (true), else stamps its start and
    /// asks for the poll that starts the sweep. The view changes only when the field lands
    /// or the sweep starts.
    fn poll_generation(&mut self, tx: &mut Tx) -> bool {
        let Some(job) = &mut self.generating else { return false };
        if let Some(g) = job.take() {
            let first = job.first;
            self.land(g, first);
            // The clock starts in this same poll.
            self.clock_waits = false;
            self.start_clock();
            tx.wake_after(Duration::from_secs(1));
            return true;
        }
        // Without a waker nothing says when the field lands: look a few times a second.
        let check = if self.waker.is_some() { f64::INFINITY } else { 1.0 / 30.0 };
        let since = *self.generating_since.get_or_insert(self.now);
        let due = since + Self::SWEEP_AFTER - self.now;
        if due > 0.0 {
            tx.wake_after(Duration::from_secs_f64(due.min(check)));
            return false;
        }
        if check.is_finite() {
            tx.wake_after(Duration::from_secs_f64(check));
        }
        // The sweep mounts at the first poll past 600 ms.
        if self.sweep_from.is_none() {
            self.sweep_from = Some(since + Self::SWEEP_AFTER);
            return true;
        }
        false
    }

    /// A review's field being laid: opens the review once it lands (true).
    fn poll_review_laying(&mut self, tx: &mut Tx) -> bool {
        let Some((i, job)) = &mut self.review_laying else { return false };
        let Some(base) = job.take() else {
            // Without a waker nothing says when it lands: look a few times a second.
            if self.waker.is_none() {
                tx.wake_after(Duration::from_secs_f64(1.0 / 30.0));
            }
            return false;
        };
        let i = *i;
        self.review_laying = None;
        if let Some(r) = self.history.get(i) {
            self.review = Some(Review::of(r, base));
        }
        true
    }

    /// Arrow keys move the selector a square at a time, at once (keys never animate).
    fn step(&mut self, dx: isize, dy: isize, tx: &mut Tx) {
        let (w, h) = (self.game.w as isize, self.game.h as isize);
        let i = self.hover_cell.unwrap_or(self.game.idx(self.game.w / 2, self.game.h / 2));
        let (x, y) = ((i as isize % w + dx).clamp(0, w - 1), (i as isize / w + dy).clamp(0, h - 1));
        self.hover(Some((y * w + x) as usize), tx);
    }

    pub fn key(&mut self, e: &KeyEvent, tx: &mut Tx) -> bool {
        let plain = !e.mods.cmd && !e.mods.ctrl && !e.mods.alt;
        // A review: the arrows step through its moves, Escape leaves it, and the play keys
        // wait (keys never animate).
        if self.review.is_some() {
            match &e.key {
                Key::Left | Key::Right if plain => {
                    self.review_step(if e.key == Key::Left { -1 } else { 1 });
                    tx.instant();
                    return true;
                }
                Key::Escape if self.panel.is_none() => {
                    self.leave_review();
                    tx.instant();
                    return true;
                }
                Key::Up | Key::Down => return true,
                Key::Text(t) if plain && [" ", "f", "h"].contains(&t.to_ascii_lowercase().as_str()) => return true,
                _ => {}
            }
        }
        match &e.key {
            Key::Up if plain => {
                self.step(0, -1, tx);
                true
            }
            Key::Down if plain => {
                self.step(0, 1, tx);
                true
            }
            Key::Left if plain => {
                self.step(-1, 0, tx);
                true
            }
            Key::Right if plain => {
                self.step(1, 0, tx);
                true
            }
            // Space opens the selected square, or chords on a number.
            Key::Text(t) if plain && t == " " && self.hover_cell.is_some() => {
                let i = self.hover_cell.unwrap();
                let chord = self.game.cells[i].cover == Cover::Open;
                self.release(chord, Some(i));
                self.wake_clock(tx);
                true
            }
            Key::Text(t) if plain && t.eq_ignore_ascii_case("f") && self.hover_cell.is_some() => {
                let i = self.hover_cell.unwrap();
                self.mark_at(i);
                true
            }
            // H steps one stage deeper into a hint, at once (keys never animate).
            Key::Text(t) if plain && t.eq_ignore_ascii_case("h") => {
                self.press_hint();
                tx.instant();
                true
            }
            Key::Function(2) => {
                self.new_game_now();
                // A key swaps the field in one frame: no deal, no fade of the old one.
                tx.instant();
                tx.settle();
                true
            }
            // A key closes the panel in one frame: keys never animate.
            Key::Escape if self.panel.is_some() => {
                self.panel = None;
                tx.instant();
                true
            }
            Key::Escape if self.hint.is_some() => {
                self.hint = None;
                tx.instant();
                true
            }
            // Once the word is typed, each Shift press turns the cheat on or off.
            Key::Modifier { modifier: Modifier::ShiftLeft | Modifier::ShiftRight, pressed: true } if self.xyzzy >= 5 => {
                self.cheat = !self.cheat;
                true
            }
            // Other keys no longer count once the word is in.
            Key::Text(_) if self.xyzzy >= 5 => false,
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
            Key::Command('n') => {
                self.new_game_now();
                // A key swaps the field in one frame: no deal, no fade of the old one.
                tx.instant();
                tx.settle();
                true
            }
            Key::Command('q') | Key::Command('w') => {
                self.abandon();
                tx.quit();
                true
            }
            _ => false,
        }
    }
}

/// Text is shaped in the game's language, so Chinese is shaped as Chinese and falls back to
/// Simplified Chinese fonts.
fn shape_in(lang: Lang) {
    rune::text::set_language(Some(lang.tag()));
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
    fn flag_mode_flags_on_a_tap_and_chords_on_a_number() {
        let mut a = App::with(Saved::default(), 3);
        let safe = (0..a.game.cells.len()).find(|&i| !a.game.cells[i].mine && a.game.cells[i].adj > 0).unwrap();
        a.release(false, Some(safe));
        assert_eq!(a.game.cells[safe].cover, Cover::Open);
        a.flag_mode = true;
        let hidden = (0..a.game.cells.len()).find(|&i| a.game.cells[i].cover == Cover::Hidden).unwrap();
        a.release(false, Some(hidden));
        assert_eq!(a.game.cells[hidden].cover, Cover::Flag, "a tap flags");
        let opened = a.game.opened;
        a.release(false, Some(safe));
        assert!(a.game.opened >= opened, "a tap on a number chords, never flags it");
        assert_eq!(a.game.cells[safe].cover, Cover::Open);
    }

    #[test]
    fn hit_testing_maps_points_to_cells() {
        let a = App::with(Saved::default(), 3);
        assert_eq!(a.cell_at(WELL + 1.0, WELL + 1.0), Some(0));
        assert_eq!(a.cell_at(WELL + PITCH * 8.5, WELL + PITCH * 8.5), Some(80));
        assert_eq!(a.cell_at(WELL + PITCH * 9.5, WELL), None);
        assert_eq!(a.cell_at(1.0, 1.0), None);
    }
}
