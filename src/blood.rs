//! Permanent blood on the ground. Every `Splat` from the simulation is painted once into
//! world-space chunk images (`BLOOD_CHUNK` pixels square), uploaded as textures and drawn
//! under everything else, so a field soaked by thousands of deaths costs the same to draw
//! as a single pool. Shapes come from a seed: no two pools look alike.

use crate::config::*;
use crate::game::Splat;
use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shade {
    /// The dark lower and side edge, where the pool is thickest.
    Rim,
    Body,
    /// Wet highlight near the top, so it reads as liquid.
    Glint,
}

impl Shade {
    fn color(self) -> ([u8; 3], f32) {
        let (hex, alpha) = match self {
            Shade::Rim => (POOL_COLORS[0], 0.92),
            Shade::Body => (POOL_COLORS[1], 0.95),
            Shade::Glint => (POOL_COLORS[2], 0.9),
        };
        let [_, r, g, b] = hex.to_be_bytes();
        ([r, g, b], alpha)
    }
}

/// Xorshift, enough to vary shapes from a seed.
struct Rng(u32);

impl Rng {
    fn new(seed: u32) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * (self.next() >> 8) as f32 / (1 << 24) as f32
    }
}

/// An ellipse lying on the ground (half as tall as wide, the view's slant).
struct Lobe {
    c: Vec2,
    rx: f32,
    ry: f32,
}

impl Lobe {
    fn contains(&self, x: i32, y: i32) -> bool {
        let d = (vec2(x as f32 + 0.5, y as f32 + 0.5) - self.c) / vec2(self.rx, self.ry);
        d.length_squared() <= 1.0
    }
}

/// Half extent of a pool's pixels around its point, trail and droplets included.
fn pool_extent(radius: f32) -> (i32, i32) {
    ((radius * 3.4).ceil() as i32, (radius * 1.8).ceil() as i32)
}

/// Calls `put(dx, dy, shade)` for every pixel of a splat around its point: a pool of
/// a main lobe and loose blobs, sometimes a splashed trail, with a dark rim, a glint and
/// droplets around it, all shifted a little from the point; or for
/// `radius` 0 the small splash of a single drop.
pub fn splat_pixels(radius: f32, seed: u32, mut put: impl FnMut(i32, i32, Shade)) {
    let mut rng = Rng::new(seed);
    if radius <= 0.0 {
        put(0, 0, Shade::Body);
        let side = [(1, 0), (-1, 0), (0, 1), (1, 1)][rng.next() as usize % 4];
        if !rng.next().is_multiple_of(3) {
            put(side.0, side.1, Shade::Rim);
        }
        return;
    }
    let r = radius;
    let tau = std::f32::consts::TAU;
    // The pool lands a little off the body, so piles of deaths spread out.
    let shift = vec2(rng.range(-0.5, 0.5), rng.range(-0.25, 0.25)) * r;
    let main_rx = r * rng.range(0.6, 0.85);
    let mut lobes = vec![Lobe {
        c: shift,
        rx: main_rx,
        ry: main_rx * rng.range(0.4, 0.6),
    }];
    // Loose blobs around it, often apart from the main one.
    for _ in 0..2 + rng.next() % 3 {
        let dir = Vec2::from_angle(rng.range(0.0, tau));
        let rx = r * rng.range(0.25, 0.5);
        lobes.push(Lobe {
            c: shift + dir * r * rng.range(0.8, 1.5) * vec2(1.0, 0.5),
            rx,
            ry: (rx * rng.range(0.45, 0.65)).max(0.8),
        });
    }
    // Half the pools trail a broken run of shrinking blobs, as if splashed sideways.
    if rng.next().is_multiple_of(2) {
        let dir = Vec2::from_angle(rng.range(0.0, tau)) * vec2(1.0, 0.5);
        let mut d = main_rx * 0.8;
        let mut rx = r * 0.45;
        for _ in 0..2 + rng.next() % 3 {
            d += rx * 1.8;
            lobes.push(Lobe {
                c: shift + dir * d,
                rx,
                ry: (rx * 0.6).max(0.8),
            });
            rx *= 0.7;
        }
    }
    let inside = |x: i32, y: i32| lobes.iter().any(|l| l.contains(x, y));
    let (ext_x, ext_y) = pool_extent(r);
    let glint = (shift + vec2(-main_rx * 0.4, -main_rx * 0.15)).floor();
    let glint_len = 1 + (r / 4.0) as i32;
    for y in -ext_y..=ext_y {
        for x in -ext_x..=ext_x {
            if !inside(x, y) {
                continue;
            }
            let edge = !inside(x, y + 1) || !inside(x - 1, y) || !inside(x + 1, y);
            let is_glint = y == glint.y as i32 && (0..glint_len).contains(&(x - glint.x as i32));
            let shade = if edge {
                Shade::Rim
            } else if is_glint && inside(x, y - 1) {
                Shade::Glint
            } else {
                Shade::Body
            };
            put(x, y, shade);
        }
    }
    // Droplets flung just past the edge.
    for _ in 0..4 + rng.next() % (2 + r as u32 / 2) {
        let dir = Vec2::from_angle(rng.range(0.0, tau));
        let p = (shift + dir * r * rng.range(1.2, 2.4) * vec2(1.0, 0.5)).floor();
        let shade = if rng.next().is_multiple_of(2) {
            Shade::Rim
        } else {
            Shade::Body
        };
        put(p.x as i32, p.y as i32, shade);
    }
}

