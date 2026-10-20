//! Arts (Veyra's weapons) during a run: cooldowns, effective stats and the fixed
//! pools of what they throw. Pure logic, driven by `Game::tick`.

use crate::config::*;
use crate::enemies::Enemies;
use crate::game::Player;
use crate::spatial::SpatialHash;
use crate::upgrades::{PlayerStats, Upgrades};
use macroquad::math::{vec2, Vec2};
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

/// A Kiss of Ruin heart. It goes through `pierce` more enemies, never hitting the
/// same one twice (`hits` holds their ids).
pub struct Bolt {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pierce: u32,
    hits: [u32; BOLT_MAX_HITS],
    hit_count: usize,
}

/// Brimstone Halo: burns for `duration`, rests for `cooldown`, and again. `t` is the
/// time spent in the current phase.
#[derive(Default)]
pub struct Halo {
    pub on: bool,
    pub t: f32,
    pub angle: f32,
}

/// What the Arts did this tick, turned into sound cues by the caller.
#[derive(Default)]
pub struct ArtCues {
    pub fired: bool,
    pub lashed: bool,
    pub kissed: bool,
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
    /// Level of each Art (0 = not owned), copied from `Upgrades` on `refresh`.
    levels: [u8; ARTS.len()],
    /// Effective numbers: level stats with the passive upgrades applied.
    pub stats: [ArtStats; ARTS.len()],
    cooldown: [f32; ARTS.len()],
}

impl Arts {
    pub fn new(upgrades: &Upgrades) -> Self {
        let mut arts = Self {
            projectiles: Vec::with_capacity(MAX_PROJECTILES),
            lashes: Vec::with_capacity(MAX_LASHES),
            bolts: Vec::with_capacity(MAX_BOLTS),
            halo: Halo::default(),
            levels: [0; ARTS.len()],
            stats: ARTS.map(|d| d.base),
            cooldown: [0.0; ARTS.len()],
        };
        arts.refresh(upgrades);
        arts
    }

    /// Picks up new levels and passive bonuses; call after every upgrade.
    pub fn refresh(&mut self, upgrades: &Upgrades) {
        let PlayerStats {
            might,
            cooldown,
            amount,
            ..
        } = upgrades.stats();
        for (i, def) in ARTS.iter().enumerate() {
            let fresh = self.levels[i] == 0 && upgrades.arts[i] > 0;
            self.levels[i] = upgrades.arts[i];
            let mut s = def.stats(self.levels[i]);
            s.damage *= might;
            s.cooldown *= cooldown;
            s.amount += amount;
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
                    ArtKind::KissOfRuin => self.kiss(w, &mut cues),
                    ArtKind::BrimstoneHalo => false,
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
        cues
    }

    /// Hellflare: a volley at the nearest enemy in range, fanned out.
    fn fire(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let from = w.player.pos;
        let Some(target) = w.hash.nearest(from, FIRE_RANGE) else {
            return false;
        };
        let s = &self.stats[HELLFLARE];
        let aim = (w.enemies.pos(target) - from).normalize_or(Vec2::X);
        for k in 0..s.amount {
            if self.projectiles.len() >= MAX_PROJECTILES {
                break;
            }
            let angle = (k as f32 - (s.amount - 1) as f32 * 0.5) * PROJ_SPREAD;
            self.projectiles.push(Projectile {
                pos: from,
                vel: Vec2::from_angle(angle).rotate(aim) * s.speed,
                life: s.duration,
            });
        }
        w.player.since_cast = 0.0;
        cues.fired = true;
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

    /// Kiss of Ruin: hearts side by side, where she is heading.
    fn kiss(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = &self.stats[KISS_OF_RUIN];
        let dir = w.player.aim;
        let across = dir.perp();
        for k in 0..s.amount {
            if self.bolts.len() >= MAX_BOLTS {
                break;
            }
            let off = (k as f32 - (s.amount - 1) as f32 * 0.5) * BOLT_SPACING;
            self.bolts.push(Bolt {
                pos: w.player.pos + across * off,
                vel: dir * s.speed,
                life: s.duration,
                pierce: s.pierce,
                hits: [u32::MAX; BOLT_MAX_HITS],
                hit_count: 0,
            });
        }
        w.player.since_cast = 0.0;
        cues.kissed = true;
        true
    }

    /// Moves fireballs and applies hits. Dead enemies stay in place (hp <= 0) until
    /// the game removes them, so hash indices stay valid for the whole pass.
    fn update_projectiles(&mut self, w: &mut World, cues: &mut ArtCues) {
        let damage = self.stats[HELLFLARE].damage;
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
                strike(enemies, j, damage, &mut cues.hits);
            }
            if hit.is_some() || pr.life <= 0.0 {
                self.projectiles.swap_remove(k);
            }
        }
    }

