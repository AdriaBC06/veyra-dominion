//! Simulation state. Pure logic: no drawing, input or window calls.

use crate::arts::{Arts, World};
use crate::assets::{SpriteId, FLASH_FRAME_OFFSET};
use crate::config::*;
use crate::enemies::Enemies;
use crate::spatial::SpatialHash;
use crate::upgrades::{Choice, PlayerStats, Upgrades};
use macroquad::math::{vec2, Vec2};
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
    /// An Abyssal Offering was opened; `reward` says what it offers. Call
    /// `accept_offering` or `sacrifice_offering`.
    Offering,
}

/// What an opened Abyssal Offering holds, until Veyra accepts or sacrifices it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reward {
    /// Index into `ARTS`: it takes its evolved form.
    Evolution(usize),
    /// Nothing can evolve: a free level of a random Art or Sin.
    Gift(Choice),
    /// Nothing left to level either: `OFFERING_HEAL` health.
    Heal,
}

/// An Abyssal Offering lying where an elite died.
pub struct Offering {
    pub pos: Vec2,
}

/// Something audible happened this tick. The frontend drains `Game::cues` and
/// turns them into sounds; each cue is kept once until drained.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cue {
    Fire,
    Lash,
    Kiss,
    Allure,
    Hellgate,
    Chain,
    /// A charmed enemy burst (Siren's Dominion).
    SirenBurst,
    /// An Abyssal Offering was opened.
    Offering,
    /// A fireball landed; armour rings differently from flesh.
    Hit {
        armored: bool,
    },
    Kill(Family),
    Soul,
    Hurt,
    Died,
    /// A new enemy kind joined the waves after the start.
    Arrival,
}

/// Every distinct cue fits: 13 plus a second hit kind and one kill per family.
const MAX_CUES: usize = 16;

pub struct Player {
    pub pos: Vec2,
    /// Position at the previous tick, for render interpolation.
    pub prev_pos: Vec2,
    pub vel: Vec2,
    /// -1.0 facing left, 1.0 facing right.
    pub facing: f32,
    /// Unit vector of the last direction moved in.
    pub aim: Vec2,
    /// Smoothed tail offset trailing behind the movement.
    pub tail_lag: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    /// Remaining invulnerability after a hit.
    pub iframes: f32,
    /// Seconds since a hand-thrown Art (Hellflare, Chain of Torment) last fired, for
    /// the cast animation.
    pub since_cast: f32,
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

/// One pixel of an enemy's death burst. It flies from `pos` (its spot on the ground) at
/// height `z`, lands, and lies there fading for `BURST_REST_TIME`.
pub struct BurstPixel {
    pub pos: Vec2,
    pub vel: Vec2,
    pub z: f32,
    pub vz: f32,
    /// RGB from the enemy's `EnemyDef::burst`.
    pub color: u32,
    /// Time spent on the ground; negative while airborne.
    pub rest: f32,
    /// A blood drop: it splashes the ground where it touches and is gone when it lands.
    pub blood: bool,
}

impl BurstPixel {
    pub fn landed(&self) -> bool {
        self.rest >= 0.0
    }

    /// Opacity: full in the air, fading out on the ground.
    pub fn alpha(&self) -> f32 {
        1.0 - (self.rest / BURST_REST_TIME).clamp(0.0, 1.0)
    }
}

/// Blood reaching the ground, queued for the renderer to bake (see `BloodLayer`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Splat {
    pub pos: Vec2,
    /// Pool radius in pixels; 0 for the small splash of a drop.
    pub radius: f32,
    /// Picks the shape, so no two pools look the same.
    pub seed: u32,
}

pub struct Game {
    pub time: f32,
    /// Playing on after victory, with ever tougher enemies.
    pub endless: bool,
    pub player: Player,
    pub enemies: Enemies,
    pub arts: Arts,
    pub souls: Vec<Soul>,
    pub offerings: Vec<Offering>,
    /// What the open Abyssal Offering holds, while its screen is up.
    pub reward: Option<Reward>,
    pub fades: Vec<Fade>,
    /// Fixed pool (capacity `MAX_BURST_PIXELS`), never grown.
    pub bursts: Vec<BurstPixel>,
    /// Blood that hit the ground since the renderer last drained it (capacity
    /// `MAX_SPLATS`; more are dropped).
    pub splats: Vec<Splat>,
    pub kills: u32,
    pub level: u32,
    /// XP gathered toward the next level.
    pub xp: u32,
    /// Fraction of a point of XP left over from Envy's bonus.
    xp_carry: f32,
    /// Level-ups earned but not yet spent on an upgrade.
    pub pending_levels: u32,
    pub upgrades: Upgrades,
    pub stats: PlayerStats,
    /// Cards offered on the level-up screen.
    pub choices: [Option<Choice>; 3],
    /// Seconds since the killing blow, while the death animation plays.
    pub dying: Option<f32>,
    /// Cues since the frontend last drained them, without repeats.
    pub cues: Vec<Cue>,
    hash: SpatialHash,
    /// Spawn ring radius, derived from the view size (see `set_view`).
    spawn_ring: f32,
    pub(crate) spawn_acc: f32,
    next_event: usize,
    unlocked: [bool; ENEMY_DEFS.len()],
}

