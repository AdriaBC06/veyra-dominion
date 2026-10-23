//! Arts behaviour tuning: hit sizes, ranges, pools and the evolutions' numbers
//! (docs/design.md 7.7). The per-level stats are in `arts.rs`.

// --- Arts (weapons), see `ARTS` -------------------------------------------
/// Hellflare only targets enemies within this distance.
pub const FIRE_RANGE: f32 = 340.0;
pub const PROJ_RADIUS: f32 = 4.0;
/// Fireballs are launched from Veyra's hands, this high above her ground point.
pub const PROJ_DRAW_HEIGHT: f32 = 18.0;
pub const MAX_PROJECTILES: usize = 256;
/// A spent shot (fireball, heart, scythe, cross: it hit or ran out of life) never just
/// vanishes: it stops hurting and fades out over `SHOT_FADE`, slowing by
/// `SHOT_FADE_DRAG` each tick. One spent on a hit keeps only `SHOT_HIT_SLOW` of its
/// speed, so it dies on the foe.
pub const SHOT_FADE: f32 = 0.2;
pub const SHOT_FADE_DRAG: f32 = 0.85;
pub const SHOT_HIT_SLOW: f32 = 0.25;
/// Spread between Hellflare fireballs of one volley.
pub const PROJ_SPREAD: f32 = 0.16;
/// Tail Lash hits a box on one side of Veyra: it starts `LASH_NEAR` from her ground
/// point, reaches `LASH_REACH` (times area) out and covers `LASH_DEPTH` of ground depth.
pub const LASH_NEAR: f32 = 4.0;
pub const LASH_REACH: f32 = 52.0;
pub const LASH_DEPTH: f32 = 24.0;
/// Extra lashes of one attack follow this long after each other, alternating sides.
pub const LASH_DELAY: f32 = 0.12;
/// How long a lash stays on screen (its 3-frame strip).
pub const LASH_SHOW_TIME: f32 = 0.19;
/// Lashes queued or on screen at once.
pub const MAX_LASHES: usize = 16;
/// Kiss of Ruin hearts fan out like the fireballs.
pub const BOLT_RADIUS: f32 = 4.0;
/// Hearts fly at this height above the ground, like the fireballs.
pub const BOLT_DRAW_HEIGHT: f32 = 16.0;
pub const MAX_BOLTS: usize = 128;
/// A heart remembers this many enemies it went through (more pierce than this
/// just stops counting).
pub const BOLT_MAX_HITS: usize = 8;
/// Brimstone Halo: fireballs circle her middle at this radius (times the Art's area),
/// drawn where they burn, and burn an enemy at most once per `HALO_IMMUNE` seconds,
/// pushing it `HALO_KNOCKBACK` pixels outwards.
pub const HALO_RADIUS: f32 = 38.0;
pub const EMBER_RADIUS: f32 = 8.0;
pub const HALO_IMMUNE: f32 = 0.6;
pub const HALO_KNOCKBACK: f32 = 6.0;
/// Embers fade in and out over this long at the ends of each burning spell.
pub const HALO_FADE: f32 = 0.2;
/// Allure: each pulse burns every enemy within `ALLURE_RADIUS` (times the Art's area)
/// and charms up to `amount` of them for the Art's duration. Charmed enemies hunt the
/// nearest free enemy within `CHARM_SEEK`, bite the free ones they touch every
/// `CHARM_BITE` seconds for the Art's damage, and never hurt Veyra.
pub const ALLURE_RADIUS: f32 = 56.0;
pub const CHARM_SEEK: f32 = 110.0;
pub const CHARM_BITE: f32 = 0.5;
/// A charmed enemy bites free enemies up to this far beyond its body (separation keeps
/// bodies a little apart).
pub const CHARM_REACH: f32 = 4.0;
/// The pulse's ring of hearts takes this long to reach its edge, then fades as long.
pub const ALLURE_WAVE_TIME: f32 = 0.35;
/// Hearts on the pulse ring, and how high the charm mark floats over a head.
pub const ALLURE_WAVE_HEARTS: usize = 14;
pub const CHARM_MARK_HEIGHT: f32 = 6.0;
/// Charmed enemies are drawn tinted toward this colour.
pub const CHARM_TINT: u32 = 0xff9ac8;
/// Hellgate pools: ellipses `HELLGATE_RADIUS` (times area) wide and half as deep, centred on a
/// random enemy within `HELLGATE_RANGE`. Everything inside burns every `HELLGATE_TICK`.
pub const HELLGATE_RADIUS: f32 = 16.0;
pub const HELLGATE_RANGE: f32 = 200.0;
pub const HELLGATE_TICK: f32 = 0.35;
/// Pools fade in and out over this long at the ends of their life.
pub const HELLGATE_FADE: f32 = 0.2;
pub const MAX_GATES: usize = 32;
/// A pool is drawn as scorched ground in `GATE_CELL` px cells: char, scorch, faint
/// edge and ember (RGB); one cell in `GATE_EMBER_ODDS` of the char glows. One small
/// flame burns per `GATE_FLAME_AREA` square pixels, between `GATE_MIN_FLAMES` and
/// `GATE_MAX_FLAMES`.
pub const GATE_COLORS: [u32; 4] = [0x1c0a0a, 0x3a1810, 0x5a3420, 0xd04818];
pub const GATE_CELL: f32 = 2.0;
pub const GATE_EMBER_ODDS: u32 = 9;
pub const GATE_FLAME_AREA: f32 = 70.0;
pub const GATE_MIN_FLAMES: usize = 4;
pub const GATE_MAX_FLAMES: usize = 40;
/// Abyssal Rift look: void, deep purple, violet edge, and the glow of the arms that
/// turn into its centre (`RIFT_ARMS`); `RIFT_MOTES` motes per pool are sucked in from
/// the edge of its pull over `RIFT_MOTE_FALL` seconds.
pub const RIFT_COLORS: [u32; 4] = [0x08030e, 0x24093a, 0x4a1a6a, 0xa860ff];
pub const RIFT_ARMS: f32 = 3.0;
pub const RIFT_MOTES: u32 = 10;
pub const RIFT_MOTE_FALL: f32 = 0.9;
/// Whip of Torment: it bites a random enemy within `CHAIN_RANGE`, then jumps to the
/// nearest enemy it hasn't bitten within `CHAIN_JUMP` (times area), `amount` in all.
pub const CHAIN_RANGE: f32 = 180.0;
pub const CHAIN_JUMP: f32 = 64.0;
pub const MAX_CHAIN_LINKS: usize = 24;
pub const MAX_CHAINS: usize = 8;
/// How long a chain stays on screen, and the height of its links over the ground.
pub const CHAIN_SHOW_TIME: f32 = 0.2;
pub const CHAIN_DRAW_HEIGHT: f32 = 9.0;
/// The drawn chain kinks every this many pixels.
pub const CHAIN_KINK: f32 = 8.0;
/// Torment Eternal look: a black and blood bolt with a molten core (glow, body, core),
/// forking from the first victim to all the others, and a ring of its reach flashing
/// out from the first bite.
pub const TORMENT_COLORS: [u32; 3] = [0x16091a, 0x9a1028, 0xffb060];
/// Chain colours: dark glow, body, white-hot core.
pub const CHAIN_COLORS: [u32; 3] = [0x5c0c1c, 0xe04454, 0xffd0dc];
/// Scent of Sin: a haze `SCENT_RADIUS` (times area) around her. Every burn (its
/// cooldown) it hurts everything inside and weakens it for the Art's duration:
/// weakened enemies take `SCENT_WEAKEN` more damage from every Art.
pub const SCENT_RADIUS: f32 = 40.0;
pub const SCENT_WEAKEN: f32 = 0.25;
/// Weakened enemies are drawn tinted toward this colour.
pub const WEAK_TINT: u32 = 0xe6c8ff;
/// Motes drifting up through the haze, and its colour (RGB).
pub const SCENT_MOTES: usize = 18;
pub const SCENT_COLOR: u32 = 0xff4fb0;
/// Reaper's Scythe: scythes leave her hands upwards at the Art's speed, always
/// symmetric: an odd one straight up, the rest in pairs to either side, each pair
/// `SCYTHE_SPREAD` px/s wider. They fall at `SCYTHE_GRAVITY`; each hits an enemy once,
/// through `pierce` more. Hit radii here and below are at area 1.
pub const SCYTHE_RADIUS: f32 = 8.0;
pub const SCYTHE_GRAVITY: f32 = 620.0;
pub const SCYTHE_SPREAD: f32 = 80.0;
pub const MAX_SCYTHES: usize = 64;
/// Ids a scythe, cross or sigil remembers between resets (a fuller list stops
/// hitting until it resets).
pub const PIERCE_MEMORY: usize = 48;
/// Spinning sprites turn this many times per second and float this high.
pub const SCYTHE_SPIN: f32 = 2.5;
pub const SCYTHE_DRAW_HEIGHT: f32 = 14.0;
/// Profaned Cross: thrown at a foe within `CROSS_RANGE` (the first at the nearest),
/// it slows by `CROSS_DECEL` px/s², turns back and chases her, speeding up by as much,
/// until it is back in her hands (within `CROSS_CATCH`; `CROSS_MAX_LIFE` at most). It
/// hits each enemy once on the way out and once on the way back.
pub const CROSS_RANGE: f32 = 260.0;
pub const CROSS_RADIUS: f32 = 10.0;
pub const CROSS_CATCH: f32 = 10.0;
pub const CROSS_MAX_LIFE: f32 = 10.0;
pub const CROSS_DECEL: f32 = 240.0;
/// A caught cross fades into her hands (`SHOT_FADE`), closing this share of the gap
/// to her each tick.
pub const CROSS_CATCH_PULL: f32 = 0.35;
pub const MAX_CROSSES: usize = 32;
pub const CROSS_SPIN: f32 = 3.0;
pub const CROSS_DRAW_HEIGHT: f32 = 14.0;
/// Wandering Sigil: it flies off in a random direction and bounces off the edges of
/// the view (`SIGIL_MARGIN` inside them); it forgets whom it hit at each bounce.
pub const SIGIL_RADIUS: f32 = 7.0;
pub const SIGIL_MARGIN: f32 = 8.0;
pub const MAX_SIGILS: usize = 32;
pub const SIGIL_SPIN: f32 = 1.0;
pub const SIGIL_DRAW_HEIGHT: f32 = 12.0;
/// Sigils fade out over their last this many seconds.
pub const SIGIL_FADE: f32 = 0.3;
/// Bat Swarm: `amount` bats circle her on an ellipse `BAT_ORBIT` wide (half as deep),
/// `BAT_DRAW_HEIGHT` up, at the Art's speed (rad/s). Each attack every bat lobs a
/// blood bomb at a random enemy within `BAT_RANGE`; it lands after `BAT_DROP_TIME`
/// and bursts over `BAT_BLAST_RADIUS` (times area), shown for `BAT_BLAST_TIME`.
pub const BAT_ORBIT: f32 = 44.0;
pub const BAT_DRAW_HEIGHT: f32 = 30.0;
pub const BAT_RANGE: f32 = 220.0;
pub const BAT_DROP_TIME: f32 = 0.45;
pub const BAT_BLAST_RADIUS: f32 = 22.0;
pub const BAT_BLAST_TIME: f32 = 0.35;
/// Bombs land this far off the enemy they were aimed at, at most.
pub const BAT_SCATTER: f32 = 8.0;
pub const MAX_BATS: usize = 16;
pub const MAX_BOMBS: usize = 48;
/// Bats bob up and down this much, each on its own beat; drops of a burst.
pub const BAT_BOB: f32 = 3.0;
pub const BAT_BLAST_SPARKS: usize = 10;
/// Black Mass: every cooldown (only while an enemy is in view) it strikes every enemy
/// within the view (plus `MASS_MARGIN`) for the Art's damage. Never an instant kill.
/// The ritual circle shows under her for `MASS_SHOW`, the view flashes red for
/// `MASS_FLASH` at `MASS_FLASH_ALPHA` at most.
pub const MASS_MARGIN: f32 = 16.0;
pub const MASS_SHOW: f32 = 1.2;
pub const MASS_FLASH: f32 = 0.45;
pub const MASS_FLASH_ALPHA: f32 = 0.35;
pub const MASS_FLASH_COLOR: u32 = 0xa8182e;
/// Gaze of Desire: `amount` beams, the first at the nearest enemy within `GAZE_RANGE`,
/// the rest evenly around; each `GAZE_REACH` long and `GAZE_WIDTH` wide (times area).
/// What a beam touches is frozen for the Art's duration: it neither moves nor hurts
/// her. Beams show for `GAZE_SHOW`, from her eyes `GAZE_DRAW_HEIGHT` up.
pub const GAZE_RANGE: f32 = 220.0;
pub const GAZE_REACH: f32 = 150.0;
pub const GAZE_WIDTH: f32 = 6.0;
pub const GAZE_SHOW: f32 = 0.3;
pub const GAZE_DRAW_HEIGHT: f32 = 22.0;
pub const MAX_GAZES: usize = 16;
/// Gaze beam colours (dark glow, body, core) and the tint of frozen enemies: her gold.
pub const GAZE_COLORS: [u32; 3] = [0x8a5a1c, 0xe8b04a, 0xfff2c0];
pub const FROZEN_TINT: u32 = 0xffd890;
/// Thralls: every summon `amount` of them rise `THRALL_SPAWN_RADIUS` around her, only
/// while an enemy is within `THRALL_SEEK` and the pool (`MAX_THRALLS`, shared with the
/// Infernal Court's) has room. Each walks at the Art's speed to the nearest enemy within
/// `THRALL_SEEK` of itself, or back to her (keeping `THRALL_LEASH` away), and hits one
/// it touches every `THRALL_BLOW` for the Art's damage, `THRALL_REACH` (times area)
/// beyond its body. It has `THRALL_HP` health; free enemies touching it hurt it with
/// their contact damage at most every `THRALL_IFRAMES`. It serves for the Art's
/// duration, fading in over `THRALL_RISE` and out over `THRALL_FADE`.
pub const THRALL_SPAWN_RADIUS: f32 = 22.0;
pub const THRALL_SEEK: f32 = 180.0;
pub const THRALL_LEASH: f32 = 34.0;
pub const THRALL_BLOW: f32 = 0.6;
pub const THRALL_REACH: f32 = 5.0;
pub const THRALL_HP: f32 = 24.0;
pub const THRALL_IFRAMES: f32 = 0.4;
pub const THRALL_RISE: f32 = 0.3;
pub const THRALL_FADE: f32 = 0.4;
pub const MAX_THRALLS: usize = 64;
/// Thralls keep this far apart from each other (they are not in the enemy hash, so
/// the horde's separation doesn't move them).
pub const THRALL_SPACING: f32 = 10.0;
pub const THRALL_SPACING_PUSH: f32 = 60.0;
/// Free enemies checked for contact around a thrall (dense hordes bail out early).
pub const THRALL_CONTACT_MAX: u32 = 8;

