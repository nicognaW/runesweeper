//! Runesweeper: Minesweeper on Rune.

pub mod app;
pub mod board;
pub mod callout;
pub mod fonts;
pub mod game;
pub mod generate;
pub mod generating;
pub mod gfx;
pub mod hint;
pub mod history;
pub mod icons;
pub mod locale;
pub mod record;
pub mod review;
pub mod solver;
pub mod storage;
pub mod strings;
pub mod style;
pub mod timeline;
pub mod toolbar;
pub mod ui;

#[cfg(test)]
mod toolbar_tests;
#[cfg(test)]
mod lang_tests;
#[cfg(test)]
mod generate_tests;
#[cfg(test)]
mod mode_tests;
#[cfg(test)]
mod record_tests;
#[cfg(test)]
mod hint_tests;
#[cfg(test)]
mod generating_tests;
#[cfg(test)]
mod max_guess_tests;
#[cfg(test)]
mod review_tests;
#[cfg(test)]
mod motion_tests;
#[cfg(test)]
mod test_fields;

/// README's Layout section names every source file and folder, so an agent can find its way.
#[cfg(test)]
#[test]
fn readme_layout_names_every_source_file() {
    let readme = include_str!("../README.md");
    let layout = &readme[readme.find("## Layout").expect("README has a Layout section")..];
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(&src).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        let named = if src.join(&name).is_dir() { format!("`src/{name}/") } else { format!("`src/{name}`") };
        let covered = layout.contains(&named) || (name.ends_with("_tests.rs") && layout.contains("`src/*_tests.rs`"));
        assert!(covered, "README's Layout section doesn't mention {named}");
    }
}

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

