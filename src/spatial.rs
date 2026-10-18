//! Uniform grid over a fixed window centred on the player, rebuilt every tick with a
//! counting sort into flat arrays (no per-tick allocations). Points outside the window
//! are ignored; the simulation relocates far enemies so this never loses gameplay.

use macroquad::math::{vec2, Vec2};

const OUTSIDE: u32 = u32::MAX;

pub struct SpatialHash {
    cell: f32,
    cols: usize,
    rows: usize,
    origin: Vec2,
    /// Prefix sums: entries of cell `c` are `entries[cell_start[c]..cell_start[c + 1]]`.
    cell_start: Vec<u32>,
    /// Point indices sorted by cell.
    entries: Vec<u32>,
    /// Positions parallel to `entries`, a snapshot taken at rebuild time.
    points: Vec<Vec2>,
    cell_of: Vec<u32>,
}

impl SpatialHash {
    pub fn new(cell: f32, cols: usize, rows: usize, capacity: usize) -> Self {
        Self {
            cell,
            cols,
            rows,
            origin: Vec2::ZERO,
            cell_start: vec![0; cols * rows + 1],
            entries: Vec::with_capacity(capacity),
            points: Vec::with_capacity(capacity),
            cell_of: Vec::with_capacity(capacity),
        }
    }

    fn cell_coords(&self, p: Vec2) -> (i32, i32) {
        let c = (p - self.origin) / self.cell;
        (c.x.floor() as i32, c.y.floor() as i32)
    }

    fn cell_index(&self, cx: i32, cy: i32) -> Option<usize> {
        let inside = cx >= 0 && cy >= 0 && (cx as usize) < self.cols && (cy as usize) < self.rows;
        inside.then(|| cy as usize * self.cols + cx as usize)
    }

    pub fn rebuild(&mut self, center: Vec2, xs: &[f32], ys: &[f32]) {
        let half = vec2(self.cols as f32, self.rows as f32) * self.cell * 0.5;
        self.origin = center - half;
        self.cell_start.fill(0);
        self.cell_of.clear();
        for (&x, &y) in xs.iter().zip(ys) {
            let (cx, cy) = self.cell_coords(vec2(x, y));
            let c = self.cell_index(cx, cy).map_or(OUTSIDE, |c| c as u32);
            if c != OUTSIDE {
                self.cell_start[c as usize + 1] += 1;
            }
            self.cell_of.push(c);
        }
        for c in 1..self.cell_start.len() {
            self.cell_start[c] += self.cell_start[c - 1];
        }
        let total = self.cell_start[self.cols * self.rows] as usize;
        self.entries.clear();
        self.entries.resize(total, 0);
        self.points.clear();
        self.points.resize(total, Vec2::ZERO);
        // Fill using cell_start as running cursors, then shift back.
        for (i, &c) in self.cell_of.iter().enumerate() {
            if c != OUTSIDE {
                let slot = self.cell_start[c as usize] as usize;
                self.entries[slot] = i as u32;
                self.points[slot] = vec2(xs[i], ys[i]);
                self.cell_start[c as usize] += 1;
            }
        }
        for c in (1..self.cell_start.len()).rev() {
            self.cell_start[c] = self.cell_start[c - 1];
        }
        self.cell_start[0] = 0;
    }

    fn for_cell(&self, cx: i32, cy: i32, mut f: impl FnMut(usize, Vec2)) {
        self.try_for_cell(cx, cy, |i, q| {
            f(i, q);
            true
        });
    }

    /// Like `for_cell`, stopping (and returning false) as soon as `f` returns false.
    fn try_for_cell(&self, cx: i32, cy: i32, mut f: impl FnMut(usize, Vec2) -> bool) -> bool {
        if let Some(c) = self.cell_index(cx, cy) {
            let (a, b) = (self.cell_start[c] as usize, self.cell_start[c + 1] as usize);
            for s in a..b {
                if !f(self.entries[s] as usize, self.points[s]) {
                    return false;
                }
            }
        }
        true
    }

    /// Calls `f(index, position)` for every point within `r` of `p`.
    pub fn query_radius(&self, p: Vec2, r: f32, mut f: impl FnMut(usize, Vec2)) {
        self.try_query_radius(p, r, |i, q| {
            f(i, q);
            true
        });
    }

