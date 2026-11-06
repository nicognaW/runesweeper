//! The compositor: every `El::effect` slot in the view becomes one rune-fx layer. A slot's
//! mark names the effect; its `effect_data` carries the few numbers that change (a start time,
//! a point, a preset). Its presented rect and corner radius are the effect's shape.

use rune::compose::{Compositor, Frame};
use rune::render::{EffectPhase, EffectSite, KIND_EFFECT};
use rune_fx::*;

use crate::app::App;

pub const BACKDROP: u32 = 1;
pub const GRAIN: u32 = 2;
/// data: [preset (0 chrome, 1 gold, 2 rose), border pt, 0, 0]
pub const RIM: u32 = 3;
/// data: [colour hex as f32 bits.., intensity, radius, 0]
pub const LED: u32 = 4;
/// data: [colour index, 0, 0, 0]
pub const FACE: u32 = 5;
/// data: [x, y (slot points), t0, amount]
pub const SHOCK: u32 = 6;
pub const FIRE: u32 = 7;
/// data: [t0 (burst) or -1, count, size, 0]
pub const SPARKLES: u32 = 8;
/// data: [t0, 0, 0, 0]
pub const AURA: u32 = 9;
/// data: [x, y (slot points), t0, colour index]
pub const RIPPLE: u32 = 10;
/// data: [intensity, 0, 0, 0]
pub const SELECT: u32 = 11;
/// data: [t0, 0, 0, 0]: a red flash that cools.
pub const HEAT: u32 = 12;
pub const SHIMMER: u32 = 13;

const COLORS: [u32; 8] = [0xffc531, 0x6aa8ff, 0xff3b4e, 0x4ade80, 0xffffff, 0xd8d8d8, 0xa78bfa, 0xff2d2d];

fn col(i: f32) -> Rgb {
    rgb(COLORS[(i.max(0.0) as usize).min(COLORS.len() - 1)])
}

/// Effects that live on the ambient clock, which slows to a stop while nobody plays.
fn ambient(kind: u32) -> bool {
    matches!(kind, BACKDROP | RIM | SHIMMER | SELECT | AURA | FIRE | SPARKLES | LED | FACE | GRAIN)
}

/// `t` is the real clock (one-shots start on it); `amb` the ambient one.
fn layer(kind: u32, d: [f32; 4], shape: Shape, origin: [f32; 2], t: f32, amb: f32) -> Option<FxLayer> {
    let fx = match kind {
        BACKDROP => Fx::fill(MeshGradientParams {
            colors: if d[0] > 0.5 {
                Stops::hex(&[0x0b0b0d, 0x26262b, 0x111114, 0x1d1d22])
            } else {
                Stops::hex(&[0x05060e, 0x1a1f5c, 0x0b2a4a, 0x3a1450, 0x070914])
            },
            speed: 0.12,
            drift: 0.18,
            ..MeshGradientParams::dusk()
        }),
        GRAIN => Fx::grain(GrainParams { amount: 0.035, ..Default::default() }),
        RIM => {
            let p = match d[0] as u32 {
                1 => LiquidMetalParams::gold(),
                2 => LiquidMetalParams::rose(),
                3 => LiquidMetalParams::gunmetal(),
                _ => LiquidMetalParams { light: rgb(0xd6e8ff), iridescence: 0.25, ..LiquidMetalParams::chrome() },
            };
            Fx::fill(LiquidMetalParams { flow: 0.35, speed: 0.25, ..p }.border(d[1]))
        }
        LED => Fx::glow(GlowParams { style: GlowStyle::Soft, color: col(d[0]), intensity: d[1], radius: d[2], occlusion: 0.0, ..Default::default() }),
        FACE => Fx::fill(PlasticParams { color: col(d[0]), roughness: 0.22, dome: 0.55, bevel: 7.0, angle: -0.7 }),
        SHOCK => Fx::distort(DistortParams::Shockwave { at: [origin[0] + d[0], origin[1] + d[1]], reach: 900.0, width: 70.0, amount: d[3], chroma: 0.7, t0: d[2], duration: 1.0 }),
        FIRE => Fx::fire(FireParams { style: FireStyle::Top, height: 46.0, intensity: 1.15, scale: 16.0, embers: 18, ember_rise: 70.0, ..FireParams::default() }),
        SPARKLES if d[0] < 0.0 && d[1] > 0.0 => Fx::sparkles(SparkleParams { count: d[1] as u32, size: d[2], color: rgb(0xffe6a8), margin: 30.0, ..Default::default() }),
        SPARKLES => Fx::sparkles(SparkleParams {
            burst: (d[0] >= 0.0).then_some(d[0]),
            count: d[1] as u32,
            size: d[2],
            color: rgb(0xffe6a8),
            margin: 30.0,
            life: 1.6,
            ..Default::default()
        }),
        AURA => {
            let k = ((t - d[0]) / 0.8).clamp(0.0, 1.0);
            Fx::glow(GlowParams {
                style: GlowStyle::Aura,
                color: rgb(0xffc531),
                core: rgb(0xfff4c2),
                third: rgb(0x6aa8ff),
                radius: 22.0,
                intensity: 0.9 * k,
                aura_speed: 0.3,
                ..Default::default()
            })
        }
        RIPPLE => Fx::pulse(PulseParams {
            style: PulseStyle::Ripple { at: [origin[0] + d[0], origin[1] + d[1]] },
            color: col(d[3]),
            t0: d[2],
            duration: 0.9,
            width: 2.2,
            intensity: 0.55,
            reach: 0.0,
            ..Default::default()
        }),
        SELECT => Fx::glow(GlowParams { style: GlowStyle::Neon, color: rgb(0x6aa8ff), intensity: d[0], radius: 7.0, occlusion: 0.0, ..Default::default() }),
        HEAT => {
            let heat = (-(t - d[0]).max(0.0) / 0.9).exp();
            if heat < 0.01 {
                return None;
            }
            Fx::glow(GlowParams { style: GlowStyle::Neon, color: rgb(0xff2d2d), radius: 26.0, intensity: 1.3, heat, ..Default::default() })
        }
        SHIMMER => Fx::shimmer(ShimmerParams { intensity: 0.1, period: 3.2, band: 60.0, ..Default::default() }),
        _ => return None,
    };
    // A burst is a one-shot on the real clock; twinkling stars are ambient.
    let at = if ambient(kind) && !(kind == SPARKLES && d[0] >= 0.0) { amb } else { t };
    Some(fx.on(shape).at(at))
}

