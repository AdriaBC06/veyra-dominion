//! Title backdrop (V14): the painted night over razed Ashford (`assets/title_backdrop.px.js`) with
//! smoke and embers rising from the village, Veyra's portrait and a subtle parallax that follows
//! the pointer. The title, the credits and the settings opened from the title all sit on it.

use crate::assets::{self, TitleArt};
use macroquad::prelude::*;

/// Width of the sky and hills canvas. It is drawn right-anchored (Veyra and the moon stay put on
/// any aspect ratio) and hangs this far past both screen edges so the parallax never shows a gap.
const CANVAS_W: f32 = 880.0;
const OVERHANG: f32 = 8.0;
/// Top-left of the village strip on the canvas (`VILLAGE` in the .px.js).
const VILLAGE: Vec2 = vec2(376.0, 208.0);
const VILLAGE_FPS: f64 = 12.0;

/// Fires in the village strip (`FIRES` in the .px.js): x, base y, size, and how high its smoke
/// climbs (fraction of `SMOKE_RISE`). The left column stays low so the moon keeps showing.
const FIRES: [(f32, f32, f32, f32); 3] = [
    (76.0, 60.0, 0.9, 0.55),
    (124.0, 34.0, 1.2, 1.0),
    (210.0, 62.0, 0.9, 0.8),
];
/// Puffs per column; a new one leaves the fire every `PUFF_SECONDS`.
const PUFFS: usize = 30;
const PUFF_SECONDS: f64 = 1.2;
const PUFF_TIERS: u32 = 24;
const PUFF_SIZE: f32 = 56.0;
const SMOKE_RISE: f32 = 270.0;
/// The wind bends the columns right, behind Veyra.
const SMOKE_DRIFT: f32 = 170.0;
/// Offsets of the shade and rim passes; the body pass covers them except on the outer edge.
const SMOKE_PASSES: [(u32, Vec2); 3] =
    [(0, vec2(-1.0, 2.0)), (1, vec2(1.0, -1.0)), (2, Vec2::ZERO)];

const EMBERS: u32 = 44;
const EMBER_COLORS: [Color; 3] = [
    Color::new(1.0, 0.82, 0.44, 1.0),
    Color::new(0.94, 0.54, 0.16, 1.0),
    Color::new(0.66, 0.19, 0.1, 1.0),
];

/// Pixels each layer moves at full pointer deflection (far to near).
const PARALLAX_SKY: f32 = 1.0;
const PARALLAX_HILLS: f32 = 2.0;
const PARALLAX_VILLAGE: f32 = 3.0;
const PARALLAX_EMBERS: f32 = 4.0;
const PARALLAX_PORTRAIT: f32 = 5.0;
const PARALLAX_CREST: f32 = 7.0;
/// How fast the view follows the pointer (per second).
const PARALLAX_EASE: f32 = 2.5;

/// Deterministic 0..1 hash, so smoke and embers need no state.
fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

#[derive(Default)]
pub struct Backdrop {
    /// Eased horizontal pointer position, -1 (left edge) to 1 (right edge).
    look: f32,
}

impl Backdrop {
    pub fn update(&mut self, mouse: Vec2, view: Vec2, dt: f32) {
        let target = (mouse.x / view.x * 2.0 - 1.0).clamp(-1.0, 1.0);
        self.look += (target - self.look) * (PARALLAX_EASE * dt).min(1.0);
    }

    /// Whole-pixel offset of a layer, so the pixel art never resamples.
    fn shift(&self, depth: f32) -> f32 {
        (-self.look * depth).round()
    }

    /// Top-left of the canvas for a layer at `depth`.
    fn canvas(&self, view: Vec2, depth: f32) -> Vec2 {
        vec2(view.x - CANVAS_W + OVERHANG + self.shift(depth), 0.0)
    }