// --- Evolutions (7.7), opened by Abyssal Offerings --------------------------
/// Kiss of Ruin (Hellflare evolved): each volley throws `KISS_EXTRA` more shots, as
/// hearts that fly at `KISS_SPEED` for `KISS_LIFE` and go through `KISS_PIERCE` enemies.
pub const KISS_EXTRA: u32 = 2;
pub const KISS_SPEED: f32 = 380.0;
pub const KISS_LIFE: f32 = 1.0;
pub const KISS_PIERCE: u32 = 2;
/// Barbed Scourge: a lashed enemy bleeds for the Lash's duration, taking
/// `BLEED_SHARE` of its damage every `BLEED_TICK`.
pub const BLEED_TICK: f32 = 0.5;
pub const BLEED_SHARE: f32 = 0.25;
/// Chance per tick that a bleeding enemy with blood drips on the ground.
pub const BLEED_DRIP: f32 = 0.05;
/// Barbed Scourge look: `BLEED_DROPS` 2x2 drops run down each bleeding body, every
/// `BLEED_DROP_FALL` seconds, fresh then darkening.
pub const BLEED_DROPS: u32 = 2;
pub const BLEED_DROP_FALL: f32 = 0.7;
pub const BLEED_DROP_COLORS: [u32; 2] = [0xd42a3c, 0x7a1020];
/// Infernal Crown: the flames never rest; one more joins them.
pub const CROWN_EXTRA: u32 = 1;
/// Infernal Crown look: the faint ring its fireballs circle on.
pub const CROWN_RING_COLOR: u32 = 0xe8b04a;
/// Siren's Dominion: when a charm runs out the enemy bursts, hurting the free enemies
/// within `SIREN_BURST_RADIUS` (times area) for `SIREN_BURST_DAMAGE` times the Art's
/// damage. It never kills: the enemy that bursts lives on, and the burst leaves
/// everyone it hurts at `SIREN_SPARE_HP` at least. It shows for `SIREN_BURST_TIME`.
pub const SIREN_BURST_RADIUS: f32 = 36.0;
pub const SIREN_BURST_DAMAGE: f32 = 1.0;
pub const SIREN_SPARE_HP: f32 = 1.0;
pub const SIREN_BURST_TIME: f32 = 0.35;
pub const SIREN_BURST_HEARTS: usize = 8;
pub const MAX_SIREN_BURSTS: usize = 32;
/// Abyssal Rift: enemies within `RIFT_RADIUS` (times area) of a pool are dragged to its
/// centre at `RIFT_PULL` px/s.
pub const RIFT_RADIUS: f32 = 56.0;
pub const RIFT_PULL: f32 = 70.0;
/// Torment Eternal: the chain keeps jumping, up to `MAX_CHAIN_LINKS`, to any enemy
/// within `TORMENT_RADIUS` (times area) of the first one it bit.
pub const TORMENT_RADIUS: f32 = 96.0;
/// Devouring Musk: every enemy the haze burns gives her `MUSK_HEAL` health, at most
/// `MUSK_HEAL_CAP` per second.
pub const MUSK_HEAL: f32 = 0.5;
pub const MUSK_HEAL_CAP: f32 = 3.0;
/// Devouring Musk look: a wine-red haze; after each burn, `MUSK_STRAND_MOTES` motes
/// per burned enemy run from it to her over `MUSK_DRAIN_TIME`.
pub const MUSK_COLOR: u32 = 0xb01848;
pub const MUSK_STRAND_COLOR: u32 = 0xff5a6e;
pub const MUSK_STRAND_MOTES: usize = 3;
pub const MUSK_DRAIN_TIME: f32 = 0.5;
/// Midnight Harvest: `HARVEST_EXTRA` more scythes fly out evenly all round at
/// `HARVEST_SPEED`, curling at `HARVEST_CURL` rad/s, hitting everything they cross
/// (`HARVEST_RADIUS` times area) for `HARVEST_LIFE`.
pub const HARVEST_EXTRA: u32 = 5;
pub const HARVEST_SPEED: f32 = 170.0;
pub const HARVEST_CURL: f32 = 1.2;
pub const HARVEST_RADIUS: f32 = 11.0;
pub const HARVEST_LIFE: f32 = 2.4;
/// Midnight Harvest look: each scythe trails ghosts of itself `HARVEST_TRAIL_LAG`
/// seconds apart, tinted `HARVEST_TRAIL_COLOR` and fading.
pub const HARVEST_TRAIL: usize = 3;
pub const HARVEST_TRAIL_LAG: f32 = 0.035;
pub const HARVEST_TRAIL_COLOR: u32 = 0x8a6aff;
/// Heretic's Brand: one more cross, and every enemy a cross hits burns for
/// `BRAND_BURN` (times Sloth), taking `BRAND_SHARE` of its damage every `BRAND_TICK`.
pub const BRAND_EXTRA: u32 = 1;
pub const BRAND_BURN: f32 = 2.0;
pub const BRAND_TICK: f32 = 0.5;
pub const BRAND_SHARE: f32 = 0.3;
/// Broken Seal: every bounce the sigil blasts everything within `SEAL_BLAST_RADIUS`
/// (times area) for `SEAL_BLAST_DAMAGE` times its damage; the blast shows for
/// `SEAL_BLAST_TIME`.
pub const SEAL_BLAST_RADIUS: f32 = 30.0;
pub const SEAL_BLAST_DAMAGE: f32 = 1.5;
pub const SEAL_BLAST_TIME: f32 = 0.4;
pub const MAX_SEAL_BLASTS: usize = 32;
/// Sparks on a blast's ring.
pub const SEAL_BLAST_SPARKS: usize = 16;
/// Crimson Host (Bat Swarm evolved): `HOST_EXTRA` more bats and blasts
/// `HOST_BLAST` times as wide. No healing (two healing evolutions were too many).
pub const HOST_EXTRA: u32 = 3;
pub const HOST_BLAST: f32 = 1.4;
/// Witches' Sabbath (Black Mass evolved): comes `SABBATH_COOLDOWN` times as often and
/// draws every soul on the map to her at `SABBATH_SOUL_SPEED`.
pub const SABBATH_COOLDOWN: f32 = 0.8;
pub const SABBATH_SOUL_SPEED: f32 = 420.0;
/// Evil Eye (Gaze evolved): one more beam, and frozen enemies take `EVIL_EYE_VULN`
/// more damage from everything.
pub const EVIL_EYE_EXTRA: u32 = 1;
pub const EVIL_EYE_VULN: f32 = 0.5;
// --- Unions (7.7): two maxed Arts fused by an Abyssal Offering ---------------
/// Infernal Court (Allure + Thralls): each pulse turns its first `amount` free foes
/// into thralls of their own kind instead of charming them (elites are only charmed,
/// so their Offering is never lost). A converted foe keeps its health, hits with its
/// own contact damage (times Wrath) and serves `COURT_LIFE` times Allure's duration.
/// It drops its soul and counts as a kill when it falls; with Siren's Dominion it
/// bursts when its time runs out. The Thralls' own summons go on.
pub const COURT_LIFE: f32 = 2.5;
/// Abyssal Offerings: dropped by elites, opened when Veyra gets this close. One with
/// nothing to evolve or level up heals `OFFERING_HEAL` instead. Sacrificed (rejected),
/// an Offering heals `SACRIFICE_HEAL` and gives `SACRIFICE_XP` souls' worth of XP.
pub const OFFERING_PICKUP: f32 = 12.0;
pub const MAX_OFFERINGS: usize = 16;
pub const OFFERING_HEAL: f32 = 50.0;
pub const SACRIFICE_HEAL: f32 = 20.0;
pub const SACRIFICE_XP: u32 = 25;

