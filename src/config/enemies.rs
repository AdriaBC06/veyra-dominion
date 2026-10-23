//! Enemy kinds, behaviours, spawning and the run timeline (docs/design.md 7.8).

use super::{POOL_RADIUS_SCALE, THRALL_HP};
use crate::assets::SpriteId;

// --- Enemies & spawning ---------------------------------------------------
pub const MAX_ENEMIES: usize = 2500;
/// Enemies spawn on a ring this far beyond the view's half-diagonal, just off screen.
pub const SPAWN_RING_MARGIN: f32 = 32.0;
/// Enemies farther than the spawn ring plus this are moved back to the ring ahead.
pub const RELOCATE_MARGIN: f32 = 200.0;
/// Enemies per second, ramping linearly from start to end over SPAWN_RAMP_SECS.
pub const SPAWN_RATE_START: f32 = 1.0;
pub const SPAWN_RATE_END: f32 = 12.0;
pub const SPAWN_RAMP_SECS: f32 = 600.0;
/// Members of a spawn group appear scattered this far around the group's ring point.
pub const SPAWN_GROUP_SPREAD: f32 = 14.0;
/// Soft separation: neighbours closer than this push each other apart.
pub const SEPARATION_RADIUS: f32 = 12.0;
pub const SEPARATION_PUSH: f32 = 60.0;
/// Only this many neighbours contribute to separation (bounds the cost in dense hordes).
pub const SEPARATION_MAX_NEIGHBOURS: u32 = 6;
pub const ENEMY_ANIM_FPS: f32 = 6.0;

/// How an enemy kind moves and attacks (docs/design.md 7.8, V18 bestiary).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Behavior {
    /// Straight at her.
    Chase,
    /// Chase while weaving across the approach: `amp` px/s sideways at `hz`.
    Swarm { amp: f32, hz: f32 },
    /// Flies with inertia (`turn` is how fast its velocity bends toward her), so it
    /// swoops past and comes round again; ignores separation.
    Flyer { turn: f32 },
    /// Walks in; within `range` it stops for `windup` (the telegraph), dashes at
    /// `dash_speed` for `dash_time`, then recovers for `rest`.
    Charger {
        range: f32,
        windup: f32,
        dash_speed: f32,
        dash_time: f32,
        rest: f32,
    },
    /// Holds `keep` px away, strafing; every `cooldown` s within `range` it throws `shot`
    /// for `damage`.
    Ranged {
        keep: f32,
        range: f32,
        cooldown: f32,
        shot: ShotKind,
        damage: f32,
    },
    /// Holds `keep` px away behind the horde; every `SUPPORT_PULSE` s it heals the
    /// allies within `radius` by `heal` of their max HP.
    Support { keep: f32, radius: f32, heal: f32 },
    /// Rushes her; within `prime_range` it primes for `fuse` s (glowing, slowed), then
    /// explodes: `damage` to Veyra and `enemy_damage` to every enemy within `radius`.
    /// Killed before that, it fizzles.
    Bomber {
        prime_range: f32,
        fuse: f32,
        radius: f32,
        damage: f32,
        enemy_damage: f32,
    },
}

/// What a `Ranged` enemy throws.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShotKind {
    /// Flies straight at `STONE_SPEED` and hits whatever is in the way.
    Stone,
    /// Lobbed at her spot, lands after `TORCH_FLIGHT` s and leaves a burning patch.
    Torch,
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
    /// Chance that a kill drops its soul at 0:00 and at `VICTORY_TIME` (linear between,
    /// held after): cheap fodder stops paying as the run goes on.
    pub drop: (f32, f32),
    /// Spawn weight once unlocked: `weight_start + weight_growth_per_min * minutes`
    /// since it arrived.
    pub weight_start: f32,
    pub weight_growth_per_min: f32,
    /// Inclusive range of enemies spawned together.
    pub group: (u8, u8),
    pub behavior: Behavior,
    pub family: Family,
    /// Flat damage taken off every hit, down to `ARMOR_FLOOR` of it (Knight).
    pub armor: f32,
    /// Carries a shield: Hellflare's fireballs coming from the front glance off.
    pub shield: bool,
    /// Never pushed back by hits (Brute).
    pub heavy: bool,
    /// On death, releases this many enemies of this kind (Ratcatcher).
    pub split: Option<(usize, u8)>,
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
pub const CROW: usize = 2;
pub const HOUND: usize = 3;
pub const TORCHBEARER: usize = 4;
pub const SHEPHERD: usize = 5;
pub const BARRICADER: usize = 6;
pub const CURATE: usize = 7;
pub const RATCATCHER: usize = 8;
pub const KNIGHT: usize = 9;
pub const ZEALOT: usize = 10;
pub const BRUTE: usize = 11;

