# Veyra: Dominion — Plan

Single source of truth for scope, architecture, task order and progress.
The MVP (sections 1–6) is complete; **V2 (section 7) is the active plan**.

## 0. Session protocol

Work happens in short sessions to save account usage. Every session starts with the same prompt and follows these rules:

1. Read **only** this file, then the **Session log** at the end, then only the source files the current session's tasks need. No broad repo exploration.
2. The current session is the first one in **8. Progress → V2 sessions** that has unchecked tasks. Do only that session's tasks, never the next session's, even if budget remains.
3. Each task follows this loop: implement → `cargo clippy -- -D warnings` and `cargo test` (trimmed output) → tick the task in Progress → commit (message from the task table in 7.12) → `git push`. One task = one commit. Commit messages carry no tool attribution or Co-Authored-By trailers.
4. Tasks marked *(user approval)* stop when they are ready to review. Show the result (PNG / WAV path or screenshot), wait for the user's OK, then commit. If there's no OK, leave the task unchecked and note it in the log.
5. Before ending, append an entry to the **Session log**: date, tasks done, decisions taken, anything left half-done, and gotchas for the next session. Keep it to 3–8 lines and commit it with the last task (or as `docs: session N notes`).
6. Record any new decision or design change in the relevant section of this file in the same commit. This file must stay enough to resume without the chat history.
7. At a checkpoint: run `cargo build --release`, put the binary size in the log and tick the checkpoint.

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

**assets.rs** (was `sprites.rs` until V1) — pixel art as embedded PNG strips in `assets/sprites/` (one file per sprite, equal-width frames side by side, frame count in the `SHEETS` table), packed at startup into one atlas `Image` → one `Texture2D` (`FilterMode::Nearest`, good batching). A white silhouette copy of every enemy frame is generated for hit flash. Ground variants are a strip too (`ground.png`).
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
WASD / arrows move · Esc pause · Enter start/confirm · 1/2/3, ↑/↓/W/S + Enter or mouse click pick upgrade · R restart · E endless (on victory) · T title · F3 debug · F10 scale mode · F11 fullscreen.

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
- Sprites: every embedded PNG decodes, every sprite rect exists inside the atlas, flash frames are white silhouettes (CPU-only decode, no texture creation).
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

## 7. V2 plan

The MVP (sections 1–6) is done. V2 turns it into a sellable game. Same workflow: one task = one commit, `cargo clippy -- -D warnings` + `cargo test` before each commit, checkpoint stops. Resume at the first unchecked V2 task in **Progress**.

### 7.1 Decisions (2026-10-18)

| Topic | Decision |
|---|---|
| Audio | Own chip synthesizer + own compositions as tracker data. No third-party audio, 100% owned, commercial-safe. |
| Art | Sprite sheets move to PNG files in `assets/`, embedded with `include_bytes!` (still a single executable). Editable in Aseprite/LibreSprite. The matrices were exported once to PNG (V1, via a throwaway test, so no `tools/` script remains); new art is drawn directly in the PNGs. Format: one horizontal strip per sprite, frames of equal width; a new sprite needs a `SpriteId` and a `SHEETS` row in `assets.rs`. |
| Veyra redesign | **Blocked until the user sends reference images.** Larger, reclining while levitating, suggestive but clothed (no nudity; Steam "suggestive" tier at most). |
| Language | English + Spanish from the start; every string goes through `i18n`. `i18n.rs` (V4): `keys!` macro defines `Key` + the `[en, es]` table; `tr(key)` / `trf(key, &[args])` (`{}` slots, filled in order); global language in an atomic, **F9** switches it until the Settings screen (V7). Tests check no empty string, equal slot counts and that every char is ASCII or in `assets::EXTRA_GLYPHS` (add new letters there). The F3 debug overlay and enemy debug names stay English (developer tool). Upgrade names are translated as they are now; the Sin renames happen in V17. |
| Monetisation | Chests ("Summoning Circles") are bought **only with in-game currency earned by playing**. Never sell them or the currency for real money (loot box laws in BE/NL and others, Steam disclosure). Drop rates are shown in-game. |
| Fonts | Only OFL fonts (embedding in a commercial game is allowed; ship the licence text and credit). **Chosen by the user (2026-10-19, after V3):** **Jacquard 24** for titles (gothic with more detail; crisp only at 43 px), **Jersey 15** for body text (crisp only at 27 px) and **Press Start 2P** at 8 px for dense text such as the debug overlay. Rejected: Jacquarda Bastarda 9 (title too crude), Silkscreen 8 and Micro 5 (hard to read), Pixelify Sans (never crisp in macroquad). TTFs + `OFL-*.txt` live in `assets/fonts/`, credited in the README. All text goes through `assets::draw_ui_text` / `measure_ui_text` with a `TextStyle` (Title 43, Body 27, Small 8); never call `draw_text` with a raw size. Body text is big for 360p, so wide layouts stack vertically (the level-up cards are now a vertical list; ↑/↓ also move the selection).
| Dependencies | Still only `macroquad`, now with `features = ["audio"]`. Saves use a hand-written `key=value` text format (no serde). |

