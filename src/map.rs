//! The map in the simulation (design.md 7.14, V19d): a grid of 16 px cells baked from an
//! authored layout, with what blocks walking and what blocks thrown things, circles pushed out
//! of solid cells, and a flow field towards Veyra that walkers follow around obstacles.
//!
//! Layout text (`assets/maps/*.map`): `size <cols> <rows>`, `start <col> <row>`, then `grid`
//! and one line of terrain characters per row, then `props` and one `<name> <x> <y>` per line
//! (bottom-left corner in map pixels; render only). `#` starts a comment at a line's start.
//! World (0, 0) is the centre of the start cell.

use crate::config::*;
use macroquad::math::{vec2, Rect, Vec2};

/// Cell flags.
const WALK: u8 = 1;
const SHOT: u8 = 2;
const UNREACHED: u16 = u16::MAX;
const CLEAR_CAP: u8 = 12;
/// The 8 neighbours with their path cost (orthogonal 2, diagonal 3).
const STEPS: [(i32, i32, u16); 8] = [
    (1, 0, 2),
    (-1, 0, 2),
    (0, 1, 2),
    (0, -1, 2),
    (1, 1, 3),
    (-1, 1, 3),
    (1, -1, 3),
    (-1, -1, 3),
];

/// What a terrain character blocks: water stops walkers only; walls, buildings, the ridge
/// and the woods stop thrown things too.
fn flags_of(t: u8) -> u8 {
    match t {
        b'~' => WALK,
        b'W' | b'H' | b'R' | b'T' => WALK | SHOT,
        _ => 0,
    }
}

pub struct Map {
    pub cols: usize,
    pub rows: usize,
    /// World position of the grid's top-left corner.
    pub origin: Vec2,
    /// One layout character per cell (`.` grass, `:` path, `#` cobbles, `~` water, `=` bridge,
    /// `W` wall, `H` building, `R` ridge, `T` woods).
    pub terrain: Vec<u8>,
    flags: Vec<u8>,
    /// Cells to the nearest one that blocks walking (4-way steps, capped at `CLEAR_CAP`).
    clear: Vec<u8>,
    /// Flow field: path cost from each cell to Veyra's, `UNREACHED` past `FLOW_RANGE`.
    dist: Vec<u16>,
    /// Dial's bucket queue (costs 2 and 3 need 4 buckets); kept between builds.
    buckets: [Vec<u32>; 4],
    /// Veyra's cell at the last build, and ticks since it.
    flow_from: Option<usize>,
    since_flow: u32,
}

impl Map {
    /// No grid at all: nothing blocks and walkers head straight for Veyra (tests, endless
    /// open ground).
    pub fn open() -> Self {
        Self {
            cols: 0,
            rows: 0,
            origin: Vec2::ZERO,
            terrain: Vec::new(),
            flags: Vec::new(),
            clear: Vec::new(),
            dist: Vec::new(),
            buckets: Default::default(),
            flow_from: None,
            since_flow: 0,
        }
    }

