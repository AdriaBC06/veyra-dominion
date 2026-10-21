//! Arts (Veyra's weapons) during a run: cooldowns, effective stats and the fixed
//! pools of what they throw. Pure logic, driven by `Game::tick`.

use crate::config::*;
use crate::enemies::Enemies;
use crate::game::Player;
use crate::spatial::SpatialHash;
use crate::upgrades::{PlayerStats, Upgrades};
use macroquad::math::{vec2, Vec2};
use macroquad::rand::gen_range;
use std::f32::consts::TAU;

/// A Hellflare fireball.
pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
}

/// One Tail Lash on `side` (-1 left, 1 right of Veyra). It strikes when `delay`
/// runs out, then stays on screen for `LASH_SHOW_TIME` (`t` counts that).
pub struct Lash {
    pub side: f32,
    pub delay: f32,
    pub t: f32,
}

/// A Kiss of Ruin heart (Hellflare evolved). It goes through `pierce` more enemies,
/// never hitting the same one twice (`hits` holds their ids).
pub struct Bolt {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pierce: u32,
    hits: [u32; BOLT_MAX_HITS],
    hit_count: usize,
}

/// Brimstone Halo: its flames burn for `duration`, rest for `cooldown`, and again. `t` is the
/// time spent in the current phase.
#[derive(Default)]
pub struct Halo {
    pub on: bool,
    pub t: f32,
    pub angle: f32,
}

/// Allure: time since the last pulse and the radius it reached (for its ring of
/// hearts), and the clock of the charmed enemies' bites.
pub struct Allure {
    pub since_pulse: f32,
    pub radius: f32,
    bite: f32,
}

/// A Hellgate pool burning at `pos` for `life` seconds; `t` is its age and `burn` the
/// time left to its next burn.
pub struct Gate {
    pub pos: Vec2,
    pub t: f32,
    pub life: f32,
    burn: f32,
    /// Places its flames, so each pool burns in its own pattern.
    pub seed: u32,
}

impl Gate {
    /// Opacity: pools open and close over `HELLGATE_FADE`.
    pub fn alpha(&self) -> f32 {
        (self.t.min(self.life - self.t) / HELLGATE_FADE).clamp(0.0, 1.0)
    }
}

/// A Chain of Torment strike: Veyra, then every enemy it bit, in order. `seed` keeps
/// its jagged shape the same while it shows.
pub struct Chain {
    links: [Vec2; MAX_CHAIN_LINKS + 1],
    len: usize,
    pub t: f32,
    pub seed: u32,
}

impl Chain {
    pub fn links(&self) -> &[Vec2] {
        &self.links[..self.len]
    }
}

/// Siren's Dominion: a charmed enemy burst at `pos`, a ring of hearts spreading to
/// `radius` over `SIREN_BURST_TIME`.
pub struct SirenBurst {
    pub pos: Vec2,
    pub radius: f32,
    pub t: f32,
}

/// What the Arts did this tick, turned into sound cues by the caller.
#[derive(Default)]
pub struct ArtCues {
    pub fired: bool,
    pub lashed: bool,
    pub kissed: bool,
    pub charmed: bool,
    pub gated: bool,
    pub chained: bool,
    /// A charmed enemy burst (Siren's Dominion).
    pub burst: bool,
    /// A hit landed on flesh / on armour.
    pub hits: [bool; 2],
}

/// The parts of the simulation the Arts read and change.
pub struct World<'a> {
    pub player: &'a mut Player,
    pub enemies: &'a mut Enemies,
    pub hash: &'a SpatialHash,
    pub time: f32,
}

pub struct Arts {
    pub projectiles: Vec<Projectile>,
    pub lashes: Vec<Lash>,
    pub bolts: Vec<Bolt>,
    pub halo: Halo,
    pub allure: Allure,
    pub gates: Vec<Gate>,
    pub chains: Vec<Chain>,
    pub siren_bursts: Vec<SirenBurst>,
    /// Seconds since the Scent of Sin last burned (the haze swells a little then).
    pub since_scent: f32,
    /// Level of each Art (0 = not owned), copied from `Upgrades` on `refresh`.
    levels: [u8; ARTS.len()],
    /// Arts in their evolved form, copied from `Upgrades` on `refresh`.
    evolved: [bool; ARTS.len()],
    /// Effective numbers: level stats with the Sins applied.
    pub stats: [ArtStats; ARTS.len()],
    cooldown: [f32; ARTS.len()],
    /// Devouring Musk: health it may still give back, refilled at `MUSK_HEAL_CAP`/s.
    heal_budget: f32,
    /// Time left to the next Barbed Scourge bleed.
    bleed_clock: f32,
}

impl Arts {
    pub fn new(upgrades: &Upgrades) -> Self {
        let mut arts = Self {
            projectiles: Vec::with_capacity(MAX_PROJECTILES),
            lashes: Vec::with_capacity(MAX_LASHES),
            bolts: Vec::with_capacity(MAX_BOLTS),
            halo: Halo::default(),
            allure: Allure {
                since_pulse: f32::INFINITY,
                radius: 0.0,
                bite: 0.0,
            },
            gates: Vec::with_capacity(MAX_GATES),
            chains: Vec::with_capacity(MAX_CHAINS),
            siren_bursts: Vec::with_capacity(MAX_SIREN_BURSTS),
            since_scent: f32::INFINITY,
            levels: [0; ARTS.len()],
            evolved: [false; ARTS.len()],
            stats: ARTS.map(|d| d.base),
            cooldown: [0.0; ARTS.len()],
            heal_budget: MUSK_HEAL_CAP,
            bleed_clock: 0.0,
        };
        arts.refresh(upgrades);
        arts
    }