fn splat(splats: &mut Vec<Splat>, pos: Vec2, radius: f32) {
    if splats.len() < MAX_SPLATS {
        let seed = gen_range(0, u32::MAX);
        splats.push(Splat { pos, radius, seed });
    }
}

impl Game {
    pub fn new() -> Self {
        let upgrades = Upgrades::default();
        Self {
            time: 0.0,
            endless: false,
            player: Player {
                pos: Vec2::ZERO,
                prev_pos: Vec2::ZERO,
                vel: Vec2::ZERO,
                facing: -1.0,
                aim: vec2(-1.0, 0.0),
                tail_lag: Vec2::ZERO,
                hp: PLAYER_MAX_HP,
                max_hp: PLAYER_MAX_HP,
                iframes: 0.0,
                since_cast: f32::INFINITY,
            },
            enemies: Enemies::with_capacity(MAX_ENEMIES),
            arts: Arts::new(&upgrades),
            souls: Vec::with_capacity(MAX_SOULS),
            offerings: Vec::with_capacity(MAX_OFFERINGS),
            reward: None,
            fades: Vec::with_capacity(MAX_FADES),
            bursts: Vec::with_capacity(MAX_BURST_PIXELS),
            splats: Vec::with_capacity(MAX_SPLATS),
            kills: 0,
            level: 1,
            xp: 0,
            xp_carry: 0.0,
            pending_levels: 0,
            stats: upgrades.stats(),
            upgrades,
            choices: [None; 3],
            dying: None,
            cues: Vec::with_capacity(MAX_CUES),
            hash: SpatialHash::new(HASH_CELL, HASH_COLS, HASH_ROWS, MAX_ENEMIES),
            spawn_ring: Self::ring_radius(Vec2::new(DEFAULT_VIEW_W, LOGICAL_H)),
            spawn_acc: 0.0,
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
        let speed = 1.0 + self.stats.curse * PRIDE_SPEED_SHARE;
        self.enemies
            .steer(TICK_DT, center, &self.hash, self.time, speed);
        self.use_arts(alive);
        self.drip_bleeding();
        if alive {
            self.regenerate();
            self.contact_damage();
        }
        self.remove_dead_enemies();
        let mut opened = false;
        if alive {
            self.update_souls();
            opened = self.collect_offering();
        }
        self.update_fades();
        self.update_bursts();

        if let Some(t) = &mut self.dying {
            *t += TICK_DT;
            if *t >= DEATH_TIME {
                return SimEvent::PlayerDied;
            }
        } else if opened {
            return SimEvent::Offering;
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

    fn cue(&mut self, cue: Cue) {
        if !self.cues.contains(&cue) && self.cues.len() < MAX_CUES {
            self.cues.push(cue);
        }
    }

    /// Takes the upgrade in `slot` of `choices`; rerolls if more levels are pending.
    pub fn choose_upgrade(&mut self, slot: usize) {
        let Some(idx) = self.choices.get(slot).copied().flatten() else {
            return;
        };
        self.upgrades.apply(idx, &mut self.player);
        self.stats = self.upgrades.stats();
        self.arts.refresh(&self.upgrades);
        self.pending_levels = self.pending_levels.saturating_sub(1);
        self.choices = [None; 3];
        if self.pending_levels > 0 {
            self.upgrades.roll_choices(&mut self.choices);
        }
    }

    /// Opens the first Abyssal Offering Veyra touches, if any.
    fn collect_offering(&mut self) -> bool {
        let p = self.player.pos;
        let reach = OFFERING_PICKUP * OFFERING_PICKUP;
        let Some(k) = self
            .offerings
            .iter()
            .position(|o| o.pos.distance_squared(p) <= reach)
        else {
            return false;
        };
        self.offerings.swap_remove(k);
        self.open_offering();
        true
    }

    /// Opens an Offering: decides what it holds (the first Art that can evolve, else a
    /// random level, else health) without giving it yet.
    pub fn open_offering(&mut self) -> Reward {
        let reward = if let Some(art) = self.upgrades.evolvable() {
            Reward::Evolution(art)
        } else {
            let mut pick = [None];
            self.upgrades.roll_choices(&mut pick);
            pick[0].map_or(Reward::Heal, Reward::Gift)
        };
        self.reward = Some(reward);
        self.cue(Cue::Offering);
        reward
    }

    /// Takes what the open Offering holds.
    pub fn accept_offering(&mut self) {
        match self.reward.take() {
            Some(Reward::Evolution(art)) => self.upgrades.evolve(art),
            Some(Reward::Gift(choice)) => self.upgrades.apply(choice, &mut self.player),
            Some(Reward::Heal) => self.heal(OFFERING_HEAL),
            None => return,
        }
        self.stats = self.upgrades.stats();
        self.arts.refresh(&self.upgrades);
    }

    /// Destroys the open Offering for a minor reward: some health and XP.
    pub fn sacrifice_offering(&mut self) {
        if self.reward.take().is_some() {
            self.heal(SACRIFICE_HEAL);
            self.gain_xp(SACRIFICE_XP);
        }
    }

    fn heal(&mut self, hp: f32) {
        let p = &mut self.player;
        p.hp = (p.hp + hp).min(p.max_hp);
    }

    /// Gluttony: a little health back every tick.
    fn regenerate(&mut self) {
        if self.stats.regen > 0.0 {
            self.heal(self.stats.regen * TICK_DT);
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
        let time = self.time;
        let mut damage = 0.0f32;
        let query = PLAYER_RADIUS + enemies.max_radius();
        self.hash.query_radius(p.pos, query, |j, _| {
            let def = &ENEMY_DEFS[enemies.kind[j] as usize];
            let reach = PLAYER_RADIUS + def.radius();
            let close = enemies.pos(j).distance_squared(p.pos) <= reach * reach;
            if enemies.hp[j] > 0.0 && !enemies.charmed(j, time) && close {
                damage = damage.max(def.contact_damage);
            }
        });
        if damage <= 0.0 {
            return;
        }
        let damage = (damage - self.stats.armor).max(MIN_CONTACT_DAMAGE);
        p.hp = (p.hp - damage).max(0.0);
        p.iframes = PLAYER_IFRAMES;
        if p.hp <= 0.0 {
            self.dying = Some(0.0);
            self.spawn_death_smoke();
            self.cue(Cue::Died);
        } else {
            self.cue(Cue::Hurt);
        }
    }

    fn spawn_death_smoke(&mut self) {
        // Where her body burns: she is drawn `VEYRA_DROP` below a figure on her hitbox.
        let center = self.player.pos + Vec2::Y * (VEYRA_DROP - PROJ_DRAW_HEIGHT);
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

    fn use_arts(&mut self, alive: bool) {
        let mut world = World {
            player: &mut self.player,
            enemies: &mut self.enemies,
            hash: &self.hash,
            time: self.time,
        };
        let cues = self.arts.update(&mut world, alive);
        for (on, cue) in [
            (cues.fired, Cue::Fire),
            (cues.lashed, Cue::Lash),
            (cues.kissed, Cue::Kiss),
            (cues.charmed, Cue::Allure),
            (cues.gated, Cue::Hellgate),
            (cues.chained, Cue::Chain),
            (cues.burst, Cue::SirenBurst),
            (cues.hits[0], Cue::Hit { armored: false }),
            (cues.hits[1], Cue::Hit { armored: true }),
        ] {
            if on {
                self.cue(cue);
            }
        }
    }

    /// Barbed Scourge: bleeding enemies that have blood leave drops behind them.
    fn drip_bleeding(&mut self) {
        if !self.arts.evolved(TAIL_LASH) {
            return;
        }
        let e = &self.enemies;
        for i in 0..e.len() {
            let def = &ENEMY_DEFS[e.kind[i] as usize];
            if e.bleed_until[i] > self.time && def.blood && gen_range(0.0, 1.0) < BLEED_DRIP {
                splat(&mut self.splats, e.pos(i), 0.0);
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
            let def = &ENEMY_DEFS[self.enemies.kind[i] as usize];
            let (xp, family) = (def.soul_xp, def.family);
            self.spawn_burst(def, self.enemies.pos(i));
            if def.blood {
                splat(&mut self.splats, self.enemies.pos(i), def.pool_radius());
            }
            self.cue(Cue::Kill(family));
            self.drop_soul(self.enemies.pos(i), xp);
            if self.enemies.elite[i] && self.offerings.len() < MAX_OFFERINGS {
                self.offerings.push(Offering {
                    pos: self.enemies.pos(i),
                });
            }
            self.enemies.swap_remove(i);
        }
    }

    /// Scatters `def.burst_count()` bits from the body, thrown away from Veyra, plus
    /// half as many blood drops when the enemy bleeds.
    fn spawn_burst(&mut self, def: &EnemyDef, pos: Vec2) {
        let away = (pos.x - self.player.pos.x).signum();
        let h = def.size.1 as f32;
        let bits = def.burst_count();
        let drops = if def.blood { bits / 2 } else { 0 };
        for k in 0..bits + drops {
            if self.bursts.len() >= MAX_BURST_PIXELS {
                return;
            }
            let blood = k >= bits;
            let (color, jump) = if blood {
                (BLOOD[k % BLOOD.len()], BLOOD_JUMP_SCALE)
            } else {
                (def.burst[k % def.burst.len()], 1.0)
            };
            let dir = Vec2::from_angle(gen_range(0.0, TAU));
            let speed = gen_range(BURST_SPEED.0, BURST_SPEED.1);
            // The ground plane is seen at an angle: depth moves at half speed.
            let vel = vec2(dir.x * speed + away * BURST_AWAY, dir.y * speed * 0.5);
            self.bursts.push(BurstPixel {
                pos,
                vel,
                z: h * gen_range(0.3, 0.75),
                vz: gen_range(BURST_JUMP.0, BURST_JUMP.1) * jump,
                color,
                rest: -1.0,
                blood,
            });
        }
    }

    /// Ballistic flight, one small bounce on a hard landing, then rest and fade.
    /// Blood drops splash the ground at each contact and vanish on landing.
    fn update_bursts(&mut self) {
        let splats = &mut self.splats;
        self.bursts.retain_mut(|b| {
            if b.landed() {
                b.rest += TICK_DT;
                return b.rest < BURST_REST_TIME;
            }
            b.vz -= BURST_GRAVITY * TICK_DT;
            b.z += b.vz * TICK_DT;
            b.pos += b.vel * TICK_DT;
            if b.z <= 0.0 {
                b.z = 0.0;
                if b.blood {
                    splat(splats, b.pos, 0.0);
                }
                if b.vz < -BURST_JUMP.0 * 1.5 {
                    b.vz *= -0.3;
                    b.vel *= 0.5;
                } else {
                    b.rest = 0.0;
                    return !b.blood;
                }
            }
            true
        });
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
        if gained > 0 {
            self.cue(Cue::Soul);
        }
        // Envy: every soul is worth a little more; fractions carry over.
        let xp = gained as f32 * self.stats.growth + self.xp_carry;
        self.xp_carry = xp.fract();
        self.gain_xp(xp as u32);
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
                EventKind::Unlock(kind) => {
                    self.unlocked[kind] = true;
                    if ev.time > 0.0 {
                        self.cue(Cue::Arrival);
                    }
                }
                EventKind::Victory => victory = !self.endless,
            }
        }
        victory
    }

    pub fn start_endless(&mut self) {
        self.endless = true;
    }

    /// Enemy HP multiplier: 1 during the run, growing after victory in endless mode.
    /// Pride (`curse`) adds to it either way.
    fn hp_scale(&self) -> f32 {
        let pride = 1.0 + self.stats.curse;
        if !self.endless {
            return pride;
        }
        pride * (1.0 + ENDLESS_HP_GROWTH_PER_MIN * (self.time - VICTORY_TIME).max(0.0) / 60.0)
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
        self.spawn_acc += Self::spawn_rate(self.time) * (1.0 + self.stats.curse) * TICK_DT;
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

    /// Debug (F4): one group of every enemy kind on the spawn ring, spread around Veyra.
    pub fn debug_spawn_all(&mut self) {
        let hp_scale = self.hp_scale();
        for kind in 0..ENEMY_DEFS.len() {
            let angle = kind as f32 * TAU / ENEMY_DEFS.len() as f32 + gen_range(0.0, 0.5);
            let center = self.player.pos + Vec2::from_angle(angle) * self.spawn_ring * 0.6;
            for _ in 0..[8, 6, 4, 2][kind % 4] {
                if self.enemies.len() >= MAX_ENEMIES {
                    return;
                }
                let off = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(0.0, 24.0);
                self.enemies.spawn(kind, center + off, hp_scale);
            }
        }
    }

    /// Debug (F5): enemies near Veyra become elites (real elites spawn from V18).
    pub fn debug_make_elites(&mut self) {
        let e = &mut self.enemies;
        let near = DEBUG_ELITE_RADIUS * DEBUG_ELITE_RADIUS;
        for i in 0..e.len() {
            if e.pos(i).distance_squared(self.player.pos) <= near {
                e.elite[i] = true;
            }
        }
    }

    /// Debug (F8): an Abyssal Offering on the ground just ahead of Veyra.
    pub fn debug_drop_offering(&mut self) {
        if self.offerings.len() < MAX_OFFERINGS {
            let pos = self.player.pos + self.player.aim * 40.0;
            self.offerings.push(Offering { pos });
        }
    }

    /// Debug (F7): every Art gains a level (new ones join at level 1).
    pub fn debug_level_arts(&mut self) {
        for art in 0..ARTS.len() {
            self.upgrades.apply(Choice::Art(art), &mut self.player);
        }
        self.arts.refresh(&self.upgrades);
    }

    /// Debug (F2 panel): Art `art` one level up (`up`) or down, within the slot limit.
    pub fn debug_art_level(&mut self, art: usize, up: bool) {
        if up {
            self.upgrades.apply(Choice::Art(art), &mut self.player);
        } else {
            self.upgrades.drop_art_level(art);
        }
        self.stats = self.upgrades.stats();
        self.arts.refresh(&self.upgrades);
    }

    /// Debug (F2 panel): Sin `sin` one level up (`up`) or down, within the slot limit.
    pub fn debug_sin_level(&mut self, sin: usize, up: bool) {
        if up {
            self.upgrades.apply(Choice::Sin(sin), &mut self.player);
        } else {
            self.upgrades.drop_sin_level(sin, &mut self.player);
        }
        self.stats = self.upgrades.stats();
        self.arts.refresh(&self.upgrades);
    }

    /// Debug (F2 panel): evolves `art` straight away if it could evolve (top level and
    /// its Sin owned). Returns whether it did.
    pub fn debug_evolve(&mut self, art: usize) -> bool {
        if !self.upgrades.can_evolve(art) {
            return false;
        }
        self.upgrades.evolve(art);
        self.arts.refresh(&self.upgrades);
        true
    }

    /// Debug (F6): every enemy inside a view of `view` size around Veyra dies on the
    /// next tick, with its burst, blood and soul.
    pub fn debug_kill_visible(&mut self, view: Vec2) {
        let e = &mut self.enemies;
        let half = view * 0.5;
        for i in 0..e.len() {
            let off = (e.pos(i) - self.player.pos).abs();
            if off.x <= half.x && off.y <= half.y {
                e.hp[i] = 0.0;
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
        if dir != Vec2::ZERO {
            p.aim = dir;
        }
        let blend = 1.0 - (-PLAYER_ACCEL * TICK_DT).exp();
        p.vel = p.vel.lerp(dir * self.stats.speed, blend);
        p.prev_pos = p.pos;
        p.pos += p.vel * TICK_DT;
        p.since_cast += TICK_DT;
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

    fn largest_radius() -> f32 {
        ENEMY_DEFS.iter().map(|d| d.radius()).fold(0.0, f32::max)
    }

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
                g.enemies.pos(i).distance(g.player.pos) <= g.spawn_ring + SPAWN_GROUP_SPREAD + 1.0
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
            let reach = 2.0 * (g.spawn_ring + RELOCATE_MARGIN + largest_radius());
            assert!(reach <= HASH_CELL * HASH_COLS.min(HASH_ROWS) as f32);
        }
    }

    #[test]
    fn radii_and_bursts_follow_sprite_size() {
        // Derived from the sprites, these match the hand-tuned radii they replace.
        let radii: Vec<f32> = ENEMY_DEFS.iter().map(|d| d.radius()).collect();
        assert_eq!(radii, [5.0, 6.0, 7.0, 11.0]);
        assert_eq!(largest_radius(), 11.0);
        assert!(ENEMY_DEFS
            .iter()
            .all(|d| (6..=10).contains(&d.burst_count())));
        assert_eq!(ENEMY_DEFS[RAT].burst_count(), 6);
        assert_eq!(ENEMY_DEFS[BRUTE].burst_count(), 10);
    }

    #[test]
    fn brute_edge_is_hit_with_only_rats_nearby() {
        // The query must widen by the largest radius in play, not the nearest kind's.
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold();
        g.enemies
            .spawn(BRUTE, vec2(PLAYER_RADIUS + 10.5, 0.0), 1000.0);
        g.enemies.spawn(RAT, vec2(-200.0, 0.0), 1.0);
        g.tick(&InputState::default());
        assert_eq!(
            g.player.hp,
            PLAYER_MAX_HP - ENEMY_DEFS[BRUTE].contact_damage
        );
    }

    #[test]
    fn death_burst_falls_lands_and_fades() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold();
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 1.0);
        g.enemies.hp[0] = 0.0;
        g.tick(&InputState::default());
        let def = &ENEMY_DEFS[KNIGHT];
        let (bits, drops) = (def.burst_count(), def.burst_count() / 2);
        assert_eq!(g.bursts.len(), bits + drops);
        let own = |b: &BurstPixel| def.burst.contains(&b.color) as usize;
        assert_eq!(g.bursts.iter().map(own).sum::<usize>(), bits);
        assert!(g.bursts[bits..].iter().all(|b| BLOOD.contains(&b.color)));
        assert!(g.bursts.iter().all(|b| b.z > 0.0));
        assert!(g
            .bursts
            .iter()
            .all(|b| b.vel.x > -BURST_SPEED.1 + BURST_AWAY));
        let mut landed = false;
        for _ in 0..120 {
            g.tick(&InputState::default());
            landed |= g.bursts.iter().any(|b| b.landed() && b.z == 0.0);
        }
        assert!(landed && g.bursts.is_empty());
        assert!(g.bursts.capacity() == MAX_BURST_PIXELS);
    }

    #[test]
    fn bleeding_deaths_leave_a_pool_and_drop_splashes() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold();
        g.enemies.spawn(VILLAGER, vec2(60.0, 0.0), 1.0);
        g.enemies.hp[0] = 0.0;
        g.tick(&InputState::default());
        // Where it fell: it still stepped toward Veyra during the tick.
        assert_eq!(g.splats.len(), 1);
        assert!(g.splats[0].pos.distance(vec2(60.0, 0.0)) < 2.0);
        assert_eq!(g.splats[0].radius, ENEMY_DEFS[VILLAGER].pool_radius());
        let drops = g.bursts.iter().filter(|b| b.blood).count();
        assert!(drops > 0);
        for _ in 0..120 {
            g.tick(&InputState::default());
        }
        assert!(g.bursts.iter().all(|b| !b.blood));
        let splashes = g.splats[1..].iter().filter(|s| s.radius == 0.0).count();
        assert!(splashes >= drops, "one splash per contact");
    }

    #[test]
    fn splat_queue_is_capped() {
        let mut g = Game::new();
        for _ in 0..MAX_SPLATS + 5 {
            splat(&mut g.splats, Vec2::ZERO, 1.0);
        }
        assert_eq!(g.splats.len(), MAX_SPLATS);
        assert_eq!(g.splats.capacity(), MAX_SPLATS);
    }

    #[test]
    fn bloodless_enemies_leave_no_blood() {
        let mut g = Game::new();
        let def = EnemyDef {
            blood: false,
            ..ENEMY_DEFS[KNIGHT]
        };
        g.spawn_burst(&def, Vec2::ZERO);
        assert_eq!(g.bursts.len(), def.burst_count());
        assert!(g.bursts.iter().all(|b| !BLOOD.contains(&b.color)));
        for _ in 0..120 {
            g.update_bursts();
        }
        assert!(g.splats.is_empty());
    }

    #[test]
    fn debug_kill_takes_only_the_visible_enemies() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold();
        let view = vec2(DEFAULT_VIEW_W, LOGICAL_H);
        g.enemies.spawn(RAT, vec2(300.0, 170.0), 1.0);
        g.enemies.spawn(BRUTE, vec2(-250.0, -100.0), 1.0);
        g.enemies.spawn(RAT, vec2(330.0, 0.0), 1.0);
        g.debug_kill_visible(view);
        g.tick(&InputState::default());
        assert_eq!(g.kills, 2);
        assert_eq!(g.enemies.len(), 1);
        assert_eq!(g.souls.len(), 2);
    }

    #[test]
    fn debug_elites_are_the_nearby_ones() {
        let mut g = Game::new();
        g.enemies
            .spawn(RAT, vec2(DEBUG_ELITE_RADIUS - 1.0, 0.0), 1.0);
        g.enemies
            .spawn(RAT, vec2(DEBUG_ELITE_RADIUS + 1.0, 0.0), 1.0);
        g.debug_make_elites();
        assert_eq!(g.enemies.elite, [true, false]);
    }

    #[test]
    fn hellflare_kills_a_nearby_enemy() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY; // no random spawns
        g.enemies.spawn(RAT, vec2(80.0, 0.0), 1.0);
        for _ in 0..30 {
            g.tick(&InputState::default());
            if g.kills > 0 {
                break;
            }
        }
        assert_eq!(g.kills, 1);
        assert_eq!(g.enemies.len(), 0);
        let hit = Cue::Hit { armored: false };
        for cue in [Cue::Fire, hit, Cue::Kill(Family::Beast)] {
            assert_eq!(g.cues.iter().filter(|&&c| c == cue).count(), 1, "{cue:?}");
        }
        assert_eq!(g.fades.len(), 1);
        let bits = ENEMY_DEFS[RAT].burst_count();
        assert_eq!(g.bursts.len(), bits + bits / 2);
        assert_eq!(g.souls.len(), 1);
        for _ in 0..90 {
            g.tick(&InputState::default());
        }
        assert!(g.fades.is_empty() && g.bursts.is_empty() && g.arts.projectiles.is_empty());
    }

    #[test]
    fn contact_hurts_once_per_iframe_window_then_kills() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold(); // keep the attacker alive
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
        assert!(g.cues.contains(&Cue::Hurt) && g.cues.contains(&Cue::Died));
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
        assert_eq!(g.cues, [Cue::Soul]);
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
        let arts: u8 = g.upgrades.arts.iter().sum();
        assert_eq!(
            g.upgrades.sins.iter().sum::<u8>() + arts,
            3,
            "Hellflare + 2"
        );
        assert_eq!(g.tick(&InputState::default()), SimEvent::None);
    }

