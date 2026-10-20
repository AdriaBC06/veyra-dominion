//! Enemy horde stored as structure-of-arrays for a cache-friendly hot loop.

use crate::config::*;
use crate::spatial::SpatialHash;
use macroquad::math::{vec2, Vec2};

pub struct Enemies {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub hp: Vec<f32>,
    pub kind: Vec<u8>,
    /// Remaining hit-flash time.
    pub flash: Vec<f32>,
    /// Animation phase offset so the horde does not walk in lockstep.
    pub phase: Vec<f32>,
    /// Elites wear a bright outline (only the flag and the look so far; V18 gives them stats).
    pub elite: Vec<bool>,
    /// Stable id, so a piercing heart can tell the enemies it already went through.
    pub uid: Vec<u32>,
    /// Game time until which the Brimstone Halo can't burn it again.
    pub halo_until: Vec<f32>,
    next_uid: u32,
    /// Live enemies per kind, so the largest radius in play is cheap to find.
    counts: [u32; ENEMY_DEFS.len()],
}

impl Enemies {
    pub fn with_capacity(n: usize) -> Self {
        Self {
            x: Vec::with_capacity(n),
            y: Vec::with_capacity(n),
            hp: Vec::with_capacity(n),
            kind: Vec::with_capacity(n),
            flash: Vec::with_capacity(n),
            phase: Vec::with_capacity(n),
            elite: Vec::with_capacity(n),
            uid: Vec::with_capacity(n),
            halo_until: Vec::with_capacity(n),
            next_uid: 0,
            counts: [0; ENEMY_DEFS.len()],
        }
    }

    pub fn len(&self) -> usize {
        self.x.len()
    }

    pub fn pos(&self, i: usize) -> Vec2 {
        vec2(self.x[i], self.y[i])
    }

    pub fn set_pos(&mut self, i: usize, p: Vec2) {
        self.x[i] = p.x;
        self.y[i] = p.y;
    }

    pub fn spawn(&mut self, kind: usize, pos: Vec2, hp_scale: f32) {
        self.x.push(pos.x);
        self.y.push(pos.y);
        self.hp.push(ENEMY_DEFS[kind].hp * hp_scale);
        self.kind.push(kind as u8);
        self.flash.push(0.0);
        self.phase.push(macroquad::rand::gen_range(0.0, 2.0));
        self.elite.push(false);
        self.uid.push(self.next_uid);
        self.next_uid = self.next_uid.wrapping_add(1);
        self.halo_until.push(0.0);
        self.counts[kind] += 1;
    }

    pub fn swap_remove(&mut self, i: usize) {
        self.counts[self.kind[i] as usize] -= 1;
        self.x.swap_remove(i);
        self.y.swap_remove(i);
        self.hp.swap_remove(i);
        self.kind.swap_remove(i);
        self.flash.swap_remove(i);
        self.phase.swap_remove(i);
        self.elite.swap_remove(i);
        self.uid.swap_remove(i);
        self.halo_until.swap_remove(i);
    }

    /// Largest collision radius among the kinds currently alive (0 when empty); spatial
    /// queries widen by this so they never miss a big enemy's edge.
    pub fn max_radius(&self) -> f32 {
        let alive = ENEMY_DEFS.iter().zip(self.counts).filter(|(_, n)| *n > 0);
        alive.map(|(d, _)| d.radius()).fold(0.0, f32::max)
    }

    /// Moves every enemy according to its behaviour plus soft separation, using the
    /// position snapshot in `hash` for neighbours.
    pub fn steer(&mut self, dt: f32, target: Vec2, hash: &SpatialHash) {
        for i in 0..self.len() {
            let def = &ENEMY_DEFS[self.kind[i] as usize];
            let p = self.pos(i);
            let desired = match def.behavior {
                Behavior::Chase => (target - p).normalize_or_zero() * def.speed,
            };
            let mut push = Vec2::ZERO;
            let mut n = 0;
            hash.try_query_radius(p, SEPARATION_RADIUS, |j, q| {
                if j == i {
                    return true;
                }
                let d = p - q;
                let len = d.length();
                // Perfectly stacked enemies split along an index-based direction.
                let dir = if len > 1e-3 {
                    d / len
                } else {
                    Vec2::from_angle(i as f32)
                };
                push += dir * (1.0 - len / SEPARATION_RADIUS);
                n += 1;
                n < SEPARATION_MAX_NEIGHBOURS
            });
            self.set_pos(i, p + (desired + push * SEPARATION_PUSH) * dt);
            self.flash[i] = (self.flash[i] - dt).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chasers_close_in_and_spread_out() {
        let mut e = Enemies::with_capacity(8);
        let mut hash = SpatialHash::new(HASH_CELL, HASH_COLS, HASH_ROWS, 8);
        for _ in 0..4 {
            e.spawn(RAT, vec2(100.0, 0.0), 1.0);
        }
        for _ in 0..60 {
            hash.rebuild(Vec2::ZERO, &e.x, &e.y);
            e.steer(TICK_DT, Vec2::ZERO, &hash);
        }
        for i in 0..e.len() {
            assert!(e.pos(i).x < 60.0);
            for j in 0..i {
                assert!(e.pos(i).distance(e.pos(j)) > 1.0, "{i} and {j} stacked");
            }
        }
    }

    #[test]
    fn swap_remove_keeps_columns_aligned() {
        let mut e = Enemies::with_capacity(4);
        e.spawn(RAT, vec2(1.0, 1.0), 1.0);
        e.spawn(VILLAGER, vec2(2.0, 2.0), 2.0);
        e.swap_remove(0);
        assert_eq!(e.len(), 1);
        assert_eq!(e.kind[0] as usize, VILLAGER);
        assert_eq!(e.hp[0], ENEMY_DEFS[VILLAGER].hp * 2.0);
        assert_eq!(e.pos(0), vec2(2.0, 2.0));
        assert!(!e.elite[0]);
    }

    #[test]
    fn max_radius_follows_the_kinds_alive() {
        let mut e = Enemies::with_capacity(4);
        assert_eq!(e.max_radius(), 0.0);
        e.spawn(RAT, Vec2::ZERO, 1.0);
        e.spawn(BRUTE, Vec2::ZERO, 1.0);
        e.spawn(RAT, Vec2::ZERO, 1.0);
        assert_eq!(e.max_radius(), ENEMY_DEFS[BRUTE].radius());
        e.swap_remove(1);
        assert_eq!(e.max_radius(), ENEMY_DEFS[RAT].radius());
    }
}