    /// Picks up new levels, evolutions and Sins; call after every change.
    pub fn refresh(&mut self, upgrades: &Upgrades) {
        let PlayerStats {
            might,
            cooldown,
            amount,
            duration,
            area,
            ..
        } = upgrades.stats();
        for (i, def) in ARTS.iter().enumerate() {
            let fresh = self.levels[i] == 0 && upgrades.arts[i] > 0;
            self.levels[i] = upgrades.arts[i];
            self.evolved[i] = upgrades.evolved[i];
            let mut s = def.stats(self.levels[i]);
            s.damage *= might;
            s.cooldown *= cooldown;
            s.amount += amount;
            s.area *= area;
            // Projectile lifetimes are their range, not an effect that lasts.
            if def.kind != ArtKind::Hellflare {
                s.duration *= duration;
            }
            if self.evolved[i] {
                match def.kind {
                    ArtKind::Hellflare => s.amount += KISS_EXTRA,
                    ArtKind::BrimstoneHalo => s.amount += CROWN_EXTRA,
                    _ => {}
                }
            }
            self.stats[i] = s;
            if fresh && i == BRIMSTONE_HALO {
                // A new Halo lights up straight away.
                self.halo = Halo {
                    on: false,
                    t: s.cooldown,
                    angle: 0.0,
                };
            }
        }
    }

    pub fn owns(&self, art: usize) -> bool {
        self.levels[art] > 0
    }

    pub fn evolved(&self, art: usize) -> bool {
        self.evolved[art]
    }

    /// Stops every attack (tests that need an enemy to stay alive).
    #[cfg(test)]
    pub fn hold(&mut self) {
        self.cooldown = [f32::INFINITY; ARTS.len()];
    }

    /// One fixed step: attacks come off cooldown while Veyra is `alive`; whatever is
    /// already flying keeps going either way.
    pub fn update(&mut self, w: &mut World, alive: bool) -> ArtCues {
        let mut cues = ArtCues::default();
        if alive {
            for (art, def) in ARTS.iter().enumerate() {
                if !self.owns(art) || art == BRIMSTONE_HALO {
                    continue;
                }
                self.cooldown[art] = (self.cooldown[art] - TICK_DT).max(0.0);
                if self.cooldown[art] > 0.0 {
                    continue;
                }
                let attacked = match def.kind {
                    ArtKind::Hellflare => self.fire(w, &mut cues),
                    ArtKind::TailLash => self.lash(w, &mut cues),
                    ArtKind::BrimstoneHalo => false,
                    ArtKind::Allure => self.pulse(w, &mut cues),
                    ArtKind::Hellgate => self.open_gates(w, &mut cues),
                    ArtKind::ChainOfTorment => self.chain(w, &mut cues),
                    ArtKind::ScentOfSin => self.scent(w, &mut cues),
                };
                if attacked {
                    self.cooldown[art] = self.stats[art].cooldown;
                }
            }
        }
        self.update_projectiles(w, &mut cues);
        self.update_lashes(w, &mut cues);
        self.update_bolts(w, &mut cues);
        self.update_halo(w, alive, &mut cues);
        self.update_charmed(w, &mut cues);
        self.update_gates(w, &mut cues);
        self.update_bleed(w, &mut cues);
        self.heal_budget = (self.heal_budget + MUSK_HEAL_CAP * TICK_DT).min(MUSK_HEAL_CAP);
        self.since_scent += TICK_DT;
        self.siren_bursts.retain_mut(|b| {
            b.t += TICK_DT;
            b.t < SIREN_BURST_TIME
        });
        self.chains.retain_mut(|c| {
            c.t += TICK_DT;
            c.t < CHAIN_SHOW_TIME
        });
        cues
    }

    /// Hellflare: a volley at the nearest enemy in range, fanned out. Kiss of Ruin
    /// throws piercing hearts instead.
    fn fire(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let from = w.player.pos;
        let Some(target) = w.hash.nearest(from, FIRE_RANGE) else {
            return false;
        };
        let s = &self.stats[HELLFLARE];
        let aim = (w.enemies.pos(target) - from).normalize_or(Vec2::X);
        let kiss = self.evolved[HELLFLARE];
        for k in 0..s.amount {
            let angle = (k as f32 - (s.amount - 1) as f32 * 0.5) * PROJ_SPREAD;
            let dir = Vec2::from_angle(angle).rotate(aim);
            if kiss {
                if self.bolts.len() >= MAX_BOLTS {
                    break;
                }
                self.bolts.push(Bolt {
                    pos: from,
                    vel: dir * KISS_SPEED,
                    life: KISS_LIFE,
                    pierce: KISS_PIERCE,
                    hits: [u32::MAX; BOLT_MAX_HITS],
                    hit_count: 0,
                });
            } else {
                if self.projectiles.len() >= MAX_PROJECTILES {
                    break;
                }
                self.projectiles.push(Projectile {
                    pos: from,
                    vel: dir * s.speed,
                    life: s.duration,
                });
            }
        }
        w.player.since_cast = 0.0;
        if kiss {
            cues.kissed = true;
        } else {
            cues.fired = true;
        }
        true
    }

    /// Tail Lash: the first lash on the side she faces, the next ones alternate sides,
    /// `LASH_DELAY` apart.
    fn lash(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        for k in 0..self.stats[TAIL_LASH].amount {
            if self.lashes.len() >= MAX_LASHES {
                break;
            }
            let side = if k % 2 == 0 { 1.0 } else { -1.0 };
            self.lashes.push(Lash {
                side: w.player.facing * side,
                delay: k as f32 * LASH_DELAY,
                t: 0.0,
            });
        }
        cues.lashed = true;
        true
    }

    /// Allure: charms the first `amount` free enemies within reach and burns the rest
    /// (the charmed are spared).
    /// Waits while nobody is in reach.
    fn pulse(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[ALLURE];
        let (center, time) = (w.player.pos, w.time);
        let radius = ALLURE_RADIUS * s.area;
        let enemies = &mut *w.enemies;
        let (mut any, mut charms) = (false, s.amount);
        w.hash
            .query_radius(center, radius + enemies.max_radius(), |j, _| {
                if !touches(enemies, j, center, radius) {
                    return;
                }
                any = true;
                if enemies.charmed(j, time) {
                    return;
                }
                if charms > 0 {
                    // Seduced, not burned: she wants it alive and fighting for her.
                    enemies.charm_until[j] = time + s.duration;
                    charms -= 1;
                    return;
                }
                strike(enemies, j, s.damage, time, &mut cues.hits);
            });
        if any {
            self.allure.since_pulse = 0.0;
            self.allure.radius = radius;
            cues.charmed = true;
        }
        any
    }

