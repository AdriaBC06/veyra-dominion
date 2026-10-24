//! The Arts table: per-Art base numbers and level gains (docs/design.md 7.7).

use super::{
    BLOODLUST, DARK_SIGIL, ENVY, GLUTTONY, INFERNAL_HIDE, LEGION, LUST, NIGHT_WINGS, PRIDE, SLOTH,
    WRATH,
};
use crate::i18n::Key;

pub const MAX_ARTS: usize = 6;
pub const ART_MAX_LEVEL: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArtKind {
    Hellflare,
    TailLash,
    BrimstoneHalo,
    Allure,
    Hellgate,
    WhipOfTorment,
    ScentOfSin,
    ReapersScythe,
    ProfanedCross,
    WanderingSigil,
    BatSwarm,
    BlackMass,
    GazeOfDesire,
    Thralls,
}

/// An Art's numbers at some level. Each Art reads the ones it uses:
/// - `amount`: fireballs per volley, lashes per attack, flames, enemies charmed per
///   pulse, pools per cast, enemies a chain bites, scythes, crosses, sigils, bats and
///   gaze beams and thralls summoned (the Scent and Black Mass have none);
/// - `cooldown`: seconds between attacks (the Halo: seconds off between spells; the
///   Scent: between burns; Thralls: between summons);
/// - `area`: lash reach, Halo orbit radius, Allure radius, pool size, chain jump, haze
///   radius, rift pull multiplier, the size of scythes, crosses and sigils (sprite
///   and hit), bat blasts, gaze beams (length and width) and a thrall's reach;
/// - `speed`: projectile pixels per second (the Halo and the bats: radians per second;
///   thralls: their walk);
/// - `pierce`: enemies a scythe goes through after the first;
/// - `duration`: projectile lifetime (the Halo: seconds each spell burns; Allure:
///   charm; Hellgate: pool life; Tail Lash: bleed once evolved; the Scent: weakness;
///   the Sigil: how long it wanders; the Gaze: freeze; Thralls: how long one serves;
///   the Cross has none, it flies until it is back).
///   Sloth lengthens every one but the projectile flights (see `ArtDef::sloth`).
/// - `damage`: per hit (Allure: per pulse and per charmed bite; Hellgate and the
///   Scent: per burn; Thralls: per blow; the Gaze deals none).
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
    /// "+{} fireball(s)" and the like, for the level-up card (unused by an Art whose
    /// levels never add amount).
    pub amount_key: Key,
    pub base: ArtStats,
    /// Gains for levels 2..=ART_MAX_LEVEL.
    pub gains: [ArtGain; ART_MAX_LEVEL as usize - 1],
    /// `None` for an Art that only grows through a union (see `UNIONS`).
    pub evolution: Option<Evolution>,
    /// What it throws stops at walls, buildings, the ridge and the woods (design.md 7.14).
    /// Sweeping, orbiting, falling and instant Arts pass over them.
    pub walls: bool,
    /// Its marks on the village set things alight (V19h); the others only break them.
    pub fire: bool,
}

/// Two Arts at `ART_MAX_LEVEL` that an Abyssal Offering fuses into one (no Sin
/// needed). The first keeps its slot under the union's name, the second leaves its
/// slot free but keeps working as part of the union.
pub struct UnionDef {
    pub arts: [usize; 2],
    pub name: Key,
    pub desc: Key,
}

pub const HELLFLARE: usize = 0;
pub const TAIL_LASH: usize = 1;
pub const BRIMSTONE_HALO: usize = 2;
pub const ALLURE: usize = 3;
pub const HELLGATE: usize = 4;
pub const WHIP_OF_TORMENT: usize = 5;
pub const SCENT_OF_SIN: usize = 6;
pub const REAPERS_SCYTHE: usize = 7;
pub const PROFANED_CROSS: usize = 8;
pub const WANDERING_SIGIL: usize = 9;
pub const BAT_SWARM: usize = 10;
pub const BLACK_MASS: usize = 11;
pub const GAZE_OF_DESIRE: usize = 12;
pub const THRALLS: usize = 13;

