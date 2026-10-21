//! All tunable numbers and data tables live here.

mod arts;

pub use arts::*;

use crate::assets::SpriteId;
use crate::i18n::Key;

pub const GAME_TITLE: &str = "Veyra: Dominion";

/// Logical height is fixed; the logical width follows the window aspect ratio
/// (4:3 up to 21:9) and the frame is scaled to the window with nearest filtering.
pub const LOGICAL_H: f32 = 360.0;
pub const MIN_LOGICAL_W: f32 = 480.0;
pub const MAX_LOGICAL_W: f32 = 864.0;
/// View size used by headless runs (16:9).
pub const DEFAULT_VIEW_W: f32 = 640.0;

pub const WINDOW_W: i32 = 1280;
pub const WINDOW_H: i32 = 720;

/// Fixed simulation step (60 Hz).
pub const TICK_DT: f32 = 1.0 / 60.0;
/// Longest frame time fed to the accumulator, avoids the spiral of death.
pub const MAX_FRAME_DT: f32 = 0.25;
/// Maximum simulation steps run in a single rendered frame.
pub const MAX_TICKS_PER_FRAME: u32 = 8;

// --- Player ---------------------------------------------------------------
pub const PLAYER_MAX_HP: f32 = 100.0;
pub const PLAYER_IFRAMES: f32 = 0.5;
/// Collision radius of Veyra's ground point against enemies.
pub const PLAYER_RADIUS: f32 = 6.0;
/// While invulnerable the sprite toggles visibility with this period.
pub const BLINK_PERIOD: f32 = 0.06;
/// Time from the killing blow to the game over screen.
pub const DEATH_TIME: f32 = 1.2;
pub const DEATH_SMOKE_PUFFS: usize = 12;
pub const SMOKE_RISE_SPEED: f32 = 22.0;
pub const PLAYER_SPEED: f32 = 180.0;
/// Exponential approach rate of velocity toward the input direction (floaty feel).
pub const PLAYER_ACCEL: f32 = 14.0;
/// Speed above which the move animation is shown.
pub const IMPULSE_SPEED: f32 = 40.0;
/// Hover height of the lowest pixel of the sprite above its shadow, in pixels.
pub const HOVER_HEIGHT: f32 = 6.0;
/// Veyra is drawn this much lower than a figure standing on her ground point would be,
/// so her body (not her feet) sits on her hitbox; her shadow falls this far below it.
pub const VEYRA_DROP: f32 = 29.0;
pub const BOB_AMPLITUDE: f32 = 2.0;
pub const BOB_HZ: f32 = 1.5;
/// How much the shadow shrinks at the top of the bob (fraction of its size).
pub const SHADOW_SHRINK: f32 = 0.15;
/// Half size of Veyra's shadow at rest.
pub const VEYRA_SHADOW: (f32, f32) = (15.0, 4.0);
/// Tail offset per unit of velocity, and how fast it catches up.
pub const TAIL_LAG_SCALE: f32 = 0.012;
pub const TAIL_SMOOTHING: f32 = 6.0;
/// The tail and hair layers drift at most this far from the body, so they stay attached;
/// the hair follows the tail's lag scaled down.
pub const TAIL_LAG_MAX: f32 = 2.0;
pub const HAIR_LAG_MAX: f32 = 1.0;

