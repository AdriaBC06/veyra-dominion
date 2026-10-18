//! Pixel art defined in code: character matrices + palette, packed into a single
//! nearest-filtered atlas at startup. `sprite()` is the only accessor, so the art can
//! later be swapped for a real atlas without touching callers.

use macroquad::prelude::*;
use std::cell::OnceCell;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteId {
    VeyraBody,
    VeyraWings,
    VeyraTail,
    Ground,
}

const SPRITE_COUNT: usize = 4;

pub const GROUND_TILE: u16 = 32;
pub const GROUND_VARIANTS: usize = 6;

const ATLAS_W: u16 = 256;
const ATLAS_H: u16 = 256;

/// Maps a matrix character to a colour; `.` is transparent.
fn palette(ch: char) -> Option<Color> {
    let rgb = |r, g, b| Some(Color::from_rgba(r, g, b, 255));
    match ch {
        '.' => None,
        'R' => rgb(200, 50, 60),  // skin
        'r' => rgb(142, 31, 44),  // skin shadow
        'H' => rgb(158, 27, 42),  // horn / tail tip
        'h' => rgb(214, 70, 80),  // horn highlight
        'P' => rgb(59, 31, 94),   // hair
        'p' => rgb(90, 50, 144),  // hair highlight
        'Y' => rgb(242, 165, 42), // eyes
        'D' => rgb(36, 20, 46),   // corset
        'd' => rgb(58, 32, 72),   // corset highlight
        'G' => rgb(217, 166, 64), // gold trim
        'J' => rgb(224, 48, 74),  // gem
        'S' => rgb(90, 36, 117),  // skirt
        's' => rgb(61, 24, 80),   // skirt shadow
        'L' => rgb(42, 26, 53),   // sleeves / stockings
        'W' => rgb(107, 42, 110), // wing membrane
        'w' => rgb(46, 22, 56),   // wing bone
        _ => panic!("unknown palette char {ch:?}"),
    }
}

type Frame = &'static [&'static str];

const VEYRA_REST: Frame = &[
    "..h..........h..",
    ".Hh..........hH.",
    ".HH..........HH.",
    "..HHPPPPPPPPHH..",
    "...PPPPPPPPPP...",
    "...PPpRRRRpPP...",
    "..PPPRYRRYRPPP..",
    ".RPPPRRRRRRPPPR.",
    "..PPPrRRRRrPPP..",
    "..PPPPrRRrPPPP..",
    ".PPPRRGJJGRRPPP.",
    ".PPRRDDDDDDRRPP.",
    ".PPLDdDJJDdDLPP.",
    ".PPL.DDDDDD.LPP.",
    ".PPL.dDDDDd.LPP.",
    "..PLSSGJJGSSLP..",
    "..RLSSDDDDSSLR..",
    "...SSSDDDDSSS...",
    "...SsSLDDLSsS...",
    "..SsSSLDDLSSsS..",
    "..SsS.LDDL.SsS..",
    ".S.Ss.L..L.sS.S.",
    "......L..L......",
    "......d..d......",
];

/// Leaning into movement (facing right): head forward, hair and skirt trailing.
const VEYRA_IMPULSE: Frame = &[
    "...h..........h.",
    "..Hh..........hH",
    "..HH..........HH",
    "...HHPPPPPPPPHH.",
    ".P..PPPPPPPPPP..",
    ".PPPPPpRRRRpPP..",
    ".PPPPPRYRRYRPPP.",
    ".PRPPPRRRRRRPPPR",
    ".PPPPPrRRRRrPPP.",
    ".PPPPPPrRRrPPPP.",
    "PPPPRRGJJGRRPPP.",
    "PPPRRDDDDDDRRPP.",
    "PPPLDdDJJDdDLPP.",
    "PPPL.DDDDDD.LPP.",
    ".PPL.dDDDDd.LPP.",
    ".PPLSSGJJGSSLP..",
    "..RLSSDDDDSSLR..",
    "..SSSDDDDSSS....",
    ".SSsSLDDLSsS....",
    "SSsSSLDDLSSsS...",
    "SSsS.LDDL.SsS...",
    "S.Ss.L..L.sS.S..",
    ".....L..L.......",
    "....d..d........",
];

const WINGS_UP: Frame = &[
    "..w..................w..",
    ".ww..................ww.",
    ".wWw................wWw.",
    "wWWWw..............wWWWw",
    "wWWWWw............wWWWWw",
    "wWWWWWw..........wWWWWWw",
    ".WWWWWWw........wWWWWWW.",
    ".W.WW.WW........WW.WW.W.",
    "...W...W........W...W...",
    "........................",
];

const WINGS_DOWN: Frame = &[
    "........................",
    "........................",
    "..w..................w..",
    ".wWw................wWw.",
    ".wWWw..............wWWw.",
    "wWWWWw............wWWWWw",
    "wWWWWWw..........wWWWWWw",
    ".WWWWWWw........wWWWWWW.",
    "..W.WW.W........W.WW.W..",
    "....W..W........W..W....",
];