pub const ARTS: [ArtDef; 14] = [
    // Fireballs at the nearest enemy (Magic Wand). Evolves into Kiss of Ruin.
    ArtDef {
        kind: ArtKind::Hellflare,
        walls: true,
        fire: true,
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
        evolution: Some(Evolution {
            sin: LEGION,
            name: Key::EvoKissOfRuin,
            desc: Key::EvoKissOfRuinDesc,
        }),
    },
    // A lash on the side she faces; the second one cracks behind her (Whip). Its reach
    // grows with area.
    ArtDef {
        kind: ArtKind::TailLash,
        walls: false,
        fire: false,
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
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 10.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: WRATH,
            name: Key::EvoBarbedScourge,
            desc: Key::EvoBarbedScourgeDesc,
        }),
    },
    // One big flame circling her, then more, faster and resting less (King Bible).
    ArtDef {
        kind: ArtKind::BrimstoneHalo,
        walls: false,
        fire: true,
        name: Key::ArtBrimstoneHalo,
        desc: Key::ArtBrimstoneHaloDesc,
        amount_key: Key::ArtMoreEmbers,
        base: ArtStats {
            damage: 12.0,
            cooldown: 3.0,
            amount: 1,
            area: 1.0,
            speed: 3.0,
            pierce: 0,
            duration: 3.0,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                speed: 0.25,
                cooldown: 0.15,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                speed: 0.25,
                cooldown: 0.15,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                cooldown: 0.2,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: SLOTH,
            name: Key::EvoInfernalCrown,
            desc: Key::EvoInfernalCrownDesc,
        }),
    },
    // A pulse that burns and charms the nearest few; they turn on the horde.
    ArtDef {
        kind: ArtKind::Allure,
        walls: false,
        fire: false,
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
        evolution: Some(Evolution {
            sin: LUST,
            name: Key::EvoSirensDominion,
            desc: Key::EvoSirensDominionDesc,
        }),
    },
    // Burning pools opened under random enemies, wider every few levels (Santa Water).
    ArtDef {
        kind: ArtKind::Hellgate,
        walls: false,
        fire: true,
        name: Key::ArtHellgate,
        desc: Key::ArtHellgateDesc,
        amount_key: Key::ArtMorePools,
        base: ArtStats {
            damage: 3.0,
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
                area: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                damage: 2.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                area: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                damage: 1.0,
                duration: 0.5,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 2.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: DARK_SIGIL,
            name: Key::EvoAbyssalRift,
            desc: Key::EvoAbyssalRiftDesc,
        }),
    },
    // A chain that bites one enemy and jumps from foe to foe (Lightning Ring).
    ArtDef {
        kind: ArtKind::WhipOfTorment,
        walls: false,
        fire: false,
        name: Key::ArtWhipOfTorment,
        desc: Key::ArtWhipOfTormentDesc,
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
        evolution: Some(Evolution {
            sin: BLOODLUST,
            name: Key::EvoTormentEternal,
            desc: Key::EvoTormentEternalDesc,
        }),
    },
    // A pink haze around her that burns slowly and weakens what it touches (Garlic).
    ArtDef {
        kind: ArtKind::ScentOfSin,
        walls: false,
        fire: false,
        name: Key::ArtScentOfSin,
        desc: Key::ArtScentOfSinDesc,
        amount_key: Key::ArtGainArea,
        base: ArtStats {
            damage: 4.0,
            cooldown: 1.0,
            amount: 0,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 1.5,
        },
        gains: [
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 2.0,
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
                damage: 2.0,
                duration: 0.5,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                damage: 2.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: GLUTTONY,
            name: Key::EvoDevouringMusk,
            desc: Key::EvoDevouringMuskDesc,
        }),
    },
    // Scythes thrown high that fall back through the horde (Axe), bigger every other
    // level.
    ArtDef {
        kind: ArtKind::ReapersScythe,
        walls: false,
        fire: false,
        name: Key::ArtReapersScythe,
        desc: Key::ArtReapersScytheDesc,
        amount_key: Key::ArtMoreScythes,
        base: ArtStats {
            damage: 20.0,
            cooldown: 2.0,
            amount: 1,
            area: 1.0,
            speed: 300.0,
            pierce: 2,
            duration: 1.6,
        },
        gains: [
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 10.0,
                area: 0.15,
                ..NO_GAIN
            },
            ArtGain {
                pierce: 2,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                area: 0.15,
                ..NO_GAIN
            },
            ArtGain {
                damage: 10.0,
                ..NO_GAIN
            },
            ArtGain {
                pierce: 2,
                cooldown: 0.1,
                area: 0.15,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 10.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: NIGHT_WINGS,
            name: Key::EvoMidnightHarvest,
            desc: Key::EvoMidnightHarvestDesc,
        }),
    },
    // An inverted cross thrown at a foe that comes back to her like a boomerang
    // (Cross). Speed carries it further.
    ArtDef {
        kind: ArtKind::ProfanedCross,
        walls: false,
        fire: false,
        name: Key::ArtProfanedCross,
        desc: Key::ArtProfanedCrossDesc,
        amount_key: Key::ArtMoreCrosses,
        base: ArtStats {
            damage: 10.0,
            cooldown: 1.8,
            amount: 1,
            area: 1.0,
            speed: 260.0,
            pierce: 0,
            duration: 0.0,
        },
        gains: [
            ArtGain {
                damage: 10.0,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.1,
                speed: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 10.0,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.1,
                speed: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 10.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: WRATH,
            name: Key::EvoHereticsBrand,
            desc: Key::EvoHereticsBrandDesc,
        }),
    },
    // Her sigil wandering the screen, bouncing off its edges (Runetracer).
    ArtDef {
        kind: ArtKind::WanderingSigil,
        walls: false,
        fire: false,
        name: Key::ArtWanderingSigil,
        desc: Key::ArtWanderingSigilDesc,
        amount_key: Key::ArtMoreSigils,
        base: ArtStats {
            damage: 10.0,
            cooldown: 3.0,
            amount: 1,
            area: 1.0,
            speed: 250.0,
            pierce: 0,
            duration: 2.25,
        },
        gains: [
            ArtGain {
                damage: 5.0,
                speed: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                duration: 0.3,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                speed: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                damage: 5.0,
                duration: 0.3,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                duration: 0.5,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: INFERNAL_HIDE,
            name: Key::EvoBrokenSeal,
            desc: Key::EvoBrokenSealDesc,
        }),
    },
    // A flock circling her that bombs the horde, bigger every few levels (Peachone and
    // Ebony Wings as one Art).
    ArtDef {
        kind: ArtKind::BatSwarm,
        walls: false,
        fire: true,
        name: Key::ArtBatSwarm,
        desc: Key::ArtBatSwarmDesc,
        amount_key: Key::ArtMoreBats,
        base: ArtStats {
            damage: 15.0,
            cooldown: 2.5,
            amount: 2,
            area: 1.0,
            speed: 1.6,
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
                area: 0.15,
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
                area: 0.15,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 5.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: GLUTTONY,
            name: Key::EvoCrimsonHost,
            desc: Key::EvoCrimsonHostDesc,
        }),
    },
    // Now and then a rite that strikes everything in sight hard, never an instant
    // kill (Pentagram).
    ArtDef {
        kind: ArtKind::BlackMass,
        walls: false,
        fire: false,
        name: Key::ArtBlackMass,
        desc: Key::ArtBlackMassDesc,
        amount_key: Key::ArtGainArea,
        base: ArtStats {
            damage: 12.0,
            cooldown: 20.0,
            amount: 0,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 0.0,
        },
        gains: [
            ArtGain {
                damage: 6.0,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 6.0,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 6.0,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 12.0,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: ENVY,
            name: Key::EvoWitchesSabbath,
            desc: Key::EvoWitchesSabbathDesc,
        }),
    },
    // Her gaze: beams that freeze what they touch for a moment (Clock Lancet).
    ArtDef {
        kind: ArtKind::GazeOfDesire,
        walls: false,
        fire: false,
        name: Key::ArtGazeOfDesire,
        desc: Key::ArtGazeOfDesireDesc,
        amount_key: Key::ArtMoreGazes,
        base: ArtStats {
            damage: 0.0,
            cooldown: 2.0,
            amount: 1,
            area: 1.0,
            speed: 0.0,
            pierce: 0,
            duration: 1.0,
        },
        gains: [
            ArtGain {
                duration: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                duration: 0.25,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                area: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                cooldown: 0.1,
                duration: 0.5,
                ..NO_GAIN
            },
        ],
        evolution: Some(Evolution {
            sin: PRIDE,
            name: Key::EvoEvilEye,
            desc: Key::EvoEvilEyeDesc,
        }),
    },
    // Seduced villagers that rise at her side and fight on their own (a summon).
    ArtDef {
        kind: ArtKind::Thralls,
        walls: false,
        fire: false,
        name: Key::ArtThralls,
        desc: Key::ArtThrallsDesc,
        amount_key: Key::ArtMoreThralls,
        base: ArtStats {
            damage: 8.0,
            cooldown: 4.0,
            amount: 1,
            area: 1.0,
            speed: 80.0,
            pierce: 0,
            duration: 8.0,
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
                duration: 2.0,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                ..NO_GAIN
            },
            ArtGain {
                damage: 4.0,
                cooldown: 0.1,
                ..NO_GAIN
            },
            ArtGain {
                duration: 2.0,
                speed: 0.2,
                ..NO_GAIN
            },
            ArtGain {
                amount: 1,
                damage: 4.0,
                ..NO_GAIN
            },
        ],
        // Grows only by uniting with Allure (Infernal Court).
        evolution: None,
    },
];

pub const INFERNAL_COURT: usize = 0;

pub const UNIONS: [UnionDef; 1] = [
    // Allure's charmed become thralls of the court; her own thralls keep coming.
    UnionDef {
        arts: [ALLURE, THRALLS],
        name: Key::UnionInfernalCourt,
        desc: Key::UnionInfernalCourtDesc,
    },
];

impl ArtDef {
    /// Whether Sloth lengthens `duration`: not where it is a projectile's flight, which
    /// is its range.
    pub fn sloth(&self) -> bool {
        !matches!(
            self.kind,
            ArtKind::Hellflare | ArtKind::ReapersScythe | ArtKind::ProfanedCross
        )
    }

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
