//! Procedural demonic stone for the UI kit's Stone skin (design.md 7.14): a frame of rough-hewn
//! blocks with mortar, chips, cracks and moss around a dark slate well, generated at the exact
//! size of each panel or button (no tiling, no stretching) and cached by `kit`. The cracks
//! smoulder with hellfire in `GLOW_FRAMES` frames, like the title logo's glow.
//! Pure CPU work on `Image`s, so it is tested headless.

use macroquad::prelude::Image;

pub const GLOW_FRAMES: usize = 4;

type Rgba = [u8; 4];

const fn hex(c: u32) -> Rgba {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]
}

const CLEAR: Rgba = [0, 0, 0, 0];
/// Outline, the same plum black ink as the title art.
const INK: Rgba = hex(0x0a0608);
const MORTAR: Rgba = hex(0x0e0a0c);
/// Stone ramp, dark to lit: a cold grey with a breath of plum.
const STONE: [Rgba; 6] = [
    hex(0x151014),
    hex(0x221b20),
    hex(0x30272b),
    hex(0x3f3538),
    hex(0x54484a),
    hex(0x6c5e5d),
];
/// Moss: dark and dull, so it reads as rot on the stone, not grass.
const MOSS: [Rgba; 3] = [hex(0x182014), hex(0x25301b), hex(0x364225)];
/// The slate well inside the frame: two quiet tones and the shadow the frame casts.
const WELL: [Rgba; 3] = [hex(0x0a0608), hex(0x110b0e), hex(0x140d11)];
/// Hellfire in the cracks, dim to white-hot (the logo's ramp).
const FIRE: [Rgba; 5] = [
    hex(0x3a0a0a),
    hex(0x7a1a12),
    hex(0xc8401c),
    hex(0xf08a2c),
    hex(0xffd27a),
];

/// A generated frame: the stone; its relief, laid over whatever is spilt on it (joints and
/// grooves darken it, the lit slopes catch light, the frame shades the well); then the glow
/// over everything, `GLOW_FRAMES` idle frames followed by as many lit ones.
pub struct Stone {
    pub base: Image,
    pub relief: Image,
    pub glow: Vec<Image>,
}

/// The shadow the frame casts into the well, by depth below its top edge (the left edge casts
/// a lighter one).
const WELL_SHADOW: [u8; 5] = [190, 140, 95, 55, 25];
/// Light caught by the well's far walls (bottom and right).
const WELL_LIP: u8 = 40;

/// Small deterministic RNG (xorshift), so a panel always looks the same.
struct Rng(u32);

impl Rng {
    fn new(seed: u32) -> Self {
        Self(hash(seed) | 1)
    }

    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    fn f(&mut self) -> f32 {
        (self.next() & 0xFFFF) as f32 / 65535.0
    }

    /// In `lo..=hi`.
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u32) as i32
    }
}

fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x
}

/// Smooth value noise in 0..1 with lattice cells `scale` pixels wide.
fn noise(seed: u32, x: i32, y: i32, scale: f32) -> f32 {
    let (fx, fy) = (x as f32 / scale, y as f32 / scale);
    let (x0, y0) = (fx.floor() as i32, fy.floor() as i32);
    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let at = |x: i32, y: i32| {
        let h = hash(seed ^ (x as u32).wrapping_mul(0x27D4_EB2D) ^ (y as u32).wrapping_mul(0x1656_67B1));
        (h & 0xFFFF) as f32 / 65535.0
    };
    let top = at(x0, y0) + (at(x0 + 1, y0) - at(x0, y0)) * sx;
    let bottom = at(x0, y0 + 1) + (at(x0 + 1, y0 + 1) - at(x0, y0 + 1)) * sx;
    top + (bottom - top) * sy
}

const BAYER: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

fn bayer(x: i32, y: i32) -> f32 {
    (BAYER[(y & 3) as usize][(x & 3) as usize] + 0.5) / 16.0
}

struct Canvas {
    w: i32,
    h: i32,
    px: Vec<Rgba>,
}

impl Canvas {
    fn get(&self, x: i32, y: i32) -> Rgba {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            CLEAR
        } else {
            self.px[(y * self.w + x) as usize]
        }
    }

    fn set(&mut self, x: i32, y: i32, c: Rgba) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    fn image(&self) -> Image {
        Image {
            bytes: self.px.iter().flatten().copied().collect(),
            width: self.w as u16,
            height: self.h as u16,
        }
    }
}