struct Chunk {
    image: Image,
    tex: Option<Texture2D>,
    dirty: bool,
    /// Last sync frame when it was on screen, for eviction.
    seen: u64,
}

/// A pool on the ground that later deaths nearby can feed.
#[derive(Clone, Copy, Debug)]
struct Site {
    center: Vec2,
    seed: u32,
    radius: f32,
}

impl Site {
    /// Whether `p` falls on this pool (measured on the ground ellipse).
    fn covers(&self, p: Vec2) -> bool {
        let d = (p - self.center) * vec2(1.0, 2.0);
        d.length_squared() <= self.radius * self.radius
    }
}

pub struct BloodLayer {
    chunks: HashMap<(i32, i32), Chunk>,
    frame: u64,
    /// Ring of the last `MAX_POOL_SITES` pools; `next_site` is the oldest once full.
    sites: Vec<Site>,
    next_site: usize,
}

impl Default for BloodLayer {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            frame: 0,
            sites: Vec::with_capacity(MAX_POOL_SITES),
            next_site: 0,
        }
    }
}

impl BloodLayer {
    /// Bakes a splat. A pool landing on a remembered one makes that one grow instead of
    /// stacking another on top, so a killing ground shows as more blood in one place.
    pub fn add(&mut self, s: &Splat) {
        if s.radius <= 0.0 {
            return self.paint_splat(s.pos, 0.0, s.seed);
        }
        let Some(i) = self.sites.iter().rposition(|site| site.covers(s.pos)) else {
            let site = Site {
                center: s.pos,
                seed: s.seed,
                radius: s.radius,
            };
            if self.sites.len() < MAX_POOL_SITES {
                self.sites.push(site);
            } else {
                self.sites[self.next_site] = site;
                self.next_site = (self.next_site + 1) % MAX_POOL_SITES;
            }
            return self.paint_splat(s.pos, s.radius, s.seed);
        };
        let site = self.sites[i];
        if site.radius >= POOL_MAX_RADIUS {
            let mut rng = Rng::new(s.seed);
            for _ in 0..FULL_POOL_SPLASHES {
                let dir = Vec2::from_angle(rng.range(0.0, std::f32::consts::TAU));
                let off = dir * site.radius * rng.range(0.9, 1.6) * vec2(1.0, 0.5);
                self.paint_splat(site.center + off, 0.0, rng.next());
            }
            return;
        }
        let radius = site.radius.hypot(s.radius).min(POOL_MAX_RADIUS);
        self.sites[i].radius = radius;
        self.paint_splat(site.center, radius, site.seed);
    }

    /// Paints the pixels of one splat shape into the chunks under it.
    fn paint_splat(&mut self, pos: Vec2, radius: f32, seed: u32) {
        let p = pos.round();
        splat_pixels(radius, seed, |dx, dy, shade| {
            self.paint(p.x as i32 + dx, p.y as i32 + dy, shade);
        });
    }