    #[test]
    fn elites_drop_an_offering_that_evolves_an_art() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.upgrades.arts[HELLFLARE] = ART_MAX_LEVEL;
        g.upgrades.sins[ARTS[HELLFLARE].evolution.sin] = 1;
        g.arts.refresh(&g.upgrades);
        g.arts.hold();
        g.enemies.spawn(RAT, vec2(200.0, 0.0), 1.0);
        g.enemies.elite[0] = true;
        g.enemies.hp[0] = 0.0;
        g.enemies.spawn(RAT, vec2(-200.0, 0.0), 1.0);
        g.enemies.hp[1] = 0.0;
        assert_eq!(g.tick(&InputState::default()), SimEvent::None);
        assert_eq!(g.offerings.len(), 1, "only the elite drops one");
        g.player.pos = g.offerings[0].pos;
        assert_eq!(g.tick(&InputState::default()), SimEvent::Offering);
        assert!(g.offerings.is_empty());
        assert_eq!(g.reward, Some(Reward::Evolution(HELLFLARE)));
        assert!(!g.upgrades.evolved[HELLFLARE], "not before she accepts");
        assert!(g.cues.contains(&Cue::Offering));
        g.accept_offering();
        assert_eq!(g.reward, None);
        assert!(g.upgrades.evolved[HELLFLARE] && g.arts.evolved(HELLFLARE));
    }

    #[test]
    fn debug_panel_levels_respect_slots_and_evolution_needs_its_sin() {
        let mut g = Game::new();
        assert!(!g.debug_evolve(HELLFLARE), "level 1");
        for _ in 0..10 {
            g.debug_art_level(HELLFLARE, true);
        }
        assert_eq!(g.upgrades.arts[HELLFLARE], ART_MAX_LEVEL);
        assert!(!g.debug_evolve(HELLFLARE), "no Legion");
        g.debug_sin_level(LEGION, true);
        assert!(g.debug_evolve(HELLFLARE) && g.arts.evolved(HELLFLARE));
        assert!(!g.debug_evolve(HELLFLARE), "only once");
        // Down a level loses the evolution; down to 0 frees the slot.
        g.debug_art_level(HELLFLARE, false);
        assert!(!g.upgrades.evolved[HELLFLARE]);
        for _ in 0..10 {
            g.debug_art_level(HELLFLARE, false);
        }
        assert_eq!(g.upgrades.owned_arts(), 0);
        assert!(!g.arts.owns(HELLFLARE));
        // Slots still cap new Arts and Sins.
        for art in 0..ARTS.len() {
            g.debug_art_level(art, true);
        }
        assert_eq!(g.upgrades.owned_arts(), MAX_ARTS);
        for sin in 0..SINS.len() {
            g.debug_sin_level(sin, true);
        }
        assert_eq!(g.upgrades.owned_sins(), MAX_SINS);
        // Gluttony down keeps her health within the new maximum.
        g.debug_sin_level(GLUTTONY, false);
        assert_eq!(g.player.max_hp, PLAYER_MAX_HP);
        assert!(g.player.hp <= g.player.max_hp);
    }

    #[test]
    fn a_sacrificed_offering_heals_and_gives_xp_instead() {
        let mut g = Game::new();
        g.upgrades.arts[HELLFLARE] = ART_MAX_LEVEL;
        g.upgrades.sins[ARTS[HELLFLARE].evolution.sin] = 1;
        g.player.hp = 10.0;
        assert_eq!(g.open_offering(), Reward::Evolution(HELLFLARE));
        g.sacrifice_offering();
        assert!(!g.upgrades.evolved[HELLFLARE]);
        assert_eq!(g.player.hp, 10.0 + SACRIFICE_HEAL);
        assert_eq!(
            g.xp + (1..g.level).map(xp_to_next).sum::<u32>(),
            SACRIFICE_XP
        );
        assert_eq!(g.reward, None);
    }

    #[test]
    fn offerings_without_an_evolution_give_a_level_or_heal() {
        let mut g = Game::new();
        let before = g.upgrades.arts.iter().sum::<u8>() + g.upgrades.sins.iter().sum::<u8>();
        let Reward::Gift(_) = g.open_offering() else {
            panic!("expected a gift")
        };
        g.accept_offering();
        let after = g.upgrades.arts.iter().sum::<u8>() + g.upgrades.sins.iter().sum::<u8>();
        assert_eq!(after, before + 1);
        // Everything capped: it heals.
        g.upgrades.arts = [ART_MAX_LEVEL; ARTS.len()];
        g.upgrades.evolved = [true; ARTS.len()];
        g.upgrades.sins = [0; SINS.len()];
        for (lv, def) in g.upgrades.sins.iter_mut().zip(&SINS).take(MAX_SINS) {
            *lv = def.max_level;
        }
        g.player.hp = 10.0;
        assert_eq!(g.open_offering(), Reward::Heal);
        g.accept_offering();
        assert_eq!(g.player.hp, 10.0 + OFFERING_HEAL);
    }

    #[test]
    fn infernal_hide_softens_blows_down_to_one() {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.arts.hold();
        for _ in 0..3 {
            g.upgrades.apply(Choice::Sin(INFERNAL_HIDE), &mut g.player);
        }
        g.stats = g.upgrades.stats();
        g.enemies.spawn(KNIGHT, Vec2::ZERO, 1.0);
        g.tick(&InputState::default());
        let dmg = ENEMY_DEFS[KNIGHT].contact_damage - 3.0;
        assert_eq!(g.player.hp, PLAYER_MAX_HP - dmg);
        for _ in 0..2 {
            g.upgrades.apply(Choice::Sin(INFERNAL_HIDE), &mut g.player);
        }
        g.stats = g.upgrades.stats();
        g.player.iframes = 0.0;
        g.enemies.swap_remove(0);
        g.enemies.spawn(RAT, Vec2::ZERO, 1.0);
        let hp = g.player.hp;
        g.tick(&InputState::default());
        assert_eq!(
            g.player.hp,
            hp - MIN_CONTACT_DAMAGE,
            "a rat's 5 minus 5 is still 1"
        );
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
        assert!(g.cues.is_empty(), "the opening unlocks are silent");
        assert_eq!(g.spawn_weight(KNIGHT), 0.0);
        assert!(g.spawn_weight(RAT) > 0.0 && g.spawn_weight(VILLAGER) > 0.0);
        for _ in 0..200 {
            assert_ne!(g.roll_kind(), Some(KNIGHT));
        }
        g.time = 121.0;
        g.run_events();
        assert!(g.spawn_weight(KNIGHT) > 0.0);
        assert_eq!(g.cues, [Cue::Arrival]);
    }

    #[test]
    fn spawning_respects_enemy_cap() {
        let mut g = Game::new();
        g.time = 290.0;
        g.arts.hold();
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
    fn stress_at_enemy_cap_with_every_art_evolved() {
        const STRESS_HP: f32 = 20000.0;
        macroquad::rand::srand(7);
        let mut g = Game::new();
        g.upgrades.arts = [ART_MAX_LEVEL; ARTS.len()];
        g.upgrades.evolved = [true; ARTS.len()];
        g.upgrades.sins = SINS.map(|d| d.max_level);
        g.stats = g.upgrades.stats();
        g.arts.refresh(&g.upgrades);
        g.time = VICTORY_TIME;
        // Devouring Musk heals up to max_hp, so both are endless.
        g.player.hp = f32::INFINITY;
        g.player.max_hp = f32::INFINITY;
        while g.enemies.len() < MAX_ENEMIES {
            let p = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(50.0, g.spawn_ring);
            // Tough enough that the horde stays full under the maxed Arts.
            g.enemies
                .spawn(gen_range(0, ENEMY_DEFS.len()), p, STRESS_HP);
        }
        let hp_before: f32 = g.enemies.hp.iter().sum();
        let mut input = InputState::default();
        let (mut peak, mut total) = (0, 0);
        let ticks = 20_000;
        let started = std::time::Instant::now();
        for i in 0..ticks {
            if i % 30 == 0 {
                input.move_dir = vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0));
            }
            if g.tick(&input) == SimEvent::LevelUp {
                g.choose_upgrade(0);
                g.player.hp = f32::INFINITY; // healing clamps hp to max_hp
            }
            assert!(g.dying.is_none());
            assert!(g.enemies.len() <= MAX_ENEMIES);
            assert!(g.arts.projectiles.len() <= MAX_PROJECTILES);
            assert!(g.arts.bolts.capacity() == MAX_BOLTS);
            assert!(g.arts.lashes.capacity() == MAX_LASHES);
            assert!(g.arts.gates.capacity() == MAX_GATES);
            assert!(g.arts.chains.capacity() == MAX_CHAINS);
            assert!(g.arts.siren_bursts.capacity() == MAX_SIREN_BURSTS);
            assert!(g.souls.len() <= MAX_SOULS);
            assert!(g.fades.len() <= MAX_FADES);
            assert!(g.bursts.capacity() == MAX_BURST_PIXELS);
            assert!(g.splats.capacity() == MAX_SPLATS);
            g.splats.clear(); // the renderer drains them every frame
            peak = peak.max(g.enemies.len());
            total += g.enemies.len();
        }
        let mean_ms = started.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
        println!(
            "stress: mean tick {mean_ms:.3} ms, enemies peak {peak} mean {}, kills {}",
            total / ticks,
            g.kills
        );
        assert_eq!(peak, MAX_ENEMIES);
        let hp_after: f32 = g.enemies.hp.iter().sum();
        assert!(hp_after < hp_before, "the Arts keep hitting the horde");
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