### 7.2 Theme & nomenclature

Every name describes what a succubus is and does: seduction, sin, pacts, souls, the Abyss. All names below are English keys; Spanish goes in the i18n table.

- **Currency: Tribute** (Tributo): gold coins stamped with Veyra's sigil. Earned from kills (small chance), map objectives and an end-of-run payout scaled by time, kills, map and mode.
- **XP: Souls** (as now).
- **Permanent upgrades: Infernal Pacts** (Pactos infernales), signed at the **Altar**.
- **Pets: Familiars** (Familiares), obtained from **Summoning Circles** (Círculos de invocación), which take the place of chests.
- **Run upgrades:** weapons are **Arts** (Artes) and passives are **Sins** (Pecados).
- Existing renames: Fury → **Wrath**, Frenzy → **Bloodlust**, Ember Legion → **Legion**, Light Wings → **Night Wings**, Soul Pull → **Siren's Call**, Blood Pact stays.

### 7.3 Screen & resolution adaptation

- The logical height stays at 360 px. The logical width follows the window aspect ratio, clamped to 480–864 (4:3 up to 21:9), so wide screens see more of the world and lose the black bars.
- Scale mode setting: **Pixel-perfect** (integer scale plus thin bars) or **Fill** (fractional scale, Nearest filtering).
- Fullscreen toggle (`set_fullscreen`) plus a remembered windowed size (in memory; persisted once `settings.txt` exists, V6).
- The UI uses anchors (`hud::Anchor`: centre / right edge of the view) and never fixed 640-based coordinates. The height is always 360, so only x needs anchoring.
- **No UI scale setting (decided in S2):** body text is already 27 px of 360 (crisp only at that size), so ×1.5 / ×2 would overflow the HUD and level-up list and blur the fonts.
- `screen.rs`: `Layout::new(window, mode)` is a pure function (unit-tested on 16:9, 16:10, 21:9, 4:3 and odd sizes); `Screen` owns the render target, recreates it when the logical width changes, and gives `to_logical` for the mouse. Runtime keys until the Settings screen (V7): **F10** scale mode, **F11** fullscreen.
- The spawn ring is the view's half-diagonal + `SPAWN_RING_MARGIN`; enemies beyond ring + `RELOCATE_MARGIN` are relocated. The hash window (24×24 cells of 64) is sized for the widest view; a test checks it.

### 7.4 Input

- Menus, upgrade choice, Altar, Circles and Settings all work with **mouse** (hover selects, click confirms, wheel scrolls) **and keyboard** (arrows/WASD + Enter/Esc). The last device used decides whether the hover highlight shows.
- Shared UI layer in `ui.rs`: `Button`, `Slider`, `Toggle`, `Tabs`, `ListView`, with a single focus model. Gamepad hooks stay in the same model, to be added later.
- Optional setting for in-run movement: "move toward cursor while holding LMB" (V7, `mouse_move` in `settings.txt`, off by default). Keys win over the mouse; 6 px dead zone around Veyra.
- `ui.rs` (V5): `UiInput::read(layout)` gives one frame of input in logical coordinates (Enter/Space confirm, Esc back, arrows/WASD). `Focus::update(&input, &rects) -> Action` (`Confirm`, `Adjust(±1)`, `Pointer(frac)` while held, `Back`) is the only focus model; `Focus::highlight()` returns the focused item with the keyboard and only the hovered one with the mouse. A still mouse never takes focus from the keyboard. Widgets (`Button`, `Toggle`, `Slider`, `Tabs`, `ListView`) are logic + draw, each with a `handle(i, action, …)`; logic is unit-tested headless. Screens build their rects with a shared function (e.g. `hud::level_up_cards`) so drawing and hit-testing agree.

### 7.5 Title menu & scenes

