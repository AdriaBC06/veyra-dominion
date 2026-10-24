//! The living village (design.md 7.14, V19h): what the Arts break and burn, the villagers
//! who flee and what the wreckage drops.

/// Marks (Art hits left on the village) queued per tick; more are dropped.
pub const MAX_MARKS: usize = 256;
/// A prop hit by a mark ignores further marks for this long, so Arts that strike every
/// tick don't stack: however many Arts pound it, a building takes at most one blow per
/// cooldown. A house holds out a few seconds of steady pounding (sturdy, but it falls).
pub const PROP_HIT_COOLDOWN: f32 = 0.25;
/// A struck building or breakable shakes this long and sheds `STRUCK_BITS` bits, so the
/// blows show before it cracks.
pub const STRUCK_SHAKE: f32 = 0.12;
pub const STRUCK_BITS: usize = 2;
/// Side of the square buckets that index the props for mark lookups, in pixels.
pub const PROP_CHUNK: f32 = 64.0;

/// A building: its sprite name, structure points and the folk living in it.
pub struct BuildingDef {
    pub name: &'static str,
    pub hp: f32,
    pub folk: u8,
}

pub const BUILDINGS: [BuildingDef; 4] = [
    BuildingDef {
        name: "house_cottage",
        hp: 500.0,
        folk: 2,
    },
    BuildingDef {
        name: "house_long",
        hp: 700.0,
        folk: 3,
    },
    BuildingDef {
        name: "house_slate",
        hp: 800.0,
        folk: 3,
    },
    BuildingDef {
        name: "chapel",
        hp: 1600.0,
        folk: 2,
    },
];
/// A building cracks below this share of its structure.
pub const CRACK_AT: f32 = 0.5;

/// A breakable: sprite name, hit points, whether it catches fire.
pub struct BreakableDef {
    pub name: &'static str,
    pub hp: f32,
    pub burns: bool,
}

pub const BREAKABLES: [BreakableDef; 5] = [
    BreakableDef {
        name: "barrel",
        hp: 70.0,
        burns: true,
    },
    BreakableDef {
        name: "crate",
        hp: 55.0,
        burns: true,
    },
    BreakableDef {
        name: "hay_bale",
        hp: 40.0,
        burns: true,
    },
    BreakableDef {
        name: "hay_roll",
        hp: 50.0,
        burns: true,
    },
    BreakableDef {
        name: "cart",
        hp: 140.0,
        burns: true,
    },
];

/// Plants that only fire touches: those with a burnt sprite of their own (`<name>_burnt`)...
pub const CHARRING_PLANTS: [&str; 6] = [
    "tree_oak",
    "tree_oak_small",
    "tree_birch",
    "tree_pine",
    "bush",
    "bush_berry",
];
/// ...and the low ones that burn down to stubble.
pub const LOW_PLANTS: [&str; 8] = [
    "crop_wheat",
    "crop_cabbage",
    "flowers_lav",
    "flowers_yellow",
    "flowers_white",
    "grass_tall",
    "fern",
    "reeds",
];

/// Heat a fire mark adds; heat cools by `HEAT_COOL` per second.
pub const FIRE_HEAT: f32 = 1.0;
pub const HEAT_COOL: f32 = 0.5;
/// Heat that sets a building alight, and anything else: since a prop takes at most one fire
/// mark per `PROP_HIT_COOLDOWN`, a house needs seconds of fire on it, a tree a few touches.
pub const BUILDING_IGNITE: f32 = 16.0;
pub const PLANT_IGNITE: f32 = 6.0;
/// Most things burning at once; fire marks past it only scorch.
pub const MAX_BURNING: usize = 12;
/// How long things burn, in seconds.
pub const BUILDING_BURN_TIME: f32 = 16.0;
pub const TREE_BURN_TIME: f32 = 5.0;
pub const LOW_BURN_TIME: f32 = 2.0;
pub const BREAKABLE_BURN_TIME: f32 = 2.5;
/// Share of its structure a burning building loses per second.
pub const BURN_DECAY: f32 = 0.035;
/// A burning building warms whatever burnable is within this many pixels of its sprite,
/// `SPREAD_HEAT` every `SPREAD_TICK` seconds. Plants don't spread fire.
pub const SPREAD_RANGE: f32 = 20.0;
pub const SPREAD_HEAT: f32 = 2.0;
pub const SPREAD_TICK: f32 = 1.5;

