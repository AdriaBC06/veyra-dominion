//! Simulation state. Pure logic: no drawing, input or window calls.

use crate::assets::{SpriteId, FLASH_FRAME_OFFSET};
use crate::config::*;
use crate::enemies::Enemies;
use crate::spatial::SpatialHash;
use crate::upgrades::{PlayerStats, Upgrades};
use macroquad::math::Vec2;
use macroquad::rand::gen_range;
use std::f32::consts::TAU;

/// Per-tick player intent, filled by the frontend.
#[derive(Clone, Copy, Default)]
pub struct InputState {
    pub move_dir: Vec2,
}

/// Notable outcome of a tick, for the frontend's state machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimEvent {
    None,
    /// `pending_levels > 0` and `choices` are ready; call `choose_upgrade`.
    LevelUp,
    PlayerDied,
    /// Survived until VICTORY_TIME; `start_endless` keeps the run going.
    Victory,
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
    pub hp: f32,
    pub max_hp: f32,
    /// Remaining invulnerability after a hit.
    pub iframes: f32,
}

pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
}

/// A dropped soul worth `xp`; drifts to Veyra once she is within `PICKUP_RADIUS`.
pub struct Soul {
    pub pos: Vec2,
    pub xp: u32,
}

/// A short-lived visual left by the simulation (enemy vanish, smoke puff).
pub struct Fade {
    pub pos: Vec2,
    pub vel: Vec2,
    pub sprite: SpriteId,
    pub frame: usize,
    pub t: f32,
    pub duration: f32,
}

pub struct Game {
    pub time: f32,
    /// Playing on after victory, with ever tougher enemies.
    pub endless: bool,
    pub player: Player,
    pub enemies: Enemies,
    pub projectiles: Vec<Projectile>,
    pub souls: Vec<Soul>,
    pub fades: Vec<Fade>,
    pub kills: u32,
    pub level: u32,
    /// XP gathered toward the next level.
    pub xp: u32,
    /// Level-ups earned but not yet spent on an upgrade.
    pub pending_levels: u32,
    pub upgrades: Upgrades,
    pub stats: PlayerStats,
    /// Upgrade indices offered on the level-up screen.
    pub choices: [Option<usize>; 3],
    /// Seconds since the killing blow, while the death animation plays.
    pub dying: Option<f32>,
    hash: SpatialHash,
    /// Spawn ring radius, derived from the view size (see `set_view`).
    spawn_ring: f32,
    spawn_acc: f32,
    fire_cd: f32,
    next_event: usize,
    unlocked: [bool; ENEMY_DEFS.len()],
}