pub const HIT_FLASH_TIME: f32 = 0.1;
/// Dead enemies flash as a white silhouette for this long, then burst.
pub const DEATH_FADE_TIME: f32 = 0.12;
pub const MAX_FADES: usize = 512;
/// Death burst: bits in the enemy's palette leave the body at a random height, fall to
/// the ground and fade there. `burst_count` picks 6..=10 per enemy by sprite area.
pub const MAX_BURST_PIXELS: usize = 1024;
/// Side of a burst bit in pixels; single pixels get lost in the grass speckle.
pub const BURST_BIT: f32 = 2.0;
pub const BURST_SPEED: (f32, f32) = (12.0, 38.0);
/// Blood drops (enemies with `EnemyDef::blood`): a dark, brownish red, well apart from
/// Veyra's bright crimsons. Heavier than the other bits, they jump lower.
pub const BLOOD: [u32; 2] = [0x4a0c10, 0x6a1416];
pub const BLOOD_JUMP_SCALE: f32 = 0.6;
/// Blood on the ground is permanent: every bleeding death leaves a pool sized by the
/// enemy (`pool_radius`) and every drop a small splash where it lands. `blood::BloodLayer`
/// bakes them into ground textures, so their number costs nothing to draw.
/// Pool colours: dark rim, body, wet glint.
pub const POOL_COLORS: [u32; 3] = [0x22060a, 0x3e0a0e, 0x601618];
/// Pool radius per pixel of the enemy's collision radius.
pub const POOL_RADIUS_SCALE: f32 = 0.9;
/// A pool landing inside a recent one feeds it instead (the area adds up, same shape,
/// larger) up to this radius; past it, deaths there only splash around. The last
/// `MAX_POOL_SITES` pools are remembered for this.
pub const POOL_MAX_RADIUS: f32 = 18.0;
pub const MAX_POOL_SITES: usize = 512;
/// Splashes thrown around a pool that is already at its largest.
pub const FULL_POOL_SPLASHES: usize = 3;
/// Splats the simulation can queue between two frames; the renderer drains them.
pub const MAX_SPLATS: usize = 512;
/// The blood layer is cut into square chunks of world pixels; past `MAX_BLOOD_CHUNKS`
/// the chunk seen longest ago is dropped (only far away, long-left ground loses blood).
pub const BLOOD_CHUNK: u16 = 128;
pub const MAX_BLOOD_CHUNKS: usize = 256;
/// Thrown this much faster away from Veyra.
pub const BURST_AWAY: f32 = 10.0;
pub const BURST_JUMP: (f32, f32) = (55.0, 105.0);
pub const BURST_GRAVITY: f32 = 340.0;
/// Seconds a bit lies on the ground, fading all along.
pub const BURST_REST_TIME: f32 = 0.5;
/// Elite outline colour: cold, so it never competes with Veyra's warm palette.
pub const ELITE_OUTLINE: u32 = 0xa8e4ff;
/// Debug (F5): enemies this close to Veyra become elites.
pub const DEBUG_ELITE_RADIUS: f32 = 90.0;