// --- Arts (weapons), see `ARTS` -------------------------------------------
/// Hellflare only targets enemies within this distance.
pub const FIRE_RANGE: f32 = 340.0;
pub const PROJ_RADIUS: f32 = 4.0;
/// Fireballs are launched from Veyra's hands, this high above her ground point.
pub const PROJ_DRAW_HEIGHT: f32 = 18.0;
pub const MAX_PROJECTILES: usize = 256;
/// Spread between Hellflare fireballs of one volley.
pub const PROJ_SPREAD: f32 = 0.16;
/// Tail Lash hits a box on one side of Veyra: it starts `LASH_NEAR` from her ground
/// point, reaches `LASH_REACH` (times area) out and covers `LASH_DEPTH` of ground depth.
pub const LASH_NEAR: f32 = 4.0;
pub const LASH_REACH: f32 = 52.0;
pub const LASH_DEPTH: f32 = 24.0;
/// Extra lashes of one attack follow this long after each other, alternating sides.
pub const LASH_DELAY: f32 = 0.12;
/// How long a lash stays on screen (its 3-frame strip).
pub const LASH_SHOW_TIME: f32 = 0.19;
/// Lashes queued or on screen at once.
pub const MAX_LASHES: usize = 16;
/// Kiss of Ruin hearts fan out like the fireballs.
pub const BOLT_RADIUS: f32 = 4.0;
/// Hearts fly at this height above the ground, like the fireballs.
pub const BOLT_DRAW_HEIGHT: f32 = 16.0;
pub const MAX_BOLTS: usize = 128;
/// A heart remembers this many enemies it went through (more pierce than this
/// just stops counting).
pub const BOLT_MAX_HITS: usize = 8;
/// Brimstone Halo: flames circle at this radius (times the Art's area), at height
/// `EMBER_DRAW_HEIGHT`, and burn an enemy at most once per `HALO_IMMUNE` seconds,
/// pushing it `HALO_KNOCKBACK` pixels outwards.
pub const HALO_RADIUS: f32 = 38.0;
pub const EMBER_RADIUS: f32 = 8.0;
pub const EMBER_DRAW_HEIGHT: f32 = 12.0;
pub const HALO_IMMUNE: f32 = 0.6;
pub const HALO_KNOCKBACK: f32 = 6.0;
/// Embers fade in and out over this long at the ends of each burning spell.
pub const HALO_FADE: f32 = 0.2;
/// Allure: each pulse burns every enemy within `ALLURE_RADIUS` (times the Art's area)
/// and charms up to `amount` of them for the Art's duration. Charmed enemies hunt the
/// nearest free enemy within `CHARM_SEEK`, bite the free ones they touch every
/// `CHARM_BITE` seconds for the Art's damage, and never hurt Veyra.
pub const ALLURE_RADIUS: f32 = 56.0;
pub const CHARM_SEEK: f32 = 110.0;
pub const CHARM_BITE: f32 = 0.5;
/// A charmed enemy bites free enemies up to this far beyond its body (separation keeps
/// bodies a little apart).
pub const CHARM_REACH: f32 = 4.0;
/// The pulse's ring of hearts takes this long to reach its edge, then fades as long.
pub const ALLURE_WAVE_TIME: f32 = 0.35;
/// Hearts on the pulse ring, and how high the charm mark floats over a head.
pub const ALLURE_WAVE_HEARTS: usize = 14;
pub const CHARM_MARK_HEIGHT: f32 = 6.0;
/// Charmed enemies are drawn tinted toward this colour.
pub const CHARM_TINT: u32 = 0xff9ac8;
/// Hellgate pools: ellipses `HELLGATE_RADIUS` (times area) wide and half as deep, centred on a
/// random enemy within `HELLGATE_RANGE`. Everything inside burns every `HELLGATE_TICK`.
pub const HELLGATE_RADIUS: f32 = 16.0;
pub const HELLGATE_RANGE: f32 = 200.0;
pub const HELLGATE_TICK: f32 = 0.35;
/// Pools fade in and out over this long at the ends of their life.
pub const HELLGATE_FADE: f32 = 0.2;
pub const MAX_GATES: usize = 32;
/// A pool is drawn as scorched ground in `GATE_CELL` px cells: char, scorch, faint
/// edge and ember (RGB); one cell in `GATE_EMBER_ODDS` of the char glows. One small
/// flame burns per `GATE_FLAME_AREA` square pixels, between `GATE_MIN_FLAMES` and
/// `GATE_MAX_FLAMES`.
pub const GATE_COLORS: [u32; 4] = [0x1c0a0a, 0x3a1810, 0x5a3420, 0xd04818];
pub const GATE_CELL: f32 = 2.0;
pub const GATE_EMBER_ODDS: u32 = 9;
pub const GATE_FLAME_AREA: f32 = 70.0;
pub const GATE_MIN_FLAMES: usize = 4;
pub const GATE_MAX_FLAMES: usize = 40;
/// Chain of Torment: it bites a random enemy within `CHAIN_RANGE`, then jumps to the
/// nearest enemy it hasn't bitten within `CHAIN_JUMP` (times area), `amount` in all.
pub const CHAIN_RANGE: f32 = 180.0;
pub const CHAIN_JUMP: f32 = 64.0;
pub const MAX_CHAIN_LINKS: usize = 24;
pub const MAX_CHAINS: usize = 8;
/// How long a chain stays on screen, and the height of its links over the ground.
pub const CHAIN_SHOW_TIME: f32 = 0.2;
pub const CHAIN_DRAW_HEIGHT: f32 = 9.0;
/// The drawn chain kinks every this many pixels.
pub const CHAIN_KINK: f32 = 8.0;
/// Chain colours: dark glow, body, white-hot core.
pub const CHAIN_COLORS: [u32; 3] = [0x5c0c1c, 0xe04454, 0xffd0dc];
/// Scent of Sin: a haze `SCENT_RADIUS` (times area) around her. Every burn (its
/// cooldown) it hurts everything inside and weakens it for the Art's duration:
/// weakened enemies take `SCENT_WEAKEN` more damage from every Art.
pub const SCENT_RADIUS: f32 = 40.0;
pub const SCENT_WEAKEN: f32 = 0.25;
/// Weakened enemies are drawn tinted toward this colour.
pub const WEAK_TINT: u32 = 0xe6c8ff;
/// Motes drifting up through the haze, and its colour (RGB).
pub const SCENT_MOTES: usize = 18;
pub const SCENT_COLOR: u32 = 0xff4fb0;