    fn chunk_key(x: i32, y: i32) -> (i32, i32) {
        let size = BLOOD_CHUNK as i32;
        (x.div_euclid(size), y.div_euclid(size))
    }

    /// Blends one pixel over what is already there, so overlapping pools pile up.
    fn paint(&mut self, x: i32, y: i32, shade: Shade) {
        let key = Self::chunk_key(x, y);
        if !self.chunks.contains_key(&key) {
            self.evict_for_new();
        }
        let frame = self.frame;
        let chunk = self.chunks.entry(key).or_insert_with(|| Chunk {
            image: Image::gen_image_color(BLOOD_CHUNK, BLOOD_CHUNK, BLANK),
            tex: None,
            dirty: true,
            seen: frame,
        });
        let size = BLOOD_CHUNK as i32;
        let i = 4 * (y.rem_euclid(size) * size + x.rem_euclid(size)) as usize;
        let px = &mut chunk.image.bytes[i..i + 4];
        let (rgb, a) = shade.color();
        let dst_a = px[3] as f32 / 255.0;
        let out_a = a + dst_a * (1.0 - a);
        for k in 0..3 {
            let mixed = rgb[k] as f32 * a + px[k] as f32 * dst_a * (1.0 - a);
            px[k] = (mixed / out_a).round() as u8;
        }
        px[3] = (out_a * 255.0).round() as u8;
        chunk.dirty = true;
    }

    /// Keeps the chunk count under `MAX_BLOOD_CHUNKS` by dropping the one seen longest ago.
    fn evict_for_new(&mut self) {
        if self.chunks.len() < MAX_BLOOD_CHUNKS {
            return;
        }
        let oldest = self
            .chunks
            .iter()
            .min_by_key(|(_, c)| c.seen)
            .map(|(&k, _)| k);
        if let Some(k) = oldest {
            self.chunks.remove(&k);
        }
    }

