//! Fields drawn as rows of characters, one per cell, for tests.

use crate::game::{Cover, Game, Status};

/// The cells drawn as `c`, row by row.
pub(crate) fn drawn(rows: &[&str], c: char) -> Vec<usize> {
    let w = rows[0].len();
    rows.iter().enumerate().flat_map(|(y, r)| r.char_indices().filter(move |&(_, ch)| ch == c).map(move |(x, _)| y * w + x)).collect()
}

/// The field with a mine at each `*` (and `F`), nothing open.
pub(crate) fn mined(rows: &[&str]) -> Game {
    Game::with_mines(rows[0].len(), rows.len(), &[drawn(rows, '*'), drawn(rows, 'F')].concat())
}

/// A position in play: `.` an open cell (its number follows from the mines), `#` a hidden
/// safe cell, `*` a hidden mine, `F` a flagged mine, `x` a flagged safe cell.
pub(crate) fn position(rows: &[&str]) -> Game {
    let mut g = mined(rows);
    for i in drawn(rows, '.') {
        g.cells[i].cover = Cover::Open;
        g.opened += 1;
    }
    for i in [drawn(rows, 'F'), drawn(rows, 'x')].concat() {
        g.cells[i].cover = Cover::Flag;
        g.flags += 1;
    }
    g.started = true;
    g.status = Status::Playing;
    g
}
