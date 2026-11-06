//! Vector art: the mine, the flag, the face's features and seven-segment digits, all as Rune
//! shapes so they stay sharp at any scale.

use rune::shape::{LineCap, Path, Shape};
use rune::{Color, El, el};

use crate::style::ca;

fn circle(cx: f32, cy: f32, r: f32, n: usize) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            [cx + r * a.cos(), cy + r * a.sin()]
        })
        .collect()
}

/// The mine: a ball with eight spikes and a glint, in a `s`×`s` box.
pub fn mine(s: f32, body: Color) -> El {
    let (m, r) = (s / 2.0, s * 0.27);
    let mut shape = Shape::new();
    for k in 0..4 {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        let (dx, dy) = (a.cos() * s * 0.42, a.sin() * s * 0.42);
        let w = if k % 2 == 0 { s * 0.11 } else { s * 0.085 };
        shape = shape.line([m - dx, m - dy], [m + dx, m + dy], w, LineCap::Round, body);
    }
    shape = shape
        .fill(circle(m, m, r, 28), body)
        .fill(circle(m - r * 0.36, m - r * 0.36, r * 0.3, 14), ca(0xffffff, 0.9));
    el().size(s, s).shape(shape)
}

/// The flag on its stand, in a `s`×`s` box.
pub fn flag(s: f32, cloth: Color, pole: Color) -> El {
    let u = s / 16.0;
    let shape = Shape::new()
        .fill(vec![[9.6 * u, 1.6 * u], [9.6 * u, 9.4 * u], [2.0 * u, 5.5 * u]], cloth)
        .fill(vec![[9.6 * u, 1.6 * u], [9.6 * u, 4.2 * u], [5.0 * u, 3.9 * u]], ca(0xffffff, 0.22))
        .line([9.6 * u, 1.6 * u], [9.6 * u, 12.4 * u], 1.6 * u, LineCap::Butt, pole)
        .fill(vec![[6.4 * u, 11.4 * u], [12.8 * u, 11.4 * u], [14.0 * u, 14.2 * u], [5.2 * u, 14.2 * u]], pole);
    el().size(s, s).shape(shape)
}

