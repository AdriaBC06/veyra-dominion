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

/// The last `N` enemies (by `uid`) a piercing shot went through, so it hits each one
/// once; past `N` the oldest are forgotten.
pub struct HitList<const N: usize> {
    ids: [u32; N],
    next: usize,
}

impl<const N: usize> HitList<N> {
    fn new() -> Self {
        Self {
            ids: [u32::MAX; N],
            next: 0,
        }
    }

    fn contains(&self, uid: u32) -> bool {
        self.ids.contains(&uid)
    }

    fn push(&mut self, uid: u32) {
        self.ids[self.next] = uid;
        self.next = (self.next + 1) % N;
    }

    /// Forgets them all: they can be hit again.
    fn clear(&mut self) {
        self.ids = [u32::MAX; N];
    }
}

/// A Kiss of Ruin heart (Hellflare evolved). It goes through `pierce` more enemies,
/// never hitting the same one twice.
pub struct Bolt {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pierce: u32,
    hits: HitList<BOLT_MAX_HITS>,
}

/// A Reaper's Scythe: thrown up, it falls through the horde hitting each enemy once,
/// `pierce` more after the first. Midnight Harvest scythes fly out curling instead.
pub struct Scythe {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub harvest: bool,
    pierce: u32,
    hits: HitList<PIERCE_MEMORY>,
}

/// A Profaned Cross: it flies out along `dir`, slowing at `speed` falls to zero, then
/// heads back to Veyra (`returning`) faster and faster until she catches it. It hits
/// each enemy once each way.
pub struct Cross {
    pub pos: Vec2,
    pub vel: Vec2,
    speed: f32,
    returning: bool,
    life: f32,
    hits: HitList<PIERCE_MEMORY>,
}

/// A Wandering Sigil bouncing inside the view; it hits each enemy once between
/// bounces.
pub struct Sigil {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    hits: HitList<PIERCE_MEMORY>,
}

/// Broken Seal: a sigil blast at `pos`, spreading to `radius` over `SEAL_BLAST_TIME`.
pub struct SealBlast {
    pub pos: Vec2,
    pub radius: f32,
    pub t: f32,
}

/// A Bat Swarm blood bomb lobbed from a bat over `from` onto `to`: it lands when `t`
/// reaches `BAT_DROP_TIME` and bursts over `radius`, shown for `BAT_BLAST_TIME` more.
pub struct Bomb {
    pub from: Vec2,
    pub to: Vec2,
    pub t: f32,
    pub radius: f32,
}

impl Bomb {
    pub fn landed(&self) -> bool {
        self.t >= BAT_DROP_TIME
    }
}

/// A Gaze of Desire beam from `from` along `dir`, `len` long; `t` is its age.
pub struct Gaze {
    pub from: Vec2,
    pub dir: Vec2,
    pub len: f32,
    pub t: f32,
}

/// A thrall fighting for Veyra: a summoned villager (`kind` is `None`), or a foe the
/// Infernal Court converted (`kind` is its `ENEMY_DEFS` index).
pub struct Thrall {
    pub pos: Vec2,
    pub kind: Option<u8>,
    pub hp: f32,
    /// Per blow.
    damage: f32,
    /// Age, and how long it serves.
    pub t: f32,
    pub life: f32,
    /// Faces left (-1) or right (1).
    pub facing: f32,
    pub moving: bool,
    pub flash: f32,
    /// Walk-cycle offset.
    pub phase: f32,
    /// Time left to its next blow / until it can be hurt again.
    blow: f32,
    hurt: f32,
}

impl Thrall {
    /// A thrall at `pos` serving `life` seconds, with full summoned-villager health and
    /// already risen (a summon resets `t`).
    fn new(pos: Vec2, kind: Option<u8>, life: f32) -> Self {
        Self {
            pos,
            kind,
            hp: THRALL_HP,
            damage: 0.0,
            t: THRALL_RISE,
            life: life + THRALL_RISE,
            facing: -1.0,
            moving: false,
            flash: 0.0,
            phase: 0.0,
            blow: 0.0,
            hurt: 0.0,
        }
    }

    /// What it looks like and how big its body is.
    pub fn def(&self) -> &'static EnemyDef {
        self.kind.map_or(&THRALL_DEF, |k| &ENEMY_DEFS[k as usize])
    }

    /// Opacity: summoned thralls rise over `THRALL_RISE` and every thrall fades over
    /// its last `THRALL_FADE` (a converted foe that falls first just bursts).
    pub fn alpha(&self) -> f32 {
        (self.t / THRALL_RISE)
            .min((self.life - self.t) / THRALL_FADE)
            .clamp(0.0, 1.0)
    }
}

/// A thrall that fell this tick, for the game to burst (and, for a converted foe, to
/// count as a kill and drop its soul).
#[derive(Clone, Copy)]
pub struct Fallen {
    pub pos: Vec2,
    pub kind: Option<u8>,
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
    pub reaped: bool,
    pub crossed: bool,
    pub sigiled: bool,
    /// A sigil blasted on a bounce (Broken Seal).
    pub sealed: bool,
    /// A charmed enemy burst (Siren's Dominion).
    pub burst: bool,
    /// Bats dropped bombs / a bomb burst.
    pub bats: bool,
    pub bat_blast: bool,
    /// Black Mass struck; with Witches' Sabbath every soul flies to her.
    pub mass: bool,
    pub sabbath: bool,
    pub gazed: bool,
    /// Thralls rose at her side.
    pub summoned: bool,
    /// A hit landed on flesh / on armour.
    pub hits: [bool; 2],
}