/// Defaults the table rows fill in.
const BASE: EnemyDef = EnemyDef {
    name: "",
    sprite: SpriteId::Villager,
    size: (14, 20),
    burst: [0x5f7f88, 0xb8927c, 0x7e9ea6, 0x8a7458],
    blood: true,
    hp: 10.0,
    speed: 50.0,
    contact_damage: 8.0,
    soul_xp: 1,
    drop: (1.0, 1.0),
    weight_start: 0.0,
    weight_growth_per_min: 0.0,
    group: (1, 1),
    behavior: Behavior::Chase,
    family: Family::Human,
    armor: 0.0,
    shield: false,
    heavy: false,
    split: None,
};

/// Ashford's roster, in order of arrival (docs/design.md 7.8, V18 bestiary).
pub const ENEMY_DEFS: [EnemyDef; 12] = [
    EnemyDef {
        name: "Rat",
        sprite: SpriteId::Rat,
        size: (17, 9),
        burst: [0xa6a4ae, 0x7d7b88, 0xa6a4ae, 0x9a808a],
        hp: 6.0,
        speed: 75.0,
        contact_damage: 5.0,
        soul_xp: 1,
        drop: (0.8, 0.3),
        weight_start: 60.0,
        weight_growth_per_min: -4.0,
        group: (3, 5),
        behavior: Behavior::Swarm { amp: 30.0, hz: 2.5 },
        family: Family::Beast,
        ..BASE
    },
    EnemyDef {
        name: "Villager",
        hp: 16.0,
        speed: 52.0,
        contact_damage: 8.0,
        soul_xp: 2,
        drop: (0.9, 0.35),
        weight_start: 40.0,
        weight_growth_per_min: 3.0,
        group: (1, 2),
        ..BASE
    },
    EnemyDef {
        name: "Crow",
        sprite: SpriteId::Crow,
        burst: [0x6b7892, 0xc4bca4, 0x6b7892, 0x9aa6b0],
        size: (15, 11),
        hp: 5.0,
        speed: 100.0,
        contact_damage: 4.0,
        soul_xp: 1,
        drop: (0.8, 0.4),
        weight_start: 10.0,
        weight_growth_per_min: 1.0,
        group: (4, 6),
        behavior: Behavior::Flyer { turn: 2.2 },
        family: Family::Beast,
        ..BASE
    },
    EnemyDef {
        name: "Hound",
        sprite: SpriteId::Hound,
        burst: [0x9e8f78, 0x75695a, 0x9e8f78, 0xb8927c],
        size: (21, 13),
        hp: 24.0,
        speed: 70.0,
        contact_damage: 9.0,
        soul_xp: 5,
        weight_start: 10.0,
        weight_growth_per_min: 3.0,
        group: (2, 3),
        behavior: Behavior::Charger {
            range: 90.0,
            windup: 0.45,
            dash_speed: 280.0,
            dash_time: 0.35,
            rest: 1.2,
        },
        family: Family::Beast,
        ..BASE
    },
    EnemyDef {
        name: "Torchbearer",
        sprite: SpriteId::Torchbearer,
        burst: [0x5f7f88, 0xe8dca4, 0xb8927c, 0xb8924a],
        size: (14, 22),
        hp: 22.0,
        speed: 48.0,
        contact_damage: 8.0,
        soul_xp: 7,
        weight_start: 6.0,
        weight_growth_per_min: 2.0,
        behavior: Behavior::Ranged {
            keep: 120.0,
            range: 190.0,
            cooldown: 3.5,
            shot: ShotKind::Torch,
            damage: 10.0,
        },
        ..BASE
    },
    EnemyDef {
        name: "Shepherd",
        sprite: SpriteId::Shepherd,
        burst: [0xd0c9b4, 0xa29b86, 0xb8927c, 0x9e8f78],
        size: (14, 21),
        hp: 16.0,
        speed: 50.0,
        contact_damage: 6.0,
        soul_xp: 7,
        weight_start: 6.0,
        weight_growth_per_min: 2.0,
        behavior: Behavior::Ranged {
            keep: 150.0,
            range: 220.0,
            cooldown: 2.6,
            shot: ShotKind::Stone,
            damage: 7.0,
        },
        ..BASE
    },
    EnemyDef {
        name: "Barricader",
        sprite: SpriteId::Barricader,
        burst: [0x8a7458, 0x6e5a44, 0xb8927c, 0x9aa6b0],
        size: (20, 22),
        hp: 60.0,
        speed: 36.0,
        contact_damage: 10.0,
        soul_xp: 14,
        weight_start: 5.0,
        weight_growth_per_min: 1.2,
        shield: true,
        ..BASE
    },
    EnemyDef {
        name: "Curate",
        sprite: SpriteId::Curate,
        burst: [0x6b7892, 0xc8d2da, 0xb8927c, 0x6b7892],
        size: (14, 21),
        hp: 34.0,
        speed: 44.0,
        contact_damage: 6.0,
        soul_xp: 16,
        weight_start: 3.0,
        weight_growth_per_min: 0.5,
        behavior: Behavior::Support {
            keep: 90.0,
            radius: 64.0,
            heal: 0.12,
        },
        ..BASE
    },
    EnemyDef {
        name: "Ratcatcher",
        sprite: SpriteId::Ratcatcher,
        burst: [0x9e8f78, 0x8a7458, 0xa6a4ae, 0x75695a],
        size: (20, 24),
        hp: 50.0,
        speed: 38.0,
        contact_damage: 10.0,
        soul_xp: 10,
        weight_start: 4.0,
        weight_growth_per_min: 1.0,
        split: Some((RAT, 5)),
        ..BASE
    },
    EnemyDef {
        name: "Knight",
        sprite: SpriteId::Knight,
        size: (18, 26),
        burst: [0xc8d2da, 0x9aa6b0, 0x4660a0, 0xb8c4d4],
        hp: 70.0,
        speed: 42.0,
        contact_damage: 14.0,
        soul_xp: 14,
        weight_start: 6.0,
        weight_growth_per_min: 3.0,
        family: Family::Armored,
        armor: 4.0,
        ..BASE
    },
    EnemyDef {
        name: "Zealot",
        sprite: SpriteId::Zealot,
        burst: [0xb2b0a4, 0x908e84, 0xb8927c, 0x8aa092],
        size: (14, 21),
        hp: 26.0,
        speed: 72.0,
        contact_damage: 8.0,
        soul_xp: 9,
        weight_start: 5.0,
        weight_growth_per_min: 2.0,
        behavior: Behavior::Bomber {
            prime_range: 36.0,
            fuse: 0.9,
            radius: 26.0,
            damage: 20.0,
            enemy_damage: 80.0,
        },
        ..BASE
    },
    EnemyDef {
        name: "Brute",
        sprite: SpriteId::Brute,
        size: (32, 40),
        burst: [0x4660a0, 0xb2b0a4, 0xb8927c, 0x908e84],
        hp: 260.0,
        speed: 30.0,
        contact_damage: 24.0,
        soul_xp: 45,
        weight_start: 2.0,
        weight_growth_per_min: 1.0,
        heavy: true,
        ..BASE
    },
];