    pub fn draw(&self, view: Vec2, t: f64) {
        assets::draw_title_art(TitleArt::Sky, 0, self.canvas(view, PARALLAX_SKY));
        assets::draw_title_art(TitleArt::Hills, 0, self.canvas(view, PARALLAX_HILLS));
        let village = self.canvas(view, PARALLAX_VILLAGE) + VILLAGE;
        let frame = (t * VILLAGE_FPS) as u32;
        assets::draw_title_art(TitleArt::Village, frame, village);
        draw_smoke(village, t);
        draw_embers(self.canvas(view, PARALLAX_EMBERS) + VILLAGE, view, t);
        let crest = assets::title_art_size(TitleArt::Crest);
        let crest_pos = vec2(-OVERHANG + self.shift(PARALLAX_CREST), view.y - crest.y);
        assets::draw_title_art(TitleArt::Crest, 0, crest_pos);
        let portrait = assets::title_portrait();
        let pos = view - portrait.size() + vec2(self.shift(PARALLAX_PORTRAIT), 0.0);
        portrait.draw(pos, t);
    }
}

/// Smoke columns: each fire lets out a puff every `PUFF_SECONDS` that climbs, grows, drifts with
/// the wind and thins out. All puffs are drawn oldest first, in three passes (see `SMOKE_PASSES`).
fn draw_smoke(village: Vec2, t: f64) {
    let mut puffs = [(0.0f32, Vec2::ZERO, 0u32); FIRES.len() * PUFFS];
    for (f, &(fx, fy, size, height)) in FIRES.iter().enumerate() {
        let clock = t / PUFF_SECONDS + f as f64 * 0.37;
        for i in 0..PUFFS {
            let age = (i as f64 + clock).rem_euclid(PUFFS as f64);
            let u = (age / PUFFS as f64) as f32;
            // the puff's own sideways jitter, fixed for its whole climb
            let born = (clock - age).round() as i64 as u32;
            let jitter = (hash(born.wrapping_mul(31) + f as u32) - 0.5) * 10.0 * (u * 4.0).min(1.0);
            let x = fx
                + SMOKE_DRIFT * u.powf(1.5) * height
                + jitter
                + 4.0 * (u * 8.0 + f as f32 * 2.0).sin();
            let y = fy - 20.0 * size - SMOKE_RISE * u.powf(0.85) * height;
            let tier = (u * (PUFF_TIERS - 1) as f32 * (0.55 + 0.45 * size).min(1.0)).round() as u32;
            puffs[f * PUFFS + i] = (
                u,
                village + vec2(x, y) - PUFF_SIZE * 0.5,
                tier.min(PUFF_TIERS - 1),
            );
        }
    }
    puffs.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
    for (pass, offset) in SMOKE_PASSES {
        for &(_, pos, tier) in &puffs {
            assets::draw_title_art(TitleArt::Smoke, pass * PUFF_TIERS + tier, pos + offset);
        }
    }
}

/// Embers: half rise from the fires, half from the whole burnt valley floor; each lives a few
/// seconds, drifts with the wind, cools from yellow to dull red and blinks out.
fn draw_embers(village: Vec2, view: Vec2, t: f64) {
    for i in 0..EMBERS {
        let life = 3.0 + 3.0 * hash(i * 7 + 1) as f64;
        let cycle = t / life + hash(i * 13 + 5) as f64;
        let gen = cycle.floor() as u32;
        let u = cycle.fract() as f32;
        let h = |k: u32| hash(i.wrapping_mul(92_821) ^ gen.wrapping_mul(7919) ^ (k << 20));
        let start = if i % 2 == 0 {
            let (fx, fy, _, _) = FIRES[(i / 2) as usize % FIRES.len()];
            village + vec2(fx + (h(0) - 0.5) * 12.0, fy - 6.0)
        } else {
            vec2(view.x * h(0), village.y + 60.0 + 70.0 * h(5))
        };
        let pos = start
            + vec2(
                u * (30.0 + 60.0 * h(1)) + 6.0 * (u * 7.0 + h(2) * 6.0).sin(),
                -u * (110.0 + 150.0 * h(3)),
            );
        // last stretch: blinking out
        if u > 0.8 && ((t * 14.0) as u32 + i).is_multiple_of(3) {
            continue;
        }
        let color = EMBER_COLORS[((u * 3.0) as usize).min(2)];
        let size = if h(4) > 0.8 { 2.0 } else { 1.0 };
        draw_rectangle(pos.x.round(), pos.y.round(), size, size, color);
    }
}