    /// Hellgate: a pool under `amount` random enemies in range.
    fn open_gates(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[HELLGATE];
        let mut opened = false;
        for _ in 0..s.amount {
            if self.gates.len() >= MAX_GATES {
                break;
            }
            let Some(j) = random_enemy(w, HELLGATE_RANGE) else {
                break;
            };
            let jitter = vec2(gen_range(-6.0, 6.0), gen_range(-3.0, 3.0));
            self.gates.push(Gate {
                pos: w.enemies.pos(j) + jitter,
                t: 0.0,
                life: s.duration,
                burn: 0.0,
                seed: gen_range(0, u32::MAX),
            });
            opened = true;
        }
        cues.gated |= opened;
        opened
    }

    /// Chain of Torment: bites a random enemy in range, then jumps to the nearest one
    /// it hasn't bitten, `amount` enemies in all.
    fn chain(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        if self.chains.len() >= MAX_CHAINS {
            return false;
        }
        let Some(first) = random_enemy(w, CHAIN_RANGE) else {
            return false;
        };
        let s = self.stats[CHAIN_OF_TORMENT];
        let eternal = self.evolved[CHAIN_OF_TORMENT];
        let (bites, jump) = if eternal {
            (MAX_CHAIN_LINKS, TORMENT_RADIUS * s.area)
        } else {
            (
                (s.amount as usize).min(MAX_CHAIN_LINKS),
                CHAIN_JUMP * s.area,
            )
        };
        let origin = w.enemies.pos(first);
        let mut c = Chain {
            links: [w.player.pos; MAX_CHAIN_LINKS + 1],
            len: 1,
            t: 0.0,
            seed: gen_range(0, u32::MAX),
        };
        let mut bitten = [u32::MAX; MAX_CHAIN_LINKS];
        let enemies = &mut *w.enemies;
        let mut next = Some(first);
        while let Some(j) = next {
            bitten[c.len - 1] = enemies.uid[j];
            c.links[c.len] = enemies.pos(j);
            c.len += 1;
            strike(enemies, j, s.damage, w.time, &mut cues.hits);
            if c.len > bites {
                break;
            }
            let from = enemies.pos(j);
            let mut best: Option<(f32, usize)> = None;
            w.hash.query_radius(from, jump, |k, q| {
                let d = q.distance_squared(from);
                let free = enemies.hp[k] > 0.0 && !bitten.contains(&enemies.uid[k]);
                // Torment Eternal hops anywhere near the first victim.
                let near = !eternal || q.distance_squared(origin) <= jump * jump;
                if free && near && best.is_none_or(|b| d < b.0) {
                    best = Some((d, k));
                }
            });
            next = best.map(|b| b.1);
        }
        self.chains.push(c);
        w.player.since_cast = 0.0;
        cues.chained = true;
        true
    }

    /// Scent of Sin: burns and weakens everything in the haze around her. Devouring
    /// Musk feeds her on every enemy it burns.
    fn scent(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[SCENT_OF_SIN];
        let (center, time) = (w.player.pos, w.time);
        let radius = SCENT_RADIUS * s.area;
        let enemies = &mut *w.enemies;
        let mut burned = 0;
        w.hash
            .query_radius(center, radius + enemies.max_radius(), |j, _| {
                if touches(enemies, j, center, radius) {
                    strike(enemies, j, s.damage, time, &mut cues.hits);
                    enemies.weak_until[j] = time + s.duration;
                    burned += 1;
                }
            });
        if self.evolved[SCENT_OF_SIN] && w.player.hp > 0.0 {
            let p = &mut *w.player;
            let heal = (burned as f32 * MUSK_HEAL).min(self.heal_budget);
            self.heal_budget -= heal;
            p.hp = (p.hp + heal).min(p.max_hp);
        }
        self.since_scent = 0.0;
        true
    }

    /// Every `CHARM_BITE` each charmed enemy bites the free enemies it touches.
    fn update_charmed(&mut self, w: &mut World, cues: &mut ArtCues) {
        self.end_charms(w, cues);
        let a = &mut self.allure;
        a.since_pulse += TICK_DT;
        a.bite -= TICK_DT;
        if a.bite > 0.0 {
            return;
        }
        a.bite = CHARM_BITE;
        let (damage, time) = (self.stats[ALLURE].damage, w.time);
        let enemies = &mut *w.enemies;
        let extra = enemies.max_radius();
        for i in 0..enemies.len() {
            if !enemies.charmed(i, time) || enemies.hp[i] <= 0.0 {
                continue;
            }
            let p = enemies.pos(i);
            let reach = ENEMY_DEFS[enemies.kind[i] as usize].radius() + CHARM_REACH;
            w.hash.query_radius(p, reach + extra, |j, _| {
                if j != i && !enemies.charmed(j, time) && touches(enemies, j, p, reach) {
                    strike(enemies, j, damage, time, &mut cues.hits);
                }
            });
        }
    }