- **Title scene:** Veyra reclines on air above a burning village silhouette. The scene has parallax layers (sky, smoke, burning roofs), rising embers, wing and tail animation and slow eye blinks. The cursor makes her glance toward the selected button. The logo is in the gothic pixel font with a flickering infernal glow, and the title theme plays.
- **Menu:** Play (map select → mode: Normal / Rush / Endless once unlocked), Altar, Summoning Circles, Familiars, Settings, Credits, Quit.
- **Settings:**
  - Audio: master, music and SFX volumes.
  - Graphics: fullscreen, scale mode, UI scale, screen shake, damage numbers, flash intensity.
  - Gameplay: language, mouse movement.
  - Controls: reference sheet.
  - Data: **reset progress**, with a two-step confirmation.
- **Credits:** scrolling, styled list. Includes design/code, art, music & SFX (own), the font licences (OFL text), macroquad (MIT/Apache notice) and "special thanks".
- Transitions are a short dissolve or ember wipe between scenes.
- **Built in V7:** `scenes/` holds `App` (owns settings, screen, save, the current `Run` and the `Scene` enum: Title / Run / Settings / Credits). Scenes return a `Nav` (Title, NewRun, ResumeRun, Settings, Credits, Quit) and `App::navigate` applies it; the run is ended (stats recorded) when it is replaced. Outside a run a fresh idle `Run` is the backdrop. Transition = 0.25 s fade from black (the ember wipe can come with V14).
  - Title menu is Play / Settings / Credits / Quit for now; Altar, Circles, Familiars and map/mode select join with their tasks (V20+, V23).
  - Pause, game over and victory are button menus (`ui::button_column`); the old R / E / T keys are gone. Esc in the pause menu resumes; Settings opened from pause returns to it.
  - Settings tabs: Audio (3 sliders, inert until V9), Video (fullscreen, scaling), Game (language, hold-click-to-move), Keys (controls sheet), Data (lifetime stats + reset with a second press to confirm; Esc or moving away disarms). Screen shake, damage numbers and flash intensity join the Video tab in V29 with the features themselves. Settings are written when the screen is left. F9/F10/F11 still work everywhere.
  - Credits scroll on their own after 2 s (wheel/arrows take over); they list the author (**Adrià Bonnin Catalán**, `AUTHOR` in `credits.rs`), the fonts, macroquad and a line pointing to the licence folder. The full OFL and MIT texts are **not** shown on screen (the user found them made the credits far too long): V30 must ship `assets/fonts/OFL-*.txt` and `assets/licenses/` in a `licences/` folder next to the executable.

### 7.6 Art restyle

- **Veyra:** about 48×32 reclining body (on her side, levitating) plus separate wings, tail and hair layers for secondary motion. Frames: idle float (4), move lean (4), cast (3), hurt (2), death dissolve (6). Bigger shadow and a subtle red rim light. Draw order and pivot are tuned so she reads clearly among 2,500 enemies.
- **Enemy sizes vary by type:** small (rat 12 px, bat 12), medium (villager 20, monk 20), large (knight 26, paladin 28), brutes 40, bosses 64–96. `EnemyDef` gets `size` and `radius` from the sprite. The spatial queries add the largest radius in play.
- **Effects:** an outline on elites, death-burst particles in each enemy's palette, ember trail on projectiles and hit-stop of 1–2 frames on big hits.
- **Ground:** each map has its own tileset plus scattered props (fences, graves, pews, banners, clouds). Props are not walls: they don't block movement.
- **Palette rule:** Veyra and her Arts own the warm reds, magentas and golds. Enemies and maps use colder or muted palettes so the player always reads her first.

### 7.7 Arts (weapons) and Sins (passives)

Modelled on the Vampire Survivors structure: up to **6 Arts + 6 Sins**, Arts reach level 8, and an Art **evolves** when it is at max level, its paired Sin is owned and a boss chest ("Abyssal Offering") is opened.

| Art | Archetype (VS analogue) | Behaviour | Evolution (+ Sin) |
|---|---|---|---|
| Hellflare | Magic Wand | Homing fireballs at the nearest enemy | **Hellstorm** (+ Legion): rain of fireballs |
| Tail Lash | Whip | Horizontal sweep on the facing side; alternates sides | **Barbed Scourge** (+ Wrath): bleed damage over time |
| Kiss of Ruin | Knife | Fast piercing heart bolt in the move direction | **Kiss of Death** (+ Blood Pact): lifesteal on hit |
| Brimstone Halo | King Bible | Orbiting embers | **Infernal Crown** (+ Dark Sigil): permanent ring |
| Allure | Garlic (area) plus a succubus twist | Aura that **charms** enemies: they fight for Veyra for a few seconds | **Siren's Dominion** (+ Siren's Call): charmed enemies explode on expiry |
| Hellgate | Santa Water | Burning pools at random spots near enemies | **Abyssal Rift**: pools pull enemies in |
| Chain of Torment | Lightning Ring | Chain lightning between N enemies | **Torment Eternal** (+ Bloodlust): unlimited bounces in a radius |
| Wing Gust | Clock Lancet / control | Cone knockback with a short slow | **Tempest of Wings** (+ Night Wings): pushes and damages |

