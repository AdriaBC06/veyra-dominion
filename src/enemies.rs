//! Enemy horde stored as structure-of-arrays for a cache-friendly hot loop.

use crate::config::*;
use crate::spatial::SpatialHash;
use macroquad::math::{vec2, Vec2};
use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};

/// Behaviour states (`Enemies::mode`).
pub const MODE_WALK: u8 = 0;
/// Charger standing still before the dash; Bomber burning its fuse.
pub const MODE_WINDUP: u8 = 1;
pub const MODE_DASH: u8 = 2;
pub const MODE_REST: u8 = 3;

/// Something an enemy did that reaches beyond the horde.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    /// A `Ranged` enemy threw `kind` from `from` toward `to`.
    Shot {
        kind: ShotKind,
        from: Vec2,
        to: Vec2,
        damage: f32,
    },
    /// A Bomber went off at `pos` (it is already gone from the horde).
    Blast {
        pos: Vec2,
        radius: f32,
        damage: f32,
        enemy_damage: f32,
    },
}

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
    /// Game time until which it fights for Veyra (Allure).
    pub charm_until: Vec<f32>,
    /// Game time until which it takes `SCENT_WEAKEN` more damage (Scent of Sin).
    pub weak_until: Vec<f32>,
    /// Game time until which it bleeds (Barbed Scourge).
    pub bleed_until: Vec<f32>,
    /// Game time until which it burns (Heretic's Brand).
    pub burn_until: Vec<f32>,
    /// Game time until which it stands frozen (Gaze of Desire): no moving, no hurting.
    pub frozen_until: Vec<f32>,
    /// Left the horde without dying (the Infernal Court turned it into a thrall), with
    /// `hp` at 0: removed with no kill, soul or burst.
    pub gone: Vec<bool>,
    /// Velocity kept between ticks: a flier's swoop, a charger's dash direction.
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    /// Behaviour clock: attack cooldown, wind-up, dash, rest or fuse time left.
    pub timer: Vec<f32>,
    /// Behaviour state, one of the `MODE_*` consts.
    pub mode: Vec<u8>,
    /// Health at spawn, the cap for heals.
    pub max_hp: Vec<f32>,
    /// Game time until which it shows a Curate's blessing.
    pub bless_until: Vec<f32>,
    /// What the horde did this tick that the game must resolve (shots, blasts),
    /// drained by `Game` (capacity `MAX_SHOTS`).
    pub actions: Vec<Action>,
    /// Extra damage frozen enemies take (Evil Eye; 0 otherwise), set by the Arts.
    pub frozen_vuln: f32,
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
            charm_until: Vec::with_capacity(n),
            weak_until: Vec::with_capacity(n),
            bleed_until: Vec::with_capacity(n),
            burn_until: Vec::with_capacity(n),
            frozen_until: Vec::with_capacity(n),
            gone: Vec::with_capacity(n),
            vx: Vec::with_capacity(n),
            vy: Vec::with_capacity(n),
            timer: Vec::with_capacity(n),
            mode: Vec::with_capacity(n),
            max_hp: Vec::with_capacity(n),
            bless_until: Vec::with_capacity(n),
            actions: Vec::with_capacity(MAX_SHOTS),
            frozen_vuln: 0.0,
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
        self.charm_until.push(0.0);
        self.weak_until.push(0.0);
        self.bleed_until.push(0.0);
        self.burn_until.push(0.0);
        self.frozen_until.push(0.0);
        self.gone.push(false);
        self.vx.push(0.0);
        self.vy.push(0.0);
        // Staggered, so a pack that arrives together doesn't fire in one volley.
        self.timer.push(macroquad::rand::gen_range(0.5, 1.5));
        self.mode.push(MODE_WALK);
        self.max_hp.push(ENEMY_DEFS[kind].hp * hp_scale);
        self.bless_until.push(0.0);
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
        self.charm_until.swap_remove(i);
        self.weak_until.swap_remove(i);
        self.bleed_until.swap_remove(i);
        self.burn_until.swap_remove(i);
        self.frozen_until.swap_remove(i);
        self.gone.swap_remove(i);
        self.vx.swap_remove(i);
        self.vy.swap_remove(i);
        self.timer.swap_remove(i);
        self.mode.swap_remove(i);
        self.max_hp.swap_remove(i);
        self.bless_until.swap_remove(i);
    }

    /// Turns enemy `i` into an elite: tougher, and marked.
    pub fn make_elite(&mut self, i: usize) {
        if !self.elite[i] {
            self.elite[i] = true;
            self.hp[i] *= ELITE_HP;
            self.max_hp[i] *= ELITE_HP;
        }
    }

    /// Largest collision radius among the kinds currently alive (0 when empty); spatial
    /// queries widen by this so they never miss a big enemy's edge.
    pub fn max_radius(&self) -> f32 {
        let alive = ENEMY_DEFS.iter().zip(self.counts).filter(|(_, n)| *n > 0);
        alive.map(|(d, _)| d.radius()).fold(0.0, f32::max)
    }

    pub fn charmed(&self, i: usize, time: f32) -> bool {
        self.charm_until[i] > time
    }

    pub fn weak(&self, i: usize, time: f32) -> bool {
        self.weak_until[i] > time
    }

    pub fn frozen(&self, i: usize, time: f32) -> bool {
        self.frozen_until[i] > time
    }

    /// Where charmed enemy `i` heads: the nearest free enemy within `CHARM_SEEK`, or
    /// nowhere (it stands guard).
    fn charm_target(&self, i: usize, hash: &SpatialHash, time: f32) -> Option<Vec2> {
        let p = self.pos(i);
        let mut best: Option<(f32, Vec2)> = None;
        hash.query_radius(p, CHARM_SEEK, |j, q| {
            let d = q.distance_squared(p);
            if j != i && !self.charmed(j, time) && self.hp[j] > 0.0 && best.is_none_or(|b| d < b.0)
            {
                best = Some((d, q));
            }
        });
        best.map(|b| b.1)
    }

    /// Moves every enemy according to its behaviour plus soft separation, using the
    /// position snapshot in `hash` for neighbours, and queues their shots and blasts in
    /// `actions`. Charmed enemies hunt the horde and never attack her; frozen ones stay
    /// put. `time` is the game time; `speed_scale` multiplies every speed (Pride).
    pub fn steer(
        &mut self,
        dt: f32,
        target: Vec2,
        hash: &SpatialHash,
        time: f32,
        speed_scale: f32,
    ) {
        for i in 0..self.len() {
            self.flash[i] = (self.flash[i] - dt).max(0.0);
            if self.frozen(i, time) || self.hp[i] <= 0.0 {
                continue;
            }
            let def = &ENEMY_DEFS[self.kind[i] as usize];
            let p = self.pos(i);
            let charmed = self.charmed(i, time);
            let goal = if charmed {
                self.charm_target(i, hash, time)
            } else {
                Some(target)
            };
            let mut speed = def.speed * speed_scale;
            if self.elite[i] {
                speed *= ELITE_SPEED;
            }
            self.timer[i] -= dt;
            let (desired, separate) = match goal {
                None => (Vec2::ZERO, true),
                Some(g) if charmed => ((g - p).normalize_or_zero() * speed, true),
                Some(g) => self.behave(i, def.behavior, p, g, speed, dt, time),
            };
            let mut push = Vec2::ZERO;
            if separate {
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
            }
            self.set_pos(i, p + (desired + push * SEPARATION_PUSH) * dt);
        }
        self.support(hash, time);
    }

    /// The velocity enemy `i` wants toward `goal` (Veyra), and whether separation
    /// applies; runs its attack state machine on the way.
    #[allow(clippy::too_many_arguments)]
    fn behave(
        &mut self,
        i: usize,
        behavior: Behavior,
        p: Vec2,
        goal: Vec2,
        speed: f32,
        dt: f32,
        time: f32,
    ) -> (Vec2, bool) {
        let off = goal - p;
        let dist = off.length();
        let dir = off.normalize_or_zero();
        match behavior {
            Behavior::Chase => (dir * speed, true),
            Behavior::Swarm { amp, hz } => {
                let weave = (TAU * hz * time + self.phase[i] * PI).sin() * amp;
                (dir * speed + dir.perp() * weave, true)
            }
            Behavior::Flyer { turn } => {
                let v = vec2(self.vx[i], self.vy[i]);
                // Close in, it commits to the dive and flies straight through her.
                if self.mode[i] == MODE_DASH && self.timer[i] > 0.0 {
                    return (v, false);
                }
                self.mode[i] = MODE_WALK;
                if dist < FLYER_DIVE_RANGE {
                    self.mode[i] = MODE_DASH;
                    self.timer[i] = FLYER_DIVE_TIME;
                    let v = v.normalize_or(dir) * speed;
                    (self.vx[i], self.vy[i]) = (v.x, v.y);
                    return (v, false);
                }
                let v = v + (dir * speed - v) * (turn * dt).min(1.0);
                (self.vx[i], self.vy[i]) = (v.x, v.y);
                (v, false)
            }
            Behavior::Charger {
                range,
                windup,
                dash_speed,
                dash_time,
                rest,
            } => match self.mode[i] {
                MODE_WINDUP if self.timer[i] <= 0.0 => {
                    // Aimed at the end of the wind-up, so moving away still works.
                    (self.vx[i], self.vy[i]) = (dir.x, dir.y);
                    self.mode[i] = MODE_DASH;
                    self.timer[i] = dash_time;
                    (dir * dash_speed, false)
                }
                MODE_WINDUP => (Vec2::ZERO, true),
                MODE_DASH if self.timer[i] <= 0.0 => {
                    self.mode[i] = MODE_REST;
                    self.timer[i] = rest;
                    (Vec2::ZERO, true)
                }
                MODE_DASH => (vec2(self.vx[i], self.vy[i]) * dash_speed, false),
                MODE_REST if self.timer[i] > 0.0 => (dir * speed * 0.5, true),
                _ => {
                    self.mode[i] = MODE_WALK;
                    if dist < range {
                        self.mode[i] = MODE_WINDUP;
                        self.timer[i] = windup;
                        return (Vec2::ZERO, true);
                    }
                    (dir * speed, true)
                }
            },
            Behavior::Ranged {
                keep,
                range,
                cooldown,
                shot,
                damage,
            } => {
                if self.timer[i] <= 0.0 && dist < range && self.actions.len() < MAX_SHOTS {
                    self.timer[i] = cooldown * gen_range(0.8, 1.2);
                    self.actions.push(Action::Shot {
                        kind: shot,
                        from: p,
                        to: goal,
                        damage,
                    });
                }
                (self.keep_away(i, dir, dist, keep, speed, time), true)
            }
            Behavior::Support { keep, .. } => {
                (self.keep_away(i, dir, dist, keep, speed, time), true)
            }
            Behavior::Bomber {
                prime_range,
                fuse,
                radius,
                damage,
                enemy_damage,
            } => {
                if self.mode[i] == MODE_WINDUP {
                    if self.timer[i] <= 0.0 && self.actions.len() < MAX_SHOTS {
                        let scale = if self.elite[i] { ELITE_DAMAGE } else { 1.0 };
                        self.actions.push(Action::Blast {
                            pos: p,
                            radius,
                            damage: damage * scale,
                            enemy_damage,
                        });
                        // Gone in its own fire: no kill, no soul.
                        self.hp[i] = 0.0;
                        self.gone[i] = true;
                    }
                    return (dir * speed * BOMBER_PRIME_SLOW, true);
                }
                if dist < prime_range {
                    self.mode[i] = MODE_WINDUP;
                    self.timer[i] = fuse;
                }
                (dir * speed, true)
            }
        }
    }

    /// Ranged and Support footwork: close in, back off, or strafe around her at `keep`.
    fn keep_away(&self, i: usize, dir: Vec2, dist: f32, keep: f32, speed: f32, time: f32) -> Vec2 {
        if dist > keep + KEEP_SLACK {
            dir * speed
        } else if dist < keep - KEEP_SLACK {
            -dir * speed
        } else {
            // Each one circles its own way, and turns now and then.
            let side = if (time * 0.2 + self.phase[i]).fract() < 0.5 {
                1.0
            } else {
                -1.0
            };
            dir.perp() * side * speed * 0.5
        }
    }

    /// Support enemies heal the free allies around them every `SUPPORT_PULSE`.
    fn support(&mut self, hash: &SpatialHash, time: f32) {
        for i in 0..self.len() {
            let Behavior::Support { radius, heal, .. } = ENEMY_DEFS[self.kind[i] as usize].behavior
            else {
                continue;
            };
            let busy = self.charmed(i, time) || self.frozen(i, time) || self.hp[i] <= 0.0;
            if busy || self.timer[i] > 0.0 {
                continue;
            }
            self.timer[i] = SUPPORT_PULSE;
            let p = self.pos(i);
            hash.query_radius(p, radius, |j, _| {
                let free = !self.charmed(j, time) && self.hp[j] > 0.0;
                if j != i && free && self.hp[j] < self.max_hp[j] {
                    self.hp[j] = (self.hp[j] + heal * self.max_hp[j]).min(self.max_hp[j]);
                    self.bless_until[j] = time + BLESS_SHOW;
                }
            });
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
            e.steer(TICK_DT, Vec2::ZERO, &hash, 0.0, 1.0);
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
