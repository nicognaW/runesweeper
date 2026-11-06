//! Game records: what is kept about one game that was won, lost, or abandoned after its first
//! click, and the measures they hold.
//!
//! A record is one line of space-separated `key=value` fields, so History is an append-only
//! text: a file next to settings natively, one browser-storage key on the web. Readers skip
//! keys they don't know, so later fields (hints, assisted) need no new version. `v` changes
//! only if a field changes meaning.

use crate::game::{Cover, Game, Level, Mode};
use crate::generate::{self, Dims};

/// The line format's version.
const VERSION: u32 = 1;

/// How a game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Won,
    Lost,
    /// Left after its first click: a new field, a level or mode change, or quitting.
    Abandoned,
}

impl End {
    const ALL: [End; 3] = [End::Won, End::Lost, End::Abandoned];

    fn tag(self) -> &'static str {
        match self {
            End::Won => "won",
            End::Lost => "lost",
            End::Abandoned => "abandoned",
        }
    }
}

/// What a click did to the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// A left click: opens the cell.
    Open,
    /// Both buttons, the middle one or Shift with the left on a number.
    Chord,
    /// A right click, which cycles Hidden → Flag → (? when `marks`) → Hidden.
    Mark { marks: bool },
}

impl Act {
    fn tag(self) -> char {
        match self {
            Act::Open => 'o',
            Act::Chord => 'c',
            Act::Mark { marks: false } => 'f',
            Act::Mark { marks: true } => 'q',
        }
    }

    fn from_tag(c: char) -> Option<Act> {
        Some(match c {
            'o' => Act::Open,
            'c' => Act::Chord,
            'f' => Act::Mark { marks: false },
            'q' => Act::Mark { marks: true },
            _ => return None,
        })
    }
}

/// One click on the field, in order: every click is kept, ones that changed nothing too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    /// Milliseconds since the field's first click of any kind.
    pub ms: u32,
    pub act: Act,
    pub cell: usize,
}

/// Where the field's mines came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    /// The new modes: [`generate::lay`] from the seed and first click.
    Seeded { seed: u64, first: usize },
    /// Classic: the mines as dealt at New Game, before XP moves a first-click mine.
    Mines(Vec<usize>),
}

/// What is kept about one game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// When the first click came, in milliseconds since the Unix epoch.
    pub at: u64,
    pub mode: Mode,
    /// Columns, rows, mines: the level.
    pub dims: Dims,
    pub end: End,
    /// The timer when the game ended.
    pub seconds: u32,
    pub three_bv: u32,
    pub three_bv_solved: u32,
    pub left: u32,
    pub right: u32,
    pub chords: u32,
    /// Hints shown: any makes it an assisted game.
    pub hints: u32,
    /// Cells opened when no hidden cell could be proven safe, the first click excluded.
    pub forced: u32,
    /// Of those, the ones that opened a safe cell.
    pub survived: u32,
    pub layout: Layout,
    pub moves: Vec<Move>,
}

impl Record {
    /// An empty record for a field of `mode` at `dims`.
    pub fn new(mode: Mode, dims: Dims) -> Record {
        Record {
            at: 0,
            mode,
            dims,
            end: End::Abandoned,
            seconds: 0,
            three_bv: 0,
            three_bv_solved: 0,
            left: 0,
            right: 0,
            chords: 0,
            hints: 0,
            forced: 0,
            survived: 0,
            layout: Layout::Mines(Vec::new()),
            moves: Vec::new(),
        }
    }

    /// A game in which at least one Hint was shown.
    pub fn assisted(&self) -> bool {
        self.hints > 0
    }

    /// The Level these dims are: a standard one, or Custom.
    pub fn level(&self) -> Level {
        [Level::Beginner, Level::Intermediate, Level::Expert].into_iter().find(|l| l.dims() == Some(self.dims)).unwrap_or(Level::Custom)
    }

    /// The field at the end of the game, rebuilt from its layout and moves.
    pub fn replay(&self) -> Game {
        let (w, h, _) = self.dims;
        let mut g = match &self.layout {
            Layout::Mines(m) => Game::with_mines(w, h, m),
            Layout::Seeded { seed, first } => generate::lay(self.mode, self.dims, *seed, *first),
        };
        for m in &self.moves {
            match m.act {
                Act::Open => {
                    g.start();
                    g.open(m.cell);
                }
                Act::Chord => {
                    g.start();
                    g.chord(m.cell);
                }
                Act::Mark { marks } => {
                    g.mark(m.cell, marks);
                }
            }
        }
        g
    }

