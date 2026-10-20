//! The Arts table: per-Art base numbers and level gains (PLAN.md 7.7).

use crate::i18n::Key;

pub const MAX_ARTS: usize = 6;
pub const ART_MAX_LEVEL: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArtKind {
    Hellflare,
    TailLash,
    KissOfRuin,
    BrimstoneHalo,
}

/// An Art's numbers at some level. Each Art reads the ones it uses:
/// - `amount`: fireballs per volley, lashes per attack, hearts per throw, embers;
/// - `cooldown`: seconds between attacks (the Halo: seconds off between spells);
/// - `area`: Halo orbit radius multiplier;
/// - `speed`: projectile pixels per second (the Halo: radians per second);
/// - `pierce`: extra enemies a heart goes through;
/// - `duration`: projectile lifetime (the Halo: seconds each spell burns).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ArtStats {
    pub damage: f32,
    pub cooldown: f32,
    pub amount: u32,
    pub area: f32,
    pub speed: f32,
    pub pierce: u32,
    pub duration: f32,
}

/// What one level adds. `cooldown`, `area` and `speed` are fractions of the base value
/// (a cooldown gain shortens it), the rest are flat.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ArtGain {
    pub damage: f32,
    pub cooldown: f32,
    pub amount: u32,
    pub area: f32,
    pub speed: f32,
    pub pierce: u32,
    pub duration: f32,
}

const NO_GAIN: ArtGain = ArtGain {
    damage: 0.0,
    cooldown: 0.0,
    amount: 0,
    area: 0.0,
    speed: 0.0,
    pierce: 0,
    duration: 0.0,
};

pub struct ArtDef {
    pub kind: ArtKind,
    pub name: Key,
    pub desc: Key,
    /// "+{} fireball(s)" and the like, for the level-up card.
    pub amount_key: Key,
    pub base: ArtStats,
    /// Gains for levels 2..=ART_MAX_LEVEL.
    pub gains: [ArtGain; ART_MAX_LEVEL as usize - 1],
}

pub const HELLFLARE: usize = 0;
pub const TAIL_LASH: usize = 1;
pub const KISS_OF_RUIN: usize = 2;
pub const BRIMSTONE_HALO: usize = 3;

pub const ARTS: [ArtDef; 4] = [
    // Homing fireballs at the nearest enemy (Magic Wand).
    ArtDef {
        kind: ArtKind::Hellflare,
        name: Key::ArtHellflare,
        desc: Key::ArtHellflareDesc,
        amount_key: Key::ArtMoreFireballs,
        base: ArtStats {
            damage: 10.0,
            cooldown: 0.6,
            amount: 1,
            area: 1.0,
            speed: 320.0,
            pierce: 0,
            duration: 1.4,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
        ],
    },
    // A lash on the side she faces; the second one cracks behind her (Whip).
    ArtDef {
        kind: ArtKind::TailLash,
        name: Key::ArtTailLash,
        desc: Key::ArtTailLashDesc,
        amount_key: Key::ArtMoreLashes,
        base: ArtStats {
            damage: 14.0,
            cooldown: 1.35,
            amount: 1,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 0.0,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
        ],
    },
    // Fast hearts where she moves, going through enemies (Knife).
    ArtDef {
        kind: ArtKind::KissOfRuin,
        name: Key::ArtKissOfRuin,
        desc: Key::ArtKissOfRuinDesc,
        amount_key: Key::ArtMoreHearts,
        base: ArtStats {
            damage: 8.0,
            cooldown: 0.9,
            amount: 1,
            area: 1.0,
            speed: 420.0,
            pierce: 1,
            duration: 0.9,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 4.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                pierce: 1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 4.0,
                ..NO_GAIN
            },
            ArtGain {
                pierce: 1,
                ..NO_GAIN
            },
        ],
    },
    // Embers circling her for a few seconds at a time (King Bible).
    ArtDef {
        kind: ArtKind::BrimstoneHalo,
        name: Key::ArtBrimstoneHalo,
        desc: Key::ArtBrimstoneHaloDesc,
        amount_key: Key::ArtMoreEmbers,
        base: ArtStats {
            damage: 10.0,
            cooldown: 3.0,
            amount: 1,
            area: 1.0,
            speed: 3.6,
            pierce: 0,
            duration: 3.0,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.25,
                speed: 0.3,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.25,
                speed: 0.3,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
        ],
    },
];

impl ArtDef {
    /// Numbers at `level` (1..=ART_MAX_LEVEL): the base plus every gain up to it.
    pub fn stats(&self, level: u8) -> ArtStats {
        let mut s = self.base;
        let (mut cooldown, mut area, mut speed) = (1.0, 1.0, 1.0);
        for g in &self.gains[..(level.max(1) - 1) as usize] {
            s.damage += g.damage;
            s.amount += g.amount;
            s.pierce += g.pierce;
            s.duration += g.duration;
            cooldown -= g.cooldown;
            area += g.area;
            speed += g.speed;
        }
        s.cooldown *= cooldown;
        s.area *= area;
        s.speed *= speed;
        s
    }
}