// --- Evolutions (7.7), opened by Abyssal Offerings --------------------------
/// Kiss of Ruin (Hellflare evolved): each volley throws `KISS_EXTRA` more shots, as
/// hearts that fly at `KISS_SPEED` for `KISS_LIFE` and go through `KISS_PIERCE` enemies.
pub const KISS_EXTRA: u32 = 2;
pub const KISS_SPEED: f32 = 380.0;
pub const KISS_LIFE: f32 = 1.0;
pub const KISS_PIERCE: u32 = 2;
/// Barbed Scourge: a lashed enemy bleeds for the Lash's duration, taking
/// `BLEED_SHARE` of its damage every `BLEED_TICK`.
pub const BLEED_TICK: f32 = 0.5;
pub const BLEED_SHARE: f32 = 0.25;
/// Chance per tick that a bleeding enemy with blood drips on the ground.
pub const BLEED_DRIP: f32 = 0.05;
/// Infernal Crown: the flames never rest; one more joins them.
pub const CROWN_EXTRA: u32 = 1;
/// Siren's Dominion: when a charm runs out the enemy bursts, hurting the free enemies
/// within `SIREN_BURST_RADIUS` (times area) for `SIREN_BURST_DAMAGE` times the Art's
/// damage. It never kills: the enemy that bursts lives on, and the burst leaves
/// everyone it hurts at `SIREN_SPARE_HP` at least. It shows for `SIREN_BURST_TIME`.
pub const SIREN_BURST_RADIUS: f32 = 36.0;
pub const SIREN_BURST_DAMAGE: f32 = 1.0;
pub const SIREN_SPARE_HP: f32 = 1.0;
pub const SIREN_BURST_TIME: f32 = 0.35;
pub const SIREN_BURST_HEARTS: usize = 8;
pub const MAX_SIREN_BURSTS: usize = 32;
/// Abyssal Rift: enemies within `RIFT_RADIUS` (times area) of a pool are dragged to its
/// centre at `RIFT_PULL` px/s.
pub const RIFT_RADIUS: f32 = 56.0;
pub const RIFT_PULL: f32 = 70.0;
/// Torment Eternal: the chain keeps jumping, up to `MAX_CHAIN_LINKS`, to any enemy
/// within `TORMENT_RADIUS` (times area) of the first one it bit.
pub const TORMENT_RADIUS: f32 = 96.0;
/// Devouring Musk: every enemy the haze burns gives her `MUSK_HEAL` health, at most
/// `MUSK_HEAL_CAP` per second.
pub const MUSK_HEAL: f32 = 0.5;
pub const MUSK_HEAL_CAP: f32 = 3.0;
/// Abyssal Offerings: dropped by elites, opened when Veyra gets this close. One with
/// nothing to evolve or level up heals `OFFERING_HEAL` instead. Sacrificed (rejected),
/// an Offering heals `SACRIFICE_HEAL` and gives `SACRIFICE_XP` souls' worth of XP.
pub const OFFERING_PICKUP: f32 = 12.0;
pub const MAX_OFFERINGS: usize = 16;
pub const OFFERING_HEAL: f32 = 50.0;
pub const SACRIFICE_HEAL: f32 = 20.0;
pub const SACRIFICE_XP: u32 = 25;

