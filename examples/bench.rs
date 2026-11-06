//! CPU cost of a frame without a GPU: view, diff, layout and draw lists on Rune's runtime.
//! `cargo run --release --example bench`
use std::time::Instant;

use rune::{Env, Runtime};
use runesweeper::{app::App, game::Level, storage::Saved, ui};

const N: usize = 240;

fn main() {
    let saved = Saved { level: Level::Expert, played: true, ..Saved::default() };
    let mut rt = Runtime::new(App::with(saved, 5), ui::view, (868.0, 598.0), Env { system_dark: true, scale: 2.0 });
    rt.frame(0.0);
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
    }
    let (bx, by) = rt.find("board").unwrap();
    // Sweep the whole field: from its left edge across, a row band at a time.
    let (bw, bh) = runesweeper::board::size(&rt.state);
    let at = move |k: usize| (bx - bw / 2.0 + (k % 60) as f32 * bw / 60.0, by - bh / 2.0 + (k / 60) as f32 * bh / 4.0);
    let times = |label: &str, rt: &mut Runtime<App>, f: &mut dyn FnMut(&mut Runtime<App>, usize)| {
        let mut v: Vec<f64> = (0..N)
            .map(|k| {
                let t = Instant::now();
                f(rt, k);
                rt.frame(1.0 / 60.0);
                t.elapsed().as_secs_f64() * 1000.0
            })
            .collect();
        v.sort_by(f64::total_cmp);
        println!("{label:<22} p50 {:6.3} ms  p95 {:6.3} ms  max {:6.3} ms", v[N / 2], v[N * 95 / 100], v[N - 1]);
    };
    times("idle frame", &mut rt, &mut |_, _| {});
    times("hover sweep", &mut rt, &mut |rt, k| { let (x, y) = at(k); rt.pointer_move(x, y) });
    rt.pointer_down(bx, by);
    times("held press drag", &mut rt, &mut |rt, k| { let (x, y) = at(k); rt.pointer_move(x, y) });
    rt.pointer_up(bx, by);
}
