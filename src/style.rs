//! Colours, sizes and the two palettes (Color on, and the monochrome the Color menu item
//! switches to).

use rune::{Color, Shadow, TextStyle};

/// sRGB hex to Rune's linear colour.
pub fn c(hex: u32) -> Color {
    ca(hex, 1.0)
}

pub fn ca(hex: u32, a: f32) -> Color {
    let ch = |s: u32| rune::color::srgb_decode(((hex >> s) & 0xff) as f32 / 255.0);
    Color { r: ch(16), g: ch(8), b: ch(0), a }
}

/// One cell's pitch and the tile inside it.
pub const PITCH: f32 = 26.0;
pub const GAP: f32 = 2.0;
pub const TILE: f32 = PITCH - GAP;
/// The well's inset around the cells.
pub const WELL: f32 = 6.0;
/// The console's padding.
pub const PAD: f32 = 14.0;
pub const HEADER_H: f32 = 58.0;
pub const MENU_H: f32 = 34.0;
pub const MARGIN: f32 = 22.0;

pub const NUMBER: TextStyle = TextStyle::new(16.0, 700.0).features("'tnum' 1");
pub const UI: TextStyle = TextStyle::new(13.0, 500.0);
pub const UI_BOLD: TextStyle = TextStyle::new(13.0, 600.0);
/// Instrument labels: small capitals, spaced out.
pub const HUD: TextStyle = TextStyle::new(12.0, 600.0).tracking(1.8);
pub const SMALL: TextStyle = TextStyle::new(11.5, 500.0);
pub const TITLE: TextStyle = TextStyle::new(20.0, 700.0).tracking(-0.2);

/// The window size that fits a `w`×`h` field.
pub fn window_size(w: usize, h: usize) -> (f32, f32) {
    let board_w = w as f32 * PITCH - GAP + 2.0 * WELL;
    let board_h = h as f32 * PITCH - GAP + 2.0 * WELL;
    let console_w = board_w.max(9.0 * PITCH + 2.0 * WELL) + 2.0 * PAD;
    let console_h = board_h + HEADER_H + 12.0 + 2.0 * PAD;
    ((console_w + 2.0 * MARGIN).max(330.0), console_h + MENU_H + 2.0 * MARGIN)
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub console_top: Color,
    pub console_bottom: Color,
    pub well: Color,
    pub open: Color,
    pub open_edge: Color,
    pub tile_top: Color,
    pub tile_bottom: Color,
    pub tile_hover: Color,
    pub tile_press: Color,
    pub tile_light: Color,
    pub numbers: [Color; 9],
    pub flag: Color,
    pub mine: Color,
    pub hit: Color,
    pub led_on: Color,
    pub led_off: Color,
    pub led_bg: Color,
    pub text: Color,
    pub muted: Color,
    pub face: u32,
    pub accent: Color,
}

impl Palette {
    pub fn get(color: bool) -> Palette {
        if color { Self::neon() } else { Self::mono() }
    }

    /// Sealed alien instrument: smoked-glass cells lit at the edge, cyan phosphor readouts.
    fn neon() -> Palette {
        Palette {
            bg: c(0x020308),
            console_top: c(0x111a24),
            console_bottom: c(0x070b12),
            well: c(0x02050a),
            open: c(0x060d16),
            open_edge: ca(0x3df5e0, 0.07),
            tile_top: c(0x14283a),
            tile_bottom: c(0x08101b),
            tile_hover: ca(0x3df5e0, 0.14),
            tile_press: c(0x0a121c),
            tile_light: ca(0x9ff8ee, 0.5),
            // The Windows colours in hue order (blue, green, red, navy, maroon, teal, black,
            // grey), as luminous inks on a near-black field.
            numbers: [
                c(0x000000),
                c(0x5fd0ff),
                c(0x5cf0a0),
                c(0xff5c8a),
                c(0xb79bff),
                c(0xffb454),
                c(0x3fe6d4),
                c(0xf4f6fb),
                c(0xa3aec6),
            ],
            flag: c(0xff3d6e),
            mine: c(0x0b0d14),
            hit: c(0xd11a4a),
            led_on: c(0x3df5e0),
            led_off: ca(0x3df5e0, 0.08),
            led_bg: c(0x010608),
            text: c(0xe6fbf8),
            muted: c(0x7f95a6),
            face: 0x8ef5c9,
            accent: c(0x3df5e0),
        }
    }

    fn mono() -> Palette {
        let g = |v: u32| c(v * 0x010101);
        Palette {
            bg: g(0x0a),
            console_top: g(0x26),
            console_bottom: g(0x17),
            well: g(0x0c),
            open: g(0x1c),
            open_edge: ca(0xffffff, 0.04),
            tile_top: g(0x5c),
            tile_bottom: g(0x3c),
            tile_hover: ca(0xffffff, 0.12),
            tile_press: g(0x22),
            tile_light: ca(0xffffff, 0.3),
            numbers: [g(0); 9].map(|_| g(0xe6)),
            flag: g(0xf0),
            mine: g(0x05),
            hit: g(0x80),
            led_on: g(0xf0),
            led_off: ca(0xffffff, 0.07),
            led_bg: g(0x05),
            text: g(0xec),
            muted: g(0x96),
            face: 0xd8d8d8,
            accent: g(0xd0),
        }
    }
}

pub fn inset(y: f32, blur: f32, color: Color) -> Shadow {
    Shadow { y, blur, color, inset: true, ..Shadow::default() }
}

pub fn drop(y: f32, blur: f32, color: Color) -> Shadow {
    Shadow { y, blur, color, ..Shadow::default() }
}
