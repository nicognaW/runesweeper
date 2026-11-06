//! Generation off the UI thread, driven headlessly through Rune's `Runtime`: a first click in a
//! mode that lays on it, a generation held back as long as the test likes, and what the window
//! does meanwhile.

use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use rune::{Key, Runtime};

use crate::app::App;
use crate::game::{Cover, Game, Level, Mode};
use crate::generate::{self, Dims};
use crate::generating;
use crate::record::{Act, Layout};
use crate::storage::Saved;
use crate::toolbar_tests::{click, press, runtime, shown};

/// A gate a generation waits at until the test opens it.
struct Gate {
    open: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    const fn new() -> Gate {
        Gate { open: Mutex::new(false), changed: Condvar::new() }
    }
    fn wait(&self) {
        let mut open = self.open.lock().unwrap();
        while !*open {
            open = self.changed.wait(open).unwrap();
        }
    }
    fn open(&self) {
        *self.open.lock().unwrap() = true;
        self.changed.notify_all();
    }
}

// One gate per test: tests run at the same time.
static SLOW: Gate = Gate::new();
static CANCELLED: Gate = Gate::new();
static REVIEWED: Gate = Gate::new();

fn slow_lay(mode: Mode, dims: Dims, seed: u64, first: usize) -> Game {
    SLOW.wait();
    generate::lay(mode, dims, seed, first)
}

fn cancelled_lay(mode: Mode, dims: Dims, seed: u64, first: usize) -> Game {
    CANCELLED.wait();
    generate::lay(mode, dims, seed, first)
}

fn reviewed_lay(mode: Mode, dims: Dims, seed: u64, first: usize) -> Game {
    REVIEWED.wait();
    generate::lay(mode, dims, seed, first)
}

fn never_lay(_: Mode, _: Dims, _: u64, _: usize) -> Game {
    panic!("a record that keeps its mines is never laid again")
}

fn no_guess(level: Level) -> Saved {
    Saved { mode: Mode::NoGuess, level, ..Saved::default() }
}

/// A runtime whose poll runs every frame, as the window's does.
fn polled(saved: Saved, lay: fn(Mode, Dims, u64, usize) -> Game) -> Runtime<App> {
    let mut rt = runtime(saved);
    rt.poll = Some(|s, t, tx| s.tick(t, tx));
    rt.update(|s, _| s.lay = lay);
    rt.frame(1.0 / 60.0);
    rt
}

/// Presses and lets go on cell `i`.
fn tap(rt: &mut Runtime<App>, i: usize) {
    let (bx, by) = rt.find("board").unwrap();
    let (bw, bh) = crate::board::size(&rt.state);
    let [x, y] = rt.state.cell_center(i);
    let (px, py) = (bx - bw / 2.0 + x, by - bh / 2.0 + y);
    rt.pointer_down(px, py);
    rt.pointer_up(px, py);
    rt.frame(1.0 / 60.0);
}

fn mines(g: &Game) -> Vec<usize> {
    (0..g.cells.len()).filter(|&i| g.cells[i].mine).collect()
}

fn thinking(rt: &Runtime<App>) -> bool {
    shown(rt, "mood-Think")
}

