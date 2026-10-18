//! All tunable numbers and data tables live here.

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
