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

// --- Enemies & spawning ---------------------------------------------------
pub const MAX_ENEMIES: usize = 2500;
/// Enemies spawn on this ring around the player, just outside the visible frame.
pub const SPAWN_RING_RADIUS: f32 = 400.0;
/// Enemies farther than this are moved back to the ring in front of the player.
pub const RELOCATE_RADIUS: f32 = 600.0;
pub const SPAWN_RATE_START: f32 = 1.0;
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
    pub sprite: SpriteId,
    pub hp: f32,
    pub speed: f32,
    pub radius: f32,
    pub behavior: Behavior,
}

pub const RAT: usize = 0;
pub const VILLAGER: usize = 1;

pub const ENEMY_DEFS: [EnemyDef; 3] = [
    EnemyDef {
        sprite: SpriteId::Rat,
        hp: 6.0,
        speed: 75.0,
        radius: 5.0,
        behavior: Behavior::Chase,
    },
    EnemyDef {
        sprite: SpriteId::Villager,
        hp: 14.0,
        speed: 55.0,
        radius: 6.0,
        behavior: Behavior::Chase,
    },
    EnemyDef {
        sprite: SpriteId::Knight,
        hp: 45.0,
        speed: 42.0,
        radius: 7.0,
        behavior: Behavior::Chase,
    },
];