pub const HIT_FLASH_TIME: f32 = 0.1;
/// Dead enemies flash as a white silhouette for this long, then burst.
pub const DEATH_FADE_TIME: f32 = 0.12;
pub const MAX_FADES: usize = 512;
/// Death burst: bits in the enemy's palette leave the body at a random height, fall to
/// the ground and fade there. `burst_count` picks 6..=10 per enemy by sprite area.
pub const MAX_BURST_PIXELS: usize = 1024;
/// Side of a burst bit in pixels; single pixels get lost in the grass speckle.
pub const BURST_BIT: f32 = 2.0;
pub const BURST_SPEED: (f32, f32) = (12.0, 38.0);
/// Blood drops (enemies with `EnemyDef::blood`): a dark, brownish red, well apart from
/// Veyra's bright crimsons. Heavier than the other bits, they jump lower.
pub const BLOOD: [u32; 2] = [0x4a0c10, 0x6a1416];
pub const BLOOD_JUMP_SCALE: f32 = 0.6;
/// Blood on the ground is permanent: every bleeding death leaves a pool sized by the
/// enemy (`pool_radius`) and every drop a small splash where it lands. `blood::BloodLayer`
/// bakes them into ground textures, so their number costs nothing to draw.
/// Pool colours: dark rim, body, wet glint.
pub const POOL_COLORS: [u32; 3] = [0x22060a, 0x3e0a0e, 0x601618];
/// Pool radius per pixel of the enemy's collision radius.
pub const POOL_RADIUS_SCALE: f32 = 0.9;
/// A pool landing inside a recent one feeds it instead (the area adds up, same shape,
/// larger) up to this radius; past it, deaths there only splash around. The last
/// `MAX_POOL_SITES` pools are remembered for this.
pub const POOL_MAX_RADIUS: f32 = 18.0;
pub const MAX_POOL_SITES: usize = 512;
/// Splashes thrown around a pool that is already at its largest.
pub const FULL_POOL_SPLASHES: usize = 3;
/// Splats the simulation can queue between two frames; the renderer drains them.
pub const MAX_SPLATS: usize = 512;
/// The blood layer is cut into square chunks of world pixels; past `MAX_BLOOD_CHUNKS`
/// the chunk seen longest ago is dropped (only far away, long-left ground loses blood).
pub const BLOOD_CHUNK: u16 = 128;
pub const MAX_BLOOD_CHUNKS: usize = 256;
/// Thrown this much faster away from Veyra.
pub const BURST_AWAY: f32 = 10.0;
pub const BURST_JUMP: (f32, f32) = (55.0, 105.0);
pub const BURST_GRAVITY: f32 = 340.0;
/// Seconds a bit lies on the ground, fading all along.
pub const BURST_REST_TIME: f32 = 0.5;
/// Elite outline colour: cold, so it never competes with Veyra's warm palette.
pub const ELITE_OUTLINE: u32 = 0xa8e4ff;
/// Debug (F5): enemies this close to Veyra become elites.
pub const DEBUG_ELITE_RADIUS: f32 = 90.0;

// --- Souls & XP -----------------------------------------------------------
/// Souls within this distance fly toward Veyra.
pub const PICKUP_RADIUS: f32 = 48.0;
pub const SOUL_MAGNET_SPEED: f32 = 240.0;
/// Souls this close are absorbed.
pub const SOUL_COLLECT_RADIUS: f32 = 6.0;
/// When full, a new soul's XP merges into the nearest existing one.
pub const MAX_SOULS: usize = 600;
pub const SOUL_BOB_HZ: f32 = 2.0;

/// XP needed to go from `level` to `level + 1`.
pub const fn xp_to_next(level: u32) -> u32 {
    8 + 6 * level
}

// --- Sins -----------------------------------------------------------------
/// Sins owned at once, like the Arts' slots.
pub const MAX_SINS: usize = 6;
/// Contact damage left after armour never drops below this.
pub const MIN_CONTACT_DAMAGE: f32 = 1.0;
/// Gluttony: health regenerated per second per level (on top of its max HP).
pub const GLUTTONY_REGEN: f32 = 0.2;
/// Pride: each level adds its `step` to the spawn rate and to new enemies' health, and
/// this share of it to their speed.
pub const PRIDE_SPEED_SHARE: f32 = 0.5;