/// The parts of the simulation the Arts read and change.
pub struct World<'a> {
    pub player: &'a mut Player,
    pub enemies: &'a mut Enemies,
    pub hash: &'a SpatialHash,
    pub time: f32,
    /// Size of the view around Veyra, which the Wandering Sigil bounces inside.
    pub view: Vec2,
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
    pub scythes: Vec<Scythe>,
    pub crosses: Vec<Cross>,
    pub sigils: Vec<Sigil>,
    pub seal_blasts: Vec<SealBlast>,
    pub bombs: Vec<Bomb>,
    pub gazes: Vec<Gaze>,
    pub thralls: Vec<Thrall>,
    /// Thralls that fell this tick; the game drains it.
    pub fallen: Vec<Fallen>,
    /// Where the bats are on their orbit (radians).
    pub bat_angle: f32,
    /// Seconds since the last Black Mass (its circle and flash fade over it).
    pub since_mass: f32,
    /// Seconds since the Scent of Sin last burned (the haze swells a little then).
    pub since_scent: f32,
    /// Level of each Art (0 = not owned), copied from `Upgrades` on `refresh`.
    levels: [u8; ARTS.len()],
    /// Arts in their evolved form, copied from `Upgrades` on `refresh`.
    evolved: [bool; ARTS.len()],
    /// Allure and Thralls fused (Infernal Court), copied from `Upgrades` on `refresh`.
    court: bool,
    /// Wrath's damage multiplier (converted foes hit with their own contact damage).
    might: f32,
    /// Effective numbers: level stats with the Sins applied.
    pub stats: [ArtStats; ARTS.len()],
    cooldown: [f32; ARTS.len()],
    /// Devouring Musk: health it may still give back, refilled at `MUSK_HEAL_CAP`/s.
    heal_budget: f32,
    /// Time left to the next Barbed Scourge bleed.
    bleed_clock: f32,
    /// Time left to the next Heretic's Brand burn.
    brand_clock: f32,
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
            scythes: Vec::with_capacity(MAX_SCYTHES),
            crosses: Vec::with_capacity(MAX_CROSSES),
            sigils: Vec::with_capacity(MAX_SIGILS),
            seal_blasts: Vec::with_capacity(MAX_SEAL_BLASTS),
            bombs: Vec::with_capacity(MAX_BOMBS),
            gazes: Vec::with_capacity(MAX_GAZES),
            thralls: Vec::with_capacity(MAX_THRALLS),
            fallen: Vec::with_capacity(MAX_THRALLS),
            bat_angle: 0.0,
            since_mass: f32::INFINITY,
            since_scent: f32::INFINITY,
            levels: [0; ARTS.len()],
            evolved: [false; ARTS.len()],
            court: false,
            might: 1.0,
            stats: ARTS.map(|d| d.base),
            cooldown: [0.0; ARTS.len()],
            heal_budget: MUSK_HEAL_CAP,
            bleed_clock: 0.0,
            brand_clock: 0.0,
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
        self.court = upgrades.united[INFERNAL_COURT];
        self.might = might;
        for (i, def) in ARTS.iter().enumerate() {
            let fresh = self.levels[i] == 0 && upgrades.arts[i] > 0;
            self.levels[i] = upgrades.arts[i];
            self.evolved[i] = upgrades.evolved[i];
            let mut s = def.stats(self.levels[i]);
            s.damage *= might;
            s.cooldown *= cooldown;
            s.amount += amount;
            s.area *= area;
            // Projectile flights are their range, not an effect that lasts.
            if def.sloth() {
                s.duration *= duration;
            }
            if self.evolved[i] {
                match def.kind {
                    ArtKind::Hellflare => s.amount += KISS_EXTRA,
                    ArtKind::BrimstoneHalo => s.amount += CROWN_EXTRA,
                    ArtKind::ReapersScythe => s.amount += HARVEST_EXTRA,
                    ArtKind::ProfanedCross => s.amount += BRAND_EXTRA,
                    ArtKind::BatSwarm => s.amount += HOST_EXTRA,
                    ArtKind::BlackMass => s.cooldown *= SABBATH_COOLDOWN,
                    ArtKind::GazeOfDesire => s.amount += EVIL_EYE_EXTRA,
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
        w.enemies.frozen_vuln = if self.evolved[GAZE_OF_DESIRE] {
            EVIL_EYE_VULN
        } else {
            0.0
        };
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
                    ArtKind::ReapersScythe => self.reap(w, &mut cues),
                    ArtKind::ProfanedCross => self.throw_crosses(w, &mut cues),
                    ArtKind::WanderingSigil => self.cast_sigils(w, &mut cues),
                    ArtKind::BatSwarm => self.drop_bombs(w, &mut cues),
                    ArtKind::BlackMass => self.black_mass(w, &mut cues),
                    ArtKind::GazeOfDesire => self.gaze(w, &mut cues),
                    ArtKind::Thralls => self.summon(w, &mut cues),
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
        self.update_thralls(w, &mut cues);
        self.update_gates(w, &mut cues);
        self.update_bleed(w, &mut cues);
        self.update_scythes(w, &mut cues);
        self.update_crosses(w, &mut cues);
        self.update_brand(w, &mut cues);
        self.update_sigils(w, &mut cues);
        self.update_bombs(w, &mut cues);
        self.since_mass += TICK_DT;
        self.gazes.retain_mut(|g| {
            g.t += TICK_DT;
            g.t < GAZE_SHOW
        });
        self.heal_budget = (self.heal_budget + MUSK_HEAL_CAP * TICK_DT).min(MUSK_HEAL_CAP);
        self.since_scent += TICK_DT;
        self.siren_bursts.retain_mut(|b| {
            b.t += TICK_DT;
            b.t < SIREN_BURST_TIME
        });
        self.seal_blasts.retain_mut(|b| {
            b.t += TICK_DT;
            b.t < SEAL_BLAST_TIME
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
                    hits: HitList::new(),
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
    /// (the charmed are spared). The Infernal Court turns them into thralls instead
    /// (elites are only charmed, and so is anyone once the pool is full).
    /// Waits while nobody is in reach.
    fn pulse(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[ALLURE];
        let (center, time) = (w.player.pos, w.time);
        let radius = ALLURE_RADIUS * s.area;
        let enemies = &mut *w.enemies;
        let (court, might, thralls) = (self.court, self.might, &mut self.thralls);
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
                    if court && !enemies.elite[j] && thralls.len() < MAX_THRALLS {
                        let kind = enemies.kind[j];
                        let def = &ENEMY_DEFS[kind as usize];
                        let mut t =
                            Thrall::new(enemies.pos(j), Some(kind), s.duration * COURT_LIFE);
                        t.hp = enemies.hp[j];
                        t.damage = def.contact_damage * might;
                        t.phase = enemies.phase[j];
                        thralls.push(t);
                        enemies.hp[j] = 0.0;
                        enemies.gone[j] = true;
                    } else {
                        enemies.charm_until[j] = time + s.duration;
                    }
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

    /// Reaper's Scythe: `amount` scythes thrown up, symmetric (see `scythe_spread`).
    /// Midnight Harvest throws them out evenly all round.
    fn reap(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[REAPERS_SCYTHE];
        let harvest = self.evolved[REAPERS_SCYTHE];
        let (n, turn) = (s.amount.max(1), gen_range(0.0, TAU));
        for k in 0..n {
            if self.scythes.len() >= MAX_SCYTHES {
                break;
            }
            let vel = if harvest {
                Vec2::from_angle(turn + k as f32 * TAU / n as f32) * HARVEST_SPEED
            } else {
                vec2(scythe_spread(k, n), -s.speed)
            };
            self.scythes.push(Scythe {
                pos: w.player.pos,
                vel,
                life: if harvest { HARVEST_LIFE } else { s.duration },
                harvest,
                pierce: if harvest { u32::MAX } else { s.pierce },
                hits: HitList::new(),
            });
        }
        w.player.since_cast = 0.0;
        cues.reaped = true;
        true
    }

    /// Profaned Cross: the first at the nearest enemy in range, the rest at random
    /// ones. Waits while nobody is in range.
    fn throw_crosses(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let from = w.player.pos;
        let Some(first) = w.hash.nearest(from, CROSS_RANGE) else {
            return false;
        };
        let s = self.stats[PROFANED_CROSS];
        for k in 0..s.amount {
            if self.crosses.len() >= MAX_CROSSES {
                break;
            }
            let target = if k == 0 {
                Some(first)
            } else {
                random_enemy(w, CROSS_RANGE)
            };
            let Some(j) = target else {
                break;
            };
            let dir = (w.enemies.pos(j) - from).normalize_or(Vec2::X);
            self.crosses.push(Cross {
                pos: from,
                vel: dir * s.speed,
                speed: s.speed,
                returning: false,
                life: CROSS_MAX_LIFE,
                hits: HitList::new(),
            });
        }
        w.player.since_cast = 0.0;
        cues.crossed = true;
        true
    }

    /// Wandering Sigil: `amount` sigils sent off in random directions.
    fn cast_sigils(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[WANDERING_SIGIL];
        for _ in 0..s.amount {
            if self.sigils.len() >= MAX_SIGILS {
                break;
            }
            self.sigils.push(Sigil {
                pos: w.player.pos,
                vel: Vec2::from_angle(gen_range(0.0, TAU)) * s.speed,
                life: s.duration,
                hits: HitList::new(),
            });
        }
        cues.sigiled = true;
        true
    }

    /// Bat Swarm: every bat lobs a bomb at a random enemy in range (they wait while
    /// nobody is). Crimson Host bombs burst wider.
    fn drop_bombs(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let s = self.stats[BAT_SWARM];
        let host = if self.evolved[BAT_SWARM] {
            HOST_BLAST
        } else {
            1.0
        };
        let radius = BAT_BLAST_RADIUS * s.area * host;
        let mut dropped = false;
        for off in bat_offsets(self.bat_angle, s.amount) {
            if self.bombs.len() >= MAX_BOMBS {
                break;
            }
            let Some(j) = random_enemy(w, BAT_RANGE) else {
                break;
            };
            let scatter = Vec2::from_angle(gen_range(0.0, TAU)) * gen_range(0.0, BAT_SCATTER);
            self.bombs.push(Bomb {
                from: w.player.pos + off,
                to: w.enemies.pos(j) + scatter,
                t: 0.0,
                radius,
            });
            dropped = true;
        }
        cues.bats |= dropped;
        dropped
    }

    /// The bats circle on; bombs land and burst on whatever is under them.
    fn update_bombs(&mut self, w: &mut World, cues: &mut ArtCues) {
        let s = self.stats[BAT_SWARM];
        if self.owns(BAT_SWARM) {
            self.bat_angle = (self.bat_angle + s.speed * TICK_DT) % TAU;
        }
        let time = w.time;
        let enemies = &mut *w.enemies;
        let extra = enemies.max_radius();
        self.bombs.retain_mut(|b| {
            let falling = !b.landed();
            b.t += TICK_DT;
            if falling && b.landed() {
                w.hash.query_radius(b.to, b.radius + extra, |j, _| {
                    if touches(enemies, j, b.to, b.radius) {
                        strike(enemies, j, s.damage, time, &mut cues.hits);
                    }
                });
                cues.bat_blast = true;
            }
            b.t < BAT_DROP_TIME + BAT_BLAST_TIME
        });
    }

    /// Black Mass: strikes every enemy in view (plus a margin); waits while there is
    /// none. Witches' Sabbath also calls every soul to her (the game does that).
    fn black_mass(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let damage = self.stats[BLACK_MASS].damage;
        let (center, time) = (w.player.pos, w.time);
        let half = w.view * 0.5 + Vec2::splat(MASS_MARGIN);
        let enemies = &mut *w.enemies;
        let mut any = false;
        for i in 0..enemies.len() {
            let off = (enemies.pos(i) - center).abs();
            if enemies.hp[i] > 0.0 && off.x <= half.x && off.y <= half.y {
                strike(enemies, i, damage, time, &mut cues.hits);
                any = true;
            }
        }
        if any {
            self.since_mass = 0.0;
            w.player.since_cast = 0.0;
            cues.mass = true;
            cues.sabbath = self.evolved[BLACK_MASS];
        }
        any
    }

    /// Gaze of Desire: `amount` beams, the first at the nearest enemy in range, the
    /// rest evenly around; whatever they touch is frozen. Waits while nobody is near.
    fn gaze(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        let from = w.player.pos;
        let Some(first) = w.hash.nearest(from, GAZE_RANGE) else {
            return false;
        };
        let s = self.stats[GAZE_OF_DESIRE];
        let aim = (w.enemies.pos(first) - from).normalize_or(Vec2::X);
        let (len, width) = (GAZE_REACH * s.area, GAZE_WIDTH * s.area);
        let (time, until) = (w.time, w.time + s.duration);
        let enemies = &mut *w.enemies;
        let extra = enemies.max_radius();
        let n = s.amount.max(1);
        for k in 0..n {
            let dir = Vec2::from_angle(k as f32 * TAU / n as f32).rotate(aim);
            let mid = from + dir * len * 0.5;
            w.hash.query_radius(mid, len * 0.5 + width + extra, |j, p| {
                let r = ENEMY_DEFS[enemies.kind[j] as usize].radius();
                let off = p - from;
                let along = off.dot(dir);
                let inside =
                    along >= -r && along <= len + r && off.perp_dot(dir).abs() <= width + r;
                if enemies.hp[j] > 0.0 && inside {
                    enemies.frozen_until[j] = enemies.frozen_until[j].max(until);
                    if s.damage > 0.0 {
                        strike(enemies, j, s.damage, time, &mut cues.hits);
                    }
                }
            });
            if self.gazes.len() < MAX_GAZES {
                self.gazes.push(Gaze {
                    from,
                    dir,
                    len,
                    t: 0.0,
                });
            }
        }
        w.player.since_cast = 0.0;
        cues.gazed = true;
        true
    }

    /// Where the bats are around Veyra's ground point (none unless she owns them).
    pub fn bats(&self) -> impl Iterator<Item = Vec2> {
        let n = if self.owns(BAT_SWARM) {
            self.stats[BAT_SWARM].amount
        } else {
            0
        };
        bat_offsets(self.bat_angle, n)
    }

    /// Scythes fall (Midnight Harvest ones curl outwards) and cut what they cross.
    fn update_scythes(&mut self, w: &mut World, cues: &mut ArtCues) {
        let s = self.stats[REAPERS_SCYTHE];
        let time = w.time;
        let enemies = &mut *w.enemies;
        let extra = enemies.max_radius();
        let curl = Vec2::from_angle(HARVEST_CURL * TICK_DT);
        self.scythes.retain_mut(|sc| {
            if sc.harvest {
                sc.vel = curl.rotate(sc.vel);
            } else {
                sc.vel.y += SCYTHE_GRAVITY * TICK_DT;
            }
            sc.pos += sc.vel * TICK_DT;
            sc.life -= TICK_DT;
            let base = if sc.harvest {
                HARVEST_RADIUS
            } else {
                SCYTHE_RADIUS
            };
            let radius = base * s.area;
            let mut spent = false;
            w.hash.query_radius(sc.pos, radius + extra, |j, _| {
                if spent || !touches(enemies, j, sc.pos, radius) || sc.hits.contains(enemies.uid[j])
                {
                    return;
                }
                strike(enemies, j, s.damage, time, &mut cues.hits);
                sc.hits.push(enemies.uid[j]);
                match sc.pierce {
                    0 => spent = true,
                    _ => sc.pierce -= 1,
                }
            });
            !spent && sc.life > 0.0
        });
    }

    /// Crosses slow down, turn back (forgetting whom they hit) and chase her until she
    /// catches them. Heretic's Brand sets what they hit on fire.
    fn update_crosses(&mut self, w: &mut World, cues: &mut ArtCues) {
        let s = self.stats[PROFANED_CROSS];
        let time = w.time;
        let brand = self.evolved[PROFANED_CROSS].then_some(time + BRAND_BURN);
        let enemies = &mut *w.enemies;
        let radius = CROSS_RADIUS * s.area;
        let query = radius + enemies.max_radius();
        let home = w.player.pos;
        self.crosses.retain_mut(|c| {
            if c.returning {
                c.speed += CROSS_DECEL * TICK_DT;
                let to = home - c.pos;
                if to.length() <= CROSS_CATCH + c.speed * TICK_DT {
                    return false;
                }
                c.vel = to.normalize() * c.speed;
            } else {
                c.speed -= CROSS_DECEL * TICK_DT;
                if c.speed <= 0.0 {
                    c.returning = true;
                    c.hits.clear();
                    c.speed = 0.0;
                }
                c.vel = c.vel.normalize_or(Vec2::X) * c.speed;
            }
            c.pos += c.vel * TICK_DT;
            c.life -= TICK_DT;
            w.hash.query_radius(c.pos, query, |j, _| {
                if touches(enemies, j, c.pos, radius) && !c.hits.contains(enemies.uid[j]) {
                    strike(enemies, j, s.damage, time, &mut cues.hits);
                    c.hits.push(enemies.uid[j]);
                    if let Some(until) = brand {
                        enemies.burn_until[j] = until;
                    }
                }
            });
            c.life > 0.0
        });
    }

    /// Heretic's Brand: every `BRAND_TICK` burning enemies lose a share of the Cross's
    /// damage.
    fn update_brand(&mut self, w: &mut World, cues: &mut ArtCues) {
        if !self.evolved[PROFANED_CROSS] {
            return;
        }
        self.brand_clock -= TICK_DT;
        if self.brand_clock > 0.0 {
            return;
        }
        self.brand_clock += BRAND_TICK;
        let damage = self.stats[PROFANED_CROSS].damage * BRAND_SHARE;
        let enemies = &mut *w.enemies;
        for i in 0..enemies.len() {
            if enemies.burn_until[i] > w.time && enemies.hp[i] > 0.0 {
                strike(enemies, i, damage, w.time, &mut cues.hits);
            }
        }
    }

    /// Sigils fly straight and bounce off the view's edges, forgetting whom they hit;
    /// with Broken Seal every bounce blasts the enemies around it.
    fn update_sigils(&mut self, w: &mut World, cues: &mut ArtCues) {
        let s = self.stats[WANDERING_SIGIL];
        let time = w.time;
        let seal = self.evolved[WANDERING_SIGIL];
        let (blast_r, blast_dmg) = (SEAL_BLAST_RADIUS * s.area, s.damage * SEAL_BLAST_DAMAGE);
        let enemies = &mut *w.enemies;
        let radius = SIGIL_RADIUS * s.area;
        let extra = enemies.max_radius();
        let center = w.player.pos;
        // It floats above its ground point, so the top edge comes that much sooner.
        let half = w.view * 0.5 - Vec2::splat(SIGIL_MARGIN);
        let (top, bottom) = (-half.y + SIGIL_DRAW_HEIGHT, half.y);
        let blasts = &mut self.seal_blasts;
        self.sigils.retain_mut(|g| {
            g.pos += g.vel * TICK_DT;
            g.life -= TICK_DT;
            let off = g.pos - center;
            let mut bounced = false;
            if off.x.abs() > half.x && off.x * g.vel.x > 0.0 {
                g.vel.x = -g.vel.x;
                bounced = true;
            }
            if (off.y < top && g.vel.y < 0.0) || (off.y > bottom && g.vel.y > 0.0) {
                g.vel.y = -g.vel.y;
                bounced = true;
            }
            if bounced {
                g.hits.clear();
                if seal {
                    let p = g.pos;
                    w.hash.query_radius(p, blast_r + extra, |j, _| {
                        if touches(enemies, j, p, blast_r) {
                            strike(enemies, j, blast_dmg, time, &mut cues.hits);
                        }
                    });
                    if blasts.len() < MAX_SEAL_BLASTS {
                        blasts.push(SealBlast {
                            pos: p,
                            radius: blast_r,
                            t: 0.0,
                        });
                    }
                    cues.sealed = true;
                }
            }
            w.hash.query_radius(g.pos, radius + extra, |j, _| {
                if touches(enemies, j, g.pos, radius) && !g.hits.contains(enemies.uid[j]) {
                    strike(enemies, j, s.damage, time, &mut cues.hits);
                    g.hits.push(enemies.uid[j]);
                }
            });
            g.life > 0.0
        });
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
        let time = w.time;
        for i in 0..w.enemies.len() {
            let until = w.enemies.charm_until[i];
            if until == 0.0 || until > time {
                continue;
            }
            w.enemies.charm_until[i] = 0.0;
            if dominion && w.enemies.hp[i] > 0.0 {
                let p = w.enemies.pos(i);
                self.siren_burst(w, p, Some(i), cues);
            }
        }
    }

    /// Siren's Dominion: a burst at `p` hurts the free enemies around it (not `skip`,
    /// the one bursting) without killing any.
    fn siren_burst(&mut self, w: &mut World, p: Vec2, skip: Option<usize>, cues: &mut ArtCues) {
        let s = self.stats[ALLURE];
        let (radius, damage) = (SIREN_BURST_RADIUS * s.area, s.damage * SIREN_BURST_DAMAGE);
        let time = w.time;
        let enemies = &mut *w.enemies;
        w.hash
            .query_radius(p, radius + enemies.max_radius(), |j, _| {
                if Some(j) != skip && !enemies.charmed(j, time) && touches(enemies, j, p, radius) {
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

    /// Thralls: `amount` seduced villagers rise around her, while a foe is near and
    /// the pool has room.
    fn summon(&mut self, w: &mut World, cues: &mut ArtCues) -> bool {
        if w.hash.nearest(w.player.pos, THRALL_SEEK).is_none() {
            return false;
        }
        let s = self.stats[THRALLS];
        let mut any = false;
        for _ in 0..s.amount {
            if self.thralls.len() >= MAX_THRALLS {
                break;
            }
            let pos = w.player.pos + Vec2::from_angle(gen_range(0.0, TAU)) * THRALL_SPAWN_RADIUS;
            let mut t = Thrall::new(pos, None, s.duration);
            t.t = 0.0;
            t.damage = s.damage;
            t.phase = gen_range(0.0, 2.0);
            self.thralls.push(t);
            any = true;
        }
        cues.summoned |= any;
        any
    }

    /// Every thrall walks to the nearest foe (or back to her), strikes what it reaches
    /// every `THRALL_BLOW` and is hurt by the free foes touching it. The fallen go to
    /// `fallen`; a converted foe whose time runs out falls too (bursting first with
    /// Siren's Dominion), a summoned one just fades away.
    fn update_thralls(&mut self, w: &mut World, cues: &mut ArtCues) {
        if self.thralls.is_empty() {
            return;
        }
        // Keep apart: they are not in the hash, so the horde's separation misses them.
        let n = self.thralls.len();
        for i in 0..n {
            for j in i + 1..n {
                let d = self.thralls[i].pos - self.thralls[j].pos;
                let l2 = d.length_squared();
                if l2 >= THRALL_SPACING * THRALL_SPACING {
                    continue;
                }
                let len = l2.sqrt();
                let dir = if len > 1e-3 {
                    d / len
                } else {
                    Vec2::from_angle(i as f32)
                };
                let push = dir * (1.0 - len / THRALL_SPACING) * THRALL_SPACING_PUSH * TICK_DT;
                self.thralls[i].pos += push;
                self.thralls[j].pos -= push;
            }
        }
        let s = self.stats[THRALLS];
        let (time, home) = (w.time, w.player.pos);
        let enemies = &mut *w.enemies;
        let extra = enemies.max_radius();
        for th in &mut self.thralls {
            th.t += TICK_DT;
            th.flash = (th.flash - TICK_DT).max(0.0);
            th.blow -= TICK_DT;
            th.hurt -= TICK_DT;
            let def = th.def();
            let body = def.radius();
            let reach = THRALL_REACH * s.area;
            let target = w
                .hash
                .nearest(th.pos, THRALL_SEEK)
                .filter(|&j| enemies.hp[j] > 0.0 && !enemies.charmed(j, time));
            let in_reach = target.filter(|&j| touches(enemies, j, th.pos, body + reach));
            let goal = match (target, in_reach) {
                (_, Some(_)) => None,
                (Some(j), None) => Some(enemies.pos(j)),
                (None, _) => {
                    (th.pos.distance_squared(home) > THRALL_LEASH * THRALL_LEASH).then_some(home)
                }
            };
            th.moving = goal.is_some();
            if let Some(g) = goal {
                // A converted foe keeps its own pace.
                let speed = if th.kind.is_some() {
                    def.speed
                } else {
                    s.speed
                };
                let dir = (g - th.pos).normalize_or_zero();
                th.pos += dir * speed * TICK_DT;
                if dir.x.abs() > 0.2 {
                    th.facing = dir.x.signum();
                }
            }
            if let Some(j) = in_reach.filter(|_| th.blow <= 0.0) {
                strike(enemies, j, th.damage, time, &mut cues.hits);
                th.blow = THRALL_BLOW;
                th.facing = (enemies.x[j] - th.pos.x).signum();
            }
            if th.hurt <= 0.0 {
                let (mut damage, mut seen) = (0.0f32, 0);
                w.hash.try_query_radius(th.pos, body + extra, |j, q| {
                    let d = &ENEMY_DEFS[enemies.kind[j] as usize];
                    let r = body + d.radius();
                    let free = !enemies.charmed(j, time) && !enemies.frozen(j, time);
                    if enemies.hp[j] > 0.0 && free && q.distance_squared(th.pos) <= r * r {
                        damage = damage.max(d.contact_damage);
                    }
                    seen += 1;
                    seen < THRALL_CONTACT_MAX
                });
                if damage > 0.0 {
                    th.hp -= damage;
                    th.hurt = THRALL_IFRAMES;
                    th.flash = HIT_FLASH_TIME;
                }
            }
        }
        let dominion = self.evolved[ALLURE];
        let mut k = 0;
        while k < self.thralls.len() {
            let th = &self.thralls[k];
            let (dead, done) = (th.hp <= 0.0, th.t >= th.life);
            if !dead && !done {
                k += 1;
                continue;
            }
            let th = self.thralls.swap_remove(k);
            let converted = th.kind.is_some();
            if (dead || converted) && self.fallen.len() < MAX_THRALLS {
                self.fallen.push(Fallen {
                    pos: th.pos,
                    kind: th.kind,
                });
            }
            if done && !dead && converted && dominion {
                self.siren_burst(w, th.pos, None, cues);
            }
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
                    && !b.hits.contains(enemies.uid[j])
                {
                    hit = Some(j);
                }
            });
            let Some(j) = hit else {
                return b.life > 0.0;
            };
            strike(enemies, j, damage, time, &mut cues.hits);
            b.hits.push(enemies.uid[j]);
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

/// `n` bats evenly spread on their orbit, an ellipse seen from above at an angle.
fn bat_offsets(angle: f32, n: u32) -> impl Iterator<Item = Vec2> {
    let n = (n as usize).min(MAX_BATS);
    (0..n).map(move |k| {
        let v = Vec2::from_angle(angle + k as f32 * TAU / n as f32) * BAT_ORBIT;
        vec2(v.x, v.y * 0.5)
    })
}

/// Sideways speed of scythe `k` of `n`: with an odd count the first goes straight up,
/// the rest pair off to either side, each pair `SCYTHE_SPREAD` wider.
fn scythe_spread(k: u32, n: u32) -> f32 {
    let k = if n % 2 == 1 {
        if k == 0 {
            return 0.0;
        }
        k - 1
    } else {
        k
    };
    let side = if k % 2 == 0 { -1.0 } else { 1.0 };
    side * (k / 2 + 1) as f32 * SCYTHE_SPREAD
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

/// Hurts enemy `j`; weakened enemies (Scent of Sin) take `SCENT_WEAKEN` more, frozen
/// ones `frozen_vuln` more (Evil Eye).
fn strike(enemies: &mut Enemies, j: usize, damage: f32, time: f32, hits: &mut [bool; 2]) {
    let mut scale = 1.0;
    if enemies.weak(j, time) {
        scale *= 1.0 + SCENT_WEAKEN;
    }
    if enemies.frozen(j, time) {
        scale *= 1.0 + enemies.frozen_vuln;
    }
    enemies.hp[j] -= damage * scale;
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
    fn new_arts_level_tables_and_sloth() {
        let scythe = ARTS[REAPERS_SCYTHE].stats(ART_MAX_LEVEL);
        assert_eq!((scythe.amount, scythe.pierce, scythe.damage), (4, 6, 50.0));
        assert!((scythe.area - 1.45).abs() < 1e-5, "grows every other level");
        let cross = ARTS[PROFANED_CROSS].stats(ART_MAX_LEVEL);
        assert_eq!((cross.amount, cross.damage), (3, 40.0));
        assert!((cross.speed - 260.0 * 1.5).abs() < 1e-3);
        let sigil = ARTS[WANDERING_SIGIL].stats(ART_MAX_LEVEL);
        assert_eq!(sigil.amount, 3);
        assert!((sigil.duration - 3.35).abs() < 1e-5);

        // Sloth lengthens the Sigil's wander, not the flights of scythes and crosses.
        let mut g = game_with(WANDERING_SIGIL, 1);
        g.upgrades.arts[REAPERS_SCYTHE] = 1;
        g.upgrades.arts[PROFANED_CROSS] = 1;
        g.upgrades.apply(Choice::Sin(SLOTH), &mut g.player);
        g.arts.refresh(&g.upgrades);
        let base = |i: usize| ARTS[i].base.duration;
        assert!(
            (g.arts.stats[WANDERING_SIGIL].duration - base(WANDERING_SIGIL) * 1.1).abs() < 1e-5
        );
        assert_eq!(g.arts.stats[REAPERS_SCYTHE].duration, base(REAPERS_SCYTHE));
        assert_eq!(g.arts.stats[PROFANED_CROSS].duration, base(PROFANED_CROSS));
    }

    #[test]
    fn scythes_fly_up_then_fall_through_pierce_plus_one_foes() {
        let mut g = game_with(REAPERS_SCYTHE, 1);
        tick(&mut g, 1);
        assert_eq!(g.arts.scythes.len(), 1);
        assert!(g.cues.contains(&Cue::Scythe));
        let start = g.arts.scythes[0].pos;
        g.arts.hold();
        tick(&mut g, 20);
        let sc = &g.arts.scythes[0];
        assert!(sc.pos.y < start.y && sc.vel.y > -ARTS[REAPERS_SCYTHE].base.speed);
        tick(
            &mut g,
            (ARTS[REAPERS_SCYTHE].base.duration / TICK_DT) as usize,
        );
        assert!(g.arts.scythes.is_empty(), "gone after its flight");

        // One dropped onto a column of knights cuts the first and `pierce` more.
        let mut g = game_with(REAPERS_SCYTHE, 1);
        g.arts.hold();
        for k in 0..5 {
            g.enemies
                .spawn(KNIGHT, vec2(200.0, 40.0 + k as f32 * 30.0), 100.0);
        }
        g.arts.scythes.push(Scythe {
            pos: vec2(200.0, 0.0),
            vel: vec2(0.0, 200.0),
            life: 2.0,
            harvest: false,
            pierce: ARTS[REAPERS_SCYTHE].base.pierce,
            hits: HitList::new(),
        });
        tick(&mut g, 90);
        let cut = (0..5).filter(|&i| hurt(&g, i)).count();
        assert_eq!(cut, ARTS[REAPERS_SCYTHE].base.pierce as usize + 1);
        let full = ENEMY_DEFS[KNIGHT].hp * 100.0;
        assert_eq!(
            g.enemies.hp[0],
            full - ARTS[REAPERS_SCYTHE].base.damage,
            "once each"
        );
    }

    #[test]
    fn midnight_harvest_sends_scythes_out_all_round() {
        let mut g = evolved(REAPERS_SCYTHE);
        tick(&mut g, 1);
        let n = g.arts.stats[REAPERS_SCYTHE].amount as usize;
        assert_eq!(
            n,
            ARTS[REAPERS_SCYTHE].stats(ART_MAX_LEVEL).amount as usize + 5
        );
        assert_eq!(g.arts.scythes.len(), n);
        let sum: Vec2 = g.arts.scythes.iter().map(|s| s.vel).sum();
        assert!(sum.length() < 1.0, "evenly spread");
        assert!(g.arts.scythes.iter().all(|s| s.harvest));
        // They curl without falling, and cut through a whole crowd.
        g.arts.hold();
        for k in 0..12 {
            g.enemies
                .spawn(KNIGHT, Vec2::from_angle(k as f32 * 0.5) * 40.0, 100.0);
        }
        tick(&mut g, 30);
        for s in &g.arts.scythes {
            assert!((s.vel.length() - HARVEST_SPEED).abs() < 0.5);
        }
        assert!((0..12).filter(|&i| hurt(&g, i)).count() >= 8);
    }

    #[test]
    fn profaned_cross_hits_on_the_way_out_and_back_into_her_hands() {
        let mut g = game_with(PROFANED_CROSS, 1);
        tick(&mut g, 1);
        assert!(g.arts.crosses.is_empty(), "waits for a target");
        g.enemies.spawn(KNIGHT, vec2(60.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert_eq!(g.arts.crosses.len(), 1);
        assert!(g.cues.contains(&Cue::Cross));
        g.arts.hold();
        let mut far: f32 = 0.0;
        let mut ticks = 0;
        while let Some(c) = g.arts.crosses.first() {
            far = far.max(c.pos.x);
            assert!(c.pos.x > -CROSS_CATCH, "never flies past her");
            tick(&mut g, 1);
            ticks += 1;
            assert!(ticks < 600, "comes back");
        }
        let s = ARTS[PROFANED_CROSS].base;
        let reach = s.speed * s.speed / (2.0 * CROSS_DECEL);
        assert!((far - reach).abs() < 6.0, "{far} vs {reach}");
        let full = ENEMY_DEFS[KNIGHT].hp * 100.0;
        assert_eq!(g.enemies.hp[0], full - 2.0 * s.damage, "out and back");

        // It follows her wherever she went meanwhile.
        let mut g = game_with(PROFANED_CROSS, 1);
        g.enemies.spawn(KNIGHT, vec2(60.0, 0.0), 1000.0);
        tick(&mut g, 1);
        g.arts.hold();
        g.player.pos = vec2(-150.0, 120.0);
        for _ in 0..600 {
            if g.arts.crosses.is_empty() {
                break;
            }
            let c = &g.arts.crosses[0];
            assert!(c.life > 0.0);
            tick(&mut g, 1);
        }
        assert!(g.arts.crosses.is_empty(), "caught where she is");
    }

    #[test]
    fn scythes_are_thrown_symmetrically() {
        let spread = |n: u32| {
            (0..n)
                .map(|k| scythe_spread(k, n) / SCYTHE_SPREAD)
                .collect::<Vec<_>>()
        };
        assert_eq!(spread(1), [0.0]);
        assert_eq!(spread(2), [-1.0, 1.0]);
        assert_eq!(spread(3), [0.0, -1.0, 1.0]);
        assert_eq!(spread(4), [-1.0, 1.0, -2.0, 2.0]);
        assert_eq!(spread(5), [0.0, -1.0, 1.0, -2.0, 2.0]);
        let mut g = game_with(REAPERS_SCYTHE, 4);
        tick(&mut g, 1);
        let sum: f32 = g.arts.scythes.iter().map(|s| s.vel.x).sum();
        assert_eq!(g.arts.scythes.len(), 2);
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn heretics_brand_sets_what_the_crosses_hit_ablaze() {
        let mut g = evolved(PROFANED_CROSS);
        g.enemies.spawn(KNIGHT, vec2(40.0, 0.0), 100.0);
        tick(&mut g, 4);
        assert!(g.enemies.burn_until[0] > g.time);
        g.arts.hold();
        g.arts.crosses.clear();
        let before = g.enemies.hp[0];
        tick(&mut g, (BRAND_TICK / TICK_DT) as usize * 2 + 2);
        assert!(g.enemies.hp[0] < before, "burns away from the cross");
        // An unevolved cross sets nothing alight.
        let mut g = game_with(PROFANED_CROSS, ART_MAX_LEVEL);
        g.enemies.spawn(KNIGHT, vec2(40.0, 0.0), 100.0);
        tick(&mut g, 4);
        assert!(hurt(&g, 0));
        assert_eq!(g.enemies.burn_until[0], 0.0);
    }

    #[test]
    fn wandering_sigil_bounces_inside_the_view() {
        let mut g = game_with(WANDERING_SIGIL, 1);
        tick(&mut g, 1);
        assert_eq!(g.arts.sigils.len(), 1);
        assert!(g.cues.contains(&Cue::Sigil));
        g.arts.hold();
        let v0 = g.arts.sigils[0].vel;
        let half = vec2(DEFAULT_VIEW_W, LOGICAL_H) * 0.5;
        let step = ARTS[WANDERING_SIGIL].base.speed * TICK_DT;
        let mut bounced = false;
        while let Some(sg) = g.arts.sigils.first() {
            let off = sg.pos - g.player.pos;
            assert!(off.x.abs() <= half.x + step && off.y.abs() <= half.y + step);
            bounced |= sg.vel != v0;
            tick(&mut g, 1);
        }
        assert!(bounced, "it travels further than any edge is far");
    }

    #[test]
    fn broken_seal_blasts_around_every_bounce() {
        let mut g = evolved(WANDERING_SIGIL);
        g.arts.hold();
        let edge = DEFAULT_VIEW_W * 0.5 - SIGIL_MARGIN;
        g.enemies.spawn(KNIGHT, vec2(edge - 8.0, 22.0), 100.0);
        g.enemies.spawn(KNIGHT, vec2(edge - 8.0, -120.0), 100.0);
        g.arts.sigils.push(Sigil {
            pos: vec2(edge + 1.0, 0.0),
            vel: vec2(100.0, 0.0),
            life: 1.0,
            hits: HitList::new(),
        });
        tick(&mut g, 1);
        assert!(g.arts.sigils[0].vel.x < 0.0, "bounced back");
        assert_eq!(g.arts.seal_blasts.len(), 1);
        assert!(g.cues.contains(&Cue::SealBlast));
        let full = ENEMY_DEFS[KNIGHT].hp * 100.0;
        let blast = g.arts.stats[WANDERING_SIGIL].damage * SEAL_BLAST_DAMAGE;
        assert_eq!(g.enemies.hp[0], full - blast);
        assert!(!hurt(&g, 1), "out of the blast");
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

    #[test]
    fn bats_mass_and_gaze_level_tables() {
        let bats = ARTS[BAT_SWARM].stats(ART_MAX_LEVEL);
        assert_eq!((bats.amount, bats.damage), (5, 30.0));
        assert!((bats.area - 1.3).abs() < 1e-5);
        let mass = ARTS[BLACK_MASS].stats(ART_MAX_LEVEL);
        assert_eq!(mass.damage, 140.0);
        assert!((mass.cooldown - 12.0 * 0.7).abs() < 1e-4);
        let gaze = ARTS[GAZE_OF_DESIRE].stats(ART_MAX_LEVEL);
        assert_eq!(gaze.amount, 3);
        assert!((gaze.duration - 2.0).abs() < 1e-5 && (gaze.area - 1.4).abs() < 1e-5);
    }

    #[test]
    fn bats_bomb_random_foes_and_the_blast_hurts_around_it() {
        let mut g = game_with(BAT_SWARM, 1);
        tick(&mut g, 1);
        assert!(g.arts.bombs.is_empty(), "nobody to bomb");
        assert_eq!(g.arts.bats().count(), ARTS[BAT_SWARM].base.amount as usize);
        g.enemies.spawn(KNIGHT, vec2(120.0, 0.0), 100.0);
        g.enemies.spawn(KNIGHT, vec2(120.0, 12.0), 100.0);
        // Out of the bats' range: never a target, and far from the blasts.
        g.enemies
            .spawn(KNIGHT, vec2(-(BAT_RANGE + 40.0), 0.0), 100.0);
        tick(&mut g, 1);
        assert!(g.cues.contains(&Cue::Bats));
        assert_eq!(g.arts.bombs.len(), 2);
        assert!(!hurt(&g, 0), "still falling");
        tick(&mut g, (BAT_DROP_TIME / TICK_DT) as usize + 2);
        assert!(hurt(&g, 0) && hurt(&g, 1), "the blast covers both");
        assert!(!hurt(&g, 2));
        tick(&mut g, (BAT_BLAST_TIME / TICK_DT) as usize + 2);
        assert!(g.arts.bombs.is_empty());
    }

    #[test]
    fn crimson_host_is_a_bigger_flock_with_wider_blasts() {
        let mut g = evolved(BAT_SWARM);
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 100.0);
        tick(&mut g, 1);
        let n = ARTS[BAT_SWARM].stats(ART_MAX_LEVEL).amount + HOST_EXTRA;
        assert_eq!(g.arts.bats().count(), n as usize);
        assert_eq!(g.arts.bombs.len(), n as usize);
        let area = ARTS[BAT_SWARM].stats(ART_MAX_LEVEL).area;
        let r = g.arts.bombs[0].radius;
        assert!((r - BAT_BLAST_RADIUS * area * HOST_BLAST).abs() < 1e-4);
    }

    #[test]
    fn black_mass_strikes_everything_in_view_and_waits_for_a_target() {
        let mut g = game_with(BLACK_MASS, 1);
        tick(&mut g, 1);
        assert!(g.arts.since_mass.is_infinite(), "nothing in view");
        let half = vec2(DEFAULT_VIEW_W, LOGICAL_H) * 0.5;
        g.enemies.spawn(KNIGHT, vec2(half.x - 4.0, 0.0), 100.0);
        g.enemies.spawn(KNIGHT, vec2(0.0, -half.y + 4.0), 100.0);
        g.enemies
            .spawn(KNIGHT, vec2(half.x + MASS_MARGIN + 40.0, 0.0), 100.0);
        g.enemies.spawn(RAT, vec2(30.0, 0.0), 1.0);
        tick(&mut g, 1);
        assert!(g.cues.contains(&Cue::BlackMass));
        let full = ENEMY_DEFS[KNIGHT].hp * 100.0;
        let dmg = ARTS[BLACK_MASS].base.damage;
        assert_eq!(g.enemies.hp[0], full - dmg);
        assert_eq!(g.enemies.hp[1], full - dmg);
        assert_eq!(g.enemies.hp[2], full, "out of view");
        assert_eq!(g.kills, 1, "the rat could not take it");
    }

    #[test]
    fn witches_sabbath_comes_sooner_and_calls_every_soul() {
        let mut g = evolved(BLACK_MASS);
        let top = ARTS[BLACK_MASS].stats(ART_MAX_LEVEL).cooldown;
        assert!((g.arts.stats[BLACK_MASS].cooldown - top * SABBATH_COOLDOWN).abs() < 1e-4);
        g.souls.push(crate::game::Soul {
            pos: vec2(600.0, 0.0),
            xp: 3,
            called: false,
        });
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert!(g.souls[0].called);
        let xp = g.xp;
        tick(&mut g, (600.0 / SABBATH_SOUL_SPEED / TICK_DT) as usize + 4);
        assert!(g.souls.is_empty() && g.xp > xp, "flew all the way to her");

        // Without the evolution, far souls stay where they are.
        let mut g = game_with(BLACK_MASS, ART_MAX_LEVEL);
        g.souls.push(crate::game::Soul {
            pos: vec2(600.0, 0.0),
            xp: 3,
            called: false,
        });
        g.enemies.spawn(KNIGHT, vec2(100.0, 0.0), 100.0);
        tick(&mut g, 60);
        assert_eq!(g.souls[0].pos, vec2(600.0, 0.0));
    }

    #[test]
    fn gaze_freezes_what_it_touches_harmless_until_it_thaws() {
        let mut g = game_with(GAZE_OF_DESIRE, 1);
        g.player.iframes = 0.0;
        // One in the beam right next to her, one off it.
        g.enemies.spawn(KNIGHT, vec2(8.0, 0.0), 100.0);
        g.enemies.spawn(KNIGHT, vec2(0.0, 60.0), 100.0);
        tick(&mut g, 1);
        assert!(g.cues.contains(&Cue::Gaze));
        assert_eq!(g.arts.gazes.len(), 1);
        assert!(g.enemies.frozen(0, g.time) && !g.enemies.frozen(1, g.time));
        assert!(!hurt(&g, 0), "the gaze does no damage");
        g.arts.hold();
        let at = g.enemies.pos(0);
        tick(&mut g, 20);
        assert_eq!(g.enemies.pos(0), at, "frozen in place");
        assert_eq!(g.player.hp, PLAYER_MAX_HP, "and harmless on top of her");
        tick(
            &mut g,
            (ARTS[GAZE_OF_DESIRE].base.duration / TICK_DT) as usize,
        );
        assert!(!g.enemies.frozen(0, g.time));
        assert!(g.player.hp < PLAYER_MAX_HP, "thawed, it bites");
    }

    #[test]
    fn evil_eye_frozen_foes_take_more_damage() {
        let mut g = evolved(GAZE_OF_DESIRE);
        assert_eq!(
            g.arts.stats[GAZE_OF_DESIRE].amount,
            ARTS[GAZE_OF_DESIRE].stats(ART_MAX_LEVEL).amount + EVIL_EYE_EXTRA
        );
        g.enemies.spawn(KNIGHT, vec2(40.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert!(g.enemies.frozen(0, g.time));
        let mut hits = [false; 2];
        let before = g.enemies.hp[0];
        strike(&mut g.enemies, 0, 10.0, g.time, &mut hits);
        assert!((before - g.enemies.hp[0] - 10.0 * (1.0 + EVIL_EYE_VULN)).abs() < 1e-3);
        // Unevolved, frozen foes take the plain damage.
        let mut g = game_with(GAZE_OF_DESIRE, ART_MAX_LEVEL);
        g.enemies.spawn(KNIGHT, vec2(40.0, 0.0), 100.0);
        tick(&mut g, 1);
        let before = g.enemies.hp[0];
        strike(&mut g.enemies, 0, 10.0, g.time, &mut hits);
        assert_eq!(before - g.enemies.hp[0], 10.0);
    }

    #[test]
    fn thralls_rise_only_near_foes_hunt_them_and_can_die() {
        let mut g = game_with(THRALLS, 1);
        tick(&mut g, 60);
        assert!(g.arts.thralls.is_empty(), "nobody to fight");
        g.enemies.spawn(RAT, vec2(90.0, 0.0), 100.0);
        tick(&mut g, 1);
        assert_eq!(g.arts.thralls.len(), 1);
        assert!(g.cues.contains(&Cue::Thralls));
        let th = &g.arts.thralls[0];
        assert!(th.pos.distance(g.player.pos) <= THRALL_SPAWN_RADIUS + 3.0);
        assert!(th.alpha() < 0.1, "it rises");
        g.arts.hold();
        tick(&mut g, 90);
        assert!(
            g.enemies.hp[0] < ENEMY_DEFS[RAT].hp * 100.0,
            "it reached the rat"
        );
        // The rat bites back until the thrall falls; a thrall gives no soul.
        tick(&mut g, 600);
        assert!(g.arts.thralls.is_empty());
        assert_eq!(g.kills, 0);
        assert!(g.souls.is_empty());
    }

    #[test]
    fn thralls_level_table() {
        let top = ARTS[THRALLS].stats(ART_MAX_LEVEL);
        assert_eq!(top.amount, 4);
        assert_eq!(top.damage, 20.0);
        assert_eq!(top.duration, 12.0);
        assert!(
            ARTS[THRALLS].evolution.is_none(),
            "it grows only by the union"
        );
    }

    /// Allure and Thralls at the top, fused into the Infernal Court.
    fn court() -> Game {
        let mut g = game_with(ALLURE, ART_MAX_LEVEL);
        g.upgrades.arts[THRALLS] = ART_MAX_LEVEL;
        g.upgrades.united[INFERNAL_COURT] = true;
        g.arts.refresh(&g.upgrades);
        g
    }

    #[test]
    fn infernal_court_turns_the_charmed_into_thralls_that_fall_with_their_soul() {
        let mut g = court();
        g.enemies.spawn(KNIGHT, vec2(20.0, 0.0), 1.0);
        g.enemies.spawn(KNIGHT, vec2(-20.0, 0.0), 1.0);
        g.enemies.elite[1] = true;
        tick(&mut g, 1);
        assert_eq!(g.enemies.len(), 1, "the knight left the horde");
        assert!(
            g.enemies.elite[0] && g.enemies.charmed(0, g.time),
            "elites are only charmed"
        );
        assert_eq!(g.kills, 0, "not a kill yet");
        let knights = |g: &Game| {
            let k = Some(KNIGHT as u8);
            g.arts.thralls.iter().filter(|t| t.kind == k).count()
        };
        assert_eq!(knights(&g), 1);
        g.enemies.swap_remove(0);
        g.arts.hold();
        let life = g.arts.stats[ALLURE].duration * COURT_LIFE;
        tick(&mut g, (life / TICK_DT) as usize + 4);
        assert_eq!(knights(&g), 0, "its time ran out");
        assert_eq!(g.kills, 1, "it fell");
        // Its soul dropped where it fell (she may have picked it up already).
        assert!(g.souls.len() == 1 || g.xp == ENEMY_DEFS[KNIGHT].soul_xp);
        assert!(!g.cues.contains(&Cue::SirenBurst));

        // With Siren's Dominion it bursts as its time runs out.
        let mut g = court();
        g.upgrades.evolve(ALLURE);
        g.arts.refresh(&g.upgrades);
        g.enemies.spawn(KNIGHT, vec2(20.0, 0.0), 1.0);
        tick(&mut g, 1);
        g.arts.hold();
        tick(&mut g, (life / TICK_DT) as usize + 4);
        assert!(g.cues.contains(&Cue::SirenBurst));
        assert_eq!(g.kills, 1);
    }
}
