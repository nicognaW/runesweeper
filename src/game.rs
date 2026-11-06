//! The rules, with no drawing: the same field, first click, flood fill, chord, marks and
//! scoring as the Windows game.

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Beginner,
    Intermediate,
    Expert,
    Custom,
}

impl Level {
    /// Columns, rows, mines.
    pub fn dims(self) -> Option<(usize, usize, usize)> {
        match self {
            Level::Beginner => Some((9, 9, 10)),
            Level::Intermediate => Some((16, 16, 40)),
            Level::Expert => Some((30, 16, 99)),
            Level::Custom => None,
        }
    }
    pub fn index(self) -> Option<usize> {
        match self {
            Level::Beginner => Some(0),
            Level::Intermediate => Some(1),
            Level::Expert => Some(2),
            Level::Custom => None,
        }
    }
}

/// How a field's mines are laid, chosen independently of the Level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Windows XP's fields, first-click quirks included.
    #[default]
    Classic,
    /// Fields one number, or two overlapping numbers, at a time finish from the first click
    /// (the mine count helps near the end); the first click always opens an area.
    GentleNoGuess,
    /// Fields perfect logic finishes from the first click, which always opens an area.
    NoGuess,
    /// Fields built to force as many true 50/50s as possible, finished by logic between them;
    /// the first click always opens an area.
    MaxGuess,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Classic, Mode::GentleNoGuess, Mode::NoGuess, Mode::MaxGuess];

    /// Whether mines wait for the first click (and are laid by [`crate::generate::lay`]).
    pub fn lays_on_first_click(self) -> bool {
        self != Mode::Classic
    }

    /// The name in the settings file.
    pub fn tag(self) -> &'static str {
        match self {
            Mode::Classic => "classic",
            Mode::GentleNoGuess => "gentle-no-guess",
            Mode::NoGuess => "no-guess",
            Mode::MaxGuess => "max-guess",
        }
    }

    pub fn from_tag(tag: &str) -> Option<Mode> {
        Mode::ALL.into_iter().find(|m| m.tag() == tag)
    }
}

/// The Custom Field dialog's limits, as Windows clamps them.
pub fn clamp_custom(w: usize, h: usize, mines: usize) -> (usize, usize, usize) {
    let w = w.clamp(9, 30);
    let h = h.clamp(9, 24);
    let mines = mines.clamp(10, (w - 1) * (h - 1));
    (w, h, mines)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
    Hidden,
    Flag,
    Question,
    Open,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub mine: bool,
    pub adj: u8,
    pub cover: Cover,
    /// Flood-fill distance from the click that opened it (drives the ripple).
    pub wave: u16,
    /// Which reveal opened it, so a cell's animation replays only when it opens.
    pub turn: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// A fresh field; the clock hasn't started.
    Ready,
    Playing,
    Won,
    Lost,
}

/// What a move did, for the view's effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Opened(usize),
    Won,
    Lost(usize),
}

#[derive(Clone)]
pub struct Game {
    pub w: usize,
    pub h: usize,
    pub mines: usize,
    pub cells: Vec<Cell>,
    pub status: Status,
    /// The mine that was stepped on.
    pub hit: Option<usize>,
    /// Whether any cell has been opened (first-click protection).
    pub started: bool,
    pub opened: usize,
    pub flags: usize,
    pub turn: u32,
    /// The generator after this field was laid; the next field continues from it.
    pub rng: Rng,
}

impl Game {
    pub fn new(w: usize, h: usize, mines: usize, rng: Rng) -> Game {
        let mut g = Game { rng, ..Game::unlaid(w, h, mines) };
        g.lay_mines();
        g
    }

    /// A field with mines at `mines` and nothing open (the new modes' generators lay these).
    pub fn with_mines(w: usize, h: usize, mines: &[usize]) -> Game {
        let mut g = Game::unlaid(w, h, mines.len());
        for &i in mines {
            g.cells[i].mine = true;
        }
        g.count();
        g
    }

    /// A field with no mines yet: the new modes lay them on the first click. The counter
    /// already shows `mines`.
    pub fn unlaid(w: usize, h: usize, mines: usize) -> Game {
        Game {
            w,
            h,
            mines: mines.min(w * h - 1),
            cells: vec![Cell { mine: false, adj: 0, cover: Cover::Hidden, wave: 0, turn: 0 }; w * h],
            status: Status::Ready,
            hit: None,
            started: false,
            opened: 0,
            flags: 0,
            turn: 0,
            rng: Rng::new(0),
        }
    }