| Sin | Effect / level | Max |
|---|---|---|
| Wrath | +10% damage | 5 |
| Bloodlust | −8% cooldown | 5 |
| Legion | +1 projectile / instance | 2 |
| Night Wings | +10% speed | 5 |
| Blood Pact | +20 max HP, heal 30 | 5 |
| Siren's Call | +40% pickup radius | 5 |
| Dark Sigil | +10% duration | 5 |
| Allurement | +10% area | 5 |
| Greed | +20% Tribute | 5 |
| Infernal Hide | +1 armour | 5 |

On level-up the choice offers 3 (4 with a Pact) mixed Arts/Sins. Banish and Reroll charges come from Pacts.

### 7.8 Enemies, difficulty, maps and story

- New behaviours on top of `Chase`:
  - `Ranged { keep_dist, shot }`
  - `Charger { windup, dash_speed }` (with a telegraph)
  - `Support` (aura/heal, the old Priest design)
  - `Swarm` (bats: sine motion)
  - `Shielded` (blocks frontal projectiles)
  - `Flyer` (ignores separation)
- **Elites:** a random tinted variant, ×5 HP, drops an Offering. **Bosses** use a dedicated `Boss` struct with a phase state machine (as in the old Archangel design).
- **Difficulty curve:** each map defines its own spawn curve, an HP multiplier per minute and event table, with swarm and ring events like VS (encircling wall, stampede line).
- **Map clear:** survive the timer **and** kill the final boss. Clearing a map unlocks the next map and that map's **Endless** mode.
- **Rush mode:** the clock runs ×2 (a 10-minute map lasts 5 real minutes). Spawn rate and enemy HP/speed are scaled up and Tribute is ×1.5. Unlocked per map after a Normal clear.

| # | Map | Objective / length | Enemies | Boss |
|---|---|---|---|---|
| 1 | **Ashford Village** (the current map, reworked) | Survive 10:00, kill the boss | Rat, Hound, Villager, Torchbearer (ranged), Knight, Brute farmer (large) | **Captain of the Watch**: shield charge, calls militia |
| 2 | **Abbey of Saint Veil** | Survive 12:00 and break 3 holy seals spread across the map | Bats, Monk, Nun (holy water, ranged), Priest (support), Inquisitor (elite) | **Grand Inquisitor**: chains, purifying circles |
| 3 | **Bastion of Aurum** (royal city) | Survive 15:00, defend nothing, kill the boss | Crossbowman, Paladin (shielded), Cavalry (charger), War mage | **The Lion King-Paladin** |
| 4 | **The Celestial Gate** | Survive 15:00 | Cherub (swarm), Seraph (flyer, ranged), Angelic host | **Archangel** (3 phases: feathers, charge, light pillars) |

- **Story (subtle):** Veyra was a succubus bound for centuries under the Abbey by the Church, and her seal breaks in Ashford.
  - Each map opens with a 2–3 line card in Veyra's voice. Bosses get one line each on arrival and on death.
  - Item and Familiar descriptions carry lore fragments. After each clear a "Memory" unlocks in the Codex.
  - The ending after the Archangel is a short illustrated sequence: she claims dominion.

### 7.9 Meta-progression (roguelike)

- **Infernal Pacts:**
  - Pacts: Might, Armour, Max HP, Recovery, Cooldown, Area, Speed, Duration, Amount, Magnet, Growth, Greed, Luck, Revival, Reroll, Banish.
  - Each pact has ranks, and its cost rises with the rank and with the total number of pacts bought (the VS PowerUp formula).
  - A full refund is always available.
- **Summoning Circles & Familiars:** one Familiar equipped per run. It follows Veyra, attacks and gives a passive.
  - The first copy of a Familiar unlocks it at rank 1. Duplicates raise its rank up to 5 (dupes needed: 1, 2, 3, 4).
  - Duplicates beyond rank 5 refund Tribute.
  - Higher circles give stronger Familiars.

