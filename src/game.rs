//! Simulation state. Pure logic: no drawing, input or window calls.

use crate::config::*;
use crate::enemies::Enemies;
use crate::spatial::SpatialHash;
use macroquad::math::Vec2;
use macroquad::rand::gen_range;
use std::f32::consts::TAU;

/// Per-tick player intent, filled by the frontend.
#[derive(Clone, Copy, Default)]
pub struct InputState {
    pub move_dir: Vec2,
}

pub struct Player {
    pub pos: Vec2,
    /// Position at the previous tick, for render interpolation.
    pub prev_pos: Vec2,
    pub vel: Vec2,
    /// -1.0 facing left, 1.0 facing right.
    pub facing: f32,
    /// Smoothed tail offset trailing behind the movement.
    pub tail_lag: Vec2,
}

pub struct Game {
    pub time: f32,
    pub player: Player,
    pub enemies: Enemies,
    hash: SpatialHash,
    spawn_acc: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            player: Player {
                pos: Vec2::ZERO,
                prev_pos: Vec2::ZERO,
                vel: Vec2::ZERO,
                facing: -1.0,
                tail_lag: Vec2::ZERO,
            },
            enemies: Enemies::with_capacity(MAX_ENEMIES),
            hash: SpatialHash::new(HASH_CELL, HASH_COLS, HASH_ROWS, MAX_ENEMIES),
            spawn_acc: 0.0,
        }
    }

    /// Advances the simulation by one fixed step.
    pub fn tick(&mut self, input: &InputState) {
        self.time += TICK_DT;
        self.move_player(input);
        let center = self.player.pos;
        self.hash.rebuild(center, &self.enemies.x, &self.enemies.y);
        self.spawn_enemies();
        self.relocate_far_enemies();
        self.enemies.steer(TICK_DT, center, &self.hash);
    }

    fn ring_point(&self, angle: f32) -> Vec2 {
        self.player.pos + Vec2::from_angle(angle) * SPAWN_RING_RADIUS
    }

    fn spawn_enemies(&mut self) {
        self.spawn_acc += SPAWN_RATE_START * TICK_DT;
        while self.spawn_acc >= 1.0 {
            self.spawn_acc -= 1.0;
            if self.enemies.len() >= MAX_ENEMIES {
                continue;
            }
            let kind = if gen_range(0, 100) < 60 { RAT } else { VILLAGER };
            let pos = self.ring_point(gen_range(0.0, TAU));
            self.enemies.spawn(kind, pos, 1.0);
        }
    }

    /// Enemies left far behind reappear on the ring ahead of the player.
    fn relocate_far_enemies(&mut self) {
        let center = self.player.pos;
        for i in 0..self.enemies.len() {
            let off = self.enemies.pos(i) - center;
            if off.length_squared() > RELOCATE_RADIUS * RELOCATE_RADIUS {
                let angle = (-off).to_angle() + gen_range(-0.5, 0.5);
                let p = self.ring_point(angle);
                self.enemies.set_pos(i, p);
            }
        }
    }

    fn move_player(&mut self, input: &InputState) {
        let p = &mut self.player;
        let dir = input.move_dir.normalize_or_zero();
        let blend = 1.0 - (-PLAYER_ACCEL * TICK_DT).exp();
        p.vel = p.vel.lerp(dir * PLAYER_SPEED, blend);
        p.prev_pos = p.pos;
        p.pos += p.vel * TICK_DT;
        if p.vel.x.abs() > IMPULSE_SPEED * 0.5 {
            p.facing = p.vel.x.signum();
        }
        let tail_blend = 1.0 - (-TAIL_SMOOTHING * TICK_DT).exp();
        p.tail_lag = p.tail_lag.lerp(-p.vel * TAIL_LAG_SCALE, tail_blend);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::math::vec2;

    #[test]
    fn diagonal_speed_is_capped() {
        let mut g = Game::new();
        let input = InputState {
            move_dir: vec2(1.0, 1.0),
        };
        for _ in 0..120 {
            g.tick(&input);
        }
        assert!((g.player.vel.length() - PLAYER_SPEED).abs() < 1.0);
        assert_eq!(g.player.facing, 1.0);
    }

    #[test]
    fn enemies_spawn_on_ring_and_get_relocated() {
        let mut g = Game::new();
        for _ in 0..300 {
            g.tick(&InputState::default());
        }
        assert!(g.enemies.len() >= 4);
        g.player.pos = vec2(5000.0, 0.0);
        g.tick(&InputState::default());
        for i in 0..g.enemies.len() {
            assert!(g.enemies.pos(i).distance(g.player.pos) <= SPAWN_RING_RADIUS + 1.0);
        }
    }

    #[test]
    fn player_glides_to_rest() {
        let mut g = Game::new();
        g.player.vel = vec2(PLAYER_SPEED, 0.0);
        for _ in 0..120 {
            g.tick(&InputState::default());
        }
        assert!(g.player.vel.length() < 1.0);
        assert!(g.player.tail_lag.length() < 0.5);
    }
}