    fn visible(origin: Vec2, view: Vec2) -> impl Iterator<Item = (i32, i32)> {
        let (x0, y0) = Self::chunk_key(origin.x.floor() as i32, origin.y.floor() as i32);
        let end = origin + view;
        let (x1, y1) = Self::chunk_key(end.x.ceil() as i32, end.y.ceil() as i32);
        (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| (x, y)))
    }

    /// Uploads the chunks painted since the last call and marks the visible ones as seen.
    /// Needs the window (GL context).
    pub fn sync(&mut self, origin: Vec2, view: Vec2) {
        self.frame += 1;
        for key in Self::visible(origin, view) {
            if let Some(c) = self.chunks.get_mut(&key) {
                c.seen = self.frame;
            }
        }
        for c in self.chunks.values_mut().filter(|c| c.dirty) {
            match &c.tex {
                Some(tex) => tex.update(&c.image),
                None => {
                    let tex = Texture2D::from_image(&c.image);
                    tex.set_filter(FilterMode::Nearest);
                    c.tex = Some(tex);
                }
            }
            c.dirty = false;
        }
    }

    /// Draws the visible chunks with the world camera set.
    pub fn draw(&self, origin: Vec2, view: Vec2) {
        let size = BLOOD_CHUNK as f32;
        for key in Self::visible(origin, view) {
            if let Some(tex) = self.chunks.get(&key).and_then(|c| c.tex.as_ref()) {
                draw_texture(tex, key.0 as f32 * size, key.1 as f32 * size, WHITE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixels(radius: f32, seed: u32) -> Vec<(i32, i32, Shade)> {
        let mut v = vec![];
        splat_pixels(radius, seed, |x, y, s| v.push((x, y, s)));
        v
    }

    #[test]
    fn pools_vary_by_seed_and_grow_with_the_enemy() {
        let rat = ENEMY_DEFS[RAT].pool_radius();
        let brute = ENEMY_DEFS[BRUTE].pool_radius();
        assert_ne!(pixels(rat, 1), pixels(rat, 2));
        assert_eq!(pixels(rat, 7), pixels(rat, 7));
        let area = |r| (0..20).map(|s| pixels(r, s).len()).sum::<usize>();
        assert!(area(brute) > 3 * area(rat));
        for seed in 0..50 {
            let p = pixels(brute, seed);
            assert!(p.iter().any(|&(_, _, s)| s == Shade::Rim));
            assert!(p.iter().any(|&(_, _, s)| s == Shade::Body));
            let (ext_x, ext_y) = pool_extent(brute);
            assert!(p
                .iter()
                .all(|&(x, y, _)| x.abs() <= ext_x && y.abs() <= ext_y));
        }
    }

    #[test]
    fn drop_splash_is_tiny() {
        for seed in 0..20 {
            let p = pixels(0.0, seed);
            assert!((1..=2).contains(&p.len()));
            assert_eq!(p[0], (0, 0, Shade::Body));
        }
    }

    #[test]
    fn splats_paint_across_chunks_and_pile_up() {
        let mut layer = BloodLayer::default();
        let edge = BLOOD_CHUNK as f32;
        let pool = Splat {
            pos: vec2(edge, -1.0),
            radius: 8.0,
            seed: 3,
        };
        layer.add(&pool);
        assert_eq!(
            layer.chunks.len(),
            4,
            "a pool on a corner spans four chunks"
        );
        let alpha = |l: &BloodLayer| {
            let c = &l.chunks[&(1, 0)];
            c.image.bytes[3]
        };
        let once = alpha(&layer);
        layer.add(&pool);
        assert!(alpha(&layer) >= once && once > 200);
    }

    fn pool(pos: Vec2, radius: f32, seed: u32) -> Splat {
        Splat { pos, radius, seed }
    }

    #[test]
    fn pools_on_the_same_spot_grow_one_pool_up_to_the_cap() {
        let mut layer = BloodLayer::default();
        let r = ENEMY_DEFS[VILLAGER].pool_radius();
        layer.add(&pool(vec2(10.0, 10.0), r, 1));
        layer.add(&pool(vec2(12.0, 11.0), r, 2));
        assert_eq!(layer.sites.len(), 1);
        assert_eq!(layer.sites[0].center, vec2(10.0, 10.0));
        assert_eq!(layer.sites[0].seed, 1, "it keeps its shape");
        assert!((layer.sites[0].radius - r * 2f32.sqrt()).abs() < 1e-4);
        // Far enough away it is a pool of its own.
        layer.add(&pool(vec2(10.0 + 3.0 * r, 10.0), r, 3));
        assert_eq!(layer.sites.len(), 2);
        for k in 0..100 {
            layer.add(&pool(vec2(10.0, 10.0), r, 10 + k));
        }
        assert_eq!(layer.sites.len(), 2);
        assert_eq!(layer.sites[0].radius, POOL_MAX_RADIUS);
    }

    #[test]
    fn pool_sites_are_a_fixed_ring() {
        let mut layer = BloodLayer::default();
        for i in 0..MAX_POOL_SITES + 2 {
            layer.add(&pool(vec2(i as f32 * 100.0, 0.0), 4.0, 1));
        }
        assert_eq!(layer.sites.len(), MAX_POOL_SITES);
        assert_eq!(layer.sites.capacity(), MAX_POOL_SITES);
        assert_eq!(layer.sites[0].center.x, MAX_POOL_SITES as f32 * 100.0);
    }

    #[test]
    fn chunk_count_is_capped_dropping_the_least_seen() {
        let mut layer = BloodLayer::default();
        let size = BLOOD_CHUNK as f32;
        for i in 0..MAX_BLOOD_CHUNKS + 10 {
            layer.frame = i as u64;
            let pos = vec2(i as f32 * size + 10.0, 10.0);
            layer.add(&Splat {
                pos,
                radius: 0.0,
                seed: 1,
            });
        }
        assert_eq!(layer.chunks.len(), MAX_BLOOD_CHUNKS);
        assert!(!layer.chunks.contains_key(&(0, 0)));
        assert!(layer.chunks.contains_key(&(MAX_BLOOD_CHUNKS as i32 + 9, 0)));
    }
}
