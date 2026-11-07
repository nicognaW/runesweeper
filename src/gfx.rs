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
/// How long a shockwave runs (seconds); its slot unmounts after it.
pub const SHOCK_FOR: f64 = 0.6;
/// data: [t0 (burst), count, size, 0]
pub const SPARKLES: u32 = 8;
/// data: [t0, 0, 0, 0]
pub const AURA: u32 = 9;
/// data: [intensity, 0, 0, 0]
pub const SELECT: u32 = 11;
/// data: [t0, 0, 0, 0]: a red flash that cools.
pub const HEAT: u32 = 12;
/// How long the heat takes to cool completely (seconds); the window sleeps after it.
pub const HEAT_FOR: f32 = 1.2;
/// data: [t0, colour index, 0, 0]: a slow band of light across the hidden tiles while a field
/// is generated, from off its left edge at t0; with Reduce Motion it breathes in place.
pub const SWEEP: u32 = 13;

const COLORS: [u32; 8] = [0xffc531, 0x6aa8ff, 0xff3b4e, 0x4ade80, 0xffffff, 0xd8d8d8, 0xa78bfa, 0xff2d2d];

fn col(i: f32) -> Rgb {
    rgb(COLORS[(i.max(0.0) as usize).min(COLORS.len() - 1)])
}

