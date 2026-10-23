//! Bosses: phases, attack rotations and their numbers (docs/design.md 7.8, V19).

use super::enemies::{Formation, BARRICADER, CAPTAIN, KNIGHT, SHEPHERD, TORCHBEARER, VILLAGER};
use crate::i18n::Key;

/// Something a boss does between walks, in its phase's rotation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Attack {
    /// Plants himself (the telegraph), then rushes along a locked line; `chain` rushes in a row.
    Charge { chain: u8 },
    /// Lobs `n` torches: one at her, the rest scattered around her.
    Volley { n: u8 },
    /// Raises his shield: Hellflare's fireballs glance off the front; he only creeps.
    Guard,
    /// Blows the horn: these troops arrive from the spawn ring.
    Call(&'static [Muster]),
}

/// A group a boss calls in.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Muster {
    pub kind: usize,
    pub n: u16,
    pub formation: Formation,
}

const fn muster(kind: usize, n: u16, formation: Formation) -> Muster {
    Muster { kind, n, formation }
}

/// One phase: it starts when health falls to `below` of the maximum.
pub struct Phase {
    pub below: f32,
    /// Walking speed multiplier on the boss kind's speed.
    pub speed: f32,
    /// Charge telegraph length (seconds standing still before the rush).
    pub windup: f32,
    /// Seconds walking between two attacks.
    pub gap: f32,
    pub rotation: &'static [Attack],
    /// Called at once when the phase starts (the first phase: on arrival).
    pub rally: &'static [Muster],
}

pub struct BossDef {
    /// Its row in `ENEMY_DEFS`: the body the Arts hit (sprite, size, HP, contact damage).
    pub kind: usize,
    pub name: Key,
    pub phases: &'static [Phase],
    pub charge_speed: f32,
    pub charge_time: f32,
    pub charge_damage: f32,
    /// The rush keeps tracking her until this much of the wind-up is left, then locks.
    pub charge_lock: f32,
    /// Standing still after a rush (or between chained rushes, half of it).
    pub charge_rest: f32,
    /// Shield bash: when she is within `bash_range`, he winds up for `bash_windup` and hits
    /// a circle of `bash_radius` in front of him; not again for `bash_cooldown`.
    pub bash_range: f32,
    pub bash_windup: f32,
    pub bash_radius: f32,
    pub bash_damage: f32,
    pub bash_cooldown: f32,
    /// Torches of a volley land up to this far from her; the arm goes up for `volley_windup`.
    pub volley_spread: f32,
    pub volley_windup: f32,
    pub volley_damage: f32,
    pub guard_time: f32,
    /// Share of his speed while guarding.
    pub guard_speed: f32,
    /// The horn sounds this long before the troops arrive.
    pub call_time: f32,
}

/// Ashford's boss. Phase I keeps to charges and villagers; II adds torch volleys, the raised
/// shield and the Watch's men; III chains two charges and calls the barricades.
pub const CAPTAIN_DEF: BossDef = BossDef {
    kind: CAPTAIN,
    name: Key::BossCaptain,
    phases: &[
        Phase {
            below: 1.0,
            speed: 1.0,
            windup: 0.9,
            gap: 2.4,
            rotation: &[
                Attack::Charge { chain: 1 },
                Attack::Call(&[
                    muster(VILLAGER, 16, Formation::Pack),
                    muster(TORCHBEARER, 4, Formation::Pack),
                ]),
            ],
            rally: &[],
        },
        Phase {
            below: 0.66,
            speed: 1.15,
            windup: 0.75,
            gap: 1.8,
            rotation: &[
                Attack::Charge { chain: 1 },
                Attack::Volley { n: 6 },
                Attack::Guard,
                Attack::Call(&[
                    muster(KNIGHT, 8, Formation::Pack),
                    muster(SHEPHERD, 4, Formation::Pack),
                ]),
            ],
            rally: &[muster(VILLAGER, 30, Formation::Ring)],
        },
        Phase {
            below: 0.33,
            speed: 1.3,
            windup: 0.6,
            gap: 1.3,
            rotation: &[
                Attack::Charge { chain: 2 },
                Attack::Volley { n: 8 },
                Attack::Call(&[
                    muster(KNIGHT, 10, Formation::Ring),
                    muster(TORCHBEARER, 4, Formation::Pack),
                ]),
                Attack::Charge { chain: 2 },
                Attack::Guard,
            ],
            rally: &[
                muster(BARRICADER, 16, Formation::Ring),
                muster(KNIGHT, 6, Formation::Pack),
            ],
        },
    ],
    charge_speed: 300.0,
    charge_time: 0.7,
    charge_damage: 30.0,
    charge_lock: 0.25,
    charge_rest: 0.9,
    bash_range: 44.0,
    bash_windup: 0.45,
    bash_radius: 26.0,
    bash_damage: 24.0,
    bash_cooldown: 3.0,
    volley_spread: 70.0,
    volley_windup: 0.6,
    volley_damage: 12.0,
    guard_time: 3.0,
    guard_speed: 0.35,
    call_time: 1.0,
};

/// Room in `Enemies::actions` kept for a boss beyond the horde's `MAX_SHOTS`.
pub const BOSS_ACTIONS: usize = 16;

/// A boss turns round only when she is this far (px) past its centre line, or its rush
/// points that far sideways over her distance.
pub const BOSS_TURN_DEADZONE: f32 = 12.0;

/// While a boss lives, the regular spawns run at this share of their rate.
pub const BOSS_SPAWN_SCALE: f32 = 0.5;
/// Charge lanes and bash circles on the ground: cold pale blue, like the Curate's aura.
pub const BOSS_TELEGRAPH: u32 = 0xc8e4f0;
