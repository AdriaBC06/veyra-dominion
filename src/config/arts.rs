//! The Arts table: per-Art base numbers and level gains (docs/design.md 7.7).

use super::{
    ALLUREMENT, BLOODLUST, BLOOD_PACT, DARK_SIGIL, LEGION, NIGHT_WINGS, SIRENS_CALL, WRATH,
};
use crate::i18n::Key;

pub const MAX_ARTS: usize = 6;
pub const ART_MAX_LEVEL: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArtKind {
    Hellflare,
    TailLash,
    KissOfRuin,
    BrimstoneHalo,
    Allure,
    Hellgate,
    ChainOfTorment,
    WingGust,
}

/// An Art's numbers at some level. Each Art reads the ones it uses:
/// - `amount`: fireballs per volley, lashes per attack, hearts per throw, embers,
///   enemies charmed per pulse, pools per cast, enemies a chain bites, gusts per cast;
/// - `cooldown`: seconds between attacks (the Halo: seconds off between spells);
/// - `area`: Halo orbit radius, Allure radius, chain jump, gust reach, lash depth and
///   rift pull multiplier (the sprites never scale, and Hellgate's pools don't grow);
/// - `speed`: projectile pixels per second (the Halo: radians per second);
/// - `pierce`: extra enemies a heart goes through;
/// - `duration`: projectile lifetime (the Halo: seconds each spell burns; Allure:
///   charm; Hellgate: pool life; Wing Gust: slow; Tail Lash: bleed once evolved).
///   Dark Sigil lengthens every one but the projectile lifetimes.
/// - `damage`: per hit (Allure: per pulse and per charmed bite; Hellgate: per burn).
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

/// What an Art becomes when an Abyssal Offering is opened while it is at
/// `ART_MAX_LEVEL` and `sin` (index into `SINS`) is owned.
pub struct Evolution {
    pub sin: usize,
    pub name: Key,
    pub desc: Key,
}

pub struct ArtDef {
    pub kind: ArtKind,
    pub name: Key,
    pub desc: Key,
    /// "+{} fireball(s)" and the like, for the level-up card.
    pub amount_key: Key,
    pub base: ArtStats,
    /// Gains for levels 2..=ART_MAX_LEVEL.
    pub gains: [ArtGain; ART_MAX_LEVEL as usize - 1],
    pub evolution: Evolution,
}

pub const HELLFLARE: usize = 0;
pub const TAIL_LASH: usize = 1;
pub const KISS_OF_RUIN: usize = 2;
pub const BRIMSTONE_HALO: usize = 3;
pub const ALLURE: usize = 4;
pub const HELLGATE: usize = 5;
pub const CHAIN_OF_TORMENT: usize = 6;
pub const WING_GUST: usize = 7;

pub const ARTS: [ArtDef; 8] = [
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
        evolution: Evolution {
            sin: LEGION,
            name: Key::EvoHellstorm,
            desc: Key::EvoHellstormDesc,
        },
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
            duration: 3.0,
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
        evolution: Evolution {
            sin: WRATH,
            name: Key::EvoBarbedScourge,
            desc: Key::EvoBarbedScourgeDesc,
        },
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
        evolution: Evolution {
            sin: BLOOD_PACT,
            name: Key::EvoKissOfDeath,
            desc: Key::EvoKissOfDeathDesc,
        },
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
        evolution: Evolution {
            sin: DARK_SIGIL,
            name: Key::EvoInfernalCrown,
            desc: Key::EvoInfernalCrownDesc,
        },
    },
    // A pulse that burns and charms the nearest few; they turn on the horde (Garlic).
    ArtDef {
        kind: ArtKind::Allure,
        name: Key::ArtAllure,
        desc: Key::ArtAllureDesc,
        amount_key: Key::ArtMoreCharmed,
        base: ArtStats {
            damage: 5.0,
            cooldown: 3.0,
            amount: 2,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 2.5,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 3.0,
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 3.0,
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                amount: 2,
                ..NO_GAIN
            },
        ],
        evolution: Evolution {
            sin: SIRENS_CALL,
            name: Key::EvoSirensDominion,
            desc: Key::EvoSirensDominionDesc,
        },
    },
    // Burning pools opened under random enemies (Santa Water).
    ArtDef {
        kind: ArtKind::Hellgate,
        name: Key::ArtHellgate,
        desc: Key::ArtHellgateDesc,
        amount_key: Key::ArtMorePools,
        base: ArtStats {
            damage: 7.0,
            cooldown: 4.0,
            amount: 1,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 2.5,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 3.0,
                ..NO_GAIN
            },
            ArtGain {
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 3.0,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 3.0,
                ..NO_GAIN
            },
        ],
        evolution: Evolution {
            sin: ALLUREMENT,
            name: Key::EvoAbyssalRift,
            desc: Key::EvoAbyssalRiftDesc,
        },
    },
    // A chain that bites one enemy and jumps from foe to foe (Lightning Ring).
    ArtDef {
        kind: ArtKind::ChainOfTorment,
        name: Key::ArtChainOfTorment,
        desc: Key::ArtChainOfTormentDesc,
        amount_key: Key::ArtMoreJumps,
        base: ArtStats {
            damage: 12.0,
            cooldown: 1.6,
            amount: 3,
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
                damage: 4.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 4.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                area: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 2,
                damage: 6.0,
                ..NO_GAIN
            },
        ],
        evolution: Evolution {
            sin: BLOODLUST,
            name: Key::EvoTormentEternal,
            desc: Key::EvoTormentEternalDesc,
        },
    },
    // A beat of her wings: a cone that throws enemies back and slows them (control).
    ArtDef {
        kind: ArtKind::WingGust,
        name: Key::ArtWingGust,
        desc: Key::ArtWingGustDesc,
        amount_key: Key::ArtMoreGusts,
        base: ArtStats {
            damage: 5.0,
            cooldown: 2.2,
            amount: 1,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 1.0,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 3.0,
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 3.0,
                ..NO_GAIN
            },
            ArtGain {
                duration: 0.5,
                cooldown: 0.1,
                ..NO_GAIN
            },
        ],
        evolution: Evolution {
            sin: NIGHT_WINGS,
            name: Key::EvoTempestOfWings,
            desc: Key::EvoTempestOfWingsDesc,
        },
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