| Circle | Cost | Familiars |
|---|---|---|
| Lesser Circle | 500 | Bat (bites, +5% speed), Imp (throws small embers), Black Cat (+Luck), Crow (picks up far souls) |
| Greater Circle | 2,000 | Hellhound (charges, burns), Gargoyle (+armour, stone shield), Will-o'-wisp (slows, +area), Shadow Serpent (poison pools) |
| Abyssal Circle | 8,000 | Cerberus Pup (triple bite), Lesser Drake (fire breath cone), Lilim (fellow lesser succubus, charms), Nightmare (shadow steed, aura) |

- Rates are listed in the Circle screen, and the rolls are tested for distribution.
- **Codex:** seen enemies, Arts, Familiars and Memories.
- **Save file:** `$XDG_DATA_HOME/veyra-dominion/save.txt` (Linux) or `%APPDATA%\veyra-dominion\save.txt` (Windows). It has a version field and is written atomically (tmp + rename). Settings go in a separate `settings.txt`.
  - `save.rs` (V6): `Settings` (lang, scale mode, fullscreen, windowed size, master/music/sfx volumes) and `SaveData` (runs, total kills, best time; meta fields join in V20+). `to_text` / `from_text` are pure and tested; unknown keys and bad values keep the default, so older and newer files load. `version=` is written but ignored until a key changes meaning (then add a migration in `from_text`). macOS uses `~/Library/Application Support`.
  - Settings are written on every F9/F10/F11 toggle and on quit (`prevent_quit` + `is_quit_requested`); the windowed size is final only at quit. `window_conf` reads the settings for the initial size/fullscreen. A run is recorded in `SaveData` when it is replaced (restart/title) or the app quits, if `game.time > 0`. Write failures are only printed, never fatal.

### 7.10 Audio design (own chip synth)

**Synth (`audio/synth.rs`).** NES-like core extended with 16-bit touches. It renders to PCM, deterministically, and is unit-tested.
- Voices:
  - 2 pulse voices (duty 12.5/25/50/75%, vibrato, pitch sweep)
  - 1 triangle bass
  - 1 noise voice (long/short LFSR, for drums)
  - 1 4-bit **wavetable** voice (Game Boy wave channel style, for organ and choir colours in the Abbey and Celestial maps)
- Per-instrument macros (volume/arp/pitch/duty envelopes, as in FamiTracker) plus a light SNES-style echo.
- Each track is pre-rendered to WAV bytes at load and played with macroquad's audio (`load_sound_from_bytes`, looped). If dynamic layering is needed later, the plan is to switch to a real-time callback.

**Tracks (`assets/music/*.vtrk`).** Plain-text tracker patterns, reviewable in git. Each track has its own identity, so they don't all sound alike:
- **Veyra's leitmotif:** a hook built on a minor 2nd plus a tritone. It appears on the title screen, the victory jingle, and in variations in each map theme.

| Track | Key / mode | Tempo / feel | Colour |
|---|---|---|---|
| Title "Dominion" | D Phrygian dominant | 84 BPM, slow 6/8 sway | Seductive pulse lead, triangle drone, sparse echo |
| Ashford | E harmonic minor | 150, driving 4/4 | Galloping bass ostinato, Castlevania-style energy |
| Abbey | C Dorian vs C Phrygian | 120 | Corrupted plainchant on the wave channel against a pulse counter-melody |
| Bastion | G minor march | 132 | Snare-heavy noise channel, brass-like 25% pulses |
| Celestial Gate | Lydian clashing with the leitmotif | 140 | Bright arps fighting Veyra's dark motif |
| Boss | Per map, chromatic | +10 BPM | — |
| Jingles | — | — | Level-up, victory, game over, summon reveal (per rarity) |

**How the music is made:**
- Study NES/SNES technique: Castlevania III (VRC6), Mega Man 2, Ninja Gaiden, Vampire Survivors' own "Castlevania pastiche" style.
- Never copy melodies.
- Every track is exported to WAV with `cargo run --example render_audio` so the user can listen and **approve each track before integration**.

**SFX (`audio/sfx.rs`).** sfxr-style parameter presets rendered by the same synth:
- Combat: fire per Art, hit, kill per enemy family, soul pickup (pitch rises with combo), player hurt.
- Progression: level-up, coin.
- UI: move, confirm, back.
- Events: summon (by rarity), boss warning.

**Mixer.**
- Per-sound voice cap (e.g. ≤4 simultaneous hits) and ±pitch jitter.
- Ducking under jingles.
- Volumes from Settings.

### 7.11 Architecture changes

