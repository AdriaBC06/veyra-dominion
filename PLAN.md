# Veyra: Dominion — MVP Plan

Single source of truth for scope, architecture, task order and progress.
A new session reads only this file plus the files of the current task, and resumes at the first unchecked task in **Progress**.

## 1. Summary & MVP scope

Top-down survivors-like in Rust + macroquad. Veyra, a red-skinned succubus villain, has risen to raze a village. She must survive **5:00** against animals and humans while absorbing their souls (XP) and choosing upgrades.

**In the MVP**
- Veyra: levitating 16×24 sprite (bob, shadow, tilt/impulse frames, lagging tail, wing flaps), damage blink, smoke dissolve on death. 100 HP, speed 180, 0.5 s i-frames.
- Weapon "Hellflare": auto-fires dark fireballs at the nearest enemy (spatial hash lookup).
- 6 upgrades, pick 1 of 3 on level-up, with caps.
- Souls with magnet pickup, XP curve `8 + 6n`.
- 3 enemies (Rat, Villager, Knight) driven by an `EnemyDef` table and a timed `Event` list.
- Waves: off-screen spawn ring, rate 1/s → 25/s over 5:00, max 2,500 alive.
- States: Title → Playing ⇄ Paused / LevelUp → GameOver / Victory (endless or back to title).
- HUD (HP, XP, timer, kills, level), F3 debug overlay.
- Procedural muted ground, infinite world, camera follows player.
- Fixed logical resolution 640×360, `Nearest` scaling, resizable window with letterbox.

**Out of scope**: audio, more weapons, bosses, maps, meta-progression, saving.

**Tooling facts (preflight 2026-10-18)**: rustc/cargo 1.95.0, gh logged in as AdriaBC06, git user AdriaBC06 <adriabc06@gmail.com>, Arch libs present (libx11, libxi, mesa, alsa-lib, libxcursor, libxrandr). `macroquad = "0.4.16"` (from `cargo search`).

## 2. Cargo.toml

```toml
[package]
name = "veyra-dominion"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-or-later"

[dependencies]
macroquad = "0.4.16"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
lto = "thin"
codegen-units = 1
panic = "abort"
```

## 3. Architecture

Hard rules
- Only dependency: `macroquad` (`macroquad::rand`, `macroquad::math::Vec2`).
- Simulation (`game.rs`, `enemies.rs`, `spatial.rs`, `upgrades.rs`) **never** calls drawing, input, time or window functions. Input arrives as a plain `InputState`; time as `dt`. This keeps it testable headless with `cargo test`.
- Fixed timestep 60 Hz with accumulator in `main.rs`; frame dt clamped to 0.25 s and max 8 ticks/frame (no spiral of death).
- Hot path: SoA enemies, `swap_remove`, all buffers preallocated in `Game::new`, zero allocations per tick.
- **All** balance numbers and tables live in `config.rs`.

### Modules and main types (signatures only)

**config.rs**
```rust
pub const GAME_TITLE: &str = "Veyra: Dominion";
pub const LOGICAL_W: f32 = 640.0; pub const LOGICAL_H: f32 = 360.0;
pub const TICK_DT: f32 = 1.0 / 60.0;
// player, weapon, souls, xp, spawn, caps, hash: PLAYER_MAX_HP, PLAYER_SPEED, PLAYER_IFRAMES,
// BOB_AMPLITUDE, BOB_HZ, FIRE_DAMAGE, FIRE_INTERVAL, PROJ_SPEED, PROJ_LIFETIME, PROJ_PIERCE,
// PICKUP_RADIUS, SOUL_MAGNET_SPEED, SPAWN_RATE_START, SPAWN_RATE_END, SPAWN_RAMP_SECS,
// MAX_ENEMIES, MAX_PROJECTILES, MAX_SOULS, HASH_CELL, VICTORY_TIME, ENDLESS_HP_GROWTH_PER_MIN, ...
pub fn xp_to_next(level: u32) -> u32;                 // 8 + 6n

pub enum Behavior { Chase }                            // v2: Ranged, Support, Boss
pub struct EnemyDef {
    pub name: &'static str, pub sprite: SpriteId,
    pub hp: f32, pub speed: f32, pub contact_damage: f32, pub radius: f32,
    pub soul_xp: u32, pub weight_start: f32, pub weight_growth_per_min: f32,
    pub group: (u8, u8), pub behavior: Behavior,
}
pub const RAT: usize = 0; pub const VILLAGER: usize = 1; pub const KNIGHT: usize = 2;
pub const ENEMY_DEFS: [EnemyDef; 3];                   // Rat 60 / Villager 40 / Knight from 2:00 growing

pub enum EventKind { Unlock(usize), Victory }          // v2: BossWarning, SpawnBoss(usize)
pub struct Event { pub time: f32, pub kind: EventKind }
pub const EVENTS: &[Event];                            // 0:00 Rat, 0:00 Villager, 2:00 Knight, 5:00 Victory

pub enum UpgradeKind { Fury, Frenzy, EmberLegion, LightWings, BloodPact, SoulPull }
pub struct UpgradeDef { pub kind: UpgradeKind, pub name: &'static str, pub desc: &'static str, pub max_level: u8 }
pub const UPGRADES: [UpgradeDef; 6];
```

