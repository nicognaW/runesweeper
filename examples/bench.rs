//! CPU cost of a frame without a GPU: view, diff, layout and draw lists on Rune's runtime.
//! `cargo run --release --example bench`
use std::time::Instant;

use rune::{Env, Runtime};
use runesweeper::{app::App, game::Level, storage::Saved, ui};

fn main() {
    let saved = Saved { level: Level::Expert, played: true, ..Saved::default() };
    let mut rt = Runtime::new(App::with(saved, 5), ui::view, (868.0, 598.0), Env { system_dark: true, scale: 2.0 });
    rt.frame(0.0);
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
    }
    let (bx, by) = rt.find("board").unwrap();
    let mut times = |label: &str, rt: &mut Runtime<App>, f: &mut dyn FnMut(&mut Runtime<App>, usize)| {
        let mut v: Vec<f64> = (0..240)
            .map(|k| {
                let t = Instant::now();
                f(rt, k);
                rt.frame(1.0 / 60.0);
                t.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("{label:<22} p50 {:6.3} ms  p95 {:6.3} ms  max {:6.3} ms", v[120], v[228], v[239]);
    };
    times("idle frame", &mut rt, &mut |_, _| {});
    times("hover sweep", &mut rt, &mut |rt, k| rt.pointer_move(bx - 360.0 + (k % 60) as f32 * 12.0, by - 150.0 + (k / 60) as f32 * 60.0));
    rt.pointer_down(bx, by);
    times("held press drag", &mut rt, &mut |rt, k| rt.pointer_move(bx - 360.0 + (k % 60) as f32 * 12.0, by - 150.0 + (k / 60) as f32 * 60.0));
    rt.pointer_up(bx, by);
}
