//! Draws the map (design.md 7.14, V19d): terrain tiles under everything, then props and walls
//! y-sorted together with the horde and Veyra, roofs and canopies fading while she is behind
//! them. Renderer only; the simulation's `Map` holds the grid.

use crate::assets::{
    terrain_frame, terrain_id, terrain_texture, village_anim, village_frame, village_frame_n,
    village_id,
};
use crate::config::*;
use crate::map::{props, Map};
use crate::village::{Civilian, DropKind, PropKind, Village};
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
const NO_ID: usize = usize::MAX;
/// Per pixel walked, frames of a villager's run cycle.
const STRIDE_FRAMES: f32 = 0.25;
/// A drop's hop from where it broke out.
const DROP_HOP: f32 = 8.0;
/// Side of the buckets that index the props for `cover`, in pixels.
const COVER_CELL: f32 = 32.0;

/// A prop placed in the world.
struct Prop {
    id: usize,
    /// Top-left corner in world pixels.
    pos: Vec2,
    size: Vec2,
    /// Animation offset, so copies don't sway in step.
    phase: f32,
    shadow: bool,
    /// Its index in `Village::props` (`NONE` for scenery) and the sprites of its states.
    sim: u16,
    looks: [usize; 5],
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
    /// Props (indices into `props`) whose sprite reaches into each `COVER_CELL` bucket, row
    /// by row from `cover_origin`.
    cover: Vec<Vec<u16>>,
    cover_origin: Vec2,
    cover_cols: usize,
    flames: [usize; 3],
    folk: [usize; 5],
    /// In `DropKind` order, then the opening coffer.
    drops: [usize; 6],
}

/// Which of a prop's sprites to draw: its own, one of `looks` (cracked, burnt, burnt and
/// cracked, rubble, burnt rubble), or none (smashed).
fn look(p: &Prop, village: &Village) -> Option<usize> {
    if p.sim == NONE {
        return Some(p.id);
    }
    let q = &village.props[p.sim as usize];
    let pick = |k: usize| {
        Some(if p.looks[k] == NO_ID {
            p.id
        } else {
            p.looks[k]
        })
    };
    match q.kind {
        PropKind::Building(_) => match (q.broken, q.burnt, q.cracked()) {
            (true, false, _) => pick(3),
            (true, true, _) => pick(4),
            (false, true, true) => pick(2),
            (false, true, false) => pick(1),
            (false, false, true) => pick(0),
            _ => Some(p.id),
        },
        PropKind::Breakable(_) if q.broken => None,
        PropKind::Breakable(_) if q.cracked() => pick(0),
        _ if q.burnt => pick(1),
        _ => Some(p.id),
    }
}

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