/// Villagers: the pool, never grown.
pub const MAX_CIVILIANS: usize = 48;
/// Kinds, in `CIVILIAN_SPRITES` order.
pub const CIVILIAN_SPRITES: [&str; 5] = ["folk_man", "folk_woman", "folk_child", "chicken", "pig"];
/// Odds of each kind for a villager coming out to wander (folk only spill when fleeing).
pub const WANDER_WEIGHTS: [f32; 5] = [3.0, 3.0, 1.5, 2.0, 0.6];
/// Walking and running speeds per kind, pixels per second.
pub const CIVILIAN_WALK: [f32; 5] = [14.0, 14.0, 16.0, 12.0, 10.0];
pub const CIVILIAN_RUN: [f32; 5] = [58.0, 56.0, 62.0, 50.0, 46.0];
/// Body radius for marks and walls.
pub const CIVILIAN_RADIUS: f32 = 4.0;
/// Each home lets out this many wanderers while Veyra is within `WANDER_RANGE` of its door.
pub const HOME_WANDERERS: u8 = 2;
pub const WANDER_RANGE: f32 = 260.0;
/// Wanderers stay this close to their door.
pub const WANDER_LEASH: f32 = 70.0;
/// Within this of the door the folk inside run out.
pub const PANIC_RANGE: f32 = 80.0;
/// A villager runs from Veyra while she is this close, and keeps running a while after.
pub const FLEE_RANGE: f32 = 110.0;
pub const FLEE_KEEP: f32 = 2.5;
/// Past this distance from Veyra a villager goes home (back into its house's count).
pub const CIVILIAN_DESPAWN: f32 = 420.0;
/// A slain villager's soul (folk only) and the blood pool it leaves.
pub const CIVILIAN_SOUL_XP: u32 = 1;
pub const CIVILIAN_POOL: f32 = 3.0;

/// Things lying on the ground to pick up; the pool, never grown.
pub const MAX_DROPS: usize = 64;
/// Veyra picks a drop up within this.
pub const DROP_PICKUP: f32 = 14.0;
/// Health from food.
pub const BREAD_HEAL: f32 = 4.0;
pub const HAM_HEAL: f32 = 8.0;
/// What a broken breakable drops, rolled in order (the rest drop nothing): coin, bread,
/// ham, coffer, relic.
pub const BREAKABLE_DROPS: [f32; 5] = [0.35, 0.08, 0.04, 0.02, 0.004];
/// A collapsing house: coins scattered, and its chance of a coffer.
pub const HOUSE_COINS: (u32, u32) = (2, 5);
pub const HOUSE_COFFER: f32 = 0.15;
/// The chapel always leaves its relic.
pub const CHAPEL_RELIC: f32 = 1.0;
/// A coffer opens on touch and holds, rolled in order: a soul worth `COFFER_XP`, a call
/// that pulls every soul on the map to her, or (the rest) an Abyssal Offering.
pub const COFFER_ROLL: [f32; 2] = [0.55, 0.4];
pub const COFFER_XP: u32 = 12;
/// An opened coffer lingers this long before it's gone.
pub const COFFER_LINGER: f32 = 1.2;
/// The relic: Art cooldowns run `RELIC_HASTE` times as fast for `RELIC_TIME` seconds.
pub const RELIC_HASTE: f32 = 2.0;
pub const RELIC_TIME: f32 = 10.0;
/// Drops are flung from where they break this far, and settle in this long.
pub const DROP_FLING: f32 = 10.0;
pub const DROP_SETTLE: f32 = 0.35;
/// Radius of the mark the Whip of Torment leaves at each bite.
pub const CHAIN_MARK: f32 = 6.0;
/// Blood bits from a slain villager.
pub const CIVILIAN_BITS: usize = 4;
/// Splinters from a smashed breakable (wood browns).
pub const SPLINTERS: [u32; 3] = [0x6a4a2e, 0x8a6a3e, 0x4a3220];
pub const SPLINTER_BITS: usize = 8;
/// Dust and stones from a fallen building: puffs along its front.
pub const RUBBLE_BITS: [u32; 3] = [0x787060, 0x544b44, 0xa49c86];
pub const COLLAPSE_PUFFS: usize = 5;
pub const COLLAPSE_SMOKE_TIME: f32 = 1.6;