    /// Like `query_radius`, but stops as soon as `f` returns false. Lets dense hordes
    /// bail out after a few neighbours instead of scanning crowded cells.
    pub fn try_query_radius(&self, p: Vec2, r: f32, mut f: impl FnMut(usize, Vec2) -> bool) {
        let (x0, y0) = self.cell_coords(p - Vec2::splat(r));
        let (x1, y1) = self.cell_coords(p + Vec2::splat(r));
        let (x0, y0) = (x0.max(0), y0.max(0));
        let (x1, y1) = (x1.min(self.cols as i32 - 1), y1.min(self.rows as i32 - 1));
        let r2 = r * r;
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let go_on = self.try_for_cell(cx, cy, |i, q| q.distance_squared(p) > r2 || f(i, q));
                if !go_on {
                    return;
                }
            }
        }
    }

    /// Closest point within `max_r`, searching cell rings outward from `p`.
    pub fn nearest(&self, p: Vec2, max_r: f32) -> Option<usize> {
        let (cx, cy) = self.cell_coords(p);
        let max_ring = (max_r / self.cell).ceil() as i32 + 1;
        let max_r2 = max_r * max_r;
        let mut best: Option<(usize, f32)> = None;
        let visit = |cx: i32, cy: i32, best: &mut Option<(usize, f32)>| {
            self.for_cell(cx, cy, |i, q| {
                let d2 = q.distance_squared(p);
                if d2 <= max_r2 && best.is_none_or(|(_, b)| d2 < b) {
                    *best = Some((i, d2));
                }
            });
        };
        for k in 0..=max_ring {
            if k == 0 {
                visit(cx, cy, &mut best);
            } else {
                for x in cx - k..=cx + k {
                    visit(x, cy - k, &mut best);
                    visit(x, cy + k, &mut best);
                }
                for y in cy - k + 1..cy + k {
                    visit(cx - k, y, &mut best);
                    visit(cx + k, y, &mut best);
                }
            }
            // Anything in ring k+1 or beyond is at least k cells away.
            if let Some((i, d2)) = best {
                let reach = k as f32 * self.cell;
                if d2 <= reach * reach {
                    return Some(i);
                }
            }
        }
        best.map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::rand::{gen_range, srand};

    fn random_points(n: usize, spread: f32) -> (Vec<f32>, Vec<f32>) {
        let xs = (0..n).map(|_| gen_range(-spread, spread)).collect();
        let ys = (0..n).map(|_| gen_range(-spread, spread)).collect();
        (xs, ys)
    }

    fn grid() -> SpatialHash {
        SpatialHash::new(64.0, 16, 12, 1000)
    }

    #[test]
    fn rebuild_counts_only_points_inside_window() {
        let mut h = grid();
        // Window is 1024x768 centred on the origin.
        let xs = [0.0, 500.0, -511.0, 600.0, 0.0];
        let ys = [0.0, 0.0, 383.0, 0.0, -400.0];
        h.rebuild(Vec2::ZERO, &xs, &ys);
        assert_eq!(h.entries.len(), 3);
        let mut found = vec![];
        h.query_radius(Vec2::ZERO, 2000.0, |i, _| found.push(i));
        found.sort();
        assert_eq!(found, vec![0, 1, 2]);
    }

    #[test]
    fn query_radius_matches_brute_force() {
        srand(7);
        let mut h = grid();
        let (xs, ys) = random_points(800, 370.0);
        h.rebuild(vec2(10.0, -5.0), &xs, &ys);
        for _ in 0..50 {
            let p = vec2(gen_range(-500.0, 500.0), gen_range(-400.0, 400.0));
            let r = gen_range(1.0, 200.0);
            let mut got = vec![];
            h.query_radius(p, r, |i, _| got.push(i));
            got.sort();
            let want: Vec<usize> = (0..xs.len())
                .filter(|&i| vec2(xs[i], ys[i]).distance_squared(p) <= r * r)
                .collect();
            assert_eq!(got, want);
        }
    }

    #[test]
    fn nearest_matches_brute_force() {
        srand(11);
        let mut h = grid();
        let (xs, ys) = random_points(300, 380.0);
        h.rebuild(Vec2::ZERO, &xs, &ys);
        for _ in 0..100 {
            let p = vec2(gen_range(-450.0, 450.0), gen_range(-350.0, 350.0));
            let max_r = gen_range(10.0, 600.0);
            let want = (0..xs.len())
                .map(|i| (i, vec2(xs[i], ys[i]).distance(p)))
                .filter(|&(_, d)| d <= max_r)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            let got = h.nearest(p, max_r);
            match (got, want) {
                (None, None) => {}
                (Some(g), Some((_, wd))) => {
                    assert!((vec2(xs[g], ys[g]).distance(p) - wd).abs() < 1e-3);
                }
                other => panic!("mismatch {other:?}"),
            }
        }
    }

    #[test]
    fn empty_grid_has_no_nearest() {
        let mut h = grid();
        h.rebuild(Vec2::ZERO, &[], &[]);
        assert_eq!(h.nearest(Vec2::ZERO, 1000.0), None);
        h.query_radius(Vec2::ZERO, 1000.0, |_, _| panic!("unexpected point"));
    }

    #[test]
    fn queries_outside_window_are_safe() {
        let mut h = grid();
        h.rebuild(Vec2::ZERO, &[5000.0, 0.0], &[5000.0, 0.0]);
        assert_eq!(h.nearest(vec2(5000.0, 5000.0), 100.0), None);
        assert_eq!(h.nearest(vec2(-3000.0, 0.0), 10_000.0), Some(1));
        h.query_radius(vec2(5000.0, 5000.0), 50.0, |_, _| panic!("outside point"));
    }
}
