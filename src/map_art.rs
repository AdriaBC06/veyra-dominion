//! Draws the map (design.md 7.14, V19d): terrain tiles under everything, then props and walls
//! y-sorted together with the horde and Veyra, roofs and canopies fading while she is behind
//! them. Renderer only; the simulation's `Map` holds the grid.

use crate::assets::{terrain_frame, terrain_id, terrain_texture, village_frame, village_id};
use crate::config::*;
use crate::map::{props, Map};
use macroquad::prelude::*;

/// Dual-grid terrain layers, drawn in this order: which layout characters count as the
/// terrain, its tile name in `assets/village.px.js`, and the cell row (from the tile's top
/// row) whose two cells the tile also reads, if any: water's bank face reaches over the tile
/// above, the ridge's rock face over the one below. Woods run on past the grid's edge.
const LAYERS: [(&str, &str, Option<i32>); 5] = [
    (":#", "path", None),
    ("#", "cobble", None),
    ("~=", "water", Some(-1)),
    ("T", "woods", None),
    ("R", "ridge", Some(2)),
];
/// One water cell in this many (of those clear of the banks) carries a glint.
const GLINT_ONE_IN: u32 = 2;
const NONE: u16 = u16::MAX;

/// A prop placed in the world.
struct Prop {
    id: usize,
    /// Top-left corner in world pixels.
    pos: Vec2,
    size: Vec2,
    /// Animation offset, so copies don't sway in step.
    phase: f32,
    shadow: bool,
}

impl Prop {
    fn base(&self) -> f32 {
        self.pos.y + self.size.y
    }
}

pub struct MapArt {
    /// Per layer, the tile id at each cell corner ((cols + 1) x (rows + 1), offset -1).
    layers: Vec<Vec<u16>>,
    grass: [usize; 4],
    glint: usize,
    /// Top-left of each glint and its animation offset.
    glints: Vec<(Vec2, f32)>,
    /// Sorted by base.
    props: Vec<Prop>,
    /// Lying on the ground (the bridges): drawn with the terrain, under everything.
    flat: Vec<Prop>,
    tallest: f32,
}

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