/// A summoned thrall (Thralls Art): a seduced villager. Only its sprite, size (body
/// radius), burst and blood are used; it serves Veyra, so it gives no souls.
pub const THRALL_DEF: EnemyDef = EnemyDef {
    name: "Thrall",
    sprite: SpriteId::Thrall,
    burst: [0xb0567e, 0xff6f9a, 0xb8927c, 0xc8d2da],
    hp: THRALL_HP,
    speed: 0.0,
    contact_damage: 0.0,
    soul_xp: 0,
    ..BASE
};

/// Run length for victory, in seconds.
pub const VICTORY_TIME: f32 = 600.0;
/// Enemy HP grows by this fraction per minute of the run (x2 at 10:00), so a
/// weak build meets a damage check instead of only more bodies.
pub const HP_GROWTH_PER_MIN: f32 = 0.1;
/// In endless mode, enemy HP grows by this fraction per minute past VICTORY_TIME.
pub const ENDLESS_HP_GROWTH_PER_MIN: f32 = 0.5;

pub enum EventKind {
    /// Enemy kind (index into `ENEMY_DEFS`) starts spawning.
    Unlock(usize),
    /// Enemy kind stops spawning (the ones on the field stay).
    Retire(usize),
    /// An elite of one of the unlocked kinds arrives (tougher kinds are likelier).
    Elite,
    /// A pressure event: `n` enemies of a kind at once, in a formation.
    Wave(usize, u16, Formation),
    Victory,
}

/// How a `Wave` arrives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Formation {
    /// A closed ring on the spawn ring, all round her.
    Ring,
    /// A wall `WAVE_LINE_LEN` long across one side, closing in.
    Line,
    /// A tight pack from one side.
    Pack,
}

pub struct Event {
    pub time: f32,
    pub kind: EventKind,
}

