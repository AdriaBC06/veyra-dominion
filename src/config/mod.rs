//! All tunable numbers and data tables live here.

mod art_tuning;
mod arts;
mod bosses;
mod enemies;
mod map;
mod village;

pub use art_tuning::*;
pub use arts::*;
pub use bosses::*;
pub use enemies::*;
pub use map::*;
pub use village::*;

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

/// UI blood (design.md 7.14): frames and buttons gather blood as the run's kills climb,
/// `1 - e^(-kills / UI_BLOOD_KILLS)` of the full amount.
pub const UI_BLOOD_KILLS: f32 = 900.0;
/// Blood setting (V19g): share of the blood shown on Reduced (UI blood, stained titles, gore
/// bits, pools on the ground, which are also smaller).
pub const REDUCED_BLOOD: f32 = 0.4;
/// Pools on Reduced are this much of their size.
pub const REDUCED_POOL: f32 = 0.6;
/// Blood the setup's Blood page shows for each mode, so the choice can be seen.
pub const SETUP_BLOOD_PREVIEW: [f32; 3] = [0.85, 0.4, 0.0];
/// How fast the setup's preview blood follows the highlighted mode (per second).
pub const SETUP_BLOOD_RATE: f32 = 1.6;

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
/// A soul's look tells its worth: faint below `SOUL_FAINT_BELOW` XP, radiant from
/// `SOUL_RADIANT_FROM` (merged souls follow the same rule).
pub const SOUL_FAINT_BELOW: u32 = 3;
pub const SOUL_RADIANT_FROM: u32 = 10;

/// XP curve in steps (VS-like, docs/design.md 7.8): each level costs `XP_STEPS[i].1`
/// more than the one before from level `XP_STEPS[i].0` on, and the first level of a
/// step past the first is a wall (`XP_WALL` extra).
pub const XP_FIRST: u32 = 8;
pub const XP_STEPS: [(u32, u32); 3] = [(1, 7), (20, 10), (40, 14)];
pub const XP_WALL: [u32; 3] = [0, 200, 1000];

/// XP needed to go from `level` to `level + 1`.
pub const fn xp_to_next(level: u32) -> u32 {
    let mut need = XP_FIRST;
    let mut i = 0;
    while i < XP_STEPS.len() {
        let (from, step) = XP_STEPS[i];
        let to = if i + 1 < XP_STEPS.len() {
            XP_STEPS[i + 1].0
        } else {
            u32::MAX
        };
        if level > from {
            need += step * ((if level < to { level } else { to }) - from);
        }
        if level == from {
            need += XP_WALL[i];
        }
        i += 1;
    }
    need
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

// --- Spatial hash ---------------------------------------------------------
/// The grid window (cols x rows cells) must cover the relocate radius of the widest view.
pub const HASH_CELL: f32 = 64.0;
pub const HASH_COLS: usize = 24;
pub const HASH_ROWS: usize = 24;