    /// Reads a layout; panics on a malformed one (layouts are embedded, so a test catches it).
    pub fn parse(src: &str) -> Self {
        let (mut cols, mut rows, mut start) = (0, 0, (0, 0));
        let mut terrain = Vec::new();
        let mut in_grid = false;
        for line in src.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if in_grid {
                if line == "props" {
                    break;
                }
                assert_eq!(line.len(), cols, "map row {} width", terrain.len() / cols);
                terrain.extend_from_slice(line.as_bytes());
                continue;
            }
            match line.split_whitespace().next() {
                Some("size") => (cols, rows) = (num_after(line, 1), num_after(line, 2)),
                Some("start") => start = (num_after(line, 1), num_after(line, 2)),
                Some("grid") => in_grid = true,
                _ => panic!("map: unexpected line {line:?}"),
            }
        }
        assert_eq!(terrain.len(), cols * rows, "map rows");
        let flags: Vec<u8> = terrain.iter().map(|&t| flags_of(t)).collect();
        let cap = cols * rows;
        // Multi-source BFS out from every solid cell (and the grid's edge).
        let mut clear = vec![CLEAR_CAP; cap];
        let mut queue = std::collections::VecDeque::new();
        for (i, &f) in flags.iter().enumerate() {
            let (x, y) = (i % cols, i / cols);
            if f & WALK != 0 {
                clear[i] = 0;
                queue.push_back(i);
            } else if x == 0 || y == 0 || x == cols - 1 || y == rows - 1 {
                clear[i] = 1;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            let (x, y) = ((i % cols) as i32, (i / cols) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= cols as i32 || ny >= rows as i32 {
                    continue;
                }
                let n = ny as usize * cols + nx as usize;
                if clear[n] > clear[i] + 1 {
                    clear[n] = clear[i] + 1;
                    queue.push_back(n);
                }
            }
        }
        Self {
            cols,
            rows,
            origin: -vec2(start.0 as f32 + 0.5, start.1 as f32 + 0.5) * MAP_CELL,
            terrain,
            flags,
            clear,
            dist: vec![UNREACHED; cap],
            buckets: std::array::from_fn(|_| Vec::with_capacity(cap)),
            flow_from: None,
            since_flow: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.cols == 0
    }

    /// The map's extent in world pixels (`None` on open ground).
    pub fn bounds(&self) -> Option<Rect> {
        (!self.is_open()).then(|| {
            Rect::new(
                self.origin.x,
                self.origin.y,
                self.cols as f32 * MAP_CELL,
                self.rows as f32 * MAP_CELL,
            )
        })
    }

    /// Centre of a view of `view` size following `p`, held inside the map so the camera never
    /// shows past its edge (the renderer and the spawn ring both use it).
    pub fn view_centre(&self, p: Vec2, view: Vec2) -> Vec2 {
        let Some(b) = self.bounds() else { return p };
        let half = view * 0.5;
        let axis = |v: f32, lo: f32, hi: f32, h: f32| {
            if hi - lo <= 2.0 * h {
                (lo + hi) * 0.5
            } else {
                v.clamp(lo + h, hi - h)
            }
        };
        vec2(
            axis(p.x, b.x, b.right(), half.x),
            axis(p.y, b.y, b.bottom(), half.y),
        )
    }

    fn cell(&self, p: Vec2) -> (i32, i32) {
        let c = ((p - self.origin) / MAP_CELL).floor();
        (c.x as i32, c.y as i32)
    }

    fn index(&self, cx: i32, cy: i32) -> Option<usize> {
        (cx >= 0 && cy >= 0 && (cx as usize) < self.cols && (cy as usize) < self.rows)
            .then(|| cy as usize * self.cols + cx as usize)
    }

    /// Whether the cell blocks `flag`; outside the grid everything does.
    fn solid(&self, cx: i32, cy: i32, flag: u8) -> bool {
        self.index(cx, cy).is_none_or(|i| self.flags[i] & flag != 0)
    }

    pub fn blocks_walk(&self, p: Vec2) -> bool {
        if self.is_open() {
            return false;
        }
        let (cx, cy) = self.cell(p);
        self.solid(cx, cy, WALK)
    }

    pub fn blocks_shot(&self, p: Vec2) -> bool {
        if self.is_open() {
            return false;
        }
        let (cx, cy) = self.cell(p);
        self.solid(cx, cy, SHOT)
    }

    /// A building fell: its cells under `r` (world pixels) still stop walkers, but rubble
    /// no longer stops thrown things (V19h).
    pub fn open_shot(&mut self, r: Rect) {
        if self.is_open() {
            return;
        }
        let (c0, r0) = self.cell(r.point());
        let (c1, r1) = self.cell(vec2(r.right(), r.bottom()) - 0.01);
        for cy in r0..=r1 {
            for cx in c0..=c1 {
                if let Some(i) = self.index(cx, cy) {
                    if self.terrain[i] == b'H' {
                        self.flags[i] &= !SHOT;
                    }
                }
            }
        }
    }

    /// `p` moved out of every solid cell a circle of radius `r` overlaps (the grid's edge
    /// counts as solid).
    pub fn push_out(&self, p: Vec2, r: f32) -> Vec2 {
        if self.is_open() {
            return p;
        }
        let (x0, y0) = self.cell(p - Vec2::splat(r));
        let (x1, y1) = self.cell(p + Vec2::splat(r));
        let mut q = p;
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                if !self.solid(cx, cy, WALK) {
                    continue;
                }
                let min = self.origin + vec2(cx as f32, cy as f32) * MAP_CELL;
                let max = min + Vec2::splat(MAP_CELL);
                let off = q - q.clamp(min, max);
                let d2 = off.length_squared();
                if d2 >= r * r {
                    continue;
                }
                if d2 > 1e-6 {
                    let d = d2.sqrt();
                    q += off / d * (r - d);
                } else {
                    // Centre inside the cell: out through the nearest side.
                    let sides = [q.x - min.x, max.x - q.x, q.y - min.y, max.y - q.y];
                    let (side, _) = sides.iter().enumerate().fold((0, f32::MAX), |b, (k, &v)| {
                        if v < b.1 {
                            (k, v)
                        } else {
                            b
                        }
                    });
                    match side {
                        0 => q.x = min.x - r,
                        1 => q.x = max.x + r,
                        2 => q.y = min.y - r,
                        _ => q.y = max.y + r,
                    }
                }
            }
        }
        q
    }

    /// Rebuilds the flow field towards `target` every `FLOW_REFRESH_TICKS` ticks, when Veyra
    /// has changed cell. Call once per tick.
    pub fn update_flow(&mut self, target: Vec2) {
        if self.is_open() {
            return;
        }
        self.since_flow += 1;
        let (cx, cy) = self.cell(target);
        let here = self.index(cx, cy);
        if here.is_none() || here == self.flow_from {
            return;
        }
        if self.flow_from.is_none() || self.since_flow >= FLOW_REFRESH_TICKS {
            self.build_flow(cx, cy);
        }
    }

    /// Dial's algorithm from Veyra's cell out to `FLOW_RANGE`; diagonals never cut a corner.
    fn build_flow(&mut self, sx: i32, sy: i32) {
        self.since_flow = 0;
        self.dist.fill(UNREACHED);
        let Some(s) = self.index(sx, sy) else { return };
        self.flow_from = Some(s);
        if self.flags[s] & WALK != 0 {
            return;
        }
        for b in &mut self.buckets {
            b.clear();
        }
        self.dist[s] = 0;
        self.buckets[0].push(s as u32);
        let mut pending = 1usize;
        let mut cur: u16 = 0;
        while pending > 0 {
            let b = cur as usize % 4;
            while let Some(c) = self.buckets[b].pop() {
                pending -= 1;
                let c = c as usize;
                if self.dist[c] != cur || cur >= FLOW_RANGE {
                    continue;
                }
                let (x, y) = ((c % self.cols) as i32, (c / self.cols) as i32);
                for &(dx, dy, cost) in &STEPS {
                    if self.solid(x + dx, y + dy, WALK)
                        || (dx != 0
                            && dy != 0
                            && (self.solid(x + dx, y, WALK) || self.solid(x, y + dy, WALK)))
                    {
                        continue;
                    }
                    let n = (y + dy) as usize * self.cols + (x + dx) as usize;
                    let nd = cur + cost;
                    if nd < self.dist[n] {
                        self.dist[n] = nd;
                        self.buckets[nd as usize % 4].push(n as u32);
                        pending += 1;
                    }
                }
            }
            cur += 1;
        }
    }

    /// Room around `p` in cells: steps to the nearest cell that blocks walking, up to
    /// `CLEAR_CAP` (0 inside one, `CLEAR_CAP` on open ground).
    pub fn clearance(&self, p: Vec2) -> f32 {
        if self.is_open() {
            return CLEAR_CAP as f32;
        }
        let (cx, cy) = self.cell(p);
        self.index(cx, cy).map_or(0.0, |i| self.clear[i] as f32)
    }

    /// Whether `p` is open ground the flow field reaches (spawns land only there).
    pub fn reachable(&self, p: Vec2) -> bool {
        if self.is_open() {
            return true;
        }
        let (cx, cy) = self.cell(p);
        self.index(cx, cy)
            .is_some_and(|i| self.flags[i] & WALK == 0 && self.dist[i] != UNREACHED)
    }

    /// Unit direction a walker at `p` takes towards Veyra at `goal`: straight at her while
    /// the way is about as short as the straight line, else down the flow field to the
    /// cheapest neighbour cell's centre.
    pub fn steer(&self, p: Vec2, goal: Vec2) -> Vec2 {
        let direct = (goal - p).normalize_or_zero();
        if self.is_open() {
            return direct;
        }
        let (cx, cy) = self.cell(p);
        let Some(i) = self.index(cx, cy) else {
            return direct;
        };
        let d = self.dist[i];
        if d == UNREACHED
            || d as f32 * 0.5 <= goal.distance(p) / MAP_CELL * NAV_DIRECT_RATIO + NAV_DIRECT_SLACK
        {
            return direct;
        }
        let mut best = (d, None);
        for &(dx, dy, _) in &STEPS {
            let (x, y) = (cx + dx, cy + dy);
            if self.solid(x, y, WALK)
                || (dx != 0 && dy != 0 && (self.solid(x, cy, WALK) || self.solid(cx, y, WALK)))
            {
                continue;
            }
            let n = y as usize * self.cols + x as usize;
            if self.dist[n] < best.0 {
                best = (self.dist[n], Some((x, y)));
            }
        }
        match best.1 {
            Some((x, y)) => {
                let centre = self.origin + (vec2(x as f32, y as f32) + 0.5) * MAP_CELL;
                (centre - p).normalize_or(direct)
            }
            None => direct,
        }
    }
}