    /// Charms that ran out this tick are cleared; with Siren's Dominion the enemy
    /// bursts, hurting the free enemies around it without killing any.
    fn end_charms(&mut self, w: &mut World, cues: &mut ArtCues) {
        let dominion = self.evolved[ALLURE];
        let s = self.stats[ALLURE];
        let (radius, damage) = (SIREN_BURST_RADIUS * s.area, s.damage * SIREN_BURST_DAMAGE);
        let time = w.time;
        let enemies = &mut *w.enemies;
        for i in 0..enemies.len() {
            let until = enemies.charm_until[i];
            if until == 0.0 || until > time {
                continue;
            }
            enemies.charm_until[i] = 0.0;
            if !dominion || enemies.hp[i] <= 0.0 {
                continue;
            }
            let p = enemies.pos(i);
            w.hash
                .query_radius(p, radius + enemies.max_radius(), |j, _| {
                    if j != i && !enemies.charmed(j, time) && touches(enemies, j, p, radius) {
                        let floor = enemies.hp[j].min(SIREN_SPARE_HP);
                        strike(enemies, j, damage, time, &mut cues.hits);
                        enemies.hp[j] = enemies.hp[j].max(floor);
                    }
                });
            if self.siren_bursts.len() < MAX_SIREN_BURSTS {
                self.siren_bursts.push(SirenBurst {
                    pos: p,
                    radius,
                    t: 0.0,
                });
            }
            cues.burst = true;
        }
    }

    /// Barbed Scourge: every `BLEED_TICK` bleeding enemies lose a share of the Lash's
    /// damage.
    fn update_bleed(&mut self, w: &mut World, cues: &mut ArtCues) {
        if !self.evolved[TAIL_LASH] {
            return;
        }
        self.bleed_clock -= TICK_DT;
        if self.bleed_clock > 0.0 {
            return;
        }
        self.bleed_clock += BLEED_TICK;
        let damage = self.stats[TAIL_LASH].damage * BLEED_SHARE;
        let enemies = &mut *w.enemies;
        for i in 0..enemies.len() {
            if enemies.bleed_until[i] > w.time && enemies.hp[i] > 0.0 {
                strike(enemies, i, damage, w.time, &mut cues.hits);
            }
        }
    }

    /// Pools burn everything inside their ellipse every `HELLGATE_TICK`. Abyssal Rift
    /// pools also drag the enemies around them in.
    fn update_gates(&mut self, w: &mut World, cues: &mut ArtCues) {
        let s = self.stats[HELLGATE];
        let (damage, time) = (s.damage, w.time);
        let rift = self.evolved[HELLGATE].then_some(RIFT_RADIUS * s.area);
        let enemies = &mut *w.enemies;
        let extra = enemies.max_radius();
        let rx = HELLGATE_RADIUS * s.area;
        let ry = rx * 0.5;
        self.gates.retain_mut(|g| {
            g.t += TICK_DT;
            g.burn -= TICK_DT;
            if let Some(radius) = rift {
                let step = RIFT_PULL * TICK_DT;
                w.hash.query_radius(g.pos, radius, |j, p| {
                    let off = g.pos - p;
                    let len = off.length();
                    if enemies.hp[j] > 0.0 && len > 0.0 {
                        enemies.set_pos(j, p + off * (step / len).min(1.0));
                    }
                });
            }
            if g.burn <= 0.0 {
                g.burn += HELLGATE_TICK;
                w.hash.query_radius(g.pos, rx + extra, |j, _| {
                    let r = ENEMY_DEFS[enemies.kind[j] as usize].radius();
                    let d = enemies.pos(j) - g.pos;
                    let (ex, ey) = (d.x / (rx + r), d.y / (ry + r));
                    if enemies.hp[j] > 0.0 && ex * ex + ey * ey <= 1.0 {
                        strike(enemies, j, damage, time, &mut cues.hits);
                    }
                });
            }
            g.t < g.life
        });
    }

    /// Moves fireballs and applies hits. Dead enemies stay in place (hp <= 0) until
    /// the game removes them, so hash indices stay valid for the whole pass.
    fn update_projectiles(&mut self, w: &mut World, cues: &mut ArtCues) {
        let (damage, time) = (self.stats[HELLFLARE].damage, w.time);
        let enemies = &mut *w.enemies;
        let query = PROJ_RADIUS + enemies.max_radius();
        let mut k = self.projectiles.len();
        while k > 0 {
            k -= 1;
            let pr = &mut self.projectiles[k];
            pr.pos += pr.vel * TICK_DT;
            pr.life -= TICK_DT;
            let mut hit = None;
            w.hash.query_radius(pr.pos, query, |j, _| {
                if hit.is_none() && touches(enemies, j, pr.pos, PROJ_RADIUS) {
                    hit = Some(j);
                }
            });
            if let Some(j) = hit {
                strike(enemies, j, damage, time, &mut cues.hits);
            }
            if hit.is_some() || pr.life <= 0.0 {
                self.projectiles.swap_remove(k);
            }
        }
    }

    /// Lashes strike once when their delay runs out, at Veyra's position then.
    fn update_lashes(&mut self, w: &mut World, cues: &mut ArtCues) {
        let s = self.stats[TAIL_LASH];
        let time = w.time;
        let bleed = self.evolved[TAIL_LASH].then_some(time + s.duration);
        let enemies = &mut *w.enemies;
        let center = w.player.pos;
        self.lashes.retain_mut(|l| {
            if l.delay > 0.0 {
                l.delay -= TICK_DT;
                return true;
            }
            if l.t == 0.0 {
                // Area lengthens her reach.
                let half = vec2(LASH_REACH * s.area, LASH_DEPTH) * 0.5;
                let mid = center + vec2(l.side * (LASH_NEAR + half.x), 0.0);
                let query = half.length() + enemies.max_radius();
                w.hash.query_radius(mid, query, |j, p| {
                    let r = ENEMY_DEFS[enemies.kind[j] as usize].radius();
                    let off = (p - mid).abs();
                    if enemies.hp[j] > 0.0 && off.x <= half.x + r && off.y <= half.y + r {
                        strike(enemies, j, s.damage, time, &mut cues.hits);
                        if let Some(until) = bleed {
                            enemies.bleed_until[j] = until;
                        }
                    }
                });
            }
            l.t += TICK_DT;
            l.t < LASH_SHOW_TIME
        });
    }