/// Everything persistent is still; only one-shots (shockwave, heat, burst, aura's fade) move.
fn layer(kind: u32, d: [f32; 4], shape: Shape, origin: [f32; 2], t: f32) -> Option<FxLayer> {
    let fx = match kind {
        BACKDROP => Fx::fill(MeshGradientParams {
            colors: if d[0] > 0.5 {
                Stops::hex(&[0x0b0b0d, 0x26262b, 0x111114, 0x1d1d22])
            } else {
                Stops::hex(&[0x05060e, 0x1a1f5c, 0x0b2a4a, 0x3a1450, 0x070914])
            },
            speed: 0.0,
            drift: 0.0,
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
            Fx::fill(LiquidMetalParams { flow: 0.0, speed: 0.0, ..p }.border(d[1]))
        }
        LED => Fx::glow(GlowParams { style: GlowStyle::Soft, color: col(d[0]), intensity: d[1], radius: d[2], occlusion: 0.0, ..Default::default() }),
        FACE => Fx::fill(PlasticParams { color: col(d[0]), roughness: 0.22, dome: 0.55, bevel: 7.0, angle: -0.7 }),
        SHOCK => Fx::distort(DistortParams::Shockwave { at: [origin[0] + d[0], origin[1] + d[1]], reach: 900.0, width: 70.0, amount: d[3], chroma: 0.7, t0: d[2], duration: SHOCK_FOR as f32 }),
        SPARKLES => Fx::sparkles(SparkleParams {
            burst: Some(d[0]),
            count: d[1] as u32,
            size: d[2],
            color: if d[3] > 0.5 { rgb(0xffffff) } else { rgb(0xffe6a8) },
            margin: 30.0,
            life: 1.6,
            ..Default::default()
        }),
        AURA => {
            let k = ((t - d[0]) / 0.8).clamp(0.0, 1.0);
            Fx::glow(GlowParams {
                style: GlowStyle::Aura,
                color: if d[1] > 0.5 { rgb(0xd8d8d8) } else { rgb(0xffc531) },
                core: rgb(0xfff4c2),
                third: if d[1] > 0.5 { rgb(0xffffff) } else { rgb(0x6aa8ff) },
                radius: 22.0,
                intensity: 0.9 * k,
                aura_speed: 0.0,
                ..Default::default()
            })
        }
        SELECT => Fx::glow(GlowParams { style: GlowStyle::Neon, color: if d[1] > 0.5 { rgb(0xd0d0d0) } else { rgb(0x6aa8ff) }, intensity: d[0], radius: 7.0, occlusion: 0.0, ..Default::default() }),
        HEAT => {
            let since = (t - d[0]).max(0.0);
            if since >= HEAT_FOR {
                return None;
            }
            let heat = (-since / 0.26).exp();
            Fx::glow(GlowParams { style: GlowStyle::Neon, color: if d[1] > 0.5 { rgb(0xc8c8c8) } else { rgb(0xff2d2d) }, radius: 26.0, intensity: 1.3, heat, ..Default::default() })
        }
        SWEEP => Fx::shimmer(ShimmerParams {
            color: if d[1] > 0.5 { rgb(0xffffff) } else { rgb(0xbcd4ff) },
            period: 2.4,
            band: 60.0,
            angle: 0.35,
            intensity: 0.22,
            pause: 0.25,
            feather: 3.0,
            ..Default::default()
        }),
        _ => return None,
    };
    // The sweep's clock starts with it, so its band enters from the edge.
    Some(fx.on(shape).at(if kind == SWEEP { (t - d[0]).max(0.0) } else { t }))
}

#[derive(Default)]
pub struct Gfx {
    fx: Option<FxRenderer>,
    live: bool,
}

impl Compositor<App> for Gfx {
    fn animating(&self) -> bool {
        self.live
    }

    fn render(&mut self, f: &mut Frame<'_, App>) {
        let (device, queue, size) = (f.device, f.queue, f.size);
        let t = f.rt.t as f32;
        let reduced = f.rt.state.reduced || f.rt.system_reduced_motion();
        let marks = &f.rt.marks;
        let mut at: Vec<u32> = Vec::new();
        let mut layers: Vec<FxLayer> = Vec::new();
        let mut slots: Vec<FxSlot> = Vec::new();
        // Reduce Motion stills the sweep's band, but it still breathes, so it keeps frames coming.
        let mut sweeping = false;
        // The heat's cooling is ours, not rune-fx's: it keeps frames coming until it is cold.
        let mut cooling = false;
        for (n, i) in f.rt.instances.iter().enumerate() {
            if i.params[1] != KIND_EFFECT {
                continue;
            }
            let kind = marks.get(n).copied().unwrap_or(0) & 0xff;
            let k = i.params[3];
            let [x, y, w, h] = i.rect.map(|v| v / k);
            let r = i.radii.iter().fold(0.0f32, |a, v| a.max(v.abs())) / k;
            let shape = if kind == FACE { Shape::circle(x + w / 2.0, y + h / 2.0, w.min(h) / 2.0) } else { Shape::rounded(x, y, w, h, r) };
            let Some(l) = layer(kind, i.uv, shape, [x, y], t) else { continue };
            sweeping |= kind == SWEEP;
            cooling |= kind == HEAT;
            at.push(n as u32);
            layers.push(l.reduced(reduced));
            slots.push(FxSlot::default().scale(k).clip(i.clip, i.fade));
        }
        self.live = sweeping || cooling || layers.iter().any(|l| l.is_animating());
        let fx = self.fx.get_or_insert_with(|| FxRenderer::new(device, f.format, FxOptions { dither: true }));
        fx.set_srgb_blend(f.srgb_blend());
        fx.add_format(device, wgpu::TextureFormat::Rgba16Float);
        fx.prepare_with_slots(device, queue, &FxFrame { viewport: [size.0, size.1], scale: 1.0, layers: &layers, backdrop: None, sources: &[] }, &slots);
        let clear = f.rt.clear_color();
        f.ui.prepare(device, queue, &mut f.rt.text.atlas, &f.rt.instances, size);
        f.ui.draw_with_backdrop(f.encoder, f.target, clear, &mut |phase, s: &EffectSite<'_>| {
            let Some(k) = at.iter().position(|n| *n == s.index) else { return };
            match phase {
                // Only the shockwave reads the backdrop, and it bends it as it is: no blur pass.
                // A frosted effect added later would need `blur_backdrop` here.
                EffectPhase::Backdrop(_) => {
                    if let Some(bd) = s.backdrop {
                        fx.set_layer_backdrop(device, bd, k);
                    }
                }
                EffectPhase::Draw(pass) => fx.draw_layer_in(pass, s.format, s.origin, k),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heat_cools_completely_then_is_gone() {
        let shape = Shape::rounded(0.0, 0.0, 100.0, 100.0, 12.0);
        assert!(layer(HEAT, [0.0; 4], shape, [0.0; 2], HEAT_FOR - 0.1).is_some(), "still cooling");
        assert!(layer(HEAT, [0.0; 4], shape, [0.0; 2], HEAT_FOR).is_none(), "cold: nothing keeps the window awake");
    }
}