fn num_after(line: &str, k: usize) -> usize {
    line.split_whitespace()
        .nth(k)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("map: bad number in {line:?}"))
}

/// The layout's props: (sprite name, bottom-left corner in map pixels).
pub fn props(src: &str) -> impl Iterator<Item = (&str, Vec2)> {
    src.lines()
        .skip_while(|l| *l != "props")
        .skip(1)
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let mut w = l.split_whitespace();
            let name = w.next().unwrap_or_default();
            let mut n = || w.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
            (name, vec2(n(), n()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 8x6 cells, a wall across the middle with a gap at the right end; start top left.
    const TEST_MAP: &str = "# test\nsize 8 6\nstart 1 1\ngrid\n\
        ........\n\
        ........\n\
        WWWWWW..\n\
        ........\n\
        ...~~...\n\
        ........\n\
        props\nwell 16 32\n";

    fn centre(m: &Map, cx: i32, cy: i32) -> Vec2 {
        m.origin + (vec2(cx as f32, cy as f32) + 0.5) * MAP_CELL
    }

    #[test]
    fn layouts_parse() {
        let m = Map::parse(TEST_MAP);
        assert_eq!((m.cols, m.rows), (8, 6));
        assert_eq!(centre(&m, 1, 1), Vec2::ZERO);
        assert!(m.blocks_walk(centre(&m, 0, 2)) && m.blocks_shot(centre(&m, 0, 2)));
        assert!(m.blocks_walk(centre(&m, 3, 4)) && !m.blocks_shot(centre(&m, 3, 4)));
        assert!(
            m.blocks_walk(centre(&m, -1, 0)),
            "outside the grid is solid"
        );
        assert_eq!(
            props(TEST_MAP).collect::<Vec<_>>(),
            vec![("well", vec2(16.0, 32.0))]
        );
        let a = Map::parse(ASHFORD_MAP);
        assert!(!a.blocks_walk(Vec2::ZERO), "Ashford's start is open");
        assert!(props(ASHFORD_MAP).count() > 0);
    }

    #[test]
    fn circles_are_pushed_out_of_walls() {
        let m = Map::parse(TEST_MAP);
        let wall_top = m.origin.y + 2.0 * MAP_CELL;
        // Overlapping the wall from above: pushed back up to touch it.
        let p = m.push_out(vec2(centre(&m, 2, 1).x, wall_top - 2.0), 6.0);
        assert!((p.y - (wall_top - 6.0)).abs() < 1e-3, "{p}");
        // Centre inside the wall: out through the nearest side.
        let p = m.push_out(vec2(centre(&m, 2, 2).x, wall_top + 3.0), 6.0);
        assert!(p.y <= wall_top - 6.0 + 1e-3, "{p}");
        // Free ground stays put.
        let q = centre(&m, 4, 0);
        assert_eq!(m.push_out(q, 6.0), q);
    }

    #[test]
    fn walkers_go_round_the_wall_through_the_gap() {
        let mut m = Map::parse(TEST_MAP);
        let veyra = centre(&m, 1, 1);
        m.update_flow(veyra);
        // Right under the wall: the straight line is blocked, the way is through the gap
        // on the right.
        let below = centre(&m, 1, 3);
        let dir = m.steer(below, veyra);
        assert!(dir.x > 0.5, "{dir}");
        // Close in the open, straight at her.
        let near = centre(&m, 3, 0);
        assert!(m
            .steer(near, veyra)
            .abs_diff_eq((veyra - near).normalize(), 1e-5));
        // Walking the field reaches her.
        let mut p = centre(&m, 0, 5);
        for _ in 0..400 {
            p = m.push_out(p + m.steer(p, veyra) * 2.0, 5.0);
        }
        assert!(p.distance(veyra) < 8.0, "stuck at {p}");
        assert!(m.reachable(centre(&m, 0, 5)) && !m.reachable(centre(&m, 3, 4)));
    }

    #[test]
    fn open_ground_blocks_nothing() {
        let m = Map::open();
        let p = vec2(500.0, -300.0);
        assert!(!m.blocks_walk(p) && !m.blocks_shot(p) && m.reachable(p));
        assert_eq!(m.push_out(p, 6.0), p);
    }
}