/// A stone frame `thick` pixels wide around a slate well, `w` x `h` in all.
pub fn frame(w: u16, h: u16, thick: u16, seed: u32) -> Stone {
    let (w, h, t) = (w as i32, h as i32, thick as i32);
    let mut rng = Rng::new(seed);
    let mut c = Canvas {
        w,
        h,
        px: vec![CLEAR; (w * h) as usize],
    };
    let in_frame = |x: i32, y: i32| x < t || y < t || x >= w - t || y >= h - t;

    // The well: two quiet tones dithered together, darker where the frame shades it.
    for y in t..h - t {
        for x in t..w - t {
            let n = noise(seed ^ 0x5EED, x, y, 26.0);
            let mut k = if n + (bayer(x, y) - 0.5) * 0.1 > 0.6 { 2 } else { 1 };
            if y <= t + 1 || x == t || (y == t + 2 && (x + y) % 2 == 0) {
                k = 0;
            }
            c.set(x, y, WELL[k]);
        }
    }

    // A carved moulding all round: each pixel takes the profile at its depth into the frame
    // (outer lip, face, groove, inner bead), lit by how that slope faces the top-left light.
    let profile = profile(t);
    // Corner stones (wide frames only) carry a carved rune; joints cut the moulding into
    // long blocks.
    let cs = if t >= 8 { t + 4 } else { 0 };
    let in_corner = |x: i32, y: i32| {
        cs > 0 && (x < cs || x >= w - cs) && (y < cs || y >= h - cs)
    };
    let mut joints: Vec<(bool, i32)> = Vec::new(); // (horizontal edge, position along it)
    for (horizontal, len) in [(true, w), (false, h)] {
        let mut at = cs.max(t) + rng.range(20, 60);
        while at < len - cs.max(t) - 12 {
            joints.push((horizontal, at));
            at += rng.range(48, 96);
        }
    }
    let shade_jitter = rng.f() * 0.4;
    for y in 0..h {
        for x in 0..w {
            if !in_frame(x, y) {
                continue;
            }
            let grain = (noise(seed, x, y, 3.0) - 0.5) * 1.0
                + (noise(seed ^ 0x77, x, y, 10.0) - 0.5) * 0.9
                + shade_jitter;
            let light = if y < t { 0.35 } else if y >= h - t { -0.35 } else { 0.0 };
            if in_corner(x, y) {
                // a square block, bevelled on all four sides
                let bx = if x < cs { x } else { x - (w - cs) };
                let by = if y < cs { y } else { y - (h - cs) };
                let bevel = if by <= 1 || bx <= 1 {
                    1.3
                } else if by >= cs - 2 || bx >= cs - 2 {
                    -1.3
                } else {
                    0.3
                };
                let v = (2.6 + bevel + grain * 0.8 + light).round() as i32;
                c.set(x, y, STONE[v.clamp(0, 5) as usize]);
                continue;
            }
            let (dt, db, dl, dr) = (y, h - 1 - y, x, w - 1 - x);
            let d = dt.min(db).min(dl).min(dr);
            // top and left edges face the light; bottom and right face away
            let facing = if d == dt || d == dl { 1.0 } else { -1.0 };
            let (base, normal) = profile[d as usize];
            let v = (base + normal * facing * 1.3 + grain + light).round() as i32;
            c.set(x, y, STONE[v.clamp(0, 5) as usize]);
        }
    }
    // joints across the moulding: a mortar line, the block end on its right catching light
    for &(horizontal, at) in &joints {
        for d in 0..t {
            let spots = if horizontal {
                [(at, d), (at, h - 1 - d)]
            } else {
                [(d, at), (w - 1 - d, at)]
            };
            for (x, y) in spots {
                c.set(x, y, MORTAR);
                let (nx, ny) = if horizontal { (x + 1, y) } else { (x, y + 1) };
                let lit = c.get(nx, ny);
                if let Some(k) = STONE.iter().position(|&s| s == lit) {
                    c.set(nx, ny, STONE[(k + 1).min(5)]);
                }
            }
        }
    }
    // corner stones: mortar round them and an inverted pentagram cut into each
    let mut runes: Vec<(i32, i32)> = Vec::new();
    if cs > 0 {
        for (ox, oy) in [(0, 0), (w - cs, 0), (0, h - cs), (w - cs, h - cs)] {
            for i in 0..cs {
                // seams on the inner sides of the block
                let ix = if ox == 0 { cs } else { ox - 1 };
                let iy = if oy == 0 { cs } else { oy - 1 };
                c.set(ix, oy + i, MORTAR);
                c.set(ox + i, iy, MORTAR);
            }
            let (cx, cy) = (ox as f32 + cs as f32 * 0.5 - 0.5, oy as f32 + cs as f32 * 0.5 - 0.5);
            let r = cs as f32 * 0.5 - 2.6;
            let pts: Vec<(f32, f32)> = (0..5)
                .map(|k| {
                    let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::TAU / 5.0;
                    (cx + a.cos() * r, cy + a.sin() * r)
                })
                .collect();
            for k in 0..5 {
                let (a, b) = (pts[k], pts[(k + 2) % 5]);
                let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 1.5) as i32 + 1;
                for i in 0..=steps {
                    let f = i as f32 / steps as f32;
                    let (x, y) = ((a.0 + (b.0 - a.0) * f).round() as i32, (a.1 + (b.1 - a.1) * f).round() as i32);
                    c.set(x, y, STONE[0]);
                    if !runes.contains(&(x, y)) {
                        runes.push((x, y));
                    }
                }
            }
        }
    }

    // Chips: bites out of the outer edge.
    let perimeter = 2 * (w + h);
    for _ in 0..perimeter / 24 {
        let along = rng.range(0, perimeter - 1);
        for k in 0..rng.range(2, 4) {
            let (x, y) = perimeter_point(along + k, w, h);
            c.set(x, y, CLEAR);
        }
    }

    // Moss: a few rotten patches in the groove and on the lower faces.
    for y in 0..h {
        for x in 0..w {
            let px = c.get(x, y);
            if !in_frame(x, y) || px == CLEAR {
                continue;
            }
            let low = y as f32 / h as f32;
            let bias = if y >= h - t { 0.12 } else if y < t { -0.25 } else { low * 0.12 - 0.04 };
            let seam = if px == MORTAR || px == STONE[0] { 0.06 } else { 0.0 };
            let patch = noise(seed ^ 0x0B66, x, y, 22.0);
            let m = patch * 0.65 + noise(seed ^ 0x0A55, x, y, 3.0) * 0.35 + bias + seam;
            if m > 0.8 {
                c.set(x, y, MOSS[2]);
            } else if m > 0.74 {
                c.set(x, y, MOSS[1]);
            } else if m > 0.69 && (x + y) % 2 == 0 {
                c.set(x, y, MOSS[0]);
            }
        }
    }

    // Cracks across the frame; their cores smoulder (glow pixels: position, heat, phase).
    let mut embers: Vec<(i32, i32, f32, f32)> = Vec::new();
    for _ in 0..(perimeter / 46).max(2) {
        let along = rng.range(0, perimeter - 1);
        let (mut x, mut y) = perimeter_point(along, w, h);
        // across the frame, toward the well
        let (dx, dy) = if y == 0 {
            (0, 1)
        } else if y == h - 1 {
            (0, -1)
        } else if x == 0 {
            (1, 0)
        } else {
            (-1, 0)
        };
        let len = rng.range(t / 2 + 2, t + 5);
        let heat = 0.55 + rng.f() * 0.45;
        for step in 0..len {
            if !in_frame(x, y) && step > t + 1 {
                break;
            }
            if in_frame(x, y) {
                c.set(x, y, STONE[0]);
                if step > 0 && rng.f() < 0.8 {
                    embers.push((x, y, heat * (1.0 - step as f32 / len as f32 * 0.5), rng.f()));
                }
            } else {
                // a hairline runs on into the well
                c.set(x, y, WELL[0]);
            }
            x += dx;
            y += dy;
            if rng.f() < 0.4 {
                let side = if rng.f() < 0.5 { -1 } else { 1 };
                x += dy.abs() * side;
                y += dx.abs() * side;
                if in_frame(x, y) {
                    c.set(x, y, STONE[0]);
                }
            }
        }
    }

    for &(x, y) in &runes {
        embers.push((x, y, 0.85, (x + y) as f32 * 0.07));
    }

    // Ink outline wherever the stone meets the outside.
    let solid: Vec<bool> = c.px.iter().map(|p| p[3] > 0).collect();
    for y in 0..h {
        for x in 0..w {
            if !solid[(y * w + x) as usize] || !in_frame(x, y) {
                continue;
            }
            let open = |x: i32, y: i32| {
                x < 0 || y < 0 || x >= w || y >= h || !solid[(y * w + x) as usize]
            };
            if open(x - 1, y) || open(x + 1, y) || open(x, y - 1) || open(x, y + 1) {
                c.set(x, y, INK);
            }
        }
    }

    let glow = (0..2 * GLOW_FRAMES)
        .map(|f| {
            let lit = f >= GLOW_FRAMES;
            let phase = (f % GLOW_FRAMES) as f32 / GLOW_FRAMES as f32;
            let mut g = Canvas {
                w,
                h,
                px: vec![CLEAR; (w * h) as usize],
            };
            for &(x, y, heat, p) in &embers {
                if c.get(x, y) == INK {
                    continue;
                }
                let wave = 0.5 + 0.5 * ((phase - p) * std::f32::consts::TAU).cos();
                let level = heat * (0.45 + 0.55 * wave) * 4.0 + if lit { 1.3 } else { 0.0 };
                let k = level.floor() as i32;
                if k >= 1 {
                    g.set(x, y, FIRE[k.min(4) as usize]);
                }
            }
            g.image()
        })
        .collect();
    let mut relief = Canvas {
        w,
        h,
        px: vec![CLEAR; (w * h) as usize],
    };
    for y in 0..h {
        for x in 0..w {
            let p = c.get(x, y);
            if p[3] == 0 {
                continue;
            }
            let shade = if in_frame(x, y) {
                if MOSS.contains(&p) {
                    continue;
                }
                // how lit this bit of stone is, from mortar black to the lit lip
                let lum = (p[0] as i32 + p[1] as i32 + p[2] as i32) / 3;
                match lum {
                    l if l <= 0x18 => [0, 0, 0, 150],
                    l if l <= 0x22 => [0, 0, 0, 95],
                    l if l <= 0x30 => [0, 0, 0, 40],
                    l if l < 0x48 => CLEAR,
                    l => [255, 214, 196, ((l - 0x48) * 3).min(90) as u8],
                }
            } else {
                let (dy, dx) = ((y - t) as usize, (x - t) as usize);
                let top = WELL_SHADOW.get(dy).copied().unwrap_or(0);
                let left = WELL_SHADOW.get(dx).map_or(0, |&a| (a as u16 * 3 / 4) as u8);
                if y == h - t - 1 || x == w - t - 1 {
                    [255, 214, 196, WELL_LIP]
                } else {
                    [0, 0, 0, top.max(left)]
                }
            };
            relief.set(x, y, shade);
        }
    }
    Stone {
        base: c.image(),
        relief: relief.image(),
        glow,
    }
}

