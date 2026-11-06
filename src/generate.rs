//! Laying a field: one pure call from (mode, level, seed, first click) to the field.
//!
//! Classic is XP's field from its `rand()` stream. The new modes (ADR 0001) lay nothing until
//! the first click, keep the 3×3 around it clear, and draw from their own generator, so the
//! same inputs always give the same field. No Guess redraws uniform fields until the bot
//! finishes one from the click by perfect logic: no time limit and no mine moved, so an
//! accepted field is a uniform draw over the fields logic can finish.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game::{Game, Mode, Rng};
use crate::solver::{Strength, play};

/// Columns, rows, mines.
pub type Dims = (usize, usize, usize);

/// The field for `mode` at `dims`, from `seed`, for a first click on `first`, with nothing
/// open. Classic's seed is the `rand()` state the field is dealt from (`srand(seed)`), and it
/// ignores `first`: XP moves a first-click mine when the click lands, not here.
pub fn lay(mode: Mode, dims: Dims, seed: u64, first: usize) -> Game {
    lay_counting(mode, dims, seed, first).0
}

/// [`lay`], and how many uniform fields were drawn to find it.
pub fn lay_counting(mode: Mode, (w, h, mines): Dims, seed: u64, first: usize) -> (Game, u32) {
    match mode {
        Mode::Classic => (Game::new(w, h, mines, Rng::new(seed as u32)), 1),
        Mode::NoGuess => redraw_until_clear(w, h, mines, seed, first, Strength::Full),
    }
}

/// Draws uniform fields with the 3×3 around `first` clear until the bot at `strength`
/// finishes one from `first` without guessing.
fn redraw_until_clear(w: usize, h: usize, mines: usize, seed: u64, first: usize, strength: Strength) -> (Game, u32) {
    let mut rng = Stream::new(seed);
    let (fx, fy) = ((first % w) as isize, (first / w) as isize);
    let near = |i: usize| ((i % w) as isize - fx).abs() <= 1 && ((i / w) as isize - fy).abs() <= 1;
    let free: Vec<usize> = (0..w * h).filter(|&i| !near(i)).collect();
    let mines = mines.min(free.len());
    let mut attempts = 0;
    loop {
        attempts += 1;
        let g = Game::with_mines(w, h, &rng.pick(&free, mines));
        if play::clears(&g, first, strength) || stopped() {
            return (g, attempts);
        }
    }
}

thread_local! {
    static STOP: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
}

/// Runs `f`, a call to [`lay`], so that it gives up soon after `stop` is set: a long search
/// checks [`stopped`] between steps and returns what it holds, which the caller throws away.
/// A generation off the UI thread runs this way, so a cancelled one stops.
pub fn stoppable<T>(stop: Arc<AtomicBool>, f: impl FnOnce() -> T) -> T {
    let before = STOP.with(|s| s.replace(Some(stop)));
    let out = f();
    STOP.with(|s| s.replace(before));
    out
}

/// Whether the [`stoppable`] call this runs in was told to stop. Every search that can run
/// long checks it between steps; outside `stoppable` it is false, so [`lay`] runs to the end.
pub fn stopped() -> bool {
    STOP.with(|s| s.borrow().as_ref().is_some_and(|f| f.load(Ordering::Relaxed)))
}

/// The new modes' random numbers: SplitMix64, the same on every platform.
#[derive(Clone, Copy, Debug)]
pub struct Stream(u64);

impl Stream {
    pub fn new(seed: u64) -> Stream {
        Stream(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// 0..n, near enough uniform (the bias is n / 2⁶⁴).
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_u64() as u128 * n as u128) >> 64) as usize
    }

    /// `k` distinct items of `from`, every choice equally likely.
    fn pick(&mut self, from: &[usize], k: usize) -> Vec<usize> {
        let mut v = from.to_vec();
        for i in 0..k {
            let j = i + self.below(v.len() - i);
            v.swap(i, j);
        }
        v.truncate(k);
        v
    }
}