/// A passive (docs/design.md 7.7): the seven deadly sins plus infernal extras. `step`
/// is the per-level bonus, see `Upgrades::stats`. Wrath, Bloodlust, Legion, Sloth and
/// Dark Sigil boost every Art. Greed joins with Tribute in V20.
pub struct SinDef {
    pub name: Key,
    pub desc: Key,
    pub max_level: u8,
    pub step: f32,
}

pub const WRATH: usize = 0;
pub const BLOODLUST: usize = 1;
pub const LEGION: usize = 2;
pub const NIGHT_WINGS: usize = 3;
pub const GLUTTONY: usize = 4;
pub const LUST: usize = 5;
pub const SLOTH: usize = 6;
pub const DARK_SIGIL: usize = 7;
pub const INFERNAL_HIDE: usize = 8;
pub const ENVY: usize = 9;
pub const PRIDE: usize = 10;

macro_rules! sin {
    ($name:ident, $desc:ident, $max:expr, $step:expr) => {
        SinDef {
            name: Key::$name,
            desc: Key::$desc,
            max_level: $max,
            step: $step,
        }
    };
}

pub const SINS: [SinDef; 11] = [
    sin!(SinWrath, SinWrathDesc, 5, 0.10),
    sin!(SinBloodlust, SinBloodlustDesc, 5, 0.08),
    sin!(SinLegion, SinLegionDesc, 2, 1.0),
    sin!(SinNightWings, SinNightWingsDesc, 5, 0.10),
    sin!(SinGluttony, SinGluttonyDesc, 5, 15.0),
    sin!(SinLust, SinLustDesc, 5, 0.40),
    sin!(SinSloth, SinSlothDesc, 5, 0.10),
    sin!(SinDarkSigil, SinDarkSigilDesc, 5, 0.10),
    sin!(SinInfernalHide, SinInfernalHideDesc, 5, 1.0),
    sin!(SinEnvy, SinEnvyDesc, 5, 0.10),
    sin!(SinPride, SinPrideDesc, 5, 0.10),
];

// --- Enemies & spawning ---------------------------------------------------
pub const MAX_ENEMIES: usize = 2500;
/// Enemies spawn on a ring this far beyond the view's half-diagonal, just off screen.
pub const SPAWN_RING_MARGIN: f32 = 32.0;
/// Enemies farther than the spawn ring plus this are moved back to the ring ahead.
pub const RELOCATE_MARGIN: f32 = 200.0;
/// Enemies per second, ramping linearly from start to end over SPAWN_RAMP_SECS.
pub const SPAWN_RATE_START: f32 = 1.0;
pub const SPAWN_RATE_END: f32 = 25.0;
pub const SPAWN_RAMP_SECS: f32 = 300.0;
/// Members of a spawn group appear scattered this far around the group's ring point.
pub const SPAWN_GROUP_SPREAD: f32 = 14.0;
/// Soft separation: neighbours closer than this push each other apart.
pub const SEPARATION_RADIUS: f32 = 12.0;
pub const SEPARATION_PUSH: f32 = 60.0;
/// Only this many neighbours contribute to separation (bounds the cost in dense hordes).
pub const SEPARATION_MAX_NEIGHBOURS: u32 = 6;
pub const ENEMY_ANIM_FPS: f32 = 6.0;

// --- Spatial hash ---------------------------------------------------------
/// The grid window (cols x rows cells) must cover the relocate radius of the widest view.
pub const HASH_CELL: f32 = 64.0;
pub const HASH_COLS: usize = 24;
pub const HASH_ROWS: usize = 24;

#[derive(Clone, Copy)]
pub enum Behavior {
    Chase,
}

/// Broad enemy kind, used to pick the kill sound.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Beast,
    Human,
    Armored,
}

#[derive(Clone, Copy)]
pub struct EnemyDef {
    pub name: &'static str,
    pub sprite: SpriteId,
    /// Frame size of the sprite strip in pixels, outline included (an `assets` test
    /// checks it against the PNG). Radius and burst size derive from it.
    pub size: (u8, u8),
    /// Death-burst colours (RGB): the light end of the sprite's own ramps, since the
    /// dark ones vanish on the grass.
    pub burst: [u32; 4],
    /// Bleeds on death: a pool where it fell, plus half as many blood drops in the
    /// burst. Off for anything without blood, such as skeletons or constructs.
    pub blood: bool,
    pub hp: f32,
    pub speed: f32,
    pub contact_damage: f32,
    pub soul_xp: u32,
    /// Spawn weight once unlocked: `weight_start + weight_growth_per_min * minutes`.
    pub weight_start: f32,
    pub weight_growth_per_min: f32,
    /// Inclusive range of enemies spawned together.
    pub group: (u8, u8),
    pub behavior: Behavior,
    pub family: Family,
}