impl Game {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            endless: false,
            player: Player {
                pos: Vec2::ZERO,
                prev_pos: Vec2::ZERO,
                vel: Vec2::ZERO,
                facing: -1.0,
                tail_lag: Vec2::ZERO,
                hp: PLAYER_MAX_HP,
                max_hp: PLAYER_MAX_HP,
                iframes: 0.0,
            },
            enemies: Enemies::with_capacity(MAX_ENEMIES),
            projectiles: Vec::with_capacity(MAX_PROJECTILES),
            souls: Vec::with_capacity(MAX_SOULS),
            fades: Vec::with_capacity(MAX_FADES),
            kills: 0,
            level: 1,
            xp: 0,
            pending_levels: 0,
            upgrades: Upgrades::default(),
            stats: Upgrades::default().stats(),
            choices: [None; 3],
            dying: None,
            hash: SpatialHash::new(HASH_CELL, HASH_COLS, HASH_ROWS, MAX_ENEMIES),
            spawn_ring: Self::ring_radius(Vec2::new(DEFAULT_VIEW_W, LOGICAL_H)),
            spawn_acc: 0.0,
            fire_cd: 0.0,
            next_event: 0,
            unlocked: [false; ENEMY_DEFS.len()],
        }
    }

    /// Keeps spawns just outside a view of `size` logical pixels.
    pub fn set_view(&mut self, size: Vec2) {
        self.spawn_ring = Self::ring_radius(size);
    }

    fn ring_radius(view: Vec2) -> f32 {
        view.length() * 0.5 + SPAWN_RING_MARGIN
    }

    /// Advances the simulation by one fixed step.
    pub fn tick(&mut self, input: &InputState) -> SimEvent {
        self.time += TICK_DT;
        let alive = self.dying.is_none();
        if self.run_events() && alive {
            return SimEvent::Victory;
        }
        let idle = InputState::default();
        self.move_player(if alive { input } else { &idle });
        let center = self.player.pos;
        self.hash.rebuild(center, &self.enemies.x, &self.enemies.y);
        self.spawn_enemies();
        self.relocate_far_enemies();
        self.enemies.steer(TICK_DT, center, &self.hash);
        if alive {
            self.fire();
        }
        self.update_projectiles();
        if alive {
            self.contact_damage();
        }
        self.remove_dead_enemies();
        if alive {
            self.update_souls();
        }
        self.update_fades();

        if let Some(t) = &mut self.dying {
            *t += TICK_DT;
            if *t >= DEATH_TIME {
                return SimEvent::PlayerDied;
            }
        } else if self.pending_levels > 0 {
            if self.choices == [None; 3] {
                self.upgrades.roll_choices(&mut self.choices);
            }
            if self.choices[0].is_some() {
                return SimEvent::LevelUp;
            }
            self.pending_levels = 0; // everything is capped
        }
        SimEvent::None
    }

    /// Takes the upgrade in `slot` of `choices`; rerolls if more levels are pending.
    pub fn choose_upgrade(&mut self, slot: usize) {
        let Some(idx) = self.choices.get(slot).copied().flatten() else {
            return;
        };
        self.upgrades.apply(idx, &mut self.player);
        self.stats = self.upgrades.stats();
        self.pending_levels = self.pending_levels.saturating_sub(1);
        self.choices = [None; 3];
        if self.pending_levels > 0 {
            self.upgrades.roll_choices(&mut self.choices);
        }
    }

    /// The strongest touching enemy hurts Veyra, then she is briefly invulnerable.
    fn contact_damage(&mut self) {
        let p = &mut self.player;
        p.iframes = (p.iframes - TICK_DT).max(0.0);
        if p.iframes > 0.0 {
            return;
        }
        let enemies = &self.enemies;
        let mut damage = 0.0f32;
        self.hash
            .query_radius(p.pos, PLAYER_RADIUS + MAX_ENEMY_RADIUS + 4.0, |j, _| {
                let def = &ENEMY_DEFS[enemies.kind[j] as usize];
                let reach = PLAYER_RADIUS + def.radius;
                if enemies.hp[j] > 0.0 && enemies.pos(j).distance_squared(p.pos) <= reach * reach {
                    damage = damage.max(def.contact_damage);
                }
            });
        if damage <= 0.0 {
            return;
        }
        p.hp = (p.hp - damage).max(0.0);
        p.iframes = PLAYER_IFRAMES;
        if p.hp <= 0.0 {
            self.dying = Some(0.0);
            self.spawn_death_smoke();
        }
    }

    fn spawn_death_smoke(&mut self) {
        let center = self.player.pos - Vec2::Y * PROJ_DRAW_HEIGHT;
        for _ in 0..DEATH_SMOKE_PUFFS {
            if self.fades.len() >= MAX_FADES {
                break;
            }
            let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(0.0, 10.0);
            self.fades.push(Fade {
                pos: center + off,
                vel: off * 0.8 - Vec2::Y * SMOKE_RISE_SPEED * gen_range(0.6, 1.4),
                sprite: SpriteId::Smoke,
                frame: 0,
                t: gen_range(-0.3, 0.0),
                duration: DEATH_TIME,
            });
        }
    }

    /// Hellflare: shoots at the nearest enemy whenever the cooldown allows.
    fn fire(&mut self) {
        self.fire_cd = (self.fire_cd - TICK_DT).max(0.0);
        if self.fire_cd > 0.0 || self.projectiles.len() >= MAX_PROJECTILES {
            return;
        }
        let from = self.player.pos;
        if let Some(target) = self.hash.nearest(from, FIRE_RANGE) {
            let aim = (self.enemies.pos(target) - from).normalize_or(Vec2::X);
            let n = self.stats.projectiles;
            for k in 0..n {
                if self.projectiles.len() >= MAX_PROJECTILES {
                    break;
                }
                let angle = (k as f32 - (n - 1) as f32 * 0.5) * PROJ_SPREAD;
                self.projectiles.push(Projectile {
                    pos: from,
                    vel: Vec2::from_angle(angle).rotate(aim) * PROJ_SPEED,
                    life: PROJ_LIFETIME,
                });
            }
            self.fire_cd = self.stats.fire_interval;
        }
    }

    /// Moves fireballs and applies hits. Dead enemies stay in place (hp <= 0) until
    /// `remove_dead_enemies`, so hash indices stay valid for the whole pass.
    fn update_projectiles(&mut self) {
        let enemies = &mut self.enemies;
        let damage = self.stats.damage;
        let mut k = self.projectiles.len();
        while k > 0 {
            k -= 1;
            let pr = &mut self.projectiles[k];
            pr.pos += pr.vel * TICK_DT;
            pr.life -= TICK_DT;
            let mut hit = None;
            self.hash
                .query_radius(pr.pos, PROJ_RADIUS + MAX_ENEMY_RADIUS + 4.0, |j, _| {
                    let reach = PROJ_RADIUS + ENEMY_DEFS[enemies.kind[j] as usize].radius;
                    if hit.is_none()
                        && enemies.hp[j] > 0.0
                        && enemies.pos(j).distance_squared(pr.pos) <= reach * reach
                    {
                        hit = Some(j);
                    }
                });
            if let Some(j) = hit {
                enemies.hp[j] -= damage;
                enemies.flash[j] = HIT_FLASH_TIME;
            }
            if hit.is_some() || pr.life <= 0.0 {
                self.projectiles.swap_remove(k);
            }
        }
    }

    fn remove_dead_enemies(&mut self) {
        for i in (0..self.enemies.len()).rev() {
            if self.enemies.hp[i] > 0.0 {
                continue;
            }
            if self.fades.len() < MAX_FADES {
                let kind = self.enemies.kind[i] as usize;
                self.fades.push(Fade {
                    pos: self.enemies.pos(i),
                    vel: Vec2::ZERO,
                    sprite: ENEMY_DEFS[kind].sprite,
                    frame: FLASH_FRAME_OFFSET,
                    t: 0.0,
                    duration: DEATH_FADE_TIME,
                });
            }
            self.kills += 1;
            let xp = ENEMY_DEFS[self.enemies.kind[i] as usize].soul_xp;
            self.drop_soul(self.enemies.pos(i), xp);
            self.enemies.swap_remove(i);
        }
    }

    fn drop_soul(&mut self, pos: Vec2, xp: u32) {
        if self.souls.len() < MAX_SOULS {
            self.souls.push(Soul { pos, xp });
            return;
        }
        let nearest = self.souls.iter_mut().min_by(|a, b| {
            let (da, db) = (a.pos.distance_squared(pos), b.pos.distance_squared(pos));
            da.total_cmp(&db)
        });
        if let Some(s) = nearest {
            s.xp += xp;
        }
    }

    /// Pulls nearby souls toward Veyra and absorbs the ones that reach her.
    fn update_souls(&mut self) {
        let target = self.player.pos;
        let pickup = self.stats.pickup_radius;
        let mut gained = 0;
        self.souls.retain_mut(|s| {
            let off = target - s.pos;
            let d2 = off.length_squared();
            if d2 <= SOUL_COLLECT_RADIUS * SOUL_COLLECT_RADIUS {
                gained += s.xp;
                return false;
            }
            if d2 <= pickup * pickup {
                let step = SOUL_MAGNET_SPEED * TICK_DT;
                s.pos = if d2 <= step * step {
                    target
                } else {
                    s.pos + off.normalize() * step
                };
            }
            true
        });
        self.gain_xp(gained);
    }

    pub fn gain_xp(&mut self, amount: u32) {
        self.xp += amount;
        while self.xp >= xp_to_next(self.level) {
            self.xp -= xp_to_next(self.level);
            self.level += 1;
            self.pending_levels += 1;
        }
    }

    fn update_fades(&mut self) {
        self.fades.retain_mut(|f| {
            f.t += TICK_DT;
            f.pos += f.vel * TICK_DT;
            f.t < f.duration
        });
    }

    /// Linear ramp from SPAWN_RATE_START to SPAWN_RATE_END, clamped.
    pub fn spawn_rate(t: f32) -> f32 {
        let k = (t / SPAWN_RAMP_SECS).clamp(0.0, 1.0);
        SPAWN_RATE_START + (SPAWN_RATE_END - SPAWN_RATE_START) * k
    }

    /// Fires due timeline events; returns true when the victory beat is reached.
    fn run_events(&mut self) -> bool {
        let mut victory = false;
        while let Some(ev) = EVENTS.get(self.next_event) {
            if ev.time > self.time {
                break;
            }
            self.next_event += 1;
            match ev.kind {
                EventKind::Unlock(kind) => self.unlocked[kind] = true,
                EventKind::Victory => victory = !self.endless,
            }
        }
        victory
    }

    pub fn start_endless(&mut self) {
        self.endless = true;
    }

    /// Enemy HP multiplier: 1 during the run, growing after victory in endless mode.
    fn hp_scale(&self) -> f32 {
        if !self.endless {
            return 1.0;
        }
        1.0 + ENDLESS_HP_GROWTH_PER_MIN * (self.time - VICTORY_TIME).max(0.0) / 60.0
    }

    fn spawn_weight(&self, kind: usize) -> f32 {
        if !self.unlocked[kind] {
            return 0.0;
        }
        let def = &ENEMY_DEFS[kind];
        (def.weight_start + def.weight_growth_per_min * self.time / 60.0).max(0.0)
    }

    /// Weighted pick among unlocked kinds.
    fn roll_kind(&self) -> Option<usize> {
        let total: f32 = (0..ENEMY_DEFS.len()).map(|k| self.spawn_weight(k)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut r = gen_range(0.0, total);
        for k in 0..ENEMY_DEFS.len() {
            let w = self.spawn_weight(k);
            if r < w {
                return Some(k);
            }
            r -= w;
        }
        (0..ENEMY_DEFS.len())
            .rev()
            .find(|&k| self.spawn_weight(k) > 0.0)
    }

    fn ring_point(&self, angle: f32) -> Vec2 {
        self.player.pos + Vec2::from_angle(angle) * self.spawn_ring
    }

    fn spawn_enemies(&mut self) {
        self.spawn_acc += Self::spawn_rate(self.time) * TICK_DT;
        while self.spawn_acc >= 1.0 {
            let Some(kind) = self.roll_kind() else {
                self.spawn_acc = 0.0;
                return;
            };
            let (lo, hi) = ENEMY_DEFS[kind].group;
            let size = gen_range(lo as u32, hi as u32 + 1);
            self.spawn_acc -= size as f32;
            let center = self.ring_point(gen_range(0.0, TAU));
            let hp_scale = self.hp_scale();
            for _ in 0..size {
                if self.enemies.len() >= MAX_ENEMIES {
                    return;
                }
                let off =
                    Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(0.0, SPAWN_GROUP_SPREAD);
                self.enemies.spawn(kind, center + off, hp_scale);
            }
        }
    }

    /// Enemies left far behind reappear on the ring ahead of the player.
    fn relocate_far_enemies(&mut self) {
        let center = self.player.pos;
        let radius = self.spawn_ring + RELOCATE_MARGIN;
        for i in 0..self.enemies.len() {
            let off = self.enemies.pos(i) - center;
            if off.length_squared() > radius * radius {
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
        p.vel = p.vel.lerp(dir * self.stats.speed, blend);
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
        assert!(g.enemies.len() + g.kills as usize >= 4);
        for _ in 0..3 {
            g.enemies.spawn(RAT, Vec2::ZERO, 1.0);
        }
        g.player.pos = vec2(5000.0, 0.0);
        g.tick(&InputState::default());
        assert!(g.enemies.len() >= 3);
        for i in 0..g.enemies.len() {
            assert!(
                g.enemies.pos(i).distance(g.player.pos)
                    <= g.spawn_ring + SPAWN_GROUP_SPREAD + 1.0
            );
        }
    }

    #[test]
    fn spawn_ring_and_hash_window_fit_every_view() {
        let mut g = Game::new();
        for w in [MIN_LOGICAL_W, DEFAULT_VIEW_W, MAX_LOGICAL_W] {
            let view = vec2(w, LOGICAL_H);
            g.set_view(view);
            assert!(g.spawn_ring > view.length() * 0.5);
            let reach = 2.0 * (g.spawn_ring + RELOCATE_MARGIN + MAX_ENEMY_RADIUS);
            assert!(reach <= HASH_CELL * HASH_COLS.min(HASH_ROWS) as f32);
        }
    }

    #[test]
    fn max_enemy_radius_covers_every_def() {
        assert!(ENEMY_DEFS.iter().all(|d| d.radius <= MAX_ENEMY_RADIUS));
    }

    #[test]
    fn hellflare_kills_a_nearby_enemy() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY; // no random spawns
        g.enemies.spawn(RAT, vec2(80.0, 0.0), 1.0);
        for _ in 0..30 {
            g.tick(&InputState::default());
        }
        assert_eq!(g.kills, 1);
        assert_eq!(g.enemies.len(), 0);
        assert_eq!(g.fades.len(), 1);
        assert_eq!(g.souls.len(), 1);
        for _ in 0..60 {
            g.tick(&InputState::default());
        }
        assert!(g.fades.is_empty() && g.projectiles.is_empty());
    }

    #[test]
    fn contact_hurts_once_per_iframe_window_then_kills() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.fire_cd = f32::INFINITY; // keep the attacker alive
        g.enemies.spawn(VILLAGER, Vec2::ZERO, 1000.0);
        g.tick(&InputState::default());
        let dmg = ENEMY_DEFS[VILLAGER].contact_damage;
        assert_eq!(g.player.hp, PLAYER_MAX_HP - dmg);
        for _ in 0..10 {
            g.tick(&InputState::default());
        }
        assert_eq!(g.player.hp, PLAYER_MAX_HP - dmg);

        let mut died = false;
        for _ in
            0..(60.0 * (PLAYER_MAX_HP / dmg + 1.0) * PLAYER_IFRAMES + 60.0 * DEATH_TIME) as usize
        {
            died |= g.tick(&InputState::default()) == SimEvent::PlayerDied;
            if died {
                break;
            }
        }
        assert!(died && g.player.hp == 0.0);
        assert!(g.fades.iter().any(|f| f.sprite == SpriteId::Smoke));
    }

    #[test]
    fn xp_curve() {
        assert_eq!(xp_to_next(1), 14);
        for n in 1..50 {
            assert_eq!(xp_to_next(n), 8 + 6 * n);
        }
    }

    #[test]
    fn big_xp_gain_yields_several_pending_levels() {
        let mut g = Game::new();
        g.gain_xp(xp_to_next(1) + xp_to_next(2) + xp_to_next(3) + 1);
        assert_eq!((g.level, g.pending_levels, g.xp), (4, 3, 1));
    }

    #[test]
    fn souls_are_pulled_in_and_absorbed() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.souls.push(Soul {
            pos: vec2(PICKUP_RADIUS - 1.0, 0.0),
            xp: 5,
        });
        g.souls.push(Soul {
            pos: vec2(PICKUP_RADIUS * 3.0, 0.0),
            xp: 5,
        });
        for _ in 0..30 {
            g.tick(&InputState::default());
        }
        assert_eq!(g.xp, 5);
        assert_eq!(g.souls.len(), 1);
    }

    #[test]
    fn level_up_offers_choices_until_spent() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.gain_xp(xp_to_next(1) + xp_to_next(2));
        assert_eq!(g.tick(&InputState::default()), SimEvent::LevelUp);
        g.choose_upgrade(0);
        assert_eq!(g.pending_levels, 1);
        assert!(g.choices[0].is_some());
        g.choose_upgrade(2);
        assert_eq!(g.pending_levels, 0);
        assert_eq!(g.upgrades.levels.iter().sum::<u8>(), 2);
        assert_eq!(g.tick(&InputState::default()), SimEvent::None);
    }

    #[test]
    fn spawn_curve() {
        assert_eq!(Game::spawn_rate(0.0), 1.0);
        assert_eq!(Game::spawn_rate(150.0), 13.0);
        assert_eq!(Game::spawn_rate(300.0), 25.0);
        assert_eq!(Game::spawn_rate(900.0), 25.0);
        assert_eq!(Game::spawn_rate(-5.0), 1.0);
    }

    #[test]
    fn knights_only_after_two_minutes() {
        let mut g = Game::new();
        g.time = 119.0;
        g.run_events();
        assert_eq!(g.spawn_weight(KNIGHT), 0.0);
        assert!(g.spawn_weight(RAT) > 0.0 && g.spawn_weight(VILLAGER) > 0.0);
        for _ in 0..200 {
            assert_ne!(g.roll_kind(), Some(KNIGHT));
        }
        g.time = 121.0;
        g.run_events();
        assert!(g.spawn_weight(KNIGHT) > 0.0);
    }

    #[test]
    fn spawning_respects_enemy_cap() {
        let mut g = Game::new();
        g.time = 290.0;
        g.fire_cd = f32::INFINITY;
        g.player.hp = f32::INFINITY;
        while g.enemies.len() < MAX_ENEMIES - 3 {
            g.enemies.spawn(RAT, vec2(g.spawn_ring, 0.0), 1.0);
        }
        for _ in 0..120 {
            g.tick(&InputState::default());
            assert!(g.enemies.len() <= MAX_ENEMIES);
        }
        assert_eq!(g.enemies.len(), MAX_ENEMIES);
    }

    #[test]
    fn victory_at_five_minutes_then_endless_scales_hp() {
        let mut g = Game::new();
        g.time = VICTORY_TIME - TICK_DT * 0.5;
        g.player.hp = f32::INFINITY;
        g.run_events();
        assert_eq!(g.tick(&InputState::default()), SimEvent::Victory);
        g.start_endless();
        for _ in 0..60 {
            assert_ne!(g.tick(&InputState::default()), SimEvent::Victory);
        }
        g.time = VICTORY_TIME + 120.0;
        assert_eq!(g.hp_scale(), 1.0 + 2.0 * ENDLESS_HP_GROWTH_PER_MIN);
    }

    #[test]
    fn stress_at_enemy_cap() {
        macroquad::rand::srand(7);
        let mut g = Game::new();
        g.time = VICTORY_TIME;
        g.player.hp = f32::INFINITY;
        while g.enemies.len() < MAX_ENEMIES {
            let p = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(50.0, g.spawn_ring);
            g.enemies.spawn(gen_range(0, ENEMY_DEFS.len()), p, 1.0);
        }
        let mut input = InputState::default();
        let mut peak = 0;
        let ticks = 20_000;
        let started = std::time::Instant::now();
        for i in 0..ticks {
            if i % 30 == 0 {
                input.move_dir = vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0));
            }
            if g.tick(&input) == SimEvent::LevelUp {
                g.choose_upgrade(0);
                g.player.hp = f32::INFINITY; // BloodPact clamps hp to max_hp
            }
            assert!(g.dying.is_none());
            assert!(g.enemies.len() <= MAX_ENEMIES);
            assert!(g.projectiles.len() <= MAX_PROJECTILES);
            assert!(g.souls.len() <= MAX_SOULS);
            assert!(g.fades.len() <= MAX_FADES);
            peak = peak.max(g.enemies.len());
        }
        let mean_ms = started.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
        println!(
            "stress: mean tick {mean_ms:.3} ms, peak enemies {peak}, kills {}",
            g.kills
        );
        assert_eq!(peak, MAX_ENEMIES);
        assert!(g.kills > 0 && g.upgrades.levels.iter().sum::<u8>() > 0);
        assert!(mean_ms < 16.0, "mean tick {mean_ms:.3} ms");
    }

    #[test]
    fn souls_merge_when_full() {
        let mut g = Game::new();
        for i in 0..MAX_SOULS {
            g.drop_soul(vec2(i as f32 * 10.0, 0.0), 1);
        }
        g.drop_soul(vec2(21.0, 0.0), 4);
        assert_eq!(g.souls.len(), MAX_SOULS);
        assert_eq!(g.souls[2].xp, 5);
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