    /// Mines are laid when the field is made, as the original does: a column, then a
    /// row, each `rand() % n`, drawn again while the square already holds a mine.
    fn lay_mines(&mut self) {
        let mut placed = 0;
        while placed < self.mines {
            let x = self.rng.below(self.w);
            let y = self.rng.below(self.h);
            let i = self.idx(x, y);
            if !self.cells[i].mine {
                self.cells[i].mine = true;
                placed += 1;
            }
        }
        self.count();
    }

    fn count(&mut self) {
        for i in 0..self.cells.len() {
            let n = self.neighbours(i).filter(|&j| self.cells[j].mine).count();
            self.cells[i].adj = n as u8;
        }
    }

    pub fn neighbours(&self, i: usize) -> impl Iterator<Item = usize> + use<> {
        let (w, h) = (self.w as isize, self.h as isize);
        let (x, y) = ((i % self.w) as isize, (i / self.w) as isize);
        (-1..=1)
            .flat_map(move |dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter(|&(dx, dy)| dx != 0 || dy != 0)
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(move |&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
            .map(move |(x, y)| (y * w + x) as usize)
    }

    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    pub fn over(&self) -> bool {
        matches!(self.status, Status::Won | Status::Lost)
    }

    /// The mine counter: mines less flags, which goes negative when over-flagged.
    pub fn counter(&self) -> i32 {
        self.mines as i32 - self.flags as i32
    }

    /// Left click.
    pub fn open(&mut self, i: usize) -> Outcome {
        if self.over() || !matches!(self.cells[i].cover, Cover::Hidden | Cover::Question) {
            return Outcome::Nothing;
        }
        if !self.started {
            self.status = Status::Playing;
            if self.cells[i].mine {
                // A first click on a mine moves it to the first free square, row by row from
                // the top left. Windows' scan stops short of the last row and the last
                // column; when that area is all mines nothing moves, nothing opens, and the
                // next click is still a first click.
                let free = (0..self.h - 1).flat_map(|y| (0..self.w - 1).map(move |x| (x, y))).map(|(x, y)| self.idx(x, y)).find(|&j| !self.cells[j].mine);
                let Some(free) = free else { return Outcome::Nothing };
                self.cells[i].mine = false;
                self.cells[free].mine = true;
                self.count();
            }
            self.started = true;
        }
        self.turn += 1;
        if self.cells[i].mine {
            return self.lose(i);
        }
        let n = self.flood(&[i]);
        self.check_win().unwrap_or(Outcome::Opened(n))
    }

    /// Both buttons (or the middle one) on an open number whose flags are all placed.
    pub fn chord(&mut self, i: usize) -> Outcome {
        if self.over() || self.cells[i].cover != Cover::Open || self.cells[i].adj == 0 {
            return Outcome::Nothing;
        }
        let flags = self.neighbours(i).filter(|&j| self.cells[j].cover == Cover::Flag).count();
        if flags != self.cells[i].adj as usize {
            return Outcome::Nothing;
        }
        let targets: Vec<usize> = self.neighbours(i).filter(|&j| matches!(self.cells[j].cover, Cover::Hidden | Cover::Question)).collect();
        if targets.is_empty() {
            return Outcome::Nothing;
        }
        self.turn += 1;
        let mines: Vec<usize> = targets.iter().copied().filter(|&j| self.cells[j].mine).collect();
        let safe: Vec<usize> = targets.iter().copied().filter(|&j| !self.cells[j].mine).collect();
        let n = self.flood(&safe);
        if let Some(&m) = mines.first() {
            // A wrong flag: Windows still opens the safe squares, and every mine it uncovers
            // shows red.
            for &j in &mines[1..] {
                self.cells[j].cover = Cover::Open;
                self.cells[j].turn = self.turn;
            }
            return self.lose(m);
        }
        self.check_win().unwrap_or(Outcome::Opened(n))
    }

    /// Right click: Hidden → Flag → ? (with marks on) → Hidden.
    pub fn mark(&mut self, i: usize, marks: bool) -> bool {
        if self.over() {
            return false;
        }
        let c = &mut self.cells[i];
        c.cover = match c.cover {
            Cover::Hidden => {
                self.flags += 1;
                Cover::Flag
            }
            Cover::Flag => {
                self.flags -= 1;
                if marks { Cover::Question } else { Cover::Hidden }
            }
            Cover::Question => Cover::Hidden,
            Cover::Open => return false,
        };
        true
    }

    /// Any release on a fresh field starts the clock, even one that opens nothing.
    pub fn start(&mut self) {
        if self.status == Status::Ready {
            self.status = Status::Playing;
        }
    }

    fn flood(&mut self, from: &[usize]) -> usize {
        let mut q = VecDeque::new();
        let mut n = 0;
        for &i in from {
            if matches!(self.cells[i].cover, Cover::Hidden | Cover::Question) {
                self.open_one(i, 0);
                n += 1;
                q.push_back(i);
            }
        }
        while let Some(i) = q.pop_front() {
            if self.cells[i].adj != 0 {
                continue;
            }
            let wave = self.cells[i].wave + 1;
            for j in self.neighbours(i).collect::<Vec<_>>() {
                if matches!(self.cells[j].cover, Cover::Hidden | Cover::Question) && !self.cells[j].mine {
                    self.open_one(j, wave);
                    n += 1;
                    q.push_back(j);
                }
            }
        }
        n
    }

    fn open_one(&mut self, i: usize, wave: u16) {
        let turn = self.turn;
        let c = &mut self.cells[i];
        c.cover = Cover::Open;
        c.wave = wave;
        c.turn = turn;
        self.opened += 1;
    }

    fn lose(&mut self, i: usize) -> Outcome {
        self.status = Status::Lost;
        self.hit = Some(i);
        self.cells[i].cover = Cover::Open;
        self.cells[i].turn = self.turn;
        Outcome::Lost(i)
    }

    fn check_win(&mut self) -> Option<Outcome> {
        if self.opened + self.mines != self.w * self.h {
            return None;
        }
        self.status = Status::Won;
        for c in &mut self.cells {
            if c.mine && c.cover != Cover::Flag {
                c.cover = Cover::Flag;
            }
        }
        self.flags = self.mines;
        Some(Outcome::Won)
    }
}

/// The Microsoft C runtime's `rand()`, which Windows Minesweeper uses: seeded once with
/// `srand(GetTickCount())`, then one stream for every field.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u32);

impl Rng {
    /// `srand(seed)`.
    pub fn new(seed: u32) -> Rng {
        Rng(seed)
    }
    /// `rand()`: 0..=0x7FFF.
    fn rand(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.0 >> 16) & 0x7FFF
    }
    /// `rand() % n`, bias and all.
    fn below(&mut self, n: usize) -> usize {
        self.rand() as usize % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(rows: &[&str]) -> Game {
        let h = rows.len();
        let w = rows[0].len();
        let mut g = Game::new(w, h, 1, Rng::new(7));
        for c in &mut g.cells {
            c.mine = false;
        }
        let mut mines = 0;
        for (y, r) in rows.iter().enumerate() {
            for (x, ch) in r.chars().enumerate() {
                if ch == '*' {
                    g.cells[y * w + x].mine = true;
                    mines += 1;
                }
            }
        }
        g.mines = mines;
        g.count();
        g
    }

    #[test]
    fn levels_lay_their_mines() {
        for l in [Level::Beginner, Level::Intermediate, Level::Expert] {
            let (w, h, m) = l.dims().unwrap();
            let g = Game::new(w, h, m, Rng::new(42));
            assert_eq!(g.cells.iter().filter(|c| c.mine).count(), m);
        }
    }

    #[test]
    fn the_first_click_moves_a_mine_to_the_top_left() {
        let mut g = field(&["...", ".*.", "..."]);
        g.open(4);
        assert!(!g.cells[4].mine);
        assert!(g.cells[0].mine, "the mine moved to the first free square");
        assert_eq!(g.status, Status::Playing);
    }

    #[test]
    fn rand_matches_the_microsoft_runtime() {
        let mut r = Rng::new(1);
        assert_eq!([r.rand(), r.rand(), r.rand(), r.rand()], [41, 18467, 6334, 26500]);
    }

    #[test]
    fn mines_are_drawn_column_then_row() {
        let g = Game::new(9, 9, 1, Rng::new(1));
        let (x, y) = (41 % 9, 18467 % 9);
        assert!(g.cells[g.idx(x, y)].mine);
    }

    #[test]
    fn the_first_click_scan_skips_the_last_row_and_column() {
        // The first free square in reading order is (2,0), in the last column.
        let mut g = field(&["**.", "*..", "*.."]);
        g.open(0);
        assert!(!g.cells[0].mine);
        assert!(g.cells[4].mine, "the mine went to (1,1)");
        assert!(!g.cells[2].mine, "never the last column");
    }

    #[test]
    fn a_first_click_with_nowhere_to_move_does_nothing() {
        let mut g = field(&["**.", "**.", "..."]);
        assert_eq!(g.open(0), Outcome::Nothing);
        assert!(g.cells[0].mine && !g.started, "the next click is still a first click");
        assert_eq!(g.cells[0].cover, Cover::Hidden);
        assert_eq!(g.open(2), Outcome::Opened(1));
        assert!(g.started);
    }

    #[test]
    fn zeros_flood_and_numbers_stop_it() {
        let mut g = field(&["....", "....", "...*"]);
        assert_eq!(g.open(0), Outcome::Won);
        assert_eq!(g.opened, 11);
        assert_eq!(g.cells[11].cover, Cover::Flag, "a win flags every mine");
        assert_eq!(g.counter(), 0);
    }

    #[test]
    fn waves_count_out_from_the_click() {
        let mut g = field(&[".....", ".....", "....*"]);
        g.open(0);
        assert_eq!(g.cells[0].wave, 0);
        assert_eq!(g.cells[1].wave, 1);
        assert_eq!(g.cells[2].wave, 2);
    }

    #[test]
    fn marks_cycle_and_count() {
        let mut g = field(&["*..", "...", "..."]);
        g.mark(0, true);
        assert_eq!(g.cells[0].cover, Cover::Flag);
        assert_eq!(g.counter(), 0);
        g.mark(0, true);
        assert_eq!(g.cells[0].cover, Cover::Question);
        assert_eq!(g.counter(), 1);
        g.mark(0, true);
        assert_eq!(g.cells[0].cover, Cover::Hidden);
        g.mark(0, false);
        g.mark(0, false);
        assert_eq!(g.cells[0].cover, Cover::Hidden, "without marks a flag goes straight back");
    }

    #[test]
    fn flags_protect_and_questions_dont() {
        let mut g = field(&["*..", "...", "..*"]);
        g.started = true;
        g.mark(0, true);
        assert_eq!(g.open(0), Outcome::Nothing);
        g.mark(0, true);
        assert!(matches!(g.open(0), Outcome::Lost(0)));
    }

    #[test]
    fn chord_needs_its_flags_and_loses_on_a_wrong_one() {
        let mut g = field(&["*..", "...", "..."]);
        g.started = true;
        g.open(4);
        assert_eq!(g.chord(4), Outcome::Nothing, "no flag yet");
        g.mark(0, true);
        assert!(matches!(g.chord(4), Outcome::Won));

        let mut g = field(&["*..", "...", "..*"]);
        g.started = true;
        g.open(4);
        g.mark(1, true);
        g.mark(8, true);
        assert_eq!(g.chord(4), Outcome::Lost(0));
    }

    #[test]
    fn a_wrong_chord_opens_the_safe_squares_and_reds_every_mine() {
        let mut g = field(&["*.*", "...", "*.."]);
        g.started = true;
        g.open(4);
        g.mark(6, true);
        g.mark(1, true);
        g.mark(5, true);
        assert!(matches!(g.chord(4), Outcome::Lost(0)));
        assert_eq!(g.cells[2].cover, Cover::Open, "the other unflagged mine is shown as hit");
        assert_eq!(g.cells[3].cover, Cover::Open, "the safe squares still open");
        assert_eq!(g.cells[6].cover, Cover::Flag);
    }

    #[test]
    fn custom_limits() {
        assert_eq!(clamp_custom(3, 100, 5000), (9, 24, 8 * 23));
        assert_eq!(clamp_custom(30, 24, 1), (30, 24, 10));
    }
}