/// Frames until the field lands, in real time (the generation runs on its own clock).
fn frame_until_laid(rt: &mut Runtime<App>) {
    let start = Instant::now();
    while rt.state.generating.is_some() {
        assert!(start.elapsed() < Duration::from_secs(20), "the field never landed");
        rt.frame(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn the_window_keeps_drawing_while_a_slow_field_is_generated() {
    let mut rt = polled(no_guess(Level::Expert), slow_lay);
    let first = rt.state.game.idx(15, 8);
    let seed = rt.state.seed;
    assert!(!thinking(&rt), "the face smiles before the click");
    tap(&mut rt, first);
    assert!(thinking(&rt), "the face thinks at once");
    assert!(!shown(&rt, "sweep"), "no sweep yet");

    // Two seconds of frames while the generation is held: each one comes back at once.
    let other = rt.state.game.idx(2, 2);
    for k in 0..120 {
        let t = Instant::now();
        rt.frame(1.0 / 60.0);
        assert!(t.elapsed() < Duration::from_millis(500), "frame {k} waited for the generation");
        let elapsed = (k + 1) as f64 / 60.0;
        // The poll that stamps the start runs a frame after the click.
        if elapsed < 0.58 {
            assert!(!shown(&rt, "sweep"), "the sweep waits for 600 ms ({elapsed:.2} s)");
        }
        if elapsed > 0.65 {
            assert!(shown(&rt, "sweep"), "the sweep runs after 600 ms ({elapsed:.2} s)");
        }
        assert!(thinking(&rt));
        assert_eq!((rt.state.started_at, rt.state.seconds), (None, 0), "the timer waits");
        if k == 60 {
            // The field takes no input meanwhile: no open, no flag.
            tap(&mut rt, other);
            rt.update(|s, tx| s.on_board_secondary(crate::style::WELL + 1.0, crate::style::WELL + 1.0, tx));
            rt.frame(1.0 / 60.0);
        }
    }
    assert!(rt.state.game.cells.iter().all(|c| c.cover == Cover::Hidden), "nothing opened or flagged while generating");
    assert!(rt.state.generating.is_some());

    SLOW.open();
    frame_until_laid(&mut rt);
    assert!(!shown(&rt, "sweep"), "the sweep stops in the frame the field lands");
    assert!(!thinking(&rt));
    assert_eq!(mines(&rt.state.game), mines(&generate::lay(Mode::NoGuess, (30, 16, 99), seed, first)), "the field its inputs name");
    assert!(rt.state.game.opened > 1, "the first click opens an area");
    assert_eq!(rt.state.game.cells[other].cover, Cover::Hidden, "the ignored click stays ignored");
    // The timer starts when the field is ready.
    let landed = rt.state.now;
    assert_eq!((rt.state.started_at, rt.state.seconds), (Some(landed), 1));
    // Its record names the field by seed and first click, and starts with that click.
    let laid = Some(mines(&rt.state.game));
    assert_eq!(rt.state.record.layout, Layout::Seeded { seed, first, laid });
    assert_eq!((rt.state.record.moves[0].act, rt.state.record.moves[0].cell), (Act::Open, first));
}

#[test]
fn f2_or_the_face_cancels_and_the_next_first_click_starts_afresh() {
    let mut rt = polled(no_guess(Level::Beginner), cancelled_lay);
    for cancel in ["F2", "face"] {
        // A fresh field, generated by the held stand-in.
        press(&mut rt, Key::Function(2));
        rt.update(|s, _| s.lay = cancelled_lay);
        rt.frame(1.0 / 60.0);
        let (seed, field, kept) = (rt.state.seed, rt.state.field, rt.state.history.len());
        tap(&mut rt, 40);
        assert!(rt.state.generating.is_some() && thinking(&rt));
        if cancel == "F2" {
            press(&mut rt, Key::Function(2));
        } else {
            click(&mut rt, "face");
        }
        rt.frame(1.0 / 60.0);
        assert!(rt.state.generating.is_none(), "{cancel} cancels");
        assert!(!thinking(&rt) && !shown(&rt, "sweep"), "{cancel}: back to a smile");
        assert!(rt.state.field > field && rt.state.seed != seed, "{cancel}: a fresh field");
        assert!(rt.state.unlaid && mines(&rt.state.game).is_empty());
        assert_eq!(rt.state.history.len(), kept, "{cancel}: a field that never landed is never recorded");

        // The next first click starts a fresh generation, of the new seed.
        rt.update(|s, _| s.lay = generate::lay);
        let seed = rt.state.seed;
        tap(&mut rt, 10);
        frame_until_laid(&mut rt);
        assert_eq!(mines(&rt.state.game), mines(&generate::lay(Mode::NoGuess, (9, 9, 10), seed, 10)), "{cancel}");
    }
    // The cancelled generations finish now, and land nowhere.
    let field = mines(&rt.state.game);
    CANCELLED.open();
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(mines(&rt.state.game), field, "a cancelled field never lands");
}

#[test]
fn a_field_sent_as_a_mine_mask_is_the_field_laid() {
    // The web's worker sends a field back as one byte per cell.
    for (dims, seeds) in [((9, 9, 10), 20), ((16, 16, 40), 10), ((30, 16, 99), 3), ((20, 12, 50), 5)] {
        for seed in 0..seeds {
            let first = (seed as usize * 31) % (dims.0 * dims.1);
            let g = generate::lay(Mode::NoGuess, dims, seed, first);
            let back = generating::from_mask(dims.0, dims.1, &generating::mask(&g));
            assert_eq!(mines(&back), mines(&g), "{dims:?} seed {seed}");
            assert_eq!((back.w, back.h, back.mines), (g.w, g.h, g.mines));
            assert_eq!(back.cells, g.cells);
        }
    }
}

/// A No Guess game played to a loss on Beginner, as History keeps it.
fn lost_no_guess(rt: &mut Runtime<App>) -> crate::record::Record {
    tap(rt, 40);
    frame_until_laid(rt);
    let mine = (0..rt.state.game.cells.len()).find(|&i| rt.state.game.cells[i].mine).unwrap();
    rt.update(|s, _| s.open_at(mine));
    rt.frame(1.0 / 60.0);
    rt.state.history.last().expect("a record").clone()
}

#[test]
fn a_record_that_keeps_its_mines_opens_in_review_without_laying_anything() {
    let mut rt = polled(no_guess(Level::Beginner), generate::lay);
    let r = lost_no_guess(&mut rt);
    assert!(matches!(r.layout, Layout::Seeded { laid: Some(_), .. }), "{:?}", r.layout);
    rt.update(|s, _| s.lay = never_lay);
    rt.update(|s, _| s.open_review(s.history.len() - 1));
    let rv = rt.state.review.as_ref().expect("open at once");
    assert_eq!(rv.record, r);
    assert_eq!(rv.game.cells, rt.state.game.cells, "the field as it ended");
}

#[test]
fn an_older_record_is_laid_off_the_ui_thread_before_its_review_opens() {
    let mut rt = polled(no_guess(Level::Beginner), generate::lay);
    let mut r = lost_no_guess(&mut rt);
    let ended = rt.state.game.cells.clone();
    // Written before records kept their mines: only the seed and first click.
    let Layout::Seeded { seed, first, .. } = r.layout else { panic!("{:?}", r.layout) };
    r.layout = Layout::Seeded { seed, first, laid: None };
    let history = vec![r.clone()];
    rt.update(|s, _| {
        s.history = history;
        s.lay = reviewed_lay;
    });
    rt.frame(1.0 / 60.0);
    let t = Instant::now();
    rt.update(|s, _| s.open_review(0));
    rt.frame(1.0 / 60.0);
    assert!(t.elapsed() < Duration::from_millis(500), "opening waited for the field");
    assert!(rt.state.review.is_none(), "the review waits for its field");
    assert!(thinking(&rt), "the face thinks meanwhile");
    for _ in 0..30 {
        let t = Instant::now();
        rt.frame(1.0 / 60.0);
        assert!(t.elapsed() < Duration::from_millis(500), "a frame waited for the field");
    }
    REVIEWED.open();
    let start = Instant::now();
    while rt.state.review.is_none() {
        assert!(start.elapsed() < Duration::from_secs(20), "the review never opened");
        rt.frame(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!thinking(&rt), "the face stops thinking once it lands");
    let rv = rt.state.review.as_ref().unwrap();
    assert_eq!(rv.game.cells, ended, "the field as it ended");
    assert_eq!(rt.state.history, [r], "reviewing never changes the record");
}
