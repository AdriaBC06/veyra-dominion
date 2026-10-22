//! All tunable numbers and data tables live here.

mod art_tuning;
mod arts;

pub use art_tuning::*;
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