/// Plays the field as a strong player with lapses: proofs first, flagging mines and chording
/// once, a long pause on the 8th move and a needless guess on the 15th; then records the game.
fn reviewed_game(a: &mut App) {
    use solver::{Fact, Strength, View, deduce};
    a.open_at(a.game.idx(a.game.w / 2, a.game.h / 2));
    for k in 0..600 {
        if a.game.over() {
            break;
        }
        a.now += if k == 8 { 7.5 } else { 0.7 };
        let v = View::of(&a.game);
        let mut facts = vec![Fact::Unknown; v.len()];
        loop {
            let proofs = deduce(&v, &facts, Strength::Full);
            if proofs.is_empty() {
                break;
            }
            for p in proofs {
                facts[p.cell] = if p.mine { Fact::Mine } else { Fact::Safe };
            }
        }
        let hidden = |i: usize| a.game.cells[i].cover == game::Cover::Hidden;
        let safe = (0..v.len()).find(|&i| facts[i] == Fact::Safe);
        if let (15, Some(_)) = (k, safe) {
            let guess = (0..v.len()).find(|&i| hidden(i) && facts[i] == Fact::Unknown && !a.game.cells[i].mine);
            if let Some(i) = guess {
                a.open_at(i);
                continue;
            }
        }
        if let Some(i) = safe {
            a.open_at(i);
        } else if let Some(i) = (0..v.len()).find(|&i| facts[i] == Fact::Mine && hidden(i)) {
            a.mark_at(i);
        } else {
            let known: Vec<Fact> = facts.iter().map(|&f| if f == Fact::Mine { f } else { Fact::Unknown }).collect();
            let Some(i) = solver::play::guess(&v, &known) else { break };
            a.open_at(i);
        }
    }
    a.new_game_now();
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
    ("no-guess", |a| {
        let saved = Saved { mode: game::Mode::NoGuess, level: Level::Expert, ..Saved::default() };
        *a = App::with(saved, 5);
        a.open_at(a.game.idx(15, 8));
        a.finish_laying();
    }),
    ("max-guess", |a| {
        let saved = Saved { mode: game::Mode::MaxGuess, level: Level::Intermediate, ..Saved::default() };
        *a = App::with(saved, 5);
        a.open_at(a.game.idx(8, 8));
        a.finish_laying();
    }),
    // Max Guess lost on a true 50/50: the banner says it was forced.
    ("max-guess-forced", |a| {
        SCENES.iter().find(|s| s.0 == "max-guess").unwrap().1(a);
        if let Some(&(x, y)) = a.fifty_fifties.first() {
            a.open_at(if a.game.cells[x].mine { x } else { y });
            a.moments.boom = Some((0.05, a.cell_center(a.game.hit.unwrap_or(x))));
        }
    }),
    // A field that takes its time: the face thinks, and after 600 ms the light sweeps. It is
    // held for an hour, so it never lands while you look.
    ("generating", |a| {
        let saved = Saved { mode: game::Mode::NoGuess, level: Level::Expert, ..Saved::default() };
        *a = App::with(saved, 5);
        a.lay = |mode, dims, seed, first| {
            std::thread::sleep(std::time::Duration::from_secs(3600));
            generate::lay(mode, dims, seed, first)
        };
        a.open_at(a.game.idx(15, 8));
    }),
    ("picker", |a| {
        *a = scene(Level::Beginner, 7);
        play(a, &[(4, 4)]);
        a.toggle_panel(app::Panel::Picker);
    }),
    ("history", |a| {
        *a = scene(Level::Beginner, 7);
        let rec = |minutes: u64, end, seconds, forced, survived| record::Record {
            at: 1_793_000_000_000 + minutes * 60_000,
            end,
            seconds,
            three_bv: 31,
            three_bv_solved: if end == record::End::Won { 31 } else { 17 },
            left: 38,
            right: 9,
            chords: 4,
            forced,
            survived,
            ..record::Record::new(game::Mode::Classic, (9, 9, 10))
        };
        a.history = vec![
            rec(0, record::End::Won, 41, 1, 1),
            rec(9, record::End::Lost, 12, 2, 1),
            rec(15, record::End::Won, 33, 0, 0),
            rec(20, record::End::Abandoned, 7, 0, 0),
            rec(31, record::End::Won, 38, 1, 1),
        ];
        a.toggle_panel(app::Panel::History);
    }),
    ("hint", |a| {
        *a = scene(Level::Intermediate, 11);
        play(a, &[(8, 8)]);
        for _ in 0..4 {
            a.press_hint();
        }
    }),
    ("hint-zh-cn", |a| {
        SCENES.iter().find(|s| s.0 == "hint").unwrap().1(a);
        a.set_lang(strings::Lang::ZhCn);
    }),
    // A Review of an Intermediate game played by a bot that pauses and guesses where it
    // needn't: a missed deduction's reason beside the field, the timeline under it.
    ("review", |a| {
        // The first seed whose game runs long enough to have a few markers.
        for seed in 1.. {
            *a = scene(Level::Intermediate, seed);
            reviewed_game(a);
            if a.history[0].moves.len() >= 60 {
                break;
            }
        }
        a.open_review(0);
        // A missed deduction midway shows more of the field than the opening guess.
        let markers = &a.review.as_ref().unwrap().markers;
        let k = markers.iter().position(|m| matches!(m.kind, review::Kind::Missed { .. })).unwrap_or(0);
        a.review_select(k);
    }),
    // A loss through the App, so History has it: the banner offers a Review.
    ("lost-review", |a| {
        *a = scene(Level::Beginner, 7);
        a.open_at(a.game.idx(4, 4));
        let mine = (0..a.game.cells.len()).find(|&i| a.game.cells[i].mine).unwrap();
        a.open_at(mine);
        a.moments.boom = Some((0.05, a.cell_center(mine)));
    }),
    ("review-zh-cn", |a| {
        SCENES.iter().find(|s| s.0 == "review").unwrap().1(a);
        a.set_lang(strings::Lang::ZhCn);
    }),
    ("settings", |a| {
        *a = scene(Level::Beginner, 7);
        a.toggle_panel(app::Panel::Settings);
    }),
    ("settings-zh-cn", |a| {
        let saved = Saved { lang: Some(strings::Lang::ZhCn), ..Saved::default() };
        *a = App::with(saved, 7);
        play(a, &[(4, 4)]);
        a.toggle_panel(app::Panel::Settings);
    }),
    ("lost-zh-cn", |a| {
        SCENES.iter().find(|s| s.0 == "lost").unwrap().1(a);
        a.set_lang(strings::Lang::ZhCn);
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
        setup: Some(|s, waker| s.waker = Some(waker)),
        poll: Some(|s, t, tx| s.tick(t, tx)),
        on_key: Some(|s, e, tx| s.key(e, tx)),
        fonts: fonts::LIST,
        families: fonts::FAMILIES,
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
    // In the field generator's worker, the same wasm runs no app.
    #[cfg(target_arch = "wasm32")]
    if web_sys::window().is_some() {
        run();
    }
}