/// Ashford's 10:00 timeline (docs/design.md 7.8): arrivals and retirements from the
/// roster table, pressure events and elites. Sorted by time.
pub const EVENTS: &[Event] = &[
    unlock(0.0, RAT),
    unlock(0.0, VILLAGER),
    unlock(45.0, CROW),
    unlock(90.0, HOUND),
    elite(90.0),
    unlock(120.0, TORCHBEARER),
    wave(150.0, CROW, 30, Formation::Pack),
    unlock(180.0, SHEPHERD),
    elite(180.0),
    unlock(210.0, BARRICADER),
    retire(240.0, RAT),
    unlock(240.0, CURATE),
    wave(240.0, VILLAGER, 40, Formation::Ring),
    unlock(270.0, RATCATCHER),
    elite(270.0),
    unlock(300.0, KNIGHT),
    wave(330.0, HOUND, 16, Formation::Line),
    unlock(360.0, ZEALOT),
    elite(360.0),
    unlock(390.0, BRUTE),
    retire(420.0, VILLAGER),
    wave(420.0, RAT, 60, Formation::Pack),
    elite(450.0),
    retire(480.0, CROW),
    wave(510.0, BARRICADER, 24, Formation::Ring),
    retire(540.0, HOUND),
    elite(540.0),
    Event {
        time: VICTORY_TIME,
        kind: EventKind::Victory,
    },
];

const fn elite(time: f32) -> Event {
    Event {
        time,
        kind: EventKind::Elite,
    }
}

const fn unlock(time: f32, kind: usize) -> Event {
    Event {
        time,
        kind: EventKind::Unlock(kind),
    }
}

const fn retire(time: f32, kind: usize) -> Event {
    Event {
        time,
        kind: EventKind::Retire(kind),
    }
}

const fn wave(time: f32, kind: usize, n: u16, formation: Formation) -> Event {
    Event {
        time,
        kind: EventKind::Wave(kind, n, formation),
    }
}

/// A `Line` wave's length, and how far apart the members of a `Pack` start.
pub const WAVE_LINE_LEN: f32 = 360.0;
pub const WAVE_PACK_SPREAD: f32 = 40.0;

// --- Behaviour tuning (see `Behavior`) -------------------------------------
/// Elites: multipliers on their kind's health, damage, speed and soul; they also
/// can't be charmed and drop an Abyssal Offering.
pub const ELITE_HP: f32 = 8.0;
pub const ELITE_DAMAGE: f32 = 1.5;
pub const ELITE_SPEED: f32 = 1.1;
pub const ELITE_SOUL: u32 = 10;
/// In endless mode an elite arrives this often.
pub const ENDLESS_ELITE_EVERY: f32 = 60.0;
/// Armour never takes a hit below this share of its damage.
pub const ARMOR_FLOOR: f32 = 0.25;
/// Ranged and Support enemies hold their distance within this slack, strafing.
pub const KEEP_SLACK: f32 = 20.0;
/// Seconds between a Support enemy's heals; healed allies sparkle this long.
pub const SUPPORT_PULSE: f32 = 1.0;
pub const BLESS_SHOW: f32 = 0.6;
/// A priming Bomber moves at this share of its speed.
pub const BOMBER_PRIME_SLOW: f32 = 0.35;
/// A flier this close dives: it holds its course for `FLYER_DIVE_TIME`, through her
/// and out the other side, before turning round.
pub const FLYER_DIVE_RANGE: f32 = 40.0;
pub const FLYER_DIVE_TIME: f32 = 0.8;
/// Fliers are drawn this high above their shadow.
pub const FLYER_HEIGHT: f32 = 14.0;
/// Hostile shots in flight, burning patches and blasts on screen at once.
pub const MAX_SHOTS: usize = 128;
pub const MAX_FIRES: usize = 48;
pub const MAX_BLASTS: usize = 24;
/// Enemy spawn and split requests queued in one tick.
pub const MAX_SPLITS: usize = 32;
pub const STONE_SPEED: f32 = 170.0;
pub const STONE_RADIUS: f32 = 3.0;
pub const STONE_LIFE: f32 = 2.0;
/// A torch flies this long and peaks this high; its patch burns as long as
/// `FIRE_LIFE`, an ellipse `FIRE_RADIUS` wide and half as deep.
pub const TORCH_FLIGHT: f32 = 0.9;
pub const TORCH_ARC: f32 = 40.0;
pub const FIRE_LIFE: f32 = 2.5;
pub const FIRE_RADIUS: f32 = 14.0;
/// A Bomber's blast stays on screen this long.
pub const BLAST_SHOW: f32 = 0.36;
/// Curate's aura ring and blessing mark: cold white-blue (gold is Veyra's).
pub const SUPPORT_AURA: u32 = 0xc8e4f0;