**spatial.rs** — uniform grid of 64 px cells over a fixed window centred on the player (counting sort into flat arrays, no hashing allocations). Enemies outside the window are relocated by the sim, so the grid never overflows.
```rust
pub struct SpatialHash { cell: f32, cols: usize, rows: usize, origin: Vec2,
                         cell_start: Vec<u32>, entries: Vec<u32>, points: Vec<Vec2>, cell_of: Vec<u32> }
// points = position snapshot parallel to entries, so queries need no xs/ys and
// separation can read neighbours while the SoA columns are being written.
impl SpatialHash {
    pub fn new(cell: f32, cols: usize, rows: usize, capacity: usize) -> Self;
    pub fn rebuild(&mut self, center: Vec2, xs: &[f32], ys: &[f32]);
    pub fn query_radius(&self, p: Vec2, r: f32, f: impl FnMut(usize, Vec2));
    pub fn nearest(&self, p: Vec2, max_r: f32) -> Option<usize>; // ring-by-ring
}
```

**enemies.rs** — SoA.
```rust
pub struct Enemies { pub x: Vec<f32>, pub y: Vec<f32>, pub hp: Vec<f32>,
                     pub kind: Vec<u8>, pub flash: Vec<f32>, pub phase: Vec<f32> }
impl Enemies {
    pub fn with_capacity(n: usize) -> Self;
    pub fn len(&self) -> usize;
    pub fn spawn(&mut self, kind: usize, pos: Vec2, hp_scale: f32);
    pub fn swap_remove(&mut self, i: usize);
    pub fn steer(&mut self, dt: f32, target: Vec2, hash: &SpatialHash); // match Behavior + soft separation
}
```

**upgrades.rs**
```rust
pub struct Upgrades { pub levels: [u8; 6] }
pub struct PlayerStats { pub damage: f32, pub fire_interval: f32, pub projectiles: u32,
                         pub speed: f32, pub max_hp: f32, pub pickup_radius: f32 }
impl Upgrades {
    pub fn available(&self) -> impl Iterator<Item = usize>;   // not capped
    pub fn roll_choices(&self, out: &mut [Option<usize>; 3]); // up to 3 distinct
    pub fn apply(&mut self, idx: usize, player: &mut Player); // BloodPact heals
    pub fn stats(&self) -> PlayerStats;                       // base * (1 + k * level), additive
}
```