/// The moulding's cross-section for a frame `t` wide: (shade, slope) at each depth from the
/// outer edge. Slope +1 faces outward (lit on the top and left edges), -1 faces the well.
fn profile(t: i32) -> Vec<(f32, f32)> {
    let mut p = vec![(2.4, 0.0); t as usize];
    let t = t as usize;
    if t < 8 {
        // a slab: lip, face, a slope down to the well
        for (d, v) in p.iter_mut().enumerate() {
            *v = match d {
                0 | 1 => (3.0, 1.0),
                _ if d == t - 1 => (1.2, -1.0),
                _ if d == t - 2 => (2.2, -0.6),
                _ => (2.6, 0.0),
            };
        }
        return p;
    }
    for (d, v) in p.iter_mut().enumerate() {
        *v = match d {
            0..=2 => (3.0, 1.0),           // outer lip
            _ if d == t - 5 => (0.4, 0.0), // groove
            _ if d == t - 4 => (2.6, 1.0), // inner bead, outer side
            _ if d == t - 3 => (2.4, -1.0),
            _ if d == t - 2 => (1.6, -1.0), // slope into the well
            _ if d == t - 1 => (0.8, -0.5),
            _ => (2.5, 0.0), // flat face
        };
    }
    p
}

/// The point `along` pixels clockwise around the outer edge from the top-left corner.
fn perimeter_point(along: i32, w: i32, h: i32) -> (i32, i32) {
    let a = along.rem_euclid(2 * (w + h));
    if a < w {
        (a, 0)
    } else if a < w + h {
        (w - 1, a - w)
    } else if a < 2 * w + h {
        (w - 1 - (a - w - h), h - 1)
    } else {
        (0, h - 1 - (a - 2 * w - h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_is_deterministic_and_sized() {
        let a = frame(120, 60, 9, 7);
        let b = frame(120, 60, 9, 7);
        assert_eq!(a.base.bytes, b.base.bytes);
        assert_eq!((a.base.width, a.base.height), (120, 60));
        assert_eq!(a.glow.len(), 2 * GLOW_FRAMES);
        assert_ne!(frame(120, 60, 9, 8).base.bytes, a.base.bytes);
    }

    #[test]
    fn well_is_opaque_and_cracks_glow() {
        let s = frame(200, 80, 9, 3);
        let at = |x: usize, y: usize| s.base.bytes[(y * 200 + x) * 4 + 3];
        assert_eq!(at(100, 40), 255, "the well is solid");
        let glowing = |img: &Image| img.bytes.chunks(4).filter(|p| p[3] > 0).count();
        assert!(glowing(&s.glow[0]) > 0);
        assert!(glowing(&s.glow[GLOW_FRAMES]) >= glowing(&s.glow[0]), "lit glows more");
    }

    #[test]
    fn tiny_frames_do_not_panic() {
        for (w, h) in [(8, 8), (30, 26), (300, 26)] {
            frame(w, h, 4, 1);
        }
    }
}
