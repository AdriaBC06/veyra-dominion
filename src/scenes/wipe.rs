//! Ember wipe between scenes (V14): a black sheet burns in over the old scene from the left with a
//! glowing ragged edge, the scene changes under it, then it burns away the same way to reveal the
//! new one. Drawn at half resolution (2x2 logical pixels per cell) so the edge reads as pixel art.

use super::Nav;
use macroquad::prelude::*;

const COVER_TIME: f32 = 0.24;
const REVEAL_TIME: f32 = 0.36;
/// Width of the glowing band, in burn-field units (the field runs 0..1 across the screen).
const EDGE: f32 = 0.07;
/// Burning edge, from the sheet side (charred red) to the front (white hot).
const EDGE_COLORS: [[u8; 4]; 4] = [
    [0x5c, 0x0c, 0x1c, 255],
    [0xa8, 0x30, 0x1a, 255],
    [0xf0, 0x8a, 0x2a, 255],
    [0xff, 0xd8, 0x80, 255],
];
const CELL: f32 = 2.0;

enum Phase {
    /// Burning in over the old scene; the nav applies once it is covered.
    Cover(Nav),
    Reveal,
    Idle,
}

pub struct Wipe {
    phase: Phase,
    /// 0..1 through the current phase.
    progress: f32,
    /// Burn threshold per cell: mostly left to right with a ragged, noisy front.
    field: Vec<f32>,
    size: (u16, u16),
    image: Image,
    tex: Option<Texture2D>,
}

/// Smooth value noise in 0..1 on a lattice of `scale` cells.
fn value_noise(x: f32, y: f32, scale: f32) -> f32 {
    let hash = |i: i32, j: i32| {
        let mut n = (i.wrapping_mul(374_761_393) ^ j.wrapping_mul(668_265_263)) as u32;
        n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
        (n >> 16) as f32 / 65535.0
    };
    let (fx, fy) = (x / scale, y / scale);
    let (i, j) = (fx.floor() as i32, fy.floor() as i32);
    let (u, v) = (fx.fract(), fy.fract());
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let top = hash(i, j) + (hash(i + 1, j) - hash(i, j)) * u;
    let bottom = hash(i, j + 1) + (hash(i + 1, j + 1) - hash(i, j + 1)) * u;
    top + (bottom - top) * v
}

impl Wipe {
    pub fn new() -> Self {
        Self {
            // the game opens burning in from black
            phase: Phase::Reveal,
            progress: 0.0,
            field: Vec::new(),
            size: (0, 0),
            image: Image::empty(),
            tex: None,
        }
    }

    /// Starts covering the screen; `nav` is handed back by `update` once it is covered.
    /// Ignored while a wipe is already covering.
    pub fn start(&mut self, nav: Nav) {
        if !matches!(self.phase, Phase::Cover(_)) {
            self.phase = Phase::Cover(nav);
            self.progress = 0.0;
        }
    }

    /// True while the old scene is being covered (it should not take input).
    pub fn covering(&self) -> bool {
        matches!(self.phase, Phase::Cover(_))
    }

    /// Advances the wipe; returns the pending nav when the screen is fully covered.
    pub fn update(&mut self, dt: f32) -> Option<Nav> {
        match self.phase {
            Phase::Cover(nav) => {
                self.progress += dt / COVER_TIME;
                if self.progress >= 1.0 {
                    self.phase = Phase::Reveal;
                    self.progress = 0.0;
                    return Some(nav);
                }
            }
            Phase::Reveal => {
                self.progress += dt / REVEAL_TIME;
                if self.progress >= 1.0 {
                    self.phase = Phase::Idle;
                }
            }
            Phase::Idle => {}
        }
        None
    }

    /// Rebuilds the burn field when the view size changes.
    fn fit(&mut self, view: Vec2) {
        let size = ((view.x / CELL).ceil() as u16, (view.y / CELL).ceil() as u16);
        if size == self.size {
            return;
        }
        self.size = size;
        let (w, h) = (size.0 as usize, size.1 as usize);
        self.field = (0..w * h)
            .map(|k| {
                let (x, y) = ((k % w) as f32, (k / w) as f32);
                let sweep = (x + 0.35 * (h as f32 - y)) / (w as f32 + 0.35 * h as f32);
                let noise = 0.7 * value_noise(x, y, 14.0) + 0.3 * value_noise(x, y, 4.0);
                0.75 * sweep + 0.25 * noise
            })
            .collect();
        // the field spans roughly 0..1; stretch it so both phases start and end fully clear/covered
        let (lo, hi) = self
            .field
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        for v in &mut self.field {
            *v = (*v - lo) / (hi - lo);
        }
        self.image = Image::gen_image_color(size.0, size.1, BLANK);
        let tex = Texture2D::from_image(&self.image);
        tex.set_filter(FilterMode::Nearest);
        self.tex = Some(tex);
    }

    pub fn draw(&mut self, view: Vec2) {
        let cover = match self.phase {
            Phase::Idle => return,
            Phase::Cover(_) => true,
            Phase::Reveal => false,
        };
        self.fit(view);
        // the burn front sweeps across the field, from just before 0 to just past 1
        let front = self.progress * (1.0 + 2.0 * EDGE) - EDGE;
        let bytes = self.image.get_image_data_mut();
        for (px, &v) in bytes.iter_mut().zip(&self.field) {
            // covering: black behind the front; revealing: black ahead of it
            let ahead = if cover { front - v } else { v - front };
            *px = if ahead >= EDGE {
                [0, 0, 0, 255]
            } else if ahead >= 0.0 {
                // hottest right at the front, charred toward the black
                let heat = 1.0 - ahead / EDGE;
                EDGE_COLORS[((heat * 4.0) as usize).min(3)]
            } else {
                [0, 0, 0, 0]
            };
        }
        if let Some(tex) = &self.tex {
            tex.update(&self.image);
            let params = DrawTextureParams {
                dest_size: Some(vec2(self.size.0 as f32, self.size.1 as f32) * CELL),
                ..Default::default()
            };
            draw_texture_ex(tex, 0.0, 0.0, WHITE, params);
        }
    }
}