/// Hangs from the hip, arrowhead tip at the bottom right.
const TAIL: Frame = &[
    "R.......", "R.......", ".R......", ".R......", "..R...H.", "...R.HH.", "....HHHH", ".....HH.",
];

/// Matrix-defined sprites, indexed by `SpriteId as usize`. Ground is procedural.
const MATRIX_SPRITES: [(SpriteId, &[Frame]); 3] = [
    (SpriteId::VeyraBody, &[VEYRA_REST, VEYRA_IMPULSE]),
    (SpriteId::VeyraWings, &[WINGS_UP, WINGS_DOWN]),
    (SpriteId::VeyraTail, &[TAIL]),
];

struct Atlas {
    tex: Texture2D,
    rects: Vec<Rect>,
    /// (first rect index, frame count) per sprite id.
    ranges: [(usize, usize); SPRITE_COUNT],
}

thread_local! {
    static ATLAS: OnceCell<Atlas> = const { OnceCell::new() };
}

/// Simple shelf packer writing frames into the atlas image.
struct Packer {
    image: Image,
    x: u16,
    y: u16,
    shelf_h: u16,
}

impl Packer {
    fn place(&mut self, w: u16, h: u16) -> (u16, u16) {
        if self.x + w > ATLAS_W {
            self.x = 0;
            self.y += self.shelf_h + 1;
            self.shelf_h = 0;
        }
        assert!(self.y + h <= ATLAS_H, "sprite atlas full");
        let pos = (self.x, self.y);
        self.x += w + 1;
        self.shelf_h = self.shelf_h.max(h);
        pos
    }

    fn add(&mut self, w: u16, h: u16, pixel: impl Fn(u32, u32) -> Option<Color>) -> Rect {
        let (ox, oy) = self.place(w, h);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                if let Some(c) = pixel(x, y) {
                    self.image.set_pixel(ox as u32 + x, oy as u32 + y, c);
                }
            }
        }
        Rect::new(ox as f32, oy as f32, w as f32, h as f32)
    }
}

fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h =
        x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

/// Muted grass (variants 0..4) and dirt (4..6) so Veyra's warm palette stands out.
fn ground_pixel(variant: usize, x: u32, y: u32) -> Color {
    let n = hash(x, y, variant as u32 + 1) % 100;
    let (base, dark, light) = if variant < 4 {
        ((60, 76, 52), (50, 64, 45), (74, 90, 62))
    } else {
        ((84, 70, 56), (70, 58, 47), (98, 84, 66))
    };
    let (r, g, b) = match n {
        0..=11 => dark,
        12..=17 => light,
        _ => base,
    };
    Color::from_rgba(r, g, b, 255)
}

/// Builds the atlas texture. Must run once after the window exists.
pub fn init() {
    let mut packer = Packer {
        image: Image::gen_image_color(ATLAS_W, ATLAS_H, BLANK),
        x: 0,
        y: 0,
        shelf_h: 0,
    };
    let mut rects = Vec::new();
    let mut ranges = [(0, 0); SPRITE_COUNT];

    for (id, frames) in MATRIX_SPRITES {
        ranges[id as usize] = (rects.len(), frames.len());
        for frame in frames {
            let (w, h) = (frame[0].len() as u16, frame.len() as u16);
            rects.push(packer.add(w, h, |x, y| {
                palette(frame[y as usize].as_bytes()[x as usize] as char)
            }));
        }
    }

    ranges[SpriteId::Ground as usize] = (rects.len(), GROUND_VARIANTS);
    for v in 0..GROUND_VARIANTS {
        rects.push(packer.add(GROUND_TILE, GROUND_TILE, |x, y| Some(ground_pixel(v, x, y))));
    }

    let tex = Texture2D::from_image(&packer.image);
    tex.set_filter(FilterMode::Nearest);
    ATLAS.with(|a| {
        let _ = a.set(Atlas { tex, rects, ranges });
    });
}

/// Texture and source rect for a sprite frame (frame wraps around).
pub fn sprite(id: SpriteId, frame: usize) -> (Texture2D, Rect) {
    ATLAS.with(|a| {
        let atlas = a.get().expect("sprites::init not called");
        let (start, count) = atlas.ranges[id as usize];
        (atlas.tex.clone(), atlas.rects[start + frame % count])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrices_are_rectangular_and_use_palette() {
        for (id, frames) in MATRIX_SPRITES {
            let (w, h) = (frames[0][0].len(), frames[0].len());
            for frame in frames {
                assert_eq!(frame.len(), h, "{id:?} height");
                for row in *frame {
                    assert_eq!(row.len(), w, "{id:?} row {row:?}");
                    row.chars().for_each(|c| {
                        palette(c);
                    });
                }
            }
        }
    }

    #[test]
    fn sprite_ids_cover_every_slot_once() {
        let mut seen = [false; SPRITE_COUNT];
        for (id, _) in MATRIX_SPRITES {
            assert!(!seen[id as usize]);
            seen[id as usize] = true;
        }
        assert!(!seen[SpriteId::Ground as usize]);
    }

    #[test]
    fn veyra_body_is_16x24() {
        assert_eq!((VEYRA_REST[0].len(), VEYRA_REST.len()), (16, 24));
    }
}