    /// Hearts fly straight, hitting each enemy at most once, one per tick.
    fn update_bolts(&mut self, w: &mut World, cues: &mut ArtCues) {
        let (damage, time) = (self.stats[HELLFLARE].damage, w.time);
        let enemies = &mut *w.enemies;
        let query = BOLT_RADIUS + enemies.max_radius();
        self.bolts.retain_mut(|b| {
            b.pos += b.vel * TICK_DT;
            b.life -= TICK_DT;
            let mut hit = None;
            w.hash.query_radius(b.pos, query, |j, _| {
                if hit.is_none()
                    && touches(enemies, j, b.pos, BOLT_RADIUS)
                    && !b.hits[..b.hit_count].contains(&enemies.uid[j])
                {
                    hit = Some(j);
                }
            });
            let Some(j) = hit else {
                return b.life > 0.0;
            };
            strike(enemies, j, damage, time, &mut cues.hits);
            if b.hit_count < BOLT_MAX_HITS {
                b.hits[b.hit_count] = enemies.uid[j];
                b.hit_count += 1;
            }
            if b.pierce == 0 {
                return false;
            }
            b.pierce -= 1;
            b.life > 0.0
        });
    }

    fn update_halo(&mut self, w: &mut World, alive: bool, cues: &mut ArtCues) {
        if !self.owns(BRIMSTONE_HALO) || !alive {
            self.halo.on = false;
            return;
        }
        let s = self.stats[BRIMSTONE_HALO];
        let h = &mut self.halo;
        h.t += TICK_DT;
        if self.evolved[BRIMSTONE_HALO] {
            // Infernal Crown: it never rests.
            h.on = true;
        } else {
            let phase = if h.on { s.duration } else { s.cooldown };
            if h.t >= phase {
                h.on = !h.on;
                h.t = 0.0;
            }
        }
        if !h.on {
            return;
        }
        h.angle = (h.angle + s.speed * TICK_DT) % TAU;
        let (center, time) = (w.player.pos, w.time);
        let enemies = &mut *w.enemies;
        let query = EMBER_RADIUS + enemies.max_radius();
        for off in ember_offsets(h.angle, &s) {
            w.hash.query_radius(center + off, query, |j, p| {
                if enemies.halo_until[j] > time || !touches(enemies, j, center + off, EMBER_RADIUS)
                {
                    return;
                }
                strike(enemies, j, s.damage, time, &mut cues.hits);
                enemies.halo_until[j] = time + HALO_IMMUNE;
                let push = (p - center).normalize_or(Vec2::X) * HALO_KNOCKBACK;
                enemies.set_pos(j, p + push);
            });
        }
    }

    /// Where the Halo's embers are around Veyra's ground point (none while it rests).
    pub fn embers(&self) -> impl Iterator<Item = Vec2> + '_ {
        let s = &self.stats[BRIMSTONE_HALO];
        let n = if self.halo.on { s.amount } else { 0 };
        ember_offsets(self.halo.angle, s).take(n as usize)
    }

    /// Opacity of the embers: they fade in and out at the ends of each spell.
    pub fn halo_alpha(&self) -> f32 {
        if !self.halo.on {
            return 0.0;
        }
        if self.evolved[BRIMSTONE_HALO] {
            return (self.halo.t / HALO_FADE).min(1.0);
        }
        let left = self.stats[BRIMSTONE_HALO].duration - self.halo.t;
        (self.halo.t.min(left) / HALO_FADE).clamp(0.0, 1.0)
    }
}

fn ember_offsets(angle: f32, s: &ArtStats) -> impl Iterator<Item = Vec2> {
    let (n, radius) = (s.amount.max(1), HALO_RADIUS * s.area);
    (0..n).map(move |k| Vec2::from_angle(angle + k as f32 * TAU / n as f32) * radius)
}

/// A random live enemy within `range` of Veyra.
fn random_enemy(w: &World, range: f32) -> Option<usize> {
    let (mut seen, mut pick) = (0, None);
    w.hash.query_radius(w.player.pos, range, |j, _| {
        if w.enemies.hp[j] > 0.0 {
            seen += 1;
            if gen_range(0, seen) == 0 {
                pick = Some(j);
            }
        }
    });
    pick
}

/// Enemy `j` is alive and its body overlaps a circle of `radius` at `pos`.
fn touches(enemies: &Enemies, j: usize, pos: Vec2, radius: f32) -> bool {
    let reach = radius + ENEMY_DEFS[enemies.kind[j] as usize].radius();
    enemies.hp[j] > 0.0 && enemies.pos(j).distance_squared(pos) <= reach * reach
}