impl EnemyDef {
    /// Collision radius on the ground: `1 + 0.14 * (w + h)`, rounded. Tracks the sprite's
    /// bulk (the rat's tail and the villager's spear count for little).
    pub const fn radius(&self) -> f32 {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        (1.5 + 0.14 * (w + h)) as u32 as f32
    }

    /// Radius of the blood pool it leaves, in pixels.
    pub const fn pool_radius(&self) -> f32 {
        self.radius() * POOL_RADIUS_SCALE
    }

    /// Pixels in the death burst: 6 for the smallest, up to 10 for the brute.
    pub const fn burst_count(&self) -> usize {
        let area = self.size.0 as usize * self.size.1 as usize;
        let n = 6 + area / 250;
        if n > 10 {
            10
        } else {
            n
        }
    }
}

pub const RAT: usize = 0;
pub const VILLAGER: usize = 1;
pub const KNIGHT: usize = 2;
pub const BRUTE: usize = 3;

pub const ENEMY_DEFS: [EnemyDef; 4] = [
    EnemyDef {
        name: "Rat",
        sprite: SpriteId::Rat,
        size: (17, 9),
        burst: [0xa6a4ae, 0x7d7b88, 0xa6a4ae, 0x9a808a],
        blood: true,
        hp: 6.0,
        speed: 75.0,
        contact_damage: 5.0,
        soul_xp: 1,
        weight_start: 60.0,
        weight_growth_per_min: -4.0,
        group: (2, 4),
        behavior: Behavior::Chase,
        family: Family::Beast,
    },
    EnemyDef {
        name: "Villager",
        sprite: SpriteId::Villager,
        size: (14, 20),
        burst: [0x5f7f88, 0xb8927c, 0x7e9ea6, 0x8a7458],
        blood: true,
        hp: 14.0,
        speed: 55.0,
        contact_damage: 8.0,
        soul_xp: 2,
        weight_start: 40.0,
        weight_growth_per_min: 4.0,
        group: (1, 2),
        behavior: Behavior::Chase,
        family: Family::Human,
    },
    EnemyDef {
        name: "Knight",
        sprite: SpriteId::Knight,
        size: (18, 26),
        burst: [0xc8d2da, 0x9aa6b0, 0x4660a0, 0xb8c4d4],
        blood: true,
        hp: 45.0,
        speed: 42.0,
        contact_damage: 15.0,
        soul_xp: 5,
        weight_start: 0.0,
        weight_growth_per_min: 12.0,
        group: (1, 1),
        behavior: Behavior::Chase,
        family: Family::Armored,
    },
    EnemyDef {
        name: "Brute",
        sprite: SpriteId::Brute,
        size: (32, 40),
        burst: [0x4660a0, 0xb2b0a4, 0xb8927c, 0x908e84],
        blood: true,
        hp: 140.0,
        speed: 32.0,
        contact_damage: 25.0,
        soul_xp: 15,
        weight_start: 0.0,
        weight_growth_per_min: 3.0,
        group: (1, 1),
        behavior: Behavior::Chase,
        family: Family::Human,
    },
];

/// Run length for victory, in seconds.
pub const VICTORY_TIME: f32 = 300.0;
/// In endless mode, enemy HP grows by this fraction per minute past VICTORY_TIME.
pub const ENDLESS_HP_GROWTH_PER_MIN: f32 = 0.5;

pub enum EventKind {
    /// Enemy kind (index into `ENEMY_DEFS`) starts spawning.
    Unlock(usize),
    Victory,
}

pub struct Event {
    pub time: f32,
    pub kind: EventKind,
}

/// Timeline beats, sorted by time.
pub const EVENTS: &[Event] = &[
    Event {
        time: 0.0,
        kind: EventKind::Unlock(RAT),
    },
    Event {
        time: 0.0,
        kind: EventKind::Unlock(VILLAGER),
    },
    Event {
        time: 120.0,
        kind: EventKind::Unlock(KNIGHT),
    },
    Event {
        time: 180.0,
        kind: EventKind::Unlock(BRUTE),
    },
    Event {
        time: VICTORY_TIME,
        kind: EventKind::Victory,
    },
];