**game.rs** — whole simulation state.
```rust
pub struct InputState { pub move_dir: Vec2 }
pub struct Player { pub pos: Vec2, pub vel: Vec2, pub hp: f32, pub max_hp: f32,
                    pub iframes: f32, pub facing: f32, pub tail_lag: Vec2, pub anim_t: f32 }
pub struct Projectile { pub pos: Vec2, pub vel: Vec2, pub life: f32 }   // no pierce in MVP (no upgrade uses it)
pub struct Soul { pub pos: Vec2, pub xp: u32 }         // merged into nearest when MAX_SOULS reached
pub struct Fade { pub pos: Vec2, pub vel: Vec2, pub sprite: SpriteId, pub frame: usize, pub t: f32, pub duration: f32 }  // enemy vanish / smoke puffs
pub enum SimEvent { None, LevelUp, PlayerDied, Victory }
pub struct Game {
    pub time: f32, pub endless: bool, pub kills: u32, pub level: u32, pub xp: u32,
    pub pending_levels: u32, pub player: Player, pub enemies: Enemies,
    pub projectiles: Vec<Projectile>, pub souls: Vec<Soul>, pub fades: Vec<Fade>,
    pub upgrades: Upgrades, pub choices: [Option<usize>; 3],
    hash: SpatialHash, spawn_acc: f32, fire_cd: f32, next_event: usize,
    unlocked: [bool; 3], dying: f32,
}
impl Game {
    pub fn new() -> Self;
    pub fn tick(&mut self, input: &InputState) -> SimEvent;   // one fixed step
    pub fn choose_upgrade(&mut self, slot: usize);
    pub fn spawn_rate(t: f32) -> f32;                          // linear, clamped
}
```
Tick order: events → player move → rebuild hash → spawn/relocate → steer enemies → fire → projectiles vs enemies → contact damage → souls magnet/pickup → fades → victory/death.

**sprites.rs** — pixel art as char matrices + palette, packed at startup into one atlas `Image` → one `Texture2D` (`FilterMode::Nearest`, good batching). A white silhouette copy of every enemy frame is generated for hit flash.
```rust
pub enum SpriteId { VeyraBody, VeyraWings, VeyraTail, Rat, Villager, Knight, Fireball, Soul, Smoke, Ground }
pub struct Atlas { tex: Texture2D, rects: Vec<Rect>, index: [(u16, u8); N] }
pub fn init();                                           // build atlas once (needs window context)
pub fn sprite(id: SpriteId, frame: usize) -> (Texture2D, Rect); // only accessor -> easy swap to real atlas
```
Frames: VeyraBody 0 = rest, 1 = impulse (flip_x by facing); VeyraWings 0/1; VeyraTail 1; enemies 16×16 with frames 0/1 (+ flash variants); Fireball, Soul, Smoke 8×8; Ground 4–6 32×32 tile variants (grass/dirt, muted greens/browns).

**hud.rs** — `draw_hud(&Game)`, `draw_debug(&Game, fps, update_ms)`, `draw_level_up(&Game, selected)`, `draw_center_screen(title, lines)`.

**main.rs** — `window_conf()` (title = `GAME_TITLE`, 1280×720, resizable), `AppState { Title, Playing, Paused, LevelUp, GameOver, Victory }`, accumulator loop, input → `InputState`, world render into 640×360 render target via `Camera2D` centred on player, then letterboxed blit (integer scale when possible). Visible-only ground tiles chosen by hash(ix, iy).

### Rendering details for Veyra
- Bob: `offset_y = 2 * sin(2π · 1.5 · t)`; shadow ellipse (alpha ~0.35) width scales `1 - 0.15 * (offset_y / 2)`.
- Impulse frame while |vel| > threshold, flipped by facing; slight rotation (~±0.08 rad) toward horizontal motion.
- Tail drawn separately; `tail_lag` lerps toward `-vel * k` (smoothed delay, updated in sim).
- Wings: 2 frames toggled at ~3 Hz.
- Hurt: skip drawing every other 0.06 s during i-frames. Death: body fades while smoke puffs (Fades) rise for ~1.2 s, then GameOver.

### Controls
WASD / arrows move · Esc pause · Enter start/confirm · 1/2/3 or ←/→ + Enter pick upgrade · R restart · E endless (on victory) · T title · F3 debug.

## 4. Tasks

Each task = one commit (+ `git push`), `cargo check` before commit, Progress checklist updated in the same commit.