impl MapArt {
    /// Builds the drawing of `map` from its layout text `src` (the same one it was parsed from).
    pub fn new(map: &Map, src: &str) -> Self {
        let (cols, rows) = (map.cols as i32, map.rows as i32);
        let at = |c: i32, r: i32| -> u8 {
            if c < 0 || r < 0 || c >= cols || r >= rows {
                b'T'
            } else {
                map.terrain[(r * cols + c) as usize]
            }
        };
        let layers = LAYERS
            .iter()
            .map(|&(chars, name, far)| {
                let ids: [[u16; 64]; 16] = std::array::from_fn(|k| {
                    std::array::from_fn(|v| {
                        terrain_id(&format!("{name}_{k}_{v}")).map_or(NONE, |t| t as u16)
                    })
                });
                let has = |c, r| chars.as_bytes().contains(&at(c, r)) as usize;
                let mut tiles = Vec::with_capacity(((cols + 1) * (rows + 1)) as usize);
                for j in -1..rows {
                    for i in -1..cols {
                        let k = has(i, j)
                            + 2 * has(i + 1, j)
                            + 4 * has(i, j + 1)
                            + 8 * has(i + 1, j + 1);
                        let far = far.map_or(0, |d| has(i, j + d) + 2 * has(i + 1, j + d));
                        tiles.push(ids[k][((i & 3) + 4 * (j & 3)) as usize + 16 * far]);
                    }
                }
                tiles
            })
            .collect();
        let grass =
            std::array::from_fn(|k| terrain_id(&format!("ground_grass_{k}")).expect("grass"));
        let mut glints = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                // clear of the bank face, which reaches into the first row under the far shore
                let open = [(0, 0), (-1, 0), (1, 0), (0, -1)]
                    .iter()
                    .all(|&(dc, dr)| at(c + dc, r + dr) == b'~');
                let h = hash(c, r);
                if open && h.is_multiple_of(GLINT_ONE_IN) {
                    let off = vec2((h >> 4 & 3) as f32, 2.0 + (h >> 8) as f32 % 12.0);
                    let cell = map.origin + vec2(c as f32, r as f32) * MAP_CELL;
                    glints.push((cell + off, (h >> 12) as f32 % 100.0 * 0.0137));
                }
            }
        }

        let mut list = Vec::new();
        let mut place = |name: &str, bottom_left: Vec2, seed: usize| {
            let Some(id) = village_id(name) else { return };
            let (_, r) = village_frame(id, 0.0);
            let shadow = name.starts_with("tree")
                || name.starts_with("bush")
                || name.starts_with("rock")
                || matches!(
                    name,
                    "well"
                        | "cart"
                        | "stall"
                        | "barrel"
                        | "crate"
                        | "hay_bale"
                        | "hay_roll"
                        | "scarecrow"
                );
            list.push(Prop {
                id,
                pos: map.origin + bottom_left - vec2(0.0, r.h),
                size: vec2(r.w, r.h),
                phase: (hash(seed as i32, 7) % 1000) as f32 / 1000.0 * 4.0,
                shadow,
            });
        };
        for (k, (name, p)) in props(src).enumerate() {
            place(name, p, k);
        }
        // Dry-stone walls from the grid: a horizontal run shows cap and face, a vertical one
        // its cap, drawn over the piece it leaves from (see `wall_*` in village.px.js).
        for r in 0..rows {
            for c in 0..cols {
                if at(c, r) != b'W' {
                    continue;
                }
                let w = |dc, dr| at(c + dc, r + dr) == b'W';
                let (l, rt, u, d) = (w(-1, 0), w(1, 0), w(0, -1), w(0, 1));
                let bl = vec2(c as f32, r as f32 + 1.0) * MAP_CELL;
                if l || rt {
                    let name = match (l, rt) {
                        (true, true) => "wall_h",
                        (false, true) => "wall_h_l",
                        _ => "wall_h_r",
                    };
                    place(name, bl, 0);
                }
                if d {
                    place(
                        if u { "wall_v" } else { "wall_v_top" },
                        bl + vec2(0.0, 0.01),
                        0,
                    );
                } else if u {
                    place("wall_v_end", bl + vec2(0.0, 0.01), 0);
                }
            }
        }
        let (flat, mut list): (Vec<_>, Vec<_>) = list
            .into_iter()
            .partition(|p| p.id == village_id("bridge").unwrap_or(usize::MAX));
        list.sort_by(|a, b| a.base().total_cmp(&b.base()));
        let tallest = list.iter().map(|p| p.size.y).fold(0.0, f32::max);
        Self {
            layers,
            grass,
            glint: terrain_id("glint").expect("glint"),
            glints,
            props: list,
            flat,
            tallest,
        }
    }

    /// Grass, then the terrain layers, over the view whose top-left is `origin`.
    pub fn draw_terrain(&self, map: &Map, origin: Vec2, view: Vec2, t: f32) {
        let tex = terrain_texture();
        let draw = |src: Rect, x: f32, y: f32| {
            draw_texture_ex(
                &tex,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    source: Some(src),
                    ..Default::default()
                },
            );
        };
        let g = 32.0;
        let local = origin - map.origin;
        let (x0, y0) = ((local.x / g).floor() as i32, (local.y / g).floor() as i32);
        for iy in y0..=y0 + (view.y / g) as i32 + 1 {
            for ix in x0..=x0 + (view.x / g) as i32 + 1 {
                let v = [0, 0, 1, 2, 2, 3][hash(ix, iy) as usize % 6];
                let p = map.origin + vec2(ix as f32, iy as f32) * g;
                draw(terrain_frame(self.grass[v], 0.0), p.x, p.y);
            }
        }
        let (cols, rows) = (map.cols as i32, map.rows as i32);
        let c = MAP_CELL;
        let (i0, j0) = (
            ((local.x - c * 0.5) / c).floor() as i32,
            ((local.y - c * 0.5) / c).floor() as i32,
        );
        let (i1, j1) = (i0 + (view.x / c) as i32 + 2, j0 + (view.y / c) as i32 + 2);
        for layer in &self.layers {
            for j in j0.max(-1)..=j1.min(rows - 1) {
                for i in i0.max(-1)..=i1.min(cols - 1) {
                    let tile = layer[((j + 1) * (cols + 1) + i + 1) as usize];
                    if tile != NONE {
                        let p = map.origin + vec2(i as f32 * c + c * 0.5, j as f32 * c + c * 0.5);
                        draw(terrain_frame(tile as usize, t), p.x, p.y);
                    }
                }
            }
        }
        for p in &self.flat {
            let (tex, src) = village_frame(p.id, t);
            let params = DrawTextureParams {
                source: Some(src),
                ..Default::default()
            };
            draw_texture_ex(&tex, p.pos.x.round(), p.pos.y.round(), WHITE, params);
        }
        for &(p, phase) in &self.glints {
            if (origin.x - 12.0..origin.x + view.x).contains(&p.x)
                && (origin.y..origin.y + view.y).contains(&p.y)
            {
                draw(terrain_frame(self.glint, t + phase), p.x, p.y);
            }
        }
    }

    /// Props in view, y-sorted with `units`: calls `unit(k)` for the k-th of `units` (base y
    /// per unit, ascending) at its place in the order. A prop in front of Veyra (base below
    /// her ground, `HOVER_HEIGHT` under her `body`) that covers her body fades, so she is
    /// never hidden.
    pub fn draw_sorted(
        &self,
        origin: Vec2,
        view: Vec2,
        t: f32,
        body: Rect,
        units: &[f32],
        mut unit: impl FnMut(usize),
    ) {
        let ground = body.bottom() + HOVER_HEIGHT;
        let top = origin.y;
        let bottom = origin.y + view.y + self.tallest;
        let first = self.props.partition_point(|p| p.base() < top);
        let mut k = 0;
        for p in &self.props[first..] {
            if p.base() > bottom {
                break;
            }
            while k < units.len() && units[k] <= p.base() {
                unit(k);
                k += 1;
            }
            if p.pos.x > origin.x + view.x || p.pos.x + p.size.x < origin.x {
                continue;
            }
            if p.shadow {
                let w = p.size.x * 0.42;
                draw_ellipse(
                    p.pos.x + p.size.x * 0.5,
                    p.base() - 1.0,
                    w,
                    (w * 0.28).max(2.0),
                    0.0,
                    Color::new(0.0, 0.0, 0.0, 0.3),
                );
            }
            let behind = p.base() > ground
                && Rect::new(p.pos.x, p.pos.y, p.size.x, p.size.y).overlaps(&body);
            let (tex, src) = village_frame(p.id, t + p.phase);
            let tint = if behind {
                Color::new(1.0, 1.0, 1.0, ROOF_FADE)
            } else {
                WHITE
            };
            draw_texture_ex(
                &tex,
                p.pos.x.round(),
                p.pos.y.round(),
                tint,
                DrawTextureParams {
                    source: Some(src),
                    ..Default::default()
                },
            );
        }
        while k < units.len() {
            unit(k);
            k += 1;
        }
    }
}