impl MapArt {
    /// Builds the drawing of `map` from its layout text `src` (the same one it was parsed from).
    pub fn new(map: &Map, src: &str, village: &Village) -> Self {
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
        let stubble = village_id("stubble").unwrap_or(NO_ID);
        let mut place = |name: &str, bottom_left: Vec2, seed: usize, sim: Option<usize>| {
            let Some(id) = village_id(name) else { return };
            let state = |suffix: &str| village_id(&format!("{name}{suffix}")).unwrap_or(NO_ID);
            let looks = match sim.map(|i| village.props[i].kind) {
                Some(PropKind::Low) => [NO_ID, stubble, NO_ID, NO_ID, NO_ID],
                Some(_) => [
                    state("_cracked"),
                    state("_burnt"),
                    state("_burnt_cracked"),
                    state("_rubble"),
                    state("_rubble_burnt"),
                ],
                None => [NO_ID; 5],
            };
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
                sim: sim.map_or(NONE, |i| i as u16),
                looks,
            });
        };
        for (k, (name, p)) in props(src).enumerate() {
            place(name, p, k, village.of_layout(k));
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
                    place(name, bl, 0, None);
                }
                if d {
                    place(
                        if u { "wall_v" } else { "wall_v_top" },
                        bl + vec2(0.0, 0.01),
                        0,
                        None,
                    );
                } else if u {
                    place("wall_v_end", bl + vec2(0.0, 0.01), 0, None);
                }
            }
        }
        let (flat, mut list): (Vec<_>, Vec<_>) = list
            .into_iter()
            .partition(|p| p.id == village_id("bridge").unwrap_or(usize::MAX));
        list.sort_by(|a, b| a.base().total_cmp(&b.base()));
        let tallest = list.iter().map(|p| p.size.y).fold(0.0, f32::max);
        let cover_origin = map.origin - vec2(0.0, tallest);
        let extent = vec2(cols as f32, rows as f32) * MAP_CELL + vec2(0.0, tallest);
        let cover_cols = (extent.x / COVER_CELL).ceil() as usize + 1;
        let cover_rows = (extent.y / COVER_CELL).ceil() as usize + 1;
        let mut cover = vec![Vec::new(); cover_cols * cover_rows];
        for (k, p) in list.iter().enumerate() {
            let a = ((p.pos - cover_origin) / COVER_CELL).max(Vec2::ZERO);
            let b = ((p.pos + p.size - cover_origin) / COVER_CELL).max(Vec2::ZERO);
            for y in a.y as usize..=(b.y as usize).min(cover_rows - 1) {
                for x in a.x as usize..=(b.x as usize).min(cover_cols - 1) {
                    cover[y * cover_cols + x].push(k as u16);
                }
            }
        }
        let id = |n: &str| village_id(n).unwrap_or_else(|| panic!("village sprite {n}"));
        Self {
            layers,
            grass,
            glint: terrain_id("glint").expect("glint"),
            glints,
            props: list,
            flat,
            tallest,
            cover,
            cover_origin,
            cover_cols,
            flames: ["flame_s", "flame", "flame_l"].map(id),
            folk: CIVILIAN_SPRITES.map(id),
            drops: ["coin", "bread", "ham", "coffer", "relic", "coffer_open"].map(id),
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

    /// Props in view (`seen`, world pixels) and their flames, y-sorted with `units`: calls `unit(k)` for the k-th of `units` (base y
    /// per unit, ascending) at its place in the order. A prop in front of Veyra (base below
    /// her ground, `HOVER_HEIGHT` under her `body`) that covers her body fades, so she is
    /// never hidden.
    pub fn draw_sorted(
        &self,
        seen: Rect,
        t: f32,
        body: Rect,
        village: &Village,
        units: &[f32],
        mut unit: impl FnMut(usize),
    ) {
        let (origin, view) = (seen.point(), seen.size());
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
            let Some(id) = look(p, village) else {
                continue;
            };
            let (tex, src) = village_frame(id, t + p.phase);
            // States differ in height (rubble is low): all stand on the same base.
            let top = p.base() - src.h;
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
            let behind = p.base() > ground && Rect::new(p.pos.x, top, src.w, src.h).overlaps(&body);
            let tint = if behind {
                Color::new(1.0, 1.0, 1.0, ROOF_FADE)
            } else {
                WHITE
            };
            // A blow just landed on it (the hit cooldown started): it shakes.
            let struck = p.sim != NONE && {
                let q = &village.props[p.sim as usize];
                matches!(q.kind, PropKind::Building(_) | PropKind::Breakable(_))
                    && !q.broken
                    && q.hit_until - t > PROP_HIT_COOLDOWN - STRUCK_SHAKE
            };
            let shake = if struck {
                if (t * 60.0) as i32 % 2 == 0 {
                    1.0
                } else {
                    -1.0
                }
            } else {
                0.0
            };
            draw_texture_ex(
                &tex,
                p.pos.x.round() + shake,
                top.round(),
                tint,
                DrawTextureParams {
                    source: Some(src),
                    ..Default::default()
                },
            );
            if p.sim != NONE {
                let q = &village.props[p.sim as usize];
                if q.burning > 0.0 {
                    self.draw_flames(p, q.kind, top, src.h, t);
                }
            }
        }
        while k < units.len() {
            unit(k);
            k += 1;
        }
    }

    /// Flames over a burning prop: along a building's roof, in a tree's crown, at the foot
    /// of anything smaller.
    fn draw_flames(&self, p: &Prop, kind: PropKind, top: f32, h: f32, t: f32) {
        let seed = hash(p.pos.x as i32, p.pos.y as i32);
        let (size, n, y) = match kind {
            PropKind::Building(_) => (2, (p.size.x / 18.0) as usize, top + h * 0.6),
            PropKind::Charring => (1, 2, top + h * 0.55),
            _ => (0, 1, p.base()),
        };
        let n = n.max(1);
        for k in 0..n {
            let s = seed.rotate_left(k as u32 * 7);
            let size = if k % 2 == 1 { size.max(1) - 1 } else { size };
            let (tex, src) = village_frame(self.flames[size], t + (s % 97) as f32 * 0.031);
            let x = p.pos.x + p.size.x * (k as f32 + 0.5) / n as f32 + (s % 7) as f32 - 3.0;
            let bottom = y + (s >> 8 & 7) as f32 - 3.0;
            draw_texture_ex(
                &tex,
                (x - src.w * 0.5).round(),
                (bottom - src.h).round(),
                WHITE,
                DrawTextureParams {
                    source: Some(src),
                    ..Default::default()
                },
            );
        }
    }

    /// A villager on its ground point, running while it moves.
    pub fn draw_civilian(&self, c: &Civilian) {
        let id = self.folk[c.kind as usize];
        let frame = if c.walking {
            (c.stride * STRIDE_FRAMES) as usize % village_anim(id).0
        } else {
            0
        };
        let (tex, src) = village_frame_n(id, frame);
        let p = c.pos.round();
        draw_ellipse(
            p.x,
            p.y,
            src.w * 0.35,
            1.5,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.3),
        );
        draw_texture_ex(
            &tex,
            p.x - (src.w * 0.5).floor(),
            p.y - src.h + 1.0,
            WHITE,
            DrawTextureParams {
                source: Some(src),
                flip_x: c.facing > 0.0,
                ..Default::default()
            },
        );
    }

    /// What lies about to pick up: hopping out of the wreck, then on the ground; an opened
    /// coffer plays its lid and fades.
    /// The base line of the front-most prop drawn over `area` as it stands now, if any: a
    /// soul or drop there is drawn just after it, so a house or a tree never hides one.
    pub fn cover(&self, area: Rect, village: &Village) -> Option<f32> {
        let a = ((area.point() - self.cover_origin) / COVER_CELL).max(Vec2::ZERO);
        let b = ((area.point() + area.size() - self.cover_origin) / COVER_CELL).max(Vec2::ZERO);
        let rows = self.cover.len() / self.cover_cols;
        let mut front = None::<f32>;
        for y in a.y as usize..=(b.y as usize).min(rows - 1) {
            for x in a.x as usize..=(b.x as usize).min(self.cover_cols - 1) {
                for &k in &self.cover[y * self.cover_cols + x] {
                    let p = &self.props[k as usize];
                    let Some(id) = look(p, village) else { continue };
                    let h = village_frame(id, 0.0).1.h;
                    let drawn = Rect::new(p.pos.x, p.base() - h, p.size.x, h);
                    if drawn.overlaps(&area) && front.is_none_or(|f| p.base() > f) {
                        front = Some(p.base());
                    }
                }
            }
        }
        front
    }

    /// Where drop `i` is drawn, for `cover`.
    pub fn drop_area(village: &Village, i: usize) -> Rect {
        let p = village.drops[i].pos;
        Rect::new(p.x - 6.0, p.y - 18.0, 12.0, 18.0)
    }

    pub fn draw_drop(&self, village: &Village, i: usize, t: f32) {
        let d = &village.drops[i];
        let k = (d.t / DROP_SETTLE).min(1.0);
        let ground = d.from.lerp(d.pos, k).round();
        let lift = (4.0 * DROP_HOP * k * (1.0 - k)).round();
        let (frames, alpha) = match d.opened {
            Some(o) => {
                let id = self.drops[5];
                let step = village_anim(id).1.max(0.01);
                let fade = ((COFFER_LINGER - o) / 0.3).clamp(0.0, 1.0);
                (village_frame_n(id, (o / step) as usize), fade)
            }
            None => (
                village_frame(self.drops[d.kind as usize], t + i as f32 * 0.37),
                1.0,
            ),
        };
        let (tex, src) = frames;
        let shade = Color::new(0.0, 0.0, 0.0, 0.3 * alpha);
        draw_ellipse(ground.x, ground.y, src.w * 0.4, 1.5, 0.0, shade);
        if d.kind == DropKind::Relic {
            let glow = 0.18 + 0.08 * (t * 4.0).sin();
            let c = Color::new(1.0, 0.55, 0.2, glow);
            draw_circle(ground.x, ground.y - src.h * 0.5 - lift, src.w * 0.9, c);
        }
        draw_texture_ex(
            &tex,
            ground.x - (src.w * 0.5).floor(),
            ground.y - src.h + 1.0 - lift,
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                source: Some(src),
                ..Default::default()
            },
        );
    }
}