/// Hurts enemy `j`; weakened enemies (Scent of Sin) take `SCENT_WEAKEN` more.
fn strike(enemies: &mut Enemies, j: usize, damage: f32, time: f32, hits: &mut [bool; 2]) {
    let weak = if enemies.weak(j, time) {
        1.0 + SCENT_WEAKEN
    } else {
        1.0
    };
    enemies.hp[j] -= damage * weak;
    enemies.flash[j] = HIT_FLASH_TIME;
    let family = ENEMY_DEFS[enemies.kind[j] as usize].family;
    hits[(family == Family::Armored) as usize] = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Cue, Game, InputState};
    use crate::upgrades::Choice;

    /// A quiet game: no spawns, only `art` (at `level`) attacking.
    fn game_with(art: usize, level: u8) -> Game {
        let mut g = Game::new();
        g.spawn_acc = f32::NEG_INFINITY;
        g.upgrades.arts = [0; ARTS.len()];
        g.upgrades.arts[art] = level;
        g.arts.refresh(&g.upgrades);
        g
    }

    fn tick(g: &mut Game, n: usize) {
        for _ in 0..n {
            g.tick(&InputState::default());
        }
    }

    fn hurt(g: &Game, i: usize) -> bool {
        g.enemies.hp[i] < ENEMY_DEFS[KNIGHT].hp * 100.0
    }

    #[test]
    fn level_tables_add_up_and_passives_boost_every_art() {
        let top = ARTS[HELLFLARE].stats(ART_MAX_LEVEL);
        assert_eq!(top.amount, 4);
        assert!((top.cooldown - 0.6 * 0.8).abs() < 1e-6);
        assert_eq!(top.damage, 20.0);
        let halo = ARTS[BRIMSTONE_HALO].stats(ART_MAX_LEVEL);
        assert_eq!(halo.amount, 4);
        assert!(
            (halo.cooldown - 3.0 * 0.5).abs() < 1e-5,
            "rests half as long"
        );
        assert!((ARTS[TAIL_LASH].stats(ART_MAX_LEVEL).area - 1.6).abs() < 1e-5);
        assert_eq!(ARTS[TAIL_LASH].stats(1), ARTS[TAIL_LASH].base);

        let mut g = game_with(TAIL_LASH, 1);
        for sin in [WRATH, BLOODLUST, LEGION, SLOTH, DARK_SIGIL] {
            g.upgrades.apply(Choice::Sin(sin), &mut g.player);
        }
        g.arts.refresh(&g.upgrades);
        let s = g.arts.stats[TAIL_LASH];
        assert!((s.damage - 14.0 * 1.1).abs() < 1e-5);
        assert!((s.cooldown - 1.35 * 0.92).abs() < 1e-5);
        assert_eq!(s.amount, 2);
        assert!((s.duration - 3.0 * 1.1).abs() < 1e-5);
        assert!((s.area - 1.1).abs() < 1e-6);
        // Sloth leaves projectile lifetimes (their range) alone.
        assert_eq!(
            g.arts.stats[HELLFLARE].duration,
            ARTS[HELLFLARE].base.duration
        );
    }

    /// `game_with` and the Art evolved.
    fn evolved(art: usize) -> Game {
        let mut g = game_with(art, ART_MAX_LEVEL);
        g.upgrades.evolve(art);
        g.arts.refresh(&g.upgrades);
        g
    }

    #[test]
    fn area_lengthens_the_lash() {
        let x = LASH_NEAR + LASH_REACH + ENEMY_DEFS[KNIGHT].radius() + 3.0;
        let mut g = game_with(TAIL_LASH, 1);
        g.player.facing = 1.0;
        g.enemies.spawn(KNIGHT, vec2(x, 0.0), 100.0);
        tick(&mut g, 1);
        assert!(!hurt(&g, 0), "just out of reach");
        let mut g = game_with(TAIL_LASH, 3);
        g.player.facing = 1.0;
        g.enemies.spawn(KNIGHT, vec2(x, 0.0), 100.0);
        tick(&mut g, 1);
        assert!(hurt(&g, 0));
    }

    #[test]
    fn kiss_of_ruin_turns_hellflare_into_piercing_hearts() {
        let mut g = evolved(HELLFLARE);
        for k in 0..4 {
            g.enemies
                .spawn(KNIGHT, vec2(60.0 + k as f32 * 20.0, 0.0), 100.0);
        }
        tick(&mut g, 1);
        let s = g.arts.stats[HELLFLARE];
        assert_eq!(
            s.amount,
            ARTS[HELLFLARE].stats(ART_MAX_LEVEL).amount + KISS_EXTRA
        );
        assert!(g.arts.projectiles.is_empty());
        assert_eq!(g.arts.bolts.len(), s.amount as usize);
        assert!(g.cues.contains(&Cue::Kiss));
        g.arts.hold();
        tick(&mut g, 30);
        // The middle hearts fly down the line, through KISS_PIERCE of them.
        let hit = (0..4).filter(|&i| hurt(&g, i)).count();
        assert!(hit > KISS_PIERCE as usize, "{hit}");
    }

    #[test]
    fn barbed_scourge_makes_lashed_foes_bleed() {
        let mut g = evolved(TAIL_LASH);
        g.player.facing = 1.0;
        g.enemies.spawn(KNIGHT, vec2(30.0, 0.0), 100.0);
        tick(&mut g, 1);
        let after_lash = g.enemies.hp[0];
        assert!(g.enemies.bleed_until[0] > g.time);
        g.arts.hold();
        g.enemies.set_pos(0, vec2(300.0, 0.0));
        tick(&mut g, (BLEED_TICK / TICK_DT) as usize * 2 + 2);
        assert!(g.enemies.hp[0] < after_lash, "bleeds away from the lash");
        // An unevolved lash leaves no bleed.
        let mut g = game_with(TAIL_LASH, ART_MAX_LEVEL);
        g.player.facing = 1.0;
        g.enemies.spawn(KNIGHT, vec2(30.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert_eq!(g.enemies.bleed_until[0], 0.0);
    }

    #[test]
    fn devouring_musk_heals_her_within_a_cap() {
        let mut g = evolved(SCENT_OF_SIN);
        g.player.hp = 50.0;
        g.player.iframes = f32::INFINITY; // the knights close in
        for k in 0..20 {
            g.enemies
                .spawn(KNIGHT, Vec2::from_angle(k as f32) * 20.0, 100.0);
        }
        tick(&mut g, 1);
        let healed = g.player.hp - 50.0;
        assert!(healed > 0.0, "healed");
        assert!(healed <= MUSK_HEAL_CAP + 1e-3);
    }

    #[test]
    fn infernal_crown_never_rests() {
        let mut g = evolved(BRIMSTONE_HALO);
        let s = g.arts.stats[BRIMSTONE_HALO];
        assert_eq!(
            s.amount,
            ARTS[BRIMSTONE_HALO].stats(ART_MAX_LEVEL).amount + CROWN_EXTRA
        );
        tick(&mut g, 2);
        for _ in 0..4 {
            tick(&mut g, ((s.duration + s.cooldown) / TICK_DT) as usize / 4);
            assert!(g.arts.halo.on && g.arts.halo_alpha() == 1.0);
        }
    }

    #[test]
    fn sirens_dominion_bursts_the_charmed_when_the_charm_ends() {
        let mut g = evolved(ALLURE);
        g.arts.hold();
        g.enemies.spawn(KNIGHT, vec2(200.0, 0.0), 100.0);
        g.enemies.charm_until[0] = 0.5;
        g.enemies.spawn(RAT, vec2(215.0, 0.0), 100.0);
        g.enemies.spawn(RAT, vec2(400.0, 0.0), 100.0);
        tick(&mut g, (0.5 / TICK_DT) as usize - 2);
        assert_eq!(g.kills, 0);
        assert!(g.arts.siren_bursts.is_empty());
        tick(&mut g, 4);
        assert_eq!(g.arts.siren_bursts.len(), 1, "the charmed one burst");
        assert!(g.cues.contains(&Cue::SirenBurst));
        assert_eq!(g.kills, 0, "and lives on");
        let rat_hp = ENEMY_DEFS[RAT].hp * 100.0;
        assert!(g.enemies.hp[1] < rat_hp);
        assert_eq!(g.enemies.hp[2], rat_hp);

        // The burst never kills, even a foe on its last legs.
        let mut g = evolved(ALLURE);
        g.arts.hold();
        g.enemies.spawn(KNIGHT, vec2(200.0, 0.0), 100.0);
        // The charm ends on the first tick, before it can bite.
        g.enemies.charm_until[0] = 0.01;
        g.enemies.spawn(RAT, vec2(215.0, 0.0), 1.0);
        g.enemies.hp[1] = 1.5;
        tick(&mut g, 2);
        assert_eq!(g.kills, 0);
        assert_eq!(g.enemies.hp[1], SIREN_SPARE_HP);

        // Without the evolution a charm just ends.
        let mut g = game_with(ALLURE, ART_MAX_LEVEL);
        g.arts.hold();
        g.enemies.spawn(KNIGHT, vec2(200.0, 0.0), 100.0);
        g.enemies.charm_until[0] = 0.5;
        tick(&mut g, (0.5 / TICK_DT) as usize + 4);
        assert_eq!(g.kills, 0);
        assert_eq!(g.enemies.charm_until[0], 0.0);
    }

    #[test]
    fn abyssal_rift_drags_foes_into_the_pool() {
        let mut g = evolved(HELLGATE);
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 100.0);
        tick(&mut g, 1);
        let pool = g.arts.gates[0].pos;
        g.arts.hold();
        g.enemies.spawn(KNIGHT, pool + vec2(40.0, 0.0), 100.0);
        g.enemies
            .spawn(KNIGHT, pool + vec2(RIFT_RADIUS * 1.5 + 20.0, 0.0), 100.0);
        let before = [g.enemies.pos(1), g.enemies.pos(2)];
        tick(&mut g, 10);
        assert!(g.enemies.pos(1).distance(pool) < before[0].distance(pool) - 5.0);
        // Out of reach of the pull: the knight only walks toward Veyra.
        assert!(g.enemies.pos(2).x <= before[1].x);
    }

    #[test]
    fn torment_eternal_bites_every_foe_around_the_first() {
        let mut g = evolved(CHAIN_OF_TORMENT);
        // A tight group far from her: one chain gets them all.
        for k in 0..14 {
            let p = vec2(120.0, 0.0) + Vec2::from_angle(k as f32) * (10.0 + k as f32 * 4.0);
            g.enemies.spawn(KNIGHT, p, 100.0);
        }
        tick(&mut g, 1);
        assert_eq!((0..14).filter(|&i| hurt(&g, i)).count(), 14);
        assert_eq!(g.arts.chains[0].links().len(), 15);
    }

    #[test]
    fn tail_lash_hits_the_facing_side_then_behind() {
        let mut g = game_with(TAIL_LASH, 1);
        g.player.facing = -1.0;
        g.enemies.spawn(KNIGHT, vec2(-30.0, 5.0), 100.0);
        g.enemies.spawn(KNIGHT, vec2(30.0, 5.0), 100.0);
        g.enemies.spawn(KNIGHT, vec2(-30.0, 40.0), 100.0);
        tick(&mut g, 1);
        assert!(hurt(&g, 0) && !hurt(&g, 1) && !hurt(&g, 2));
        assert!(g.cues.contains(&Cue::Lash));

        let mut g = game_with(TAIL_LASH, 2);
        g.player.facing = 1.0;
        g.enemies.spawn(KNIGHT, vec2(-30.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert!(!hurt(&g, 0), "the back lash comes later");
        tick(&mut g, (LASH_DELAY / TICK_DT) as usize + 1);
        assert!(hurt(&g, 0));
        tick(&mut g, 60);
        assert!(g.arts.lashes.len() <= 2);
    }

    #[test]
    fn brimstone_halo_burns_on_its_orbit_and_rests() {
        let mut g = game_with(BRIMSTONE_HALO, 1);
        g.enemies.spawn(KNIGHT, vec2(HALO_RADIUS, 0.0), 100.0);
        let full = g.enemies.hp[0];
        tick(&mut g, 2);
        assert!(g.arts.halo.on);
        assert!(g.enemies.hp[0] < full, "burned as the ember passes");
        assert!(g.enemies.x[0] > HALO_RADIUS, "pushed outwards");
        // Once per immunity window at most.
        let after_first = g.enemies.hp[0];
        g.enemies.set_pos(0, vec2(HALO_RADIUS, 0.0));
        tick(&mut g, 1);
        assert!(g.enemies.hp[0] == after_first || g.time >= HALO_IMMUNE);
        let s = g.arts.stats[BRIMSTONE_HALO];
        tick(&mut g, (s.duration / TICK_DT) as usize);
        assert!(!g.arts.halo.on && g.arts.embers().count() == 0);
        assert_eq!(g.arts.halo_alpha(), 0.0);
    }

    #[test]
    fn allure_charms_the_first_few_in_reach_and_burns_the_rest() {
        let mut g = game_with(ALLURE, 1);
        for p in [vec2(20.0, 0.0), vec2(0.0, 20.0), vec2(-20.0, 0.0)] {
            g.enemies.spawn(KNIGHT, p, 100.0);
        }
        g.enemies
            .spawn(KNIGHT, vec2(0.0, ALLURE_RADIUS + 30.0), 100.0);
        tick(&mut g, 1);
        let charmed: Vec<usize> = (0..4).filter(|&i| g.enemies.charmed(i, g.time)).collect();
        assert_eq!(charmed.len(), ARTS[ALLURE].base.amount as usize);
        assert!(!charmed.contains(&3), "out of reach");
        // The one left free in reach burns; the charmed are spared.
        for i in 0..4 {
            assert_eq!(hurt(&g, i), i != 3 && !charmed.contains(&i), "{i}");
        }
        assert!(g.cues.contains(&Cue::Allure));
        assert!(
            g.arts.allure.since_pulse <= TICK_DT,
            "the ring just started"
        );
        // The charm wears off.
        tick(&mut g, (ARTS[ALLURE].base.duration / TICK_DT) as usize + 2);
        assert!((0..4).all(|i| !g.enemies.charmed(i, g.time)));
    }

    #[test]
    fn charmed_enemies_spare_her_and_hunt_the_horde() {
        let mut g = game_with(ALLURE, 1);
        g.arts.hold();
        // On top of Veyra, but charmed: no contact damage.
        g.enemies.spawn(KNIGHT, Vec2::ZERO, 100.0);
        g.enemies.charm_until[0] = 100.0;
        // Far from her: a charmed knight next to a free rat.
        g.enemies.spawn(KNIGHT, vec2(200.0, 0.0), 100.0);
        g.enemies.charm_until[1] = 100.0;
        g.enemies.spawn(RAT, vec2(230.0, 0.0), 100.0);
        tick(&mut g, 5);
        assert_eq!(g.player.hp, PLAYER_MAX_HP);
        assert!(g.enemies.x[1] > 200.0, "goes for the rat, not for her");
        tick(&mut g, 60);
        assert!(g.enemies.hp[2] < ENEMY_DEFS[RAT].hp * 100.0, "bitten");
        assert_eq!(g.enemies.hp[1], ENEMY_DEFS[KNIGHT].hp * 100.0);
    }

    #[test]
    fn hellgate_opens_under_an_enemy_and_burns_until_it_closes() {
        let mut g = game_with(HELLGATE, 1);
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 100.0);
        g.enemies
            .spawn(KNIGHT, vec2(HELLGATE_RANGE + 100.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert_eq!(g.arts.gates.len(), 1);
        assert!(g.arts.gates[0].pos.distance(vec2(100.0, 0.0)) < 8.0);
        assert!(hurt(&g, 0) && !hurt(&g, 1));
        assert!(g.cues.contains(&Cue::Hellgate));
        let after_first = g.enemies.hp[0];
        tick(&mut g, (HELLGATE_TICK / TICK_DT) as usize + 1);
        assert!(g.enemies.hp[0] < after_first, "burns again");
        g.arts.hold();
        tick(&mut g, (ARTS[HELLGATE].base.duration / TICK_DT) as usize);
        assert!(g.arts.gates.is_empty());
    }

    #[test]
    fn chain_of_torment_bites_amount_enemies_jumping_between_them() {
        let mut g = game_with(CHAIN_OF_TORMENT, 1);
        for x in [100.0, 130.0, 160.0, 190.0] {
            g.enemies.spawn(KNIGHT, vec2(x, 0.0), 100.0);
        }
        g.enemies.spawn(KNIGHT, vec2(400.0, 0.0), 100.0);
        tick(&mut g, 1);
        let bites = ARTS[CHAIN_OF_TORMENT].base.amount as usize;
        assert_eq!((0..5).filter(|&i| hurt(&g, i)).count(), bites);
        assert!(!hurt(&g, 4));
        assert_eq!(g.arts.chains.len(), 1);
        assert_eq!(g.arts.chains[0].links().len(), bites + 1);
        assert_eq!(g.arts.chains[0].links()[0], g.player.pos);
        assert!(g.cues.contains(&Cue::Chain));
        tick(&mut g, (CHAIN_SHOW_TIME / TICK_DT) as usize + 1);
        assert!(g.arts.chains.len() <= 1, "the old chain is gone");
    }

    #[test]
    fn hellgate_pools_grow_with_area() {
        let reach = HELLGATE_RADIUS + ENEMY_DEFS[KNIGHT].radius();
        let mut g = game_with(HELLGATE, 3);
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 100.0);
        tick(&mut g, 1);
        let pool = g.arts.gates[0].pos;
        g.arts.hold();
        // Past a small pool's edge, inside a level 3 one (+25% area).
        g.enemies
            .spawn(KNIGHT, pool + vec2(reach + 2.0, 0.0), 100.0);
        tick(&mut g, (HELLGATE_TICK / TICK_DT) as usize + 1);
        assert!(hurt(&g, 1));
    }

    #[test]
    fn scent_of_sin_burns_and_weakens_what_is_in_the_haze() {
        let mut g = game_with(SCENT_OF_SIN, 1);
        g.enemies.spawn(KNIGHT, vec2(20.0, 0.0), 100.0);
        g.enemies
            .spawn(KNIGHT, vec2(SCENT_RADIUS + 30.0, 0.0), 100.0);
        tick(&mut g, 1);
        let full = ENEMY_DEFS[KNIGHT].hp * 100.0;
        let dmg = ARTS[SCENT_OF_SIN].base.damage;
        assert_eq!(g.enemies.hp[0], full - dmg);
        assert!(g.enemies.weak(0, g.time) && !g.enemies.weak(1, g.time));
        assert!(!hurt(&g, 1));
        // A weakened enemy takes more from every Art.
        let before = g.enemies.hp[0];
        tick(
            &mut g,
            (ARTS[SCENT_OF_SIN].base.cooldown / TICK_DT) as usize + 2,
        );
        let taken = before - g.enemies.hp[0];
        assert!((taken - dmg * (1.0 + SCENT_WEAKEN)).abs() < 1e-3, "{taken}");
    }
}
