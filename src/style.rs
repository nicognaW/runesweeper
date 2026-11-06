//! Colours, sizes and the two palettes (Color on, and the monochrome the Color setting
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
/// The toolbar over the console.
pub const TOOLBAR_H: f32 = 44.0;
pub const MARGIN: f32 = 22.0;

pub const NUMBER: TextStyle = TextStyle::new(16.0, 700.0).features("'tnum' 1");
pub const UI: TextStyle = TextStyle::new(13.0, 500.0);
pub const UI_BOLD: TextStyle = TextStyle::new(13.0, 600.0);
pub const SMALL: TextStyle = TextStyle::new(11.5, 500.0);
pub const TITLE: TextStyle = TextStyle::new(20.0, 700.0).tracking(-0.2);

/// The window size that fits a `w`×`h` field.
pub fn window_size(w: usize, h: usize) -> (f32, f32) {
    let board_w = w as f32 * PITCH - GAP + 2.0 * WELL;
    let board_h = h as f32 * PITCH - GAP + 2.0 * WELL;
    let console_w = board_w.max(9.0 * PITCH + 2.0 * WELL) + 2.0 * PAD;
    let console_h = board_h + HEADER_H + 12.0 + 2.0 * PAD;
    ((console_w + 2.0 * MARGIN).max(330.0), console_h + TOOLBAR_H + 2.0 * MARGIN)
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
    pub accent: Color,
    /// A won field's flagged tiles (face top, bottom) and its gold: timer, banner.
    pub win_top: Color,
    pub win_bottom: Color,
    pub win: Color,
    /// The mine you hit: its cell's bright centre.
    pub hit_core: Color,
    /// A mine going off: white-hot core, flame, and the edge it fades to.
    pub blast: [Color; 3],
}

impl Palette {
    pub fn get(color: bool) -> Palette {
        if color { Self::neon() } else { Self::mono() }
    }

    fn neon() -> Palette {
        Palette {
            bg: c(0x07080f),
            console_top: c(0x1b2033),
            console_bottom: c(0x10131f),
            well: c(0x090b14),
            open: c(0x141a2b),
            open_edge: ca(0xffffff, 0.035),
            tile_top: c(0x45557c),
            tile_bottom: c(0x2b3553),
            tile_hover: ca(0x9fc3ff, 0.16),
            tile_press: c(0x1a2136),
            tile_light: ca(0xffffff, 0.28),
            // The Windows colours (blue, green, red, navy, maroon, teal, black, grey), lit for a
            // dark field.
            numbers: [
                c(0x000000),
                c(0x55a8ff),
                c(0x4ade80),
                c(0xff5c6c),
                c(0xa78bfa),
                c(0xff9f43),
                c(0x2dd4bf),
                c(0xf4f4f5),
                c(0x94a3b8),
            ],
            flag: c(0xff3b4e),
            mine: c(0x0b0d14),
            hit: c(0xe0202f),
            led_on: c(0xff2d2d),
            led_off: ca(0xff2d2d, 0.09),
            led_bg: c(0x0a0505),
            text: c(0xe8ebf5),
            muted: c(0x8a93ad),
            accent: c(0x6aa8ff),
            win_top: c(0xf6c453),
            win_bottom: c(0xa8701c),
            win: c(0xffc531),
            hit_core: c(0xff6b6b),
            blast: [c(0xfff1c2), c(0xff7a1a), c(0xff2d2d)],
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
            accent: g(0xd0),
            win_top: g(0xf2),
            win_bottom: g(0x9a),
            win: g(0xf4),
            hit_core: g(0xc8),
            blast: [g(0xff), g(0xb0), g(0x60)],
        }
    }
}

pub fn inset(y: f32, blur: f32, color: Color) -> Shadow {
    Shadow { y, blur, color, inset: true, ..Shadow::default() }
}

pub fn drop(y: f32, blur: f32, color: Color) -> Shadow {
    Shadow { y, blur, color, ..Shadow::default() }
}
