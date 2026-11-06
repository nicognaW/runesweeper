//! The rules, with no drawing: the same field, first click, flood fill, chord, marks and
//! scoring as the Windows game.

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Intermediate,
    Expert,
    Custom,
}

impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Level::Beginner => "Beginner",
            Level::Intermediate => "Intermediate",
            Level::Expert => "Expert",
            Level::Custom => "Custom",
        }
    }
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
    rng: Rng,
}

impl Game {
    pub fn new(w: usize, h: usize, mines: usize, seed: u64) -> Game {
        let mut g = Game {
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
            rng: Rng(seed | 1),
        };
        g.lay_mines();
        g
    }

    /// Mines are laid when the field is made, as the original does.
    fn lay_mines(&mut self) {
        let mut placed = 0;
        while placed < self.mines {
            let i = (self.rng.next() % (self.w * self.h) as u64) as usize;
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
            self.started = true;
            self.status = Status::Playing;
            if self.cells[i].mine {
                // The first click is never a mine: Windows moves it to the top-left-most
                // free square.
                self.cells[i].mine = false;
                let free = (0..self.cells.len()).find(|&j| j != i && !self.cells[j].mine).unwrap();
                self.cells[free].mine = true;
                self.count();
            }
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
        if let Some(&m) = targets.iter().find(|&&j| self.cells[j].mine) {
            return self.lose(m);
        }
        let n = self.flood(&targets);
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

    /// Turning marks off clears the question marks already placed.
    pub fn clear_questions(&mut self) {
        for c in &mut self.cells {
            if c.cover == Cover::Question {
                c.cover = Cover::Hidden;
            }
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

/// splitmix64: small, seedable, the same on every platform.
#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(rows: &[&str]) -> Game {
        let h = rows.len();
        let w = rows[0].len();
        let mut g = Game::new(w, h, 1, 7);
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
            let g = Game::new(w, h, m, 42);
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
    fn custom_limits() {
        assert_eq!(clamp_custom(3, 100, 5000), (9, 24, 8 * 23));
        assert_eq!(clamp_custom(30, 24, 1), (30, 24, 10));
    }
}