| # | Task | Done when | Commit |
|---|---|---|---|
| T1 | Repo bootstrap: `git init`, `gh repo create AdriaBC06/veyra-dominion --public --license gpl-3.0 --source`, `.gitignore`, README, PLAN.md | Repo on GitHub, `main` pushed | `docs: bootstrap repo with plan and readme` |
| T2 | Cargo project, `config.rs`, window, fixed-timestep loop, 640×360 render target + letterbox | `cargo check`/clippy clean; window shows cleared target | `feat: window, fixed timestep loop and logical render target` |
| T3 | `sprites.rs`: palette/matrix → atlas, `sprite()`, Veyra frames (**ask the user for their Veyra reference images first**), ground tiles | Atlas builds; unit test validates matrix sizes/palette chars | `feat: code-defined pixel art atlas` |
| T4 | Player movement + levitation (bob, shadow, impulse/flip, tilt, tail lag, wings), camera follow, procedural ground | Veyra floats with WASD, ground scrolls | `feat: levitating player, camera and procedural ground` |
| **CP1** | **Checkpoint 1 — Setup** | | |
| T5 | `spatial.rs` + tests (insert, radius query, nearest, out-of-window) | `cargo test` green | `feat: spatial hash grid with tests` |
| T6 | `enemies.rs` SoA, `EnemyDef` table, Chase + separation, ring spawn at fixed rate, relocation of far enemies | Enemies spawn off-screen and chase | `feat: SoA enemies with chase behavior and ring spawning` |
| T7 | Hellflare weapon: nearest target, projectiles, hits, white flash, fade-out on death, kills counter | Enemies die to fireballs | `feat: hellflare auto-fire and projectile collisions` |
| T8 | Contact damage, i-frames, blink, smoke death, Title/GameOver screens, restart | Full lose loop works | `feat: player damage, death, title and game over` |
| **CP2** | **Checkpoint 2 — Combat** | | |
| T9 | Souls: drop, magnet, pickup, XP curve, `pending_levels` + tests | XP/level tests green | `feat: soul drops, magnet pickup and xp curve` |
| T10 | `upgrades.rs` + LevelUp screen, 1 of 3 distinct, caps + tests | Upgrade tests green | `feat: level-up upgrades with caps` |
| T11 | Wave director: linear rate 1→25/s, weights, `EVENTS`, 2,500 cap + tests | Spawn curve tests green | `feat: wave director driven by event table` |
| T12 | HUD bars/timer/kills/level, pause, F3 debug overlay | Visible in game | `feat: hud, pause and debug overlay` |
| T13 | Victory at 5:00, endless mode (HP growth), back to title | Win loop works | `feat: victory screen and endless mode` |
| T14 | Headless stress test 20,000 ticks at cap | Test green within budget | `test: headless stress simulation at enemy cap` |
| T15 | Clippy clean, README controls + balance notes, `cargo build --release` | All green, release built | `docs: controls, balance notes and release build` |
| **CP3** | **Checkpoint 3 — Full loop** | | |

## 5. Test plan (headless, no window)

- `cargo clippy -- -D warnings` before each checkpoint (trimmed output); `cargo test` after T5+.
- **spatial**: insert/rebuild counts; `query_radius` returns exactly the points within r (vs brute force on random sets); `nearest` matches brute force; empty grid → `None`; points outside window ignored safely.
- **spawn curve**: `spawn_rate(0)=1`, `spawn_rate(150)=13`, `spawn_rate(300)=25`, clamped after; simulated spawning never exceeds `MAX_ENEMIES`; Knight weight 0 before 2:00.
- **xp**: `xp_to_next(1)=14`, `xp_to_next(n)=8+6n`; a big XP gain yields multiple `pending_levels`.
- **upgrades**: each effect matches table; caps stop offering; `roll_choices` distinct; returns <3 when fewer remain; EmberLegion stops at 5 projectiles; BloodPact heals clamped to max.
- **stress**: `Game::new`, force `time = 300`, player HP huge, 20,000 ticks with random input; assert no panic, `enemies.len() <= MAX_ENEMIES`, projectiles/souls ≤ caps, mean tick < 16 ms (`std::time::Instant`, test-only).
- Sprites: matrix dimensions and palette chars are valid (pure data test, no texture creation).
- `cargo build --release` only at CP3 (and CP1/CP2 if cheap).

## 6. Risks & mitigations