/// A red cross over a wrongly flagged mine.
pub fn cross(s: f32, color: Color) -> El {
    let (a, b) = (s * 0.14, s * 0.86);
    let shape = Shape::new()
        .line([a, a], [b, b], s * 0.1, LineCap::Round, color)
        .line([b, a], [a, b], s * 0.1, LineCap::Round, color);
    el().size(s, s).shape(shape)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mood {
    Smile,
    Oh,
    Dead,
    Cool,
}

/// The face's features over its plastic dome, in a `s`×`s` box.
pub fn face(s: f32, mood: Mood, ink: Color) -> El {
    let u = s / 32.0;
    let p = |x: f32, y: f32| [x * u, y * u];
    let w = 1.5 * u;
    // An almond eye: an ellipse pinched toward its outer corner, tilted up and out.
    let almond = |cx: f32, cy: f32, ew: f32, eh: f32, outer: f32| -> Vec<[f32; 2]> {
        let tilt = 0.42 * -outer;
        (0..28)
            .map(|k| {
                let t = k as f32 / 28.0 * std::f32::consts::TAU;
                let (c, sn) = (t.cos(), t.sin());
                let pinch = 1.0 - 0.5 * (outer * c).max(0.0);
                let (x, y) = (ew / 2.0 * c, eh / 2.0 * sn * pinch);
                let (x, y) = (x * tilt.cos() - y * tilt.sin(), x * tilt.sin() + y * tilt.cos());
                p(cx + x, cy + y)
            })
            .collect()
    };
    let glint = ca(0xffffff, 0.85);
    let mut shape = Shape::new();
    match mood {
        Mood::Smile | Mood::Oh => {
            let (ew, eh) = if mood == Mood::Oh { (10.6, 7.8) } else { (10.4, 6.2) };
            for (cx, outer) in [(10.3, -1.0), (21.7, 1.0)] {
                shape = shape.fill(almond(cx, 14.2, ew, eh, outer), ink).fill(circle((cx - 1.6 * outer.signum() * -1.0) * u, 13.0 * u, 0.9 * u, 10), glint);
            }
        }
        Mood::Dead => {
            for cx in [10.8, 21.2] {
                shape = shape
                    .line(p(cx - 2.6, 11.8), p(cx + 2.6, 16.4), w * 1.2, LineCap::Round, ink)
                    .line(p(cx + 2.6, 11.8), p(cx - 2.6, 16.4), w * 1.2, LineCap::Round, ink);
            }
        }
        Mood::Cool => {
            // A wraparound visor, lit along its top edge.
            let visor = vec![p(4.2, 11.4), p(27.8, 11.4), p(26.4, 16.6), p(16.0, 18.4), p(5.6, 16.6)];
            shape = shape
                .fill(visor, ink)
                .line(p(6.6, 12.9), p(13.0, 12.5), 0.9 * u, LineCap::Round, ca(0x3df5e0, 0.95))
                .line(p(19.0, 12.5), p(25.4, 12.9), 0.9 * u, LineCap::Round, ca(0x3df5e0, 0.95));
        }
    }
    shape = match mood {
        Mood::Oh => shape.fill(circle(16.0 * u, 23.6 * u, 1.7 * u, 16), ink),
        Mood::Dead => shape.stroke_path(&Path::new().move_to(p(12.8, 24.2)).quad_to(p(16.0, 22.4), p(19.2, 24.2)), w, LineCap::Round, ink),
        _ => shape.stroke_path(&Path::new().move_to(p(13.4, 22.6)).quad_to(p(16.0, 24.6), p(18.6, 22.6)), w, LineCap::Round, ink),
    };
    el().size(s, s).shape(shape)
}

/// A hex lattice `w`×`h`, brightest at the centre and fading out to the edges.
pub fn lattice(w: f32, h: f32, ink: Color) -> El {
    let r = 18.0f32;
    let (dx, dy) = (3f32.sqrt() * r, 1.5 * r);
    let (cx, cy, reach) = (w / 2.0, h / 2.0, (w * w + h * h).sqrt() / 2.0);
    let mut sh = Shape::new();
    let corner = |x: f32, y: f32, k: usize| {
        let a = std::f32::consts::FRAC_PI_6 + k as f32 * std::f32::consts::FRAC_PI_3;
        [x + r * a.cos(), y + r * a.sin()]
    };
    let rows = (h / dy) as i32 + 2;
    let cols = (w / dx) as i32 + 2;
    for row in -1..rows {
        for col in -1..cols {
            let x = col as f32 * dx + if row % 2 != 0 { dx / 2.0 } else { 0.0 };
            let y = row as f32 * dy;
            let d = ((x - cx).hypot(y - cy) / reach).min(1.0);
            let a = ink.a * (1.0 - d).powf(1.3);
            if a < 0.004 {
                continue;
            }
            // Three edges per cell; the neighbours draw the other three.
            for k in 0..3 {
                sh = sh.line(corner(x, y, k), corner(x, y, k + 1), 1.0, LineCap::Butt, ink.alpha(a));
            }
        }
    }
    el().size(w, h).shape(sh)
}

/// Which of a seven-segment digit's segments are lit, `abcdefg`.
fn segments(ch: char) -> [bool; 7] {
    let bits: u8 = match ch {
        '0' => 0b1111110,
        '1' => 0b0110000,
        '2' => 0b1101101,
        '3' => 0b1111001,
        '4' => 0b0110011,
        '5' => 0b1011011,
        '6' => 0b1011111,
        '7' => 0b1110000,
        '8' => 0b1111111,
        '9' => 0b1111011,
        '-' => 0b0000001,
        _ => 0,
    };
    std::array::from_fn(|i| bits & (1 << (6 - i)) != 0)
}

/// A seven-segment digit `w`×`h`, lit segments in `on` and the rest faintly in `off`.
pub fn digit(ch: char, w: f32, h: f32, on: Color, off: Color) -> El {
    let t = w * 0.2; // segment thickness
    let g = t * 0.18; // gap between segments
    let m = h / 2.0;
    // Horizontal segment centred at y, vertical at x from y0 to y1, both as hexagons.
    let hseg = |y: f32| {
        let (x0, x1) = (t * 0.5 + g, w - t * 0.5 - g);
        vec![[x0, y], [x0 + t * 0.5, y - t * 0.5], [x1 - t * 0.5, y - t * 0.5], [x1, y], [x1 - t * 0.5, y + t * 0.5], [x0 + t * 0.5, y + t * 0.5]]
    };
    let vseg = |x: f32, y0: f32, y1: f32| {
        let (y0, y1) = (y0 + g, y1 - g);
        vec![[x, y0], [x + t * 0.5, y0 + t * 0.5], [x + t * 0.5, y1 - t * 0.5], [x, y1], [x - t * 0.5, y1 - t * 0.5], [x - t * 0.5, y0 + t * 0.5]]
    };
    let (l, r) = (t * 0.5, w - t * 0.5);
    let (top, bot) = (t * 0.5, h - t * 0.5);
    let polys = [hseg(top), vseg(r, top, m), vseg(r, m, bot), hseg(bot), vseg(l, m, bot), vseg(l, top, m), hseg(m)];
    let lit = segments(ch);
    let mut shape = Shape::new();
    for (p, on_) in polys.into_iter().zip(lit) {
        shape = shape.fill(p, if on_ { on } else { off });
    }
    el().size(w, h).shape(shape)
}

/// Three digits for a counter, as the original shows them: 000 to 999, and -99 to -01.
pub fn led_text(v: i32) -> String {
    let v = v.clamp(-99, 999);
    if v < 0 { format!("-{:02}", -v) } else { format!("{v:03}") }
}

#[cfg(test)]
mod tests {
    #[test]
    fn counters_read_like_the_original() {
        assert_eq!(super::led_text(10), "010");
        assert_eq!(super::led_text(-3), "-03");
        assert_eq!(super::led_text(-150), "-99");
        assert_eq!(super::led_text(1500), "999");
    }
}
