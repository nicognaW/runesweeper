//! Runesweeper: Minesweeper on Rune.

pub mod app;
pub mod board;
pub mod game;
pub mod gfx;
pub mod icons;
pub mod solver;
pub mod storage;
pub mod style;
pub mod toolbar;
pub mod ui;

#[cfg(test)]
mod toolbar_tests;

use app::App;
use game::Level;
use rune::AppSpec;
use storage::Saved;

fn scene(level: Level, seed: u32) -> App {
    let saved = Saved { level, ..Saved::default() };
    App::with(saved, seed)
}

/// Opens squares until a few numbers show, for snapshots.
fn play(a: &mut App, opens: &[(usize, usize)]) {
    for &(x, y) in opens {
        // The nearest blank square, so each open floods.
        let g = &a.game;
        let near = (0..g.cells.len()).filter(|&i| !g.cells[i].mine && g.cells[i].adj == 0).min_by_key(|&i| {
            let (dx, dy) = ((i % g.w) as i64 - x as i64, (i / g.w) as i64 - y as i64);
            dx * dx + dy * dy
        });
        if let Some(i) = near {
            let _ = a.game.open(i);
        }
    }
    // A clock that holds still for the picture.
    a.seconds = 42;
}

const SCENES: &[(&str, fn(&mut App))] = &[
    ("beginner", |a| *a = scene(Level::Beginner, 7)),
    ("intermediate", |a| {
        *a = scene(Level::Intermediate, 11);
        play(a, &[(8, 8)]);
    }),
    ("expert", |a| {
        *a = scene(Level::Expert, 5);
        play(a, &[(15, 8), (3, 3), (26, 12)]);
        for i in 0..a.game.cells.len() {
            let c = a.game.cells[i];
            if c.mine && a.game.neighbours(i).any(|j| a.game.cells[j].cover == game::Cover::Open) {
                a.game.mark(i, true);
            }
        }
    }),
    ("lost", |a| {
        *a = scene(Level::Intermediate, 11);
        play(a, &[(8, 8)]);
        let m = (0..a.game.cells.len()).find(|&i| a.game.cells[i].mine).unwrap();
        a.game.mark((0..a.game.cells.len()).find(|&i| !a.game.cells[i].mine && a.game.cells[i].cover == game::Cover::Hidden).unwrap(), true);
        let _ = a.game.open(m);
        a.moments.boom = Some((0.05, a.cell_center(m)));
    }),
    ("won", |a| {
        *a = scene(Level::Beginner, 7);
        for i in 0..a.game.cells.len() {
            if !a.game.cells[i].mine {
                let _ = a.game.open(i);
            }
        }
        a.seconds = 37;
        a.new_best = true;
        a.moments.won = Some(0.05);
    }),
    ("picker", |a| {
        *a = scene(Level::Beginner, 7);
        play(a, &[(4, 4)]);
        a.toggle_panel(app::Panel::Picker);
    }),
    ("settings", |a| {
        *a = scene(Level::Beginner, 7);
        a.toggle_panel(app::Panel::Settings);
    }),
    ("icon", |a| {
        *a = scene(Level::Beginner, 7);
        a.icon = true;
    }),
    ("mono", |a| {
        *a = scene(Level::Intermediate, 11);
        a.saved.color = false;
        play(a, &[(8, 8)]);
    }),
    // States for checking every screen by eye.
    ("flagging", |a| {
        *a = scene(Level::Beginner, 7);
        a.touch = true;
        a.flag_mode = true;
    }),
    ("press", |a| {
        *a = scene(Level::Intermediate, 11);
        play(a, &[(8, 8)]);
        let i = (0..a.game.cells.len()).find(|&i| a.game.cells[i].cover == game::Cover::Hidden && a.game.neighbours(i).all(|j| a.game.cells[j].cover == game::Cover::Hidden)).unwrap();
        a.press = app::Press { left: true, cell: Some(i), ..Default::default() };
    }),
    ("chord", |a| {
        *a = scene(Level::Intermediate, 11);
        play(a, &[(8, 8)]);
        let i = (0..a.game.cells.len()).find(|&i| a.game.cells[i].cover == game::Cover::Open && a.game.cells[i].adj > 0).unwrap();
        a.press = app::Press { middle: true, cell: Some(i), ..Default::default() };
    }),
    ("marks", |a| {
        *a = scene(Level::Intermediate, 11);
        play(a, &[(8, 8)]);
        let hidden: Vec<usize> = (0..a.game.cells.len()).filter(|&i| a.game.cells[i].cover == game::Cover::Hidden).collect();
        for &i in hidden.iter().take(48).step_by(3) {
            a.game.mark(i, true);
        }
        for &i in hidden.iter().skip(1).take(30).step_by(5) {
            a.game.mark(i, true);
            a.game.mark(i, true);
        }
    }),
    ("over-flagged", |a| {
        *a = scene(Level::Beginner, 7);
        play(a, &[(4, 4)]);
        let hidden: Vec<usize> = (0..a.game.cells.len()).filter(|&i| a.game.cells[i].cover == game::Cover::Hidden).collect();
        for &i in hidden.iter().take(23) {
            a.game.mark(i, false);
        }
        a.seconds = 999;
    }),
    ("mono-lost", |a| {
        SCENES.iter().find(|s| s.0 == "lost").unwrap().1(a);
        a.saved.color = false;
    }),
    ("mono-won", |a| {
        SCENES.iter().find(|s| s.0 == "won").unwrap().1(a);
        a.saved.color = false;
    }),
    ("custom-max", |a| {
        let saved = Saved { level: Level::Custom, custom: (30, 24, 667), ..Saved::default() };
        *a = App::with(saved, 3);
        play(a, &[(0, 0), (29, 23)]);
    }),
];

pub fn spec() -> AppSpec<App> {
    let init = App::new;
    AppSpec {
        scenes: SCENES,
        min_size: Some((300.0, 300.0)),
        poll: Some(|s, t, tx| s.tick(t, tx)),
        on_key: Some(|s, e, tx| s.key(e, tx)),
        ..AppSpec::new("Runesweeper", initial_size(), init, ui::view)
    }
}

fn initial_size() -> (f32, f32) {
    let saved = storage::load();
    let (w, h, _) = saved.level.dims().unwrap_or(saved.custom);
    style::window_size(w, h)
}

/// Effects off: `--plain` on desktop, `?plain` on the web (for GPUs that struggle).
fn plain() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window().and_then(|w| w.location().search().ok()).is_some_and(|q| q.contains("plain"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::args().any(|a| a == "--plain")
    }
}

pub fn run() {
    // Shift on its own toggles the xyzzy cheat.
    let app = rune::App::new(spec()).modifier_keys();
    if plain() { app.run() } else { app.compositor(gfx::Gfx::default()).run() }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    run();
}