| Risk | Mitigation |
|---|---|
| Render-target camera y-flip quirk in macroquad 0.4 | Isolate in one `world_camera()` fn; verify orientation at CP1 |
| Blurry text when scaling | Draw HUD into the 640×360 target, Nearest blit, integer scale when window allows |
| Debug build too slow at 2,500 enemies | dev opt-level 1 + deps 3; grid counting sort; soft separation stops scanning after K neighbours (`try_query_radius`, 9.4 → ~0.6 ms/tick at cap); stress test measures it |
| `macroquad::rand` is global → nondeterministic parallel tests | Tests assert invariants, not exact sequences; seed with `srand` per test |
| Unbounded world breaks the grid | Grid window centred on player; enemies beyond `RELOCATE_RADIUS` are moved back to the ring |
| Spiral of death on slow frames | Clamp frame dt and max ticks per frame |
| Clippy `-D warnings` on unused v2 hooks | Only add enum variants that are used now (`Chase`, `Unlock`, `Victory`); v2 variants stay in this doc. Same for `EnemyDef` fields and consts: each is added in the task that first reads it (`KNIGHT`, weights, `group` in T11; `soul_xp` in T9; `name` in T12) |
| Sprite content rating | Clothed leather outfit, no nudity; silhouette conveyed by shape and palette |
| Context/weekly budget | State lives in PLAN.md + git; one commit per task; checkpoint stops |

## 7. Roadmap v2 (design only — do NOT implement)

### Extensibility hooks already in the MVP
- `EnemyDef.behavior: Behavior` matched in `Enemies::steer` → new variants plug in there.
- `EVENTS: &[Event]` consumed sequentially by `Game::tick` → new `EventKind`s add timeline beats.
- `pending_levels` counter → multi-level XP bursts already supported.
- `sprite(id, frame)` single accessor → new art or a real atlas without touching callers.

### Priest (appears ~2:30)
HP 60, speed 50, keeps ~200 px distance, no contact damage. Casts a slow dodgeable blessing (10 dmg). Holy aura radius ~120: slows Veyra 30% and regenerates nearby enemies. High-priority target.
- **config.rs**: `Behavior::Support { keep_dist, cast_interval, bolt_speed, bolt_damage, aura_radius, aura_slow, aura_regen }`; new `PRIEST` `EnemyDef` with `contact_damage = 0`; `Event { time: 150.0, kind: Unlock(PRIEST) }`.
- **enemies.rs**: new SoA column `cooldown: Vec<f32>`; `steer` handles keep-distance (approach/retreat/strafe); returns cast requests into a reusable buffer.
- **game.rs**: `hostile: Vec<HostileShot>` (preallocated) moved/collided with player; aura pass via spatial hash: regen enemies within radius, player `slow_mul` field applied to speed; optional target bias of Hellflare toward Support enemies within range.

### Archangel boss (5:00)
HP ~3,000, flies (ignores separation), 3 phases by HP: (1) homing feather volleys, (2) charge with ground telegraph, (3) light pillars on marked zones. On-screen warning before arrival. Defeating it becomes the victory condition; endless mode stays as an extra. Its soul grants several levels at once.
- **config.rs**: `Behavior::Boss`; `EnemyDef.flies: bool`; `EventKind::BossWarning`, `EventKind::SpawnBoss(usize)`; boss table `BossPhase { hp_frac, pattern, params }`; `VictoryCondition { Survive(f32), DefeatBoss }`; soul XP sized to N levels.
- **enemies.rs**: separation skipped when `flies`; the boss is not stored in the SoA horde but in a dedicated `Boss` struct (one entity, rich state machine) to keep the hot loop simple.
- **game.rs**: `boss: Option<Boss>` with phase/state timers; telegraph zones (`Vec<Telegraph>`) rendered by main; reuses `hostile` shots from the Priest work; `SimEvent::BossWarning`; victory on boss death; boss soul feeds `pending_levels` (multi level-up screens chained).

## 8. Progress

- [x] T1 Repo bootstrap
- [x] T2 Window, loop, render target
- [x] T3 Sprite atlas
- [x] T4 Levitating player, camera, ground
- [x] **CP1 Setup**
- [x] T5 Spatial hash
- [x] T6 SoA enemies + spawning
- [x] T7 Hellflare weapon
- [x] T8 Damage, death, title, game over
- [x] **CP2 Combat**
- [x] T9 Souls + XP
- [x] T10 Upgrades
- [x] T11 Wave director
- [x] T12 HUD, pause, debug
- [x] T13 Victory + endless
- [x] T14 Stress test
- [x] T15 Clippy, docs, release
- [x] **CP3 Full loop**