- **Modules:**
  - `assets.rs` (embedded PNGs + atlas rects manifest)
  - `ui.rs` (widgets/focus)
  - `scenes/` (title, map_select, run, settings, credits, altar, circles, codex)
  - `i18n.rs` (`Key` enum → `[&str; 2]`; a test fails if any string is empty)
  - `save.rs`, `meta.rs` (pacts, tribute, familiars)
  - `weapons.rs` (Arts as data + per-archetype update fn)
  - `familiars.rs`, `maps.rs` (`MapDef { ground, props, enemies, events, objective, music, duration }`)
  - `audio/` (synth, sfx, music, tracks)
- `config.rs` keeps the balance numbers but is split per domain if it grows past about 600 lines.
- The simulation stays headless and testable. Audio and visual feedback leave the sim as `SimEvent`s or a per-tick event buffer that `main` consumes, so the sim never calls the audio functions.
- `Game::new(map, mode, loadout)` takes the run parameters. Meta bonuses are applied as base stats.
- Hot path: Arts use preallocated pools per archetype, there are still no allocations per tick, and the stress test is extended to all Arts at max level plus 2,500 enemies.

### 7.12 V2 tasks

| # | Task | Done when | Commit |
|---|---|---|---|
| V1 | Asset pipeline: export current matrices to PNG (`tools/`), `assets.rs` loads embedded PNG atlas; matrices removed | Game looks identical, test checks every sprite rect exists | `feat: png asset pipeline` |
| V2 | Adaptive resolution: variable logical width, pixel-perfect/fill, fullscreen, anchored HUD, resize-safe target | Correct on 16:9, 16:10, 21:9, 4:3 | `feat: adaptive resolution and anchored ui` |
| V3 | Fonts (OFL gothic title + readable body), licence files in `assets/fonts/` | All text uses the new fonts | `feat: pixel gothic fonts` |
| V4 | `i18n.rs` EN/ES, every existing string migrated, language switch | Completeness test green | `feat: english and spanish localisation` |
| V5 | `ui.rs` widgets with mouse + keyboard focus; level-up screen uses it | Upgrades pickable by mouse and keys | `feat: mouse and keyboard ui layer` |
| V6 | `save.rs` + settings file, atomic writes, versioning + tests | Round-trip tests green | `feat: save and settings files` |
| V7 | Scene system + Settings screen (audio sliders inert for now, graphics, language, reset progress) + Credits | Navigable with mouse/keys | `feat: scenes, settings and credits` |
| **CP4** | **Checkpoint 4 — Foundations** | | |
| V8 | Chip synth core + WAV export example + tests | `render_audio` writes test tones | `feat: chip synthesizer core` |
| V9 | SFX presets + mixer (caps, jitter), wired to SimEvents | Sounds in game, volumes work | `feat: sound effects and mixer` |
| V10 | Tracker format + title theme & Ashford theme → **user listening approval** | User approves both | `feat: tracker music and first themes` |
| V11 | Remaining music + jingles | User approves | `feat: full soundtrack` |
| **CP5** | **Checkpoint 5 — Audio** | | |
| V12 | **Veyra redesign (needs reference images)**: proposals as PNG for approval, then reclining sprite set + animation layers | User approves, in game | `feat: veyra redesign` |
| V13 | Enemy restyle with sized sprites, `EnemyDef.size`, hash radius fix, death bursts, elite outline | Stress test still green | `feat: enemy restyle with variable sizes` |
| V14 | Animated title scene (Veyra, parallax village, embers, glowing logo, music) | User approves look | `feat: animated title scene` |
| **CP6** | **Checkpoint 6 — Look & feel** | | |
| V15 | Arts framework (6 slots, levels, pools) + Tail Lash, Kiss of Ruin, Brimstone Halo | Tests + visible | `feat: arts framework and first arts` |
| V16 | Allure (charm), Hellgate, Chain of Torment, Wing Gust | Tests + visible | `feat: remaining arts` |
| V17 | Sins table, renames, mixed level-up choices, evolutions + Offerings | Evolution tests green | `feat: sins and evolutions` |
| V18 | New behaviours (Ranged, Charger, Support, Swarm, Shielded, Flyer) + hostile shots + elites | Behaviour tests green | `feat: enemy behaviours and elites` |
| V19 | Boss framework + Captain of the Watch | Boss beatable | `feat: boss framework and first boss` |
| **CP7** | **Checkpoint 7 — Combat depth** | | |
| V20 | Tribute currency, end-of-run payout screen, Altar + Pacts (refund) | Meta tests green | `feat: tribute and infernal pacts` |
| V21 | Summoning Circles + Familiars (4 Lesser), equip, ranks, duplicate refund, rate display | Distribution tests green | `feat: summoning circles and familiars` |
| V22 | Greater + Abyssal Familiars | Visible + tested | `feat: greater and abyssal familiars` |
| **CP8** | **Checkpoint 8 — Meta** | | |
| V23 | `MapDef`, map select, objectives, Ashford reworked (10:00 + boss), Endless per map | Clear unlocks next | `feat: maps and objectives` |
| V24 | Rush mode | Tests on scaling | `feat: rush mode` |
| V25 | Abbey of Saint Veil (+ seals objective, enemies, boss, tileset) | Clearable | `feat: abbey of saint veil` |
| V26 | Bastion of Aurum | Clearable | `feat: bastion of aurum` |
| V27 | Celestial Gate + Archangel + ending | Clearable | `feat: celestial gate and ending` |
| V28 | Story cards, boss lines, Memories, Codex | All text EN/ES | `feat: story and codex` |
| **CP9** | **Checkpoint 9 — Content** | | |
| V29 | Balance pass, juice (hit-stop, shake, damage numbers), accessibility (flash/shake toggles) | Playtest notes in README | `feat: balance and game feel pass` |
| V30 | Release: Windows + Linux builds, icon, `strip = true`, licences bundle (`licences/` folder: OFL texts + macroquad MIT, the credits point to it), README/press kit | Builds run on both | `chore: release packaging` |
| **CP10** | **Checkpoint 10 — Release candidate** | | |

