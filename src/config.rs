//! All tunable numbers and data tables live here.

use crate::sprites::SpriteId;

pub const GAME_TITLE: &str = "Veyra: Dominion";

/// Fixed logical resolution; the frame is scaled to the window with nearest filtering.
pub const LOGICAL_W: f32 = 640.0;
pub const LOGICAL_H: f32 = 360.0;

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
/// Speed above which the "impulse" body frame is shown.
pub const IMPULSE_SPEED: f32 = 40.0;
/// Hover height of the sprite's feet above its shadow, in pixels.
pub const HOVER_HEIGHT: f32 = 6.0;
pub const BOB_AMPLITUDE: f32 = 2.0;
pub const BOB_HZ: f32 = 1.5;
/// How much the shadow shrinks at the top of the bob (fraction of its size).
pub const SHADOW_SHRINK: f32 = 0.15;
pub const WING_FPS: f32 = 3.0;
/// Tail offset per unit of velocity, and how fast it catches up.
pub const TAIL_LAG_SCALE: f32 = 0.012;
pub const TAIL_SMOOTHING: f32 = 6.0;

// --- Weapon: Hellflare ---------------------------------------------------
pub const FIRE_DAMAGE: f32 = 10.0;
pub const FIRE_INTERVAL: f32 = 0.6;
/// Only enemies within this distance are targeted.
pub const FIRE_RANGE: f32 = 340.0;
pub const PROJ_SPEED: f32 = 320.0;
pub const PROJ_LIFETIME: f32 = 1.4;
pub const PROJ_RADIUS: f32 = 4.0;
/// Fireballs are launched from Veyra's hands, this high above her ground point.
pub const PROJ_DRAW_HEIGHT: f32 = 18.0;
pub const MAX_PROJECTILES: usize = 256;
pub const HIT_FLASH_TIME: f32 = 0.1;
/// Dead enemies fade out as a white silhouette over this time.
pub const DEATH_FADE_TIME: f32 = 0.35;
pub const MAX_FADES: usize = 512;

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

// --- Upgrades -------------------------------------------------------------
/// Spread between Hellflare fireballs when EmberLegion adds more of them.
pub const PROJ_SPREAD: f32 = 0.16;
/// Health restored when BloodPact is taken (clamped to the new maximum).
pub const BLOOD_PACT_HEAL: f32 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UpgradeKind {
    Fury,
    Frenzy,
    EmberLegion,
    LightWings,
    BloodPact,
    SoulPull,
}

/// `step` is the per-level bonus, see `Upgrades::stats`.
pub struct UpgradeDef {
    pub kind: UpgradeKind,
    pub name: &'static str,
    pub desc: &'static str,
    pub max_level: u8,
    pub step: f32,
}

pub const UPGRADES: [UpgradeDef; 6] = [
    UpgradeDef {
        kind: UpgradeKind::Fury,
        name: "Fury",
        desc: "+25% fireball damage",
        max_level: 5,
        step: 0.25,
    },
    UpgradeDef {
        kind: UpgradeKind::Frenzy,
        name: "Frenzy",
        desc: "+15% fire rate",
        max_level: 5,
        step: 0.15,
    },
    UpgradeDef {
        kind: UpgradeKind::EmberLegion,
        name: "Ember Legion",
        desc: "+1 fireball per volley",
        max_level: 4,
        step: 1.0,
    },
    UpgradeDef {
        kind: UpgradeKind::LightWings,
        name: "Light Wings",
        desc: "+10% move speed",
        max_level: 5,
        step: 0.10,
    },
    UpgradeDef {
        kind: UpgradeKind::BloodPact,
        name: "Blood Pact",
        desc: "+20 max HP, heal 30",
        max_level: 5,
        step: 20.0,
    },
    UpgradeDef {
        kind: UpgradeKind::SoulPull,
        name: "Soul Pull",
        desc: "+40% soul pickup radius",
        max_level: 5,
        step: 0.40,
    },
];

// --- Enemies & spawning ---------------------------------------------------
pub const MAX_ENEMIES: usize = 2500;
/// Enemies spawn on this ring around the player, just outside the visible frame.
pub const SPAWN_RING_RADIUS: f32 = 400.0;
/// Enemies farther than this are moved back to the ring in front of the player.
pub const RELOCATE_RADIUS: f32 = 600.0;
/// Enemies per second, ramping linearly from start to end over SPAWN_RAMP_SECS.
pub const SPAWN_RATE_START: f32 = 1.0;
pub const SPAWN_RATE_END: f32 = 25.0;
pub const SPAWN_RAMP_SECS: f32 = 300.0;
/// Members of a spawn group appear scattered this far around the group's ring point.
pub const SPAWN_GROUP_SPREAD: f32 = 14.0;
/// Soft separation: neighbours closer than this push each other apart.
/// Largest `EnemyDef::radius`, used to size collision queries.
pub const MAX_ENEMY_RADIUS: f32 = 7.0;
pub const SEPARATION_RADIUS: f32 = 12.0;
pub const SEPARATION_PUSH: f32 = 60.0;
/// Only this many neighbours contribute to separation (bounds the cost in dense hordes).
pub const SEPARATION_MAX_NEIGHBOURS: u32 = 6;
pub const ENEMY_ANIM_FPS: f32 = 6.0;

// --- Spatial hash ---------------------------------------------------------
/// The grid window (cols x rows cells) must cover RELOCATE_RADIUS around the player.
pub const HASH_CELL: f32 = 64.0;
pub const HASH_COLS: usize = 24;
pub const HASH_ROWS: usize = 20;

pub enum Behavior {
    Chase,
}

pub struct EnemyDef {
    pub name: &'static str,
    pub sprite: SpriteId,
    pub hp: f32,
    pub speed: f32,
    pub radius: f32,
    pub contact_damage: f32,
    pub soul_xp: u32,
    /// Spawn weight once unlocked: `weight_start + weight_growth_per_min * minutes`.
    pub weight_start: f32,
    pub weight_growth_per_min: f32,
    /// Inclusive range of enemies spawned together.
    pub group: (u8, u8),
    pub behavior: Behavior,
}

pub const RAT: usize = 0;
pub const VILLAGER: usize = 1;
pub const KNIGHT: usize = 2;

pub const ENEMY_DEFS: [EnemyDef; 3] = [
    EnemyDef {
        name: "Rat",
        sprite: SpriteId::Rat,
        hp: 6.0,
        speed: 75.0,
        radius: 5.0,
        contact_damage: 5.0,
        soul_xp: 1,
        weight_start: 60.0,
        weight_growth_per_min: -4.0,
        group: (2, 4),
        behavior: Behavior::Chase,
    },
    EnemyDef {
        name: "Villager",
        sprite: SpriteId::Villager,
        hp: 14.0,
        speed: 55.0,
        radius: 6.0,
        contact_damage: 8.0,
        soul_xp: 2,
        weight_start: 40.0,
        weight_growth_per_min: 4.0,
        group: (1, 2),
        behavior: Behavior::Chase,
    },
    EnemyDef {
        name: "Knight",
        sprite: SpriteId::Knight,
        hp: 45.0,
        speed: 42.0,
        radius: 7.0,
        contact_damage: 15.0,
        soul_xp: 5,
        weight_start: 0.0,
        weight_growth_per_min: 12.0,
        group: (1, 1),
        behavior: Behavior::Chase,
    },
];

/// Run length for victory, in seconds.
pub const VICTORY_TIME: f32 = 300.0;

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
        time: VICTORY_TIME,
        kind: EventKind::Victory,
    },
];
