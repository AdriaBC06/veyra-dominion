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