### 7.13 V2 risks

| Risk | Mitigation |
|---|---|
| Music sounds generic or samey | Per-track mode/tempo/colour table, leitmotif rules, user listens to WAV exports before integration |
| Gacha legal exposure | Earned-only currency, rates shown, no real-money path ever |
| Suggestive art vs store rules | Clothed, no nudity, review proposals with the user before integration |
| Performance with more Arts and bigger enemies | Pools per Art, extended stress test, hash radius includes max enemy radius |
| Font licence | OFL only, licence files shipped and credited |
| Scope creep | Checkpoints are stop points. Content (maps 3–4, Abyssal Familiars) can slip without breaking the game |

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

### V2 sessions

- **S1 — Assets & fonts**
  - [x] V1 Asset pipeline
  - [x] V3 Fonts
- **S2 — Screen & input**
  - [x] V2 Adaptive resolution
  - [x] V5 Mouse + keyboard UI layer
- **S3 — Text & data**
  - [x] V4 English + Spanish localisation
  - [x] V6 Save and settings files
- **S4 — Menus**
  - [x] V7 Scenes, settings and credits
  - [x] **CP4 Foundations**
- **S5 — Synth**
  - [ ] V8 Chip synthesizer core
- **S6 — SFX**
  - [ ] V9 Sound effects and mixer
- **S7 — First music** *(user approval)*
  - [ ] V10 Tracker format + first themes
- **S8 — Soundtrack** *(user approval)*
  - [ ] V11 Remaining music + jingles
  - [ ] **CP5 Audio**
- **S9 — Veyra** *(user approval; ask for the reference images at the start)*
  - [ ] V12 Veyra redesign
- **S10 — Enemies look**
  - [ ] V13 Enemy restyle with variable sizes
- **S11 — Title screen** *(user approval)*
  - [ ] V14 Animated title scene
  - [ ] **CP6 Look & feel**
- **S12 — Arts I**
  - [ ] V15 Arts framework + first Arts
- **S13 — Arts II**
  - [ ] V16 Remaining Arts
- **S14 — Sins**
  - [ ] V17 Sins and evolutions
- **S15 — Enemy AI**
  - [ ] V18 Enemy behaviours and elites
- **S16 — First boss**
  - [ ] V19 Boss framework + first boss
  - [ ] **CP7 Combat depth**
- **S17 — Pacts**
  - [ ] V20 Tribute and Infernal Pacts
- **S18 — Familiars**
  - [ ] V21 Summoning Circles + Lesser Familiars
  - [ ] V22 Greater + Abyssal Familiars
  - [ ] **CP8 Meta**
- **S19 — Maps & modes**
  - [ ] V23 Maps, objectives and per-map Endless
  - [ ] V24 Rush mode
- **S20 — Abbey**
  - [ ] V25 Abbey of Saint Veil
- **S21 — Bastion**
  - [ ] V26 Bastion of Aurum
- **S22 — Celestial Gate**
  - [ ] V27 Celestial Gate + Archangel + ending