#[derive(Default)]
pub struct Gfx {
    fx: Option<FxRenderer>,
    live: bool,
    /// The ambient clock and how fast it runs (1 while someone plays, easing to 0 when idle).
    amb: f64,
    speed: f64,
}

/// How long after the last input the ambient effects keep moving.
const IDLE_AFTER: f64 = 10.0;

impl Compositor<App> for Gfx {
    fn animating(&self) -> bool {
        self.live
    }

    fn render(&mut self, f: &mut Frame<'_, App>) {
        let (device, queue, size) = (f.device, f.queue, f.size);
        let t = f.rt.t as f32;
        let reduced = f.rt.state.reduced || f.rt.system_reduced_motion();
        let dt = f.dt.clamp(0.0, 0.1);
        let awake = f.rt.state.last_input == f64::MAX || f.rt.t - f.rt.state.last_input < IDLE_AFTER;
        let target = if awake { 1.0 } else { 0.0 };
        // Ease in over ~0.3 s and out over ~2.5 s, so a stop is a slow settle, never a freeze.
        let rate = if awake { 3.5 } else { 1.8 };
        self.speed += (target - self.speed) * (dt * rate).min(1.0);
        if !awake && self.speed < 0.01 {
            self.speed = 0.0;
        }
        self.amb += dt * self.speed;
        let amb = self.amb as f32;
        let marks = &f.rt.marks;
        let mut at: Vec<u32> = Vec::new();
        let mut layers: Vec<FxLayer> = Vec::new();
        let mut slots: Vec<FxSlot> = Vec::new();
        let mut kinds: Vec<u32> = Vec::new();
        for (n, i) in f.rt.instances.iter().enumerate() {
            if i.params[1] != KIND_EFFECT {
                continue;
            }
            let kind = marks.get(n).copied().unwrap_or(0) & 0xff;
            let k = i.params[3];
            let [x, y, w, h] = i.rect.map(|v| v / k);
            let r = i.radii.iter().fold(0.0f32, |a, v| a.max(v.abs())) / k;
            let shape = if kind == FACE { Shape::circle(x + w / 2.0, y + h / 2.0, w.min(h) / 2.0) } else { Shape::rounded(x, y, w, h, r) };
            let Some(l) = layer(kind, i.uv, shape, [x, y], t, amb) else { continue };
            at.push(n as u32);
            kinds.push(if kind == SPARKLES && i.uv[0] >= 0.0 { 0 } else { kind });
            layers.push(l.reduced(reduced));
            slots.push(FxSlot::default().scale(k).clip(i.clip, i.fade));
        }
        // Ambient layers count only while their clock runs; one-shots until they finish.
        self.live = self.speed > 0.0 || layers.iter().zip(&kinds).any(|(l, &k)| !ambient(k) && l.is_animating());
        let fx = self.fx.get_or_insert_with(|| FxRenderer::new(device, f.format, FxOptions { dither: true }));
        fx.set_srgb_blend(f.srgb_blend());
        fx.add_format(device, wgpu::TextureFormat::Rgba16Float);
        fx.prepare_with_slots(device, queue, &FxFrame { viewport: [size.0, size.1], scale: 1.0, layers: &layers, backdrop: None, sources: &[] }, &slots);
        let clear = f.rt.clear_color();
        f.ui.prepare(device, queue, &mut f.rt.text.atlas, &f.rt.instances, size);
        f.ui.draw_with_backdrop(f.encoder, f.target, clear, &mut |phase, s: &EffectSite<'_>| {
            let Some(k) = at.iter().position(|n| *n == s.index) else { return };
            match phase {
                EffectPhase::Backdrop(enc) => {
                    if let Some(bd) = s.backdrop {
                        fx.blur_backdrop(device, queue, enc, bd, k);
                    }
                }
                EffectPhase::Draw(pass) => fx.draw_layer_in(pass, s.format, s.origin, k),
            }
        });
    }
}