    /// The record as one line, without its newline.
    pub fn write(&self) -> String {
        let (w, h, m) = self.dims;
        let mut out = format!(
            "v={VERSION} at={} mode={} size={w}x{h}x{m} end={} secs={} 3bv={} 3bv-solved={} left={} right={} chord={} hints={} forced={} survived={}",
            self.at,
            self.mode.tag(),
            self.end.tag(),
            self.seconds,
            self.three_bv,
            self.three_bv_solved,
            self.left,
            self.right,
            self.chords,
            self.hints,
            self.forced,
            self.survived,
        );
        match &self.layout {
            Layout::Seeded { seed, first } => out += &format!(" seed={seed} first={first}"),
            Layout::Mines(m) => out += &format!(" field={}", join(m.iter())),
        }
        out += " moves=";
        out += &join(self.moves.iter().map(|m| format!("{}{}@{}", m.act.tag(), m.cell, m.ms)));
        out
    }

    /// A line [`Record::write`] wrote, or None if it isn't one (a later version's mode, a
    /// line cut short).
    pub fn parse(line: &str) -> Option<Record> {
        let mut r = Record::new(Mode::Classic, (0, 0, 0));
        let (mut version, mut mode, mut end, mut seed, mut first, mut field) = (None, None, None, None, None, None);
        for (k, v) in line.split_whitespace().filter_map(|f| f.split_once('=')) {
            let n = || v.parse::<u32>().ok();
            match k {
                "v" => version = n(),
                "at" => r.at = v.parse().ok()?,
                "mode" => mode = Mode::from_tag(v),
                "size" => {
                    let p: Vec<usize> = v.split('x').filter_map(|n| n.parse().ok()).collect();
                    let [w, h, m] = p[..] else { return None };
                    r.dims = (w, h, m);
                }
                "end" => end = End::ALL.into_iter().find(|e| e.tag() == v),
                "secs" => r.seconds = n()?,
                "3bv" => r.three_bv = n()?,
                "3bv-solved" => r.three_bv_solved = n()?,
                "left" => r.left = n()?,
                "right" => r.right = n()?,
                "chord" => r.chords = n()?,
                "hints" => r.hints = n()?,
                "forced" => r.forced = n()?,
                "survived" => r.survived = n()?,
                "seed" => seed = v.parse::<u64>().ok(),
                "first" => first = v.parse::<usize>().ok(),
                "field" => field = Some(split(v).map(|c| c.parse().ok()).collect::<Option<Vec<usize>>>()?),
                "moves" => r.moves = split(v).map(parse_move).collect::<Option<_>>()?,
                _ => {}
            }
        }
        version.filter(|&v| v == VERSION)?;
        let (w, h, _) = r.dims;
        let cells = w * h;
        if cells == 0 || r.moves.iter().any(|m| m.cell >= cells) {
            return None;
        }
        r.mode = mode?;
        r.end = end?;
        r.layout = match (seed, first, field) {
            (Some(seed), Some(first), _) if first < cells => Layout::Seeded { seed, first },
            (_, _, Some(m)) if m.iter().all(|&c| c < cells) => Layout::Mines(m),
            _ => return None,
        };
        Some(r)
    }
}

fn join<T: std::fmt::Display>(items: impl Iterator<Item = T>) -> String {
    items.map(|i| i.to_string()).collect::<Vec<_>>().join(",")
}

fn split(v: &str) -> impl Iterator<Item = &str> {
    v.split(',').filter(|s| !s.is_empty())
}

fn parse_move(s: &str) -> Option<Move> {
    let act = Act::from_tag(s.chars().next()?)?;
    let (cell, ms) = s[1..].split_once('@')?;
    Some(Move { ms: ms.parse().ok()?, act, cell: cell.parse().ok()? })
}

/// The fastest win among `records` for `mode` at `dims`; of equal times, the earliest.
pub fn fastest(records: &[Record], mode: Mode, dims: Dims) -> Option<&Record> {
    records.iter().filter(|r| r.end == End::Won && r.mode == mode && r.dims == dims).min_by_key(|r| (r.seconds, r.at))
}