- **S23 — Story**
  - [ ] V28 Story and Codex
  - [ ] **CP9 Content**
- **S24 — Polish**
  - [ ] V29 Balance and game feel pass
- **S25 — Release**
  - [ ] V30 Release packaging
  - [ ] **CP10 Release candidate**

## 9. Session log

- **S0 (2026-10-18)** — V2 planned: decisions in 7.1 (own chip synth, PNG assets, EN+ES, earned-only gacha, OFL fonts). The MVP release binary is 1.14 MB (0.97 MB stripped). The user will send Veyra reference images for S9. Next: S1.
- **S1 (2026-10-18)** — V1 + V3 done. V1: sprites are PNG strips in `assets/sprites/` loaded by `assets.rs` (was `sprites.rs`); the exported atlas was verified byte-identical to the old matrix atlas before the matrices were deleted. V3: Jacquarda Bastarda 9 + Silkscreen, see 7.1 for sizes and why Pixelify was dropped.
  - Gotcha: `Texture2D::get_texture_data()` on the render target returns the image vertically flipped, and it only contains the UI after the batch is flushed (call it after `present`). Useful for screenshot checks with a temporary frame-counter hack in `main`.
  - Gotcha: macroquad text glyphs are rasterised per size; `assets::init` pre-populates ASCII + Spanish letters for every `TextStyle`, so new styles must be added to `TextStyle::ALL`.
  - Body text is 8 px Silkscreen (small caps look); V2's UI scale setting is the answer if it reads too small on big screens. Next: S2.
- **S1 follow-up (2026-10-19)** — The user found the fonts unreadable (the title gothic, then the small body text). Swapped to Jacquard 24 / Jersey 15 / Press Start 2P (see 7.1), level-up cards stacked vertically. To check which sizes of a font are crisp, render a test string at every integer size into a render target and count grey (anti-aliased) pixels; only sizes with ~0 grey are usable.
- **S2 (2026-10-19)** — V2 + V5 done. V2: `screen.rs` (`Layout` pure + tested, `Screen` owns the target); logical width 480–864, F10 scale mode, F11 fullscreen; spawn ring and relocate radius derive from the view (`Game::set_view`), hash window now 24×24. Decided to drop the UI scale setting (see 7.3). V5: `ui.rs` focus model + widgets; level-up cards are clickable/hoverable.
  - Gotcha: `Layout` offsets can come out as -1 from float error (360 × 2.1333 > 768); they are clamped at 0.
  - `ui.rs` has a module-level `allow(dead_code)` until V7 uses Slider/Toggle/Tabs/ListView; remove it then. "Move toward cursor" setting moved to V7. Windowed size is remembered only in memory until V6. Next: S3.
- **S3 (2026-10-19)** — V4 + V6 done. V4: `i18n.rs` (see 7.1), F9 switches EN/ES; debug overlay stays English. V6: `save.rs` (see 7.9); settings now persist language, scale mode, fullscreen and window size.
  - Checked by launching with `XDG_DATA_HOME` pointing at a temp dir holding a Spanish `settings.txt`: title in Spanish, window 960×540. Screenshots of the real window work with `spectacle -b -n -a -o <png>` (Wayland/KDE).
  - Gotcha: `Key::ALL` is `cfg(test)` only (used by the completeness test); new strings just go in the `keys!` table. Volumes in `Settings` are stored but inert until V9.
  - Leaving fullscreen with F11 stores the fullscreen size as windowed for that frame; quitting rewrites it correctly. Next: S4.
- **S4 (2026-10-19)** — V7 + CP4 done. V7: `scenes/` (`App`, `Nav`; title, run, settings, credits), see 7.5; `main.rs` is now only the window setup and loop. The pause / game over / victory screens are button menus and R/E/T are gone. Hold-click movement setting added (7.4).
  - CP4: release binary 1.95 MB (1.75 MB stripped), up from 1.14 MB mostly because of the embedded TTFs and licence texts.
  - Fixed a V5 bug: `f32::signum(0.0)` is 1.0, so `UiInput.wheel` reported "up" every frame; it now checks for 0 first.
  - Screens were checked with a temporary `VEYRA_SHOT` env hack in `App::new` (start in a given scene/tab) + `spectacle`; not committed. Follow-up: author is "Adrià Bonnin Catalán" (`à`/`À` added to `EXTRA_GLYPHS`); the on-screen licence texts were dropped as too long, see 7.5.
  - `ui::ListView` still has a local `allow(dead_code)` until the Altar/Codex lists (V20+). Next: S5 (V8 chip synth).
