# Veyra: Dominion — V2 design (section 7)

Design decisions per domain. Section numbers are kept so "see 7.x" references stay valid. The task table (7.12) lives in `PLAN.md`. Read only the subsection the current task needs; record new decisions in it.

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
  - Settings tabs: Audio (master and SFX live since V9, music since V10), Video (fullscreen, scaling), Game (language, hold-click-to-move), Keys (controls sheet), Data (lifetime stats + reset with a second press to confirm; Esc or moving away disarms). Screen shake, damage numbers and flash intensity join the Video tab in V29 with the features themselves. Settings are written when the screen is left. F9/F10/F11 still work everywhere.
  - Credits scroll on their own after 2 s (wheel/arrows take over); they list the author (**Adrià Bonnin Catalán**, `AUTHOR` in `credits.rs`), the fonts, macroquad and a line pointing to the licence folder. The full OFL and MIT texts are **not** shown on screen (the user found them made the credits far too long): V30 must ship `assets/fonts/OFL-*.txt` and `assets/licenses/` in a `licences/` folder next to the executable.

- **V14 decisions (S11, with the user):** a still "animated painting", not a generic tiling backdrop. Worked in phases, one session each, user approval at the end of every phase: (1) composition blockout ✅, (2) Veyra portrait, (3) painted backdrop, (4) logo + buttons, (5) game integration (subtle parallax, embers, ember wipe on **every** scene change), then CP6.
  - Composition (blockout source `assets/refs/title_blockout.px.js`, approved): logo top left, "DOMINION" a bit bigger under it, 4 buttons (~168 px wide) below; a leaning grave cross on the foreground crest (bottom left), no dead tree. Seen from a hill: the razed village (Ashford) sits in the valley, right of centre and **behind Veyra**, small and secondary. Smoke rises only from the village and passes behind her, darkening parts of the moon so she stands out. Night: she razed it by day and is now leaving; ruins are black with exposed beams, red embers in the rubble, 2–3 fires still alive.
  - **Portrait, update (S11, after two rejected code-drawn heads):** the user wants it to look **like the reference**, strictly (pose, face placement, expression: mature, dominant, seductive; head tilted, face 3/4; skin redder; the ear visible as in the reference, *under* the hair, **no earring**). Size: full height wins: horns at the top to the **waist at the bottom edge**, scale ≈0.67 of the reference (ref y≈40 horns → y≈540 waist ≈ 330 px), so she is ~300 px wide without wings (x≈340–640, the raised arm reaches the centre). Method chosen: **trace the reference**: downscale to the final size, quantise to a closed Veyra palette (~32 colours), then clean up with code (outline, eyes, mouth, highlights, noise). Draw the **whole body at once** (next session); only if that fails, piece by piece.
    - Segmentation notes: the moon separates by colour (`G/max < 0.33 && B/max < 0.42`), the skin doesn't (G/max > 0.36). Hair and wings are both purple, so they need a hand polygon; the horns overlap the moon and need one too. Wings are deferred: at 0.67 they would span from x≈175 and hide the village; decide their size/crop with the user.
    - **Phase 2, first try (whole at once, automated) failed:** `assets/veyra_portrait_trace.py` cuts her out (hand polygons in ref coords that work: `FIGURE`, `HORN_L/R`, `PROTECT`, `POCKET` = moon between forearm and shoulder, `ARM`, `BODICE`, `CHOKER`, `FACE`) and writes `assets/veyra_portrait.trace.js`, included by `assets/veyra_portrait.px.js` (out: `assets/sprites/title`). Downscale + k-means read as a pixelated photo (user: "no quiero que me pixeles la imagen, quiero que la dibujes en estilo pixel art"); per-material ramps + cluster smoothing came out blobby; automatic lineart (black-hat + thinning) was broken noise. Lesson: the trace is only a **guide** for shapes and placement; the art must be drawn (contours, flat ramps, hand shading clusters, hand eyes/mouth). Superseded by the AI pipeline below.
    - **Phase 2, AI pipeline (S11b, 2026-10-20, user: "buen trabajo"):** the user chose to stop hand/code drawing and use a local diffusion model. `assets/pixelize.py` (run with the fixer's venv, see its docstring) crops the input (`--crop veyra` = `CROP` from the trace script), resizes it to ~1 MP, asks **Qwen-Image-Edit-2511 fp8** in the user's ComfyUI (`~/comfyui`, `./start-comfyui.sh`, API on :8188; models were already there, no extra LoRA: none exists for Edit-2511 pixel art) to redraw it as pixel art on flat green, then **Pixel Art Fixer** (Retro Diffusion, MIT, `~/.local/src/pixel-art-fixer/python`, own `.venv`; the Rust build only links on Windows; a `detector -> pixelfixer` symlink fixes its stale imports) rebuilds the grid; green is keyed out and colours snapped to the closed portrait palette (`RAMPS`, 32 colours). Per seed it writes `ai_`, `native_` (detected grid) and `forced_` (grid forced to `--size`) PNGs + `_x4` previews.
      - Findings: very faithful to the reference (pose, face, hand, horns, corset, ear). `--fast` (Lightning, 8 steps) ≈ 72 s per image on the 4060 Ti. At ~1 MP Qwen draws ~4 px art pixels, so the **native** result is ~277×221 for the whole crop (with wings); seeds vary (one gave a clean 4 px grid, another a useless 27 px one: generate several and pick). **Forced** to 456×360 is 2.5 px per cell and reads as a pixelated photo again: don't use it. At 1.8 MP the model loses the grid (smeared pixels): stay at ~1 MP.
      - **In game (trial):** `assets/sprites/title/veyra_portrait_ai.png` (native, seed 1324928684, fast mode) is its own texture (`assets::title_portrait`, too big for the 512² atlas), drawn bottom right over the dim; the title text and buttons are centred in the space left of it (`menu_view` in `scenes/title.rs`). Not cleaned up by hand yet (orphan pixels, edge fringe, eyes).
      - Open, for the next session (try alternatives): size (native ~220 px tall vs the planned ~330; options: accept native, crop tighter without wings, or generate in two halves and stitch), full-quality runs (40 steps, several seeds), prompt variants, other reference images, palette `veyra` vs free `N` colours, then a light hand cleanup and the remaining phases (3 backdrop, 4 logo + buttons, 5 integration).
    - `assets/veyra_title.px.js` holds the rejected code-drawn head (pen/rotation helpers may still help for cleanup); `assets/title.px.js` holds the approved "VEYRA" logo plus a rejected tiling backdrop (to be replaced in phase 3).
  - **Veyra portrait (first plan, superseded above):** new art at 1:1 (~200×330), waist up, right third of the screen, full height with margins, in front of everything. Pose and style from the user's reference `~/Downloads/veyra-titile-pose.png` (copied to `assets/refs/veyra_title_pose.png`): hand at the cheek, sultry smile, corset with gold filigree, very long violet hair, wings spread, horns. Must read seductive (she is a succubus, even if evil). Draw with geometry + shading code, not hand-typed matrices; symmetric eyes.
  - **Moon:** huge **blood-red** moon (as in the reference), partly behind Veyra's head and shoulder.
  - **Logo:** "VEYRA" in hand-drawn engraved Roman capitals (`assets/title.px.js`, approved as text); Jacquard 24 read as "Negra" and is dropped for the logo. "DOMINION" needs redoing (the user didn't like it) and slightly bigger. Buttons must be restyled to look premium and read clearly (the current size reads poorly).

### 7.6 Art restyle

- **Veyra:** lounging on the air as if seated (pose chosen by the user in S9: face to the viewer with both eyes visible, legs crossed with the knees to one side, wings spread on both sides; *not* a side view lying down) plus separate wings, tail and hair layers for secondary motion. Frames: idle float (4), move lean (4), cast (3), hurt (2), death dissolve (6). Bigger shadow and a subtle red rim light. Draw order and pivot are tuned so she reads clearly among 2,500 enemies.
  - **Source (S9):** `assets/veyra.px.js`, drawn with the pxl skill (`node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/veyra.px.js --out assets/sprites/veyra`). Strips in `assets/sprites/veyra/anims/`: `veyra_idle` (4), `veyra_move` (4), `veyra_cast` (3, once), `veyra_hurt` (2, once), `veyra_death` (6, once, all layers burning away), `veyra_wings` (4), `veyra_tail` (4), `veyra_hair` (4) and `veyra_hair_move` (4, hair streaming back). Each frame is 66×50 and every layer shares the origin, so they stack directly; pivot (31, 36), under her hips; she faces left. Draw order: wings, tail, hair, body. Frame durations are in the `.json` next to each strip. The body leaves the head/hair seam without outline so head and hair read as one mass.
- **Enemy sizes vary by type:** small (rat 12 px, bat 12), medium (villager 20, monk 20), large (knight 26, paladin 28), brutes 40, bosses 64–96. `EnemyDef` gets `size` and `radius` from the sprite. The spatial queries add the largest radius in play.
  - **Source (S10):** `assets/enemies.px.js` (pxl), approved by the user. Rebuild: `node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/enemies.px.js --out assets/sprites/enemies --no-preview`. Strips of 2 walk frames in `assets/sprites/enemies/anims/` (the atlas adds the white flash copies): rat 17×9, villager 14×20, knight 18×26, brute farmer 32×40 (outline included; the rat's 12 is its body length, the tail is extra). Side views facing left, feet on the bottom row; the game draws them bottom-centre on their position, with the shadow scaled to the sprite width. Cold/muted palette and a blue-black outline (`#10131c`), never Veyra's plum.
- **Effects:** an outline on elites, death-burst particles in each enemy's palette, ember trail on projectiles and hit-stop of 1–2 frames on big hits.
- **Built in V13:**
  - `EnemyDef.size` is the sprite's frame size (an `assets` test checks it against the strip). `radius()` derives from it (`1 + 0.14·(w + h)`, rounded: 5 / 6 / 7 / 11, the old hand-tuned values) and so does `burst_count()` (6–10 by area). `MAX_ENEMY_RADIUS` is gone: `Enemies` counts live enemies per kind and `max_radius()` (largest radius in play) widens the contact and projectile queries.
  - Death: a 0.12 s white flash (`DEATH_FADE_TIME`) plus a burst of 2×2 px bits (`BURST_BIT`; single pixels got lost in the grass speckle) in the light end of the enemy's ramps (`EnemyDef::burst`), thrown away from Veyra from mid-body height; they fall, bounce once on a hard landing and fade on the ground (`BURST_*`). Pool of `MAX_BURST_PIXELS`, never grown. Landed bits draw under the horde, flying ones over it.
  - Blood (`EnemyDef::blood`, on for all four; off for future skeletons or constructs): half as many dark red drops join the burst (`BLOOD`, jump ×0.6), and a pool is left where the enemy fell. Colours are dark and brownish on purpose (user: "más oscuro"), well apart from Veyra's crimsons.
  - Blood is **permanent** (user's goal: the ground shows how many Veyra has killed). `Game` queues `Splat`s (pools and drop splashes, capacity `MAX_SPLATS`); `blood::BloodLayer` in the run bakes them once into 128 px world chunks (CPU `Image` → texture, drawn right after the ground), so thousands of pools cost the same as one. Past `MAX_BLOOD_CHUNKS` (256) the chunk seen longest ago is dropped.
  - Pool shapes come from a seed (`blood::splat_pixels`): small main lobe, 2–4 loose blobs, sometimes a broken trail, droplets, dark rim and a wet glint, shifted off the body so piles of deaths spread out ("más dispersos"). A pool landing on one of the last `MAX_POOL_SITES` pools feeds it instead (area adds up, same seed repainted larger) up to `POOL_MAX_RADIUS`; past that only splashes, so killing grounds read as more blood in one place.
  - Elites: `Enemies::elite` flag; a 1 px `ELITE_OUTLINE` ice-blue ring (the white flash frame tinted at 4 offsets) that breathes gently. Real elites (stats, spawns) are V18.
  - Debug keys in a run: F4 spawns a group of every kind, F5 turns enemies within `DEBUG_ELITE_RADIUS` (90 px) into elites, F6 kills every enemy on screen (burst, blood and soul as usual).
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
| Hellgate | Santa Water | Burning pools at random spots near enemies | **Abyssal Rift** (+ Allurement): pools pull enemies in |
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
| Greed | +20% Tribute (joins with Tribute in V20) | 5 |
| Infernal Hide | +1 armour | 5 |

On level-up the choice offers 3 (4 with a Pact) mixed Arts/Sins. Banish and Reroll charges come from Pacts.

**Built in V15 (S12):**
- `src/arts.rs` (not `weapons.rs`): `Arts` owns the fixed pools (fireballs, lashes, hearts) and the Halo state, the cooldowns and the effective stats; `Game::tick` calls `arts.update(World { player, enemies, hash, time }, alive)` where Hellflare used to fire. Levels live in `Upgrades` (`arts[i]`, 0 = not owned, plus `slots`, 6 max); every run starts with Hellflare at level 1. Call `arts.refresh(&upgrades)` after any change.
- Data in `config/arts.rs` (`config.rs` became `config/mod.rs` once it passed 600 lines): `ArtDef { base: ArtStats, gains: [ArtGain; 7] }`, `ArtDef::stats(level)`. `ArtStats` fields mean different things per Art (see its doc: e.g. the Halo's `cooldown` is its rest time and `speed` its radians per second). Cooldown, area and speed gains are fractions of the base.
- Level-up choices are mixed already (`upgrades::Choice::{Art, Upgrade}`): owned Arts below level 8, new Arts while a slot is free, and the 6 old upgrades. Cards show "New" + what the Art does, or what its next level adds (built from the gain, `hud::gain_text`). V17 only has to rename the upgrades into the Sins table and add the new Sins.
- Fury, Frenzy and Ember Legion now boost **every** Art (damage ×, cooldown ÷, +amount); their texts say so.
- Hellflare: 10 dmg, 0.6 s, levels add fireballs (2, 4, 7), −10% cooldown (3, 6), +5 damage (5, 8). Tail Lash: 14 dmg every 1.35 s on a 52×24 box on the side she faces; level 2 adds a second lash behind her 0.12 s later (extra amount keeps alternating sides); then +5 damage per level and −10% cooldown at 4 and 6. Kiss of Ruin: 8 dmg hearts at 420 px/s along the last move direction (`Player::aim`), pierce 1, 0.9 s life; +1 heart at 2, 3, 4, 6, 7, +4 damage at 3 and 7, +1 pierce at 5 and 8. A heart never hits the same enemy twice (enemies have a stable `uid` now). Brimstone Halo: 1 ember at radius 36 for 3 s, then 3 s off; 10 dmg, each enemy burned at most every 0.6 s (`Enemies::halo_until`) and pushed 6 px out; +1 ember at 2, 5, 8, +25% radius and +30% spin at 3 and 6, +5 damage and +0.5 s at 4 and 7.
- No Art scales its sprite (area only grows the Halo's radius), since non-integer scaling breaks the pixel art. Decide in V17 what Allurement (+area) does for the Lash.
- Art: `assets/arts.px.js` → `assets/sprites/arts/anims/` (tail_lash 56×20 ×3, heart_bolt 9×9 ×2, ember 7×8 ×2). The lash is drawn with its tips 2 px above her ground point, flipped by side; embers behind her when on the far side of the orbit.
- Sounds: `Sfx::Lash` (rising air, then a crack on a high D) and `Sfx::Kiss` (a breath and a sine leaning D→Eb, her minor 2nd), both measured at about −28 dB like Hellflare. Halo hits use the normal hit sound.
- Debug: **F7** in a run levels up every Art (new ones join at level 1).
- Stress test: every Art at level 8 on a full 2,500 horde (×1000 HP so it stays full): 0.53 ms mean tick, was 0.50 ms with Hellflare alone.

**Built in V17 (S14):**
- `config/mod.rs`: `SINS` table (`SinDef`, `SinKind`, index consts `WRATH`..`INFERNAL_HIDE`), values as in the table above (Wrath is +10% now, not Fury's +25%; Legion caps at 2). `Choice::Sin`; `Upgrades.sins` levels; at most `MAX_SINS` (6) owned, new Sins only offered while one is free. Greed is left out until Tribute exists (V20), so no Sin is inert.
- `PlayerStats` gained `duration`, `area`, `armor`. Dark Sigil scales every Art's `duration` except the projectile lifetimes (Hellflare, Kiss: that is their range). Infernal Hide takes 1 per level off contact damage, never below `MIN_CONTACT_DAMAGE` (1).
- **Allurement on the Lash (decision):** area deepens the lash's band (`LASH_DEPTH × area`), not its reach; the sprite never scales, so two faint copies (`LASH_ECHO_ALPHA`) mark the band's edges. Hellgate's pools still ignore area; Abyssal Rift's pull radius uses it, hence the pairing.
- Evolutions: `ArtDef.evolution { sin, name, desc }`, `Upgrades::evolvable()` (first slot at level 8 with its Sin owned) / `evolve`; `Arts` branches on `evolved[i]`. Hellstorm: +2 fireballs, each at its own random enemy (reservoir pick). Barbed Scourge: lashed enemies bleed for the Lash's `duration` (3 s base), `BLEED_SHARE` of its damage every `BLEED_TICK`, and drip blood (`Enemies.bleed_until`). Kiss of Death: 1 HP per heart hit, at most `KISS_HEAL_CAP`/s. Infernal Crown: never rests, +1 ember. Siren's Dominion: when a charm ends the enemy bursts (dies, `SIREN_BURST_DAMAGE` × damage around it, a ring of hearts); `charm_until` is reset to 0 on expiry. Abyssal Rift: pools drag enemies within `RIFT_RADIUS` × area to their centre. Torment Eternal: up to `MAX_CHAIN_LINKS` (now 24) bites, hopping anywhere within `TORMENT_RADIUS` of the first victim. Tempest of Wings: one gust all round, ×3 damage, feathers in 6 directions.
- **Abyssal Offerings:** elites drop one where they die (`Game.offerings`); touching it opens it (`SimEvent::Offering`, a screen in `run.rs` that waits `OFFERING_HOLD` before accepting a click/Enter). It evolves the first evolvable Art, else grants a random level (`Reward::Gift`), else heals `OFFERING_HEAL`. Sprite `offering` (13×14 ×4, horned reliquary with a pulsing sigil) in `assets/arts.px.js`. Debug **F8** drops one ahead of Veyra (use with F5 elites / F7 levels).
- Sounds: `Sfx::SirenBurst` (wet pop, sine falling G#→D) −28.1 dB, `Sfx::Offering` (low bell, sine climbing D minor to the Eb→D sigh, thin octave glint) about −15.5 dB like level-up.
- Stress test now runs with every Art evolved and every Sin maxed except Siren's Dominion (it kills what it charms and would thin the horde), ×20000 HP: 0.37 ms mean tick (0.57 before on the same machine with the old setup).

**Built in V16 (S13):**
- New `Enemies` columns: `charm_until`, `slow_until`, `knock` (velocity). `steer` now takes the game time: charmed enemies head for the nearest free enemy within `CHARM_SEEK` (or stand still), slowed ones move at `GUST_SLOW`, knockback is added on top and decays at `KNOCK_DECAY`. Charmed enemies never deal contact damage to Veyra. Hellflare and the other Arts still hit charmed enemies (kept simple).
- Allure (4): a pulse every 3 s, only when an enemy is within `ALLURE_RADIUS` (56 × area). The first `amount` free enemies in reach are **charmed for `duration`, not hurt** (at first the pulse burned them too and killed the weak ones as soon as they were charmed); the rest take the damage. Every `CHARM_BITE` (0.5 s) each charmed enemy bites the free enemies within its radius + `CHARM_REACH` for the Art's damage. Base 5 dmg, 2 charmed, 2.5 s; levels add charmed (2, 5, 8 ×2), area (3, 6), damage + duration (4, 7). Drawn: a ring of `charm_heart`s spreading over `ALLURE_WAVE_TIME`, charmed enemies tinted `CHARM_TINT` with a beating heart over their heads.
- Hellgate (5): every 4 s `amount` pools on random enemies within `HELLGATE_RANGE` (reservoir pick, `random_enemy`). A pool is an ellipse (`HELLGATE_RADIUS` wide, half as deep, plus the enemy radius) that burns everything inside every `HELLGATE_TICK` (0.35 s) for `duration` (2.5 s). Base 7 dmg, 1 pool. No area gains (sprite can't scale). Drawn on the ground right after the blood, fading in and out.
- Chain of Torment (6): every 1.6 s it bites a random enemy within `CHAIN_RANGE`, then jumps to the nearest unbitten enemy within `CHAIN_JUMP` × area, `amount` enemies in all (base 3, 12 dmg). It counts as a cast (cast animation). Drawn as a jagged line from her hand (a kink every `CHAIN_KINK` px, reshaped every 0.05 s): 5 px dark glow, 3 px crimson, 1 px white-hot core (`CHAIN_COLORS`), plus a `chain_spark` on each victim. The first try (1 px line, 4 kinks per link) was invisible on the grass.
- Wing Gust (7): every 2.2 s, if an enemy is within `GUST_REACH` (72 × area), `amount` cones (±`GUST_HALF_ANGLE`, the first toward that enemy, the rest spread around the circle) throw enemies out at `GUST_PUSH` (less for big ones), slow them for `duration` and deal 5 dmg. Each gust throws `GUST_FEATHERS` of her feathers (pink, they read as brown dots once faded, so they fade only in the last 30%).
- Ember Legion's +amount reaches all four (one more charmed / pool / bitten enemy / gust). With 8 Arts and 6 slots, F7 now only fills the free slots.
- Art: `charm_heart` 7×7 ×2, `hellgate` 36×22 ×4 (pool centre at (18, 16)), `chain_spark` 9×9 ×2, `gust_feather` 11×6 ×2, in `assets/arts.px.js`.
- Sounds (peak 50 ms RMS): `Sfx::Allure` -27.8 (a sine scooping up into D with a slow vibrato, the tritone G# whispered on a thin pulse), `Hellgate` -25.5 (low whoomph, fire roar, crackle), `Chain` -28.3 (snap and a zap leaping a tritone, rattles), `Gust` -27.2 (wing thump and a falling whoosh).
- Stress test with all 8 Arts at level 8: 0.56 ms mean tick (was 0.53 with 4).

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
- **Implemented (V8):** `audio/synth.rs` + `audio/wav.rs` use only `std` (the `render_audio` example includes them with `#[path]`, since the crate has no lib target). `Synth::new(sample_rate, Vec<Instrument>)`, `note_on(channel, midi_note, inst_index, velocity)`, `note_off` (jumps macros to their release point; with no volume release point it cuts), `note_cut`, `render(&mut [f32])`, `set_echo(Echo)`. Mono 44.1 kHz f32, `wav::encode` → 16-bit PCM. Macros tick at 60 Hz per voice from its note-on: volume 0–15, arp semitones, pitch in 1/16 semitone (absolute), duty = `DUTIES` index (noise: 0 long / 1 short LFSR); a macro holds its last value unless `loop_from` is inside the current part (sustain `0..release`, release `release..`). Instruments also have vibrato (depth, rate, delay), sweep (semitones/s) and a 32×4-bit wavetable. Triangle is 4-bit stepped, noise clock = note Hz × 16. Output stage: echo with damped feedback (3 kHz) → DC blocker → 12 kHz low-pass → clamp.
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

**Implemented (V9):**
- `audio/sfx.rs` (std only): `Sfx` enum with one patch each (instruments + timed notes + echo), plus per-effect `variants` (pitch offsets), `length`, `cap`, `gap` (min seconds between starts) and `gain`. `render(sfx, pitch)` normalises to peak 0.9. Effects so far: fire (Hellflare), hit (flesh / armour), kill per `Family` (Beast/Human/Armored, a new `EnemyDef.family`), soul, hurt, low-HP heartbeat, death, level-up (D Phrygian dominant arp ending on the b2 hook), upgrade pick, war horn (a new enemy kind arrives after 0:00), pause (b2 sigh), UI move/confirm/back. Coin, summon and boss warning join with their features (V19–V21).
- **Mix tiers** (peak 50 ms RMS at full volume, measured on the `render_audio` exports): UI and Hellflare about -27 to -31 dB (they repeat constantly), combat -19 to -25, events -15 to -17, hurt on top at about -12 so it always cuts through. Level-up (-15.7) is the user-approved reference: don't change its patch. Retune with `Sfx::gain`, then re-measure. UI uses sine (wave) + triangle voices; thin pulses read as harsh there.
- Heartbeat: `Run` beats at ≤ 30 % HP every 0.9 s, ≤ 15 % every 0.6 s. Upgrade pick replaces the focus's generic confirm with `audio::recue`.
- macroquad plays a fixed WAV with no pitch control, so jitter = 5 pre-rendered variants (±0.5 st) picked at random, and the soul combo = 8 variants on a minor pentatonic ladder (pickups < 0.6 s apart climb, else reset). Everything is rendered at load (`Audio::load`, ~50 short sounds).
- `audio/mixer.rs` (pure, tested): per-effect voice end times on the app clock decide cap and gap; xorshift jitter.
- Flow: `Game.cues` (`Cue` enum, deduped, ≤ 8, no per-tick allocation) → `Run` drains them after the fixed steps into `audio::cue(Sfx)` (a deduped thread-local queue that UI code also uses; `Focus::update` cues move/confirm/back) → `App::frame` calls `Audio::play_queued` once per frame with volume `master × sfx × gain`. Volume applies to sounds started afterwards (fine for short SFX; music will need `set_sound_volume`). Ducking waits for the jingles (V11).
- `cargo run --release --example render_audio` also writes `target/audio/sfx/<name>.wav` (all variants in a row, at mix gain).

**Implemented (V10):**
- `audio/tracker.rs` (std only) parses `.vtrk` and renders it; the full syntax is in its module doc. Instruments are FamiTracker-style macros (`|` loop, `/` release). Patterns are horizontal: one line per channel (`p1 p2 tri noi wav`), `.` empty, `=` release, `^` cut, notes `C4`=60, `:v` hex velocity, `drum` tokens for the noise. Each `order` line is one frame that layers patterns, with `Nx` repeat and per-item modifiers `+n`/`-n` (transpose, never the noise), `@ch` (move a one-line pattern) and `%inst` (swap its instrument). `#` starts a comment only at the start of a word (`F#4` is a note). A pattern line with the wrong step count is an error with its line number.
- The whole order loops (no intro section). `Song::render_loop` renders the song twice and keeps the second pass, so the loop's start already holds the echo and ringing notes of its end: seamless with macroquad's `looped`.
- `audio/music.rs`: `Track` (Title, Ashford) embeds `assets/music/*.vtrk`; `render` levels every track to a long-term RMS of -26 dBFS (peak ≤ 0.95), a bed under the combat effects. Both tracks render in well under a second at load.
- Playback: title, credits → Title; run → Ashford; Settings keeps what was playing. A new run restarts its theme. Music volume = master × music, applied live with `set_sound_volume`.
- Title "Dominion": 84 is the dotted-quarter (6/8, 12 rows a bar). Leitmotif = D–Eb–D then the tritone fall to G# leaning into A. Form A (pulse lead over a D drone) | A' (her "voice" on the wave, the pulse whispering answers) | B (Gm Eb Cm D, swaying bass, wave pad) | A; 32 bars, 45.7 s.
- Ashford: gallop bass, off-beat chord stabs (arp macro), the leitmotif as B–C–B–F–F# over B major; the breakdown passes it between the pulses and quotes the title's own pitches. Form A | A' (octave double) | B (long notes over 16th arps) | breakdown + build ending on D#; 32 bars, 51.2 s.
- `render_audio` writes `target/audio/music/<name>.wav`, two loops in a row so the seam can be checked. To check a mix, render each channel alone (filter `Song.events` by channel): in both tracks the lead sits about 3 dB under the full mix and the accompaniment 7–12 dB under.

**Implemented (V11):**
- Tracks: Abbey "Saint Veil" (C Dorian chant on the wave, her Phrygian answers, then the chant turns Phrygian under organum in fourths and two bars of parallel tritones; 3+3+2 pulse; 32 bars, 64 s), Bastion "Aurum" (G minor march, brass in thirds, horn double, Eb trio; the leitmotif disguised as a fanfare figure, then the anthem rises a semitone; 58 s), Gate "The Gate" (A Lydian: heaven's #4 is her tritone; choir/trumpet hymn, her Bb/A against heaven's B/A; 55 s). Bosses, map tempo +10: Captain "The Watch" (E-F grinding bass, alarm bell = her B-C sigh; 36 s), Inquisitor "The Inquisition" (wave toccata, tritone chain bass, minor chords sliding by semitones, she sings his chant back; 44 s), Paladin "The Lion of Aurum" (G major fanfare with chromatic mediants vs her G minor, her Ab chord; 41 s), Archangel "Seraphic Wrath" (feathers / chromatic charge / pillars of light / hymn with her title voice answering; 51 s).
- Jingles (`Jingle`, `assets/music/jingle_*.vtrk`): victory (bII V I in D Phrygian dominant), game over (the leitmotif sinks to G instead of leaning into A), summon Lesser / Greater / Abyssal (Abyssal states the whole leitmotif). The level-up jingle stays the approved `Sfx::LevelUp`. `Song::render_once` plays one pass then rings until a 50 ms block is silent (≤ 4 s); `music::render_jingle` levels the loudest 50 ms RMS to -16 dB (events tier). Exported to `target/audio/jingles/`.
- Playback: `audio::jingle(Jingle)` (thread-local, like `cue`), played by `play_queued` at master × music volume. Music ducks to 0.3 under a jingle and rises back over 0.8 s (`mixer::duck`, tested). Victory and game over are wired in `Run`; leaving the run (title / new run) calls `stop_jingle`. Summon jingles wait for V21.
- Loading: macroquad keeps each sound as stereo f32 (~350 KB/s), so all 9 loops would be ~160 MB. `Audio::load` renders only `PRELOADED` (Title, Ashford) plus the jingles; map and boss themes must load with their map (V19/V23), ideally freeing the previous map's.
- All tracks render in ≤ 0.25 s each; mixes checked per channel: lead 3–5 dB under the mix, accompaniment 7–14 dB under.

### 7.11 Architecture changes

- **Modules:**
  - `assets.rs` (embedded PNGs + atlas rects manifest)
  - `ui.rs` (widgets/focus)
  - `scenes/` (title, map_select, run, settings, credits, altar, circles, codex)
  - `i18n.rs` (`Key` enum → `[&str; 2]`; a test fails if any string is empty)
  - `save.rs`, `meta.rs` (pacts, tribute, familiars)
  - `arts.rs` (Arts runtime: pools + per-archetype update fn; data in `config/arts.rs`)
  - `familiars.rs`, `maps.rs` (`MapDef { ground, props, enemies, events, objective, music, duration }`)
  - `audio/` (synth, sfx, music, tracks)
- `config.rs` keeps the balance numbers but is split per domain if it grows past about 600 lines.
- The simulation stays headless and testable. Audio and visual feedback leave the sim as `SimEvent`s or a per-tick event buffer that `main` consumes, so the sim never calls the audio functions.
- `Game::new(map, mode, loadout)` takes the run parameters. Meta bonuses are applied as base stats.
- Hot path: Arts use preallocated pools per archetype, there are still no allocations per tick, and the stress test is extended to all Arts at max level plus 2,500 enemies.

### 7.13 V2 risks

| Risk | Mitigation |
|---|---|
| Music sounds generic or samey | Per-track mode/tempo/colour table, leitmotif rules, user listens to WAV exports before integration |
| Gacha legal exposure | Earned-only currency, rates shown, no real-money path ever |
| Suggestive art vs store rules | Clothed, no nudity, review proposals with the user before integration |
| Performance with more Arts and bigger enemies | Pools per Art, extended stress test, hash radius includes max enemy radius |
| Font licence | OFL only, licence files shipped and credited |
| Scope creep | Checkpoints are stop points. Content (maps 3–4, Abyssal Familiars) can slip without breaking the game |