/// A field's 3BV (the fewest clicks that clear it without chords: one per opening, one per
/// number no opening reaches), and how many of those are done. An opening is done once any
/// of its empty cells is open, a lone number once it is open.
pub fn three_bv(game: &Game) -> (u32, u32) {
    let empty = |i: usize| !game.cells[i].mine && game.cells[i].adj == 0;
    let open = |i: usize| game.cells[i].cover == Cover::Open;
    let (mut total, mut done) = (0, 0);
    let mut seen = vec![false; game.cells.len()];
    for start in 0..game.cells.len() {
        if seen[start] || !empty(start) {
            continue;
        }
        // One opening: its empty cells, joined through their neighbours.
        let (mut stack, mut any_open) = (vec![start], false);
        seen[start] = true;
        while let Some(i) = stack.pop() {
            any_open |= open(i);
            for j in game.neighbours(i) {
                if !seen[j] && empty(j) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        total += 1;
        done += any_open as u32;
    }
    for i in 0..game.cells.len() {
        let c = game.cells[i];
        if !c.mine && c.adj > 0 && !game.neighbours(i).any(empty) {
            total += 1;
            done += open(i) as u32;
        }
    }
    (total, done)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field drawn as rows, `*` for a mine.
    fn field(rows: &[&str]) -> Game {
        let w = rows[0].len();
        let mines: Vec<usize> = rows.iter().enumerate().flat_map(|(y, r)| r.char_indices().filter(|c| c.1 == '*').map(move |(x, _)| y * w + x)).collect();
        Game::with_mines(w, rows.len(), &mines)
    }

    #[test]
    fn three_bv_matches_counts_worked_by_hand() {
        // One opening reaches every number.
        assert_eq!(three_bv(&field(&["*..", "...", "..."])).0, 1);
        // No empty cell: every number is a click.
        assert_eq!(three_bv(&field(&[".*.*."])).0, 3);
        assert_eq!(three_bv(&field(&["*.*", "...", "*.*"])).0, 5);
        // Two openings walled apart by mines.
        assert_eq!(three_bv(&field(&["..*..", "..*..", "..*.."])).0, 2);
        // Two openings and the six numbers in the far corners neither reaches.
        assert_eq!(three_bv(&field(&[".....", ".*...", ".....", "...*.", "....."])).0, 8);
    }

    #[test]
    fn three_bv_solved_counts_what_is_open() {
        let mut g = field(&["..*..", "..*..", "..*.."]);
        assert_eq!(three_bv(&g), (2, 0));
        g.open(0);
        assert_eq!(three_bv(&g), (2, 1), "the left opening");
        g.open(4);
        assert_eq!(three_bv(&g), (2, 2));

        let mut g = field(&[".*.*."]);
        g.open(2);
        assert_eq!(three_bv(&g), (3, 1));
    }

    fn sample() -> Record {
        Record {
            at: 1_793_000_000_123,
            mode: Mode::Classic,
            dims: (9, 9, 10),
            end: End::Won,
            seconds: 37,
            three_bv: 24,
            three_bv_solved: 24,
            left: 20,
            right: 3,
            chords: 2,
            hints: 0,
            forced: 1,
            survived: 1,
            layout: Layout::Mines(vec![0, 5, 9, 17, 22, 40, 41, 63, 70, 80]),
            moves: vec![
                Move { ms: 0, act: Act::Mark { marks: true }, cell: 3 },
                Move { ms: 250, act: Act::Open, cell: 44 },
                Move { ms: 1900, act: Act::Mark { marks: false }, cell: 5 },
                Move { ms: 2400, act: Act::Chord, cell: 14 },
            ],
        }
    }

    #[test]
    fn a_record_round_trips_through_its_line() {
        let r = sample();
        let line = r.write();
        assert!(!line.contains('\n'), "one line: {line}");
        assert_eq!(Record::parse(&line), Some(r.clone()));
        let seeded = Record { mode: Mode::NoGuess, layout: Layout::Seeded { seed: u64::MAX - 7, first: 40 }, ..r };
        assert_eq!(Record::parse(&seeded.write()), Some(seeded));
    }

    #[test]
    fn readers_skip_fields_they_dont_know_and_lines_they_cant_read() {
        let line = sample().write();
        assert_eq!(Record::parse(&format!("{line} assisted=1 tz=+480")), Some(sample()), "a later field is skipped");
        assert_eq!(Record::parse(&line.replace("v=1", "v=2")), None, "another version");
        assert_eq!(Record::parse(&line.replace("mode=classic", "mode=from-the-future")), None, "an unknown mode");
        assert_eq!(Record::parse(&line[..line.len() / 2]), None, "a line cut short");
        assert_eq!(Record::parse(""), None);
        assert_eq!(Record::parse("v=1 at=0 mode=classic size=9x9x10 end=won moves=o81@0 field=1"), None, "a move off the field");
    }

    #[test]
    fn the_fastest_win_is_per_mode_and_level() {
        let at = |at, mode, dims, end, seconds| Record { at, mode, dims, end, seconds, ..sample() };
        let rs = [
            at(1, Mode::Classic, (9, 9, 10), End::Won, 40),
            at(2, Mode::Classic, (9, 9, 10), End::Lost, 3),
            at(3, Mode::Classic, (9, 9, 10), End::Won, 31),
            at(4, Mode::Classic, (9, 9, 10), End::Won, 31),
            at(5, Mode::NoGuess, (9, 9, 10), End::Won, 12),
            at(6, Mode::Classic, (16, 16, 40), End::Won, 9),
        ];
        assert_eq!(fastest(&rs, Mode::Classic, (9, 9, 10)).map(|r| r.at), Some(3), "a loss never counts; a tie keeps the first");
        assert_eq!(fastest(&rs, Mode::NoGuess, (9, 9, 10)).map(|r| r.at), Some(5));
        assert_eq!(fastest(&rs, Mode::Classic, (16, 16, 40)).map(|r| r.at), Some(6));
        assert_eq!(fastest(&rs, Mode::Classic, (30, 16, 99)), None);
    }
}