    /// Lashes strike once when their delay runs out, at Veyra's position then.
    fn update_lashes(&mut self, w: &mut World, cues: &mut ArtCues) {
        let damage = self.stats[TAIL_LASH].damage;
        let enemies = &mut *w.enemies;
        let center = w.player.pos;
        self.lashes.retain_mut(|l| {
            if l.delay > 0.0 {
                l.delay -= TICK_DT;
                return true;
            }
            if l.t == 0.0 {
                let half = vec2(LASH_REACH, LASH_DEPTH) * 0.5;
                let mid = center + vec2(l.side * (LASH_NEAR + half.x), 0.0);
                let query = half.length() + enemies.max_radius();
                w.hash.query_radius(mid, query, |j, p| {
                    let r = ENEMY_DEFS[enemies.kind[j] as usize].radius();
                    let off = (p - mid).abs();
                    if enemies.hp[j] > 0.0 && off.x <= half.x + r && off.y <= half.y + r {
                        strike(enemies, j, damage, &mut cues.hits);
                    }
                });
            }
            l.t += TICK_DT;
            l.t < LASH_SHOW_TIME
        });
    }

    /// Hearts fly straight, hitting each enemy at most once, one per tick.
    fn update_bolts(&mut self, w: &mut World, cues: &mut ArtCues) {
        let damage = self.stats[KISS_OF_RUIN].damage;
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
            strike(enemies, j, damage, &mut cues.hits);
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
        let phase = if h.on { s.duration } else { s.cooldown };
        if h.t >= phase {
            h.on = !h.on;
            h.t = 0.0;
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
                strike(enemies, j, s.damage, &mut cues.hits);
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
        let left = self.stats[BRIMSTONE_HALO].duration - self.halo.t;
        (self.halo.t.min(left) / HALO_FADE).clamp(0.0, 1.0)
    }
}

fn ember_offsets(angle: f32, s: &ArtStats) -> impl Iterator<Item = Vec2> {
    let (n, radius) = (s.amount.max(1), HALO_RADIUS * s.area);
    (0..n).map(move |k| Vec2::from_angle(angle + k as f32 * TAU / n as f32) * radius)
}

/// Enemy `j` is alive and its body overlaps a circle of `radius` at `pos`.
fn touches(enemies: &Enemies, j: usize, pos: Vec2, radius: f32) -> bool {
    let reach = radius + ENEMY_DEFS[enemies.kind[j] as usize].radius();
    enemies.hp[j] > 0.0 && enemies.pos(j).distance_squared(pos) <= reach * reach
}

fn strike(enemies: &mut Enemies, j: usize, damage: f32, hits: &mut [bool; 2]) {
    enemies.hp[j] -= damage;
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
        assert_eq!(ARTS[KISS_OF_RUIN].stats(ART_MAX_LEVEL).pierce, 3);
        assert_eq!(ARTS[BRIMSTONE_HALO].stats(ART_MAX_LEVEL).amount, 4);
        assert_eq!(ARTS[TAIL_LASH].stats(1), ARTS[TAIL_LASH].base);

        let mut g = game_with(TAIL_LASH, 1);
        for kind in [
            UpgradeKind::Fury,
            UpgradeKind::Frenzy,
            UpgradeKind::EmberLegion,
        ] {
            let i = UPGRADES.iter().position(|u| u.kind == kind).unwrap();
            g.upgrades.apply(Choice::Upgrade(i), &mut g.player);
        }
        g.arts.refresh(&g.upgrades);
        let s = g.arts.stats[TAIL_LASH];
        assert_eq!(s.damage, 14.0 * 1.25);
        assert!((s.cooldown - 1.35 / 1.15).abs() < 1e-5);
        assert_eq!(s.amount, 2);
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
    fn kiss_of_ruin_flies_where_she_moved_and_pierces_once() {
        let mut g = game_with(KISS_OF_RUIN, 1);
        g.tick(&InputState {
            move_dir: vec2(0.0, 1.0),
        });
        assert_eq!(g.player.aim, vec2(0.0, 1.0));
        g.arts.bolts.clear();
        g.arts.hold();
        for y in [40.0, 70.0, 100.0] {
            g.enemies
                .spawn(KNIGHT, vec2(g.player.pos.x, g.player.pos.y + y), 100.0);
        }
        g.arts.cooldown[KISS_OF_RUIN] = 0.0;
        tick(&mut g, 30);
        let full = ENEMY_DEFS[KNIGHT].hp * 100.0;
        let dmg = ARTS[KISS_OF_RUIN].base.damage;
        // Base pierce 1: the first two take one hit each, the third none.
        assert_eq!(g.enemies.hp[0], full - dmg);
        assert_eq!(g.enemies.hp[1], full - dmg);
        assert_eq!(g.enemies.hp[2], full);
        assert!(g.cues.contains(&Cue::Kiss));
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
}
