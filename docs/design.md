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
  - **Phase 2, Veyra portrait: done (S11c, 2026-10-22, user: "esta perfectisimo").** Final: `assets/sprites/title/veyra_portrait_px_sheet.png`, drawn 1:1 bottom right (`assets::title_portrait`, its own texture; `menu_view` in `scenes/title.rs` centres the title and buttons in the space left of it).
    - Source: a ChatGPT image the user made from the references (`assets/refs/veyra_gpt_fake_pixel.png`, 1254² "fake" pixel art on a ~4.015 px grid). `assets/depixelize.py` rebuilds it as **real** pixel art: detect the grid, take each cell's dominant colour (not the mean), hard alpha, ~56 colours plus reserved eye/gem colours mapped in Lab, despeckle. Result `veyra_portrait_px.png`, 312×311 (the native grid; resampling to 360 would double pixels).
    - Animation: `assets/animate_portrait.py` (input the still, output the sheet), 48 frames at 12 fps in an 8×6 sheet, 344×327 per frame (padding so the wings never clip). Every motion keeps the pixel art clean: wings and tail are cut into layers by hand polygons (purple inside a wing outline is hair only if connected to the hair outside it; the raised glove has its own outline) and swung with **RotSprite** (Scale2x ×3, rotate, sample back); the hair under the tail spade is rebuilt along the strands' slant; breathing lifts everything above the corset seam 1 px; a wave runs down the outer hair; embers rise; gems glint; a real blink (lid line + half-closed in-betweens). Pinholes the motion opens are filled; gaps the original had are kept.
    - Rejected on the way (deleted): tracing the reference (`veyra_portrait_trace.py`: a pixelated photo), the AI pipeline (`pixelize.py`: Qwen-Image-Edit in ComfyUI + Pixel Art Fixer), several code-drawn attempts with pxl (faces and hair never matched the references), and a first animation that shifted rows/columns (tail sheared, pixels swam). Lessons: copy the reference's proportions, measured at the target scale; never resample pixel art to move it.
    - `assets/title.px.js` holds the approved "VEYRA" logo (the rejected tiling backdrop is gone).
  - **Phases 3–5, built in S11d (2026-10-22, all in one session at the user's request):**
    - Backdrop (`assets/title_backdrop.px.js`, pxl): `title_sky` and `title_hills` are 880×360, drawn right-anchored (Veyra, moon and village keep their place on any aspect; 8 px hang past each edge for the parallax). Blood moon (r 112) behind her head and left wing, an orange blood red lighter than her crimson wings so they read over it; sparse stars; warm smoke haze over the horizon. Ranges, valley floor lit by the fires, a road toward the viewer. `title_village` (264×100, 12 frames at 12 fps, curated after the user asked for more detail and fire variety): stone shells with coursed walls, timber-framed cottages (studs, rail, brace), roof remnants of dark tiles with a glowing burnt edge, chimneys, a barn burnt to its skeleton, the church with buttresses, lancet windows lit from inside and a snapped stone tower; well, overturned cart, fences with missing posts, a burnt tree, the street. Fire kinds: 3 blazes (row-painted curved tongues, each on its own beat, leaning with the wind, white-hot base, licks breaking off the tips; they feed the smoke), flames out of windows, burning beams, a burning haystack and smouldering heaps. Each fire lights the ground around it (soft-seamed bands) and the wall faces turned to it, breathing with the flame. The left blaze sits in the middle row so it never shows under the Quit button at 16:9. `title_crest` (left-anchored): the hill with dry grass, grave mound, headstone and a ~90 px leaning wooden cross lit on its right edges.
    - Gradients use a `band()` helper: flat bands with only a narrow ordered-dither seam, never filler noise.
    - Smoke (`scenes/backdrop.rs`): 30 puffs per fire, stateless (position from time), climbing, growing, bending right behind Veyra and dithering out. `title_smoke` holds 3 passes × 24 size tiers (shade, rim, body); all shades are drawn offset down-left, then all rims up-right, then all bodies, so the light only shows on the column's outer edge (per-puff shading looked like stacked coins). The left column stays low so the moon keeps showing. Fire positions are duplicated in Rust (`FIRES`); keep them in sync with the .px.js.
    - Embers: 44 stateless particles, half from the fires, half from the valley floor, cooling yellow → orange → dull red and blinking out.
    - Parallax: the pointer's x eases a look value; layers shift by whole pixels, from 1 px (sky) to 7 px (crest).
    - Logo: "VEYRA" with a hellfire halo (`title_logo_glow`, hard bands, 4 strengths flickered by a sum of sines). "DOMINION" redone with the same chisel cut at 13 px (was 9 px) between gilded rules ending in rubies (the corset gems). No tagline.
    - Buttons (title only): `title_button`, 168×26, dark plum lacquer in a chamfered gilded frame with a ruby at each end; highlighted, the gold brightens, the panel warms and a 2 px ember ring surrounds it; labels muted lilac → warm white. Other menus keep `ui::draw_frame` for now.
    - The title, credits and settings opened from the title draw on this backdrop (`App::on_backdrop`); the run and its pause settings keep the world.
    - Ember wipe (`scenes/wipe.rs`) on every scene change and at start-up: a black sheet burns in from the left over the old scene (0.24 s, input ignored), the nav applies, then it burns away the same way (0.36 s). Burn field = 75% diagonal sweep + 25% value noise, at 2×2 px cells, 4-colour glowing edge; one CPU image updated in place.
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

*The two tables below are the V17 state; the redesign further down replaced them in V17b (see "Built in V17b").*

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

**Redesign direction (agreed 2026-10-21, user planning questions after S14).** Behaviours follow Vampire Survivors items; names, look and lore are Veyra's (succubus). Free colour per Art (no schools). Replaces the tables above once built.

*Existing Arts (V17b):*
- Hellflare: unchanged starting Art.
- Tail Lash: reach grows per level (too short even at high levels).
- Kiss of Ruin: no longer an Art; it is **Hellflare's evolution** (more projectiles, piercing hearts).
- Brimstone Halo: starts as one bigger flame; levels add flames, spin speed and less rest.
- Allure: unchanged; Siren's Dominion's burst deals a little damage, never kills.
- Hellgate: much less base damage, pool size grows per level.
- Chain of Torment: unchanged.
- Wing Gust: removed. Replaced by **Scent of Sin** (Garlic): a pink haze around her that burns slowly and weakens enemies (they take more damage). Allure commands, the Scent corrodes.

*New Arts (V17d–f, same session S14b, 7 more, 14 in total):*
- **Thralls** (summon): seduced villagers that turn on their own; fragile and disposable.
- **Reaper's Scythe** (Axe): a scythe thrown in an arc.
- **Profaned Cross** (Cross): an inverted cross from the Church that sealed her, thrown like a boomerang.
- **Wandering Sigil** (Runetracer): her infernal sigil bouncing off the screen edges, piercing.
- **Bat Swarm** (Peachone/Ebony as one Art): a flock that bombs areas, grows per level.
- **Black Mass** (Pentagram): a pentagram that now and then deals massive damage to everything visible (no instant kill, so bosses/elites survive).
- **Gaze of Desire** (Clock Lancet): her gaze freezes enemies for a moment.

*Union:* **Allure + Thralls** (both max + Offering) replaces both: charmed enemies become thralls that fight for her, but they can die and do not last forever. Allure can still evolve alone into Siren's Dominion.

*Sins (12): the seven deadly sins + infernal extras.* Wrath +damage · Gluttony +max HP and regen (replaces Blood Pact) · Greed +Tribute (with V20) · Sloth +duration · Lust +pickup radius · Envy +XP per soul (the user rejected luck: that belongs to permanent Pacts) · Pride = Curse (more, faster, tougher enemies for more souls) · Legion +amount · Bloodlust −cooldown · Night Wings +speed · Infernal Hide +armour · Dark Sigil +area. Old Siren's Call and Allurement go.

*Evolution pairs (agreed with the user 2026-10-21, start of S14b):* Hellflare + Legion → **Kiss of Ruin** (its card must read clearly as Hellflare's evolution) · Tail Lash + Wrath → Barbed Scourge · Brimstone Halo + Sloth → Infernal Crown · Allure + Lust → Siren's Dominion · Hellgate + Dark Sigil → Abyssal Rift · Chain + Bloodlust → Torment Eternal · Scent of Sin + Gluttony → **Devouring Musk** / Almizcle Voraz (drains and heals) · Reaper's Scythe + Night Wings → **Midnight Harvest** / Cosecha de Medianoche (scythes spiral out all round) · Profaned Cross + Wrath → **Heretic's Brand** / Estigma Hereje (burning crosses) · Wandering Sigil + Infernal Hide → **Broken Seal** / Sello Roto (explodes on each bounce) · Bat Swarm + Gluttony → **Crimson Host** / Hueste Carmesí (bigger bombing flock, **no healing**: the user found two healing evolutions too much) · Black Mass + **Envy** → **Witches' Sabbath** / Aquelarre (more often, pulls every soul; Envy is +XP like VS's Crown) · Gaze of Desire + **Pride** → **Evil Eye** / Mal de Ojo (frozen enemies take more damage) · Allure + Thralls union → **Infernal Court** / Corte Infernal. Wrath and Gluttony pair twice (11 usable Sins for 13 evolutions; Greed waits for V20).

*Offerings:* can be rejected as a **sacrifice**: destroyed for a minor reward (heal, souls or, from V20, Tribute).

*Icons (V17c):* pxl skill (https://github.com/crsolver/claude-pxl, install first); every Art, evolution (own icon) and Sin; shown on level-up/Offering cards and the HUD. Done in one pass after all 14 Arts exist (S14c).

**Built in V17b (S14b, approved by the user 2026-10-21):**
- Arts table (`config/arts.rs`, 7 now): Hellflare 0, Tail Lash 1, Brimstone Halo 2, Allure 3, Hellgate 4, Chain of Torment 5, Scent of Sin 6. `ArtStats.area` now means lash reach, pool size and haze radius too; `pierce` is unused. Sins (`config/mod.rs`, 11): Wrath, Bloodlust, Legion, Night Wings, Gluttony (+15 max HP and `GLUTTONY_REGEN` 0.2 HP/s per level; the HP it adds comes filled), Lust (+40% pickup), Sloth (+10% duration), Dark Sigil (+10% area), Infernal Hide, Envy (+10% XP, fractions carried in `Game.xp_carry`), Pride (+10% spawn rate and new enemies' HP, half of it to their speed via `steer`'s `speed_scale`; no extra XP per soul, more enemies already give more souls). `SinKind` is gone (nothing read it).
- **Kiss of Ruin** = Hellflare evolved (+ Legion): the volley becomes hearts (`Bolt`, `KISS_EXTRA` +2, `KISS_PIERCE` 2, `KISS_SPEED`, `KISS_LIFE`), aimed like the fireballs; the Offering line reads "Hellflare can evolve" and the card "Fireballs become piercing hearts".
- Tail Lash: +20% area at 3, 5, 7; area lengthens the reach (`LASH_REACH × area`), depth fixed. Drawn by stretching the lash horizontally (a motion streak; offered the user sized sprites instead, not asked for yet).
- Brimstone Halo: one big flame (`ember` 13×17 ×4, `EMBER_RADIUS` 8, 12 dmg); levels add flames (2, 5, 8), spin and −15/−15/−20% rest (3 s → 1.5 s).
- Siren's Dominion: the charmed enemy survives its burst; `SIREN_BURST_DAMAGE` 1× and the burst leaves everyone at `SIREN_SPARE_HP` at least.
- Hellgate: base 3 dmg (was 7), +25% area at 3, 5, 7 grows the pool. Drawn without a pool sprite (the user found the sprite pools and later a plain orange ellipse ugly and too bright): scorched ground in `GATE_CELL` 2 px cells with a ragged per-cell edge (`Gate.seed`), char → scorch → faint edge, a few breathing embers, plus small `gate_flame` sprites (7×10 ×4), one per `GATE_FLAME_AREA`.
- **Scent of Sin** (Garlic, replaces Wing Gust): every 1 s it burns everything within `SCENT_RADIUS` (40) × area for 4 and weakens it for 1.5 s (`Enemies.weak_until`; `strike` takes the time and multiplies by 1 + `SCENT_WEAKEN` 0.25). Weakened enemies tint `WEAK_TINT` (not during the hit flash, which stays white). Drawn: pink fill that breathes, a 2 px rim, a ring pulsing out on each burn, rising 2×2 motes. **Devouring Musk** (+ Gluttony) heals `MUSK_HEAL` per enemy burned, at most `MUSK_HEAL_CAP`/s.
- Wing Gust removed with its columns (`slow_until`, `knock`), feathers and sprite; `Sfx::Gust` kept for Bat Swarm.
- **Offerings**: `open_offering` only decides the reward; the screen shows it on card 1 (accept → `accept_offering`) and card 2 sacrifices it (`sacrifice_offering`: `SACRIFICE_HEAL` 20 HP + `SACRIFICE_XP` 25). Keys 1/2.
- **Veyra centred on her hitbox** (user request): she is drawn `VEYRA_DROP` (29 px) lower, so her body, not her feet, sits on `player.pos`; the shadow falls below. Lash, haze, Halo and death smoke follow.
- **Debug panel F2** (user request): pauses the run; every Art and Sin, owned ones filled with their level, slot counters (red "FULL"), click/Enter/→ up, right-click/← down (0 frees the slot, leaving level 8 undoes the evolution), an Evolve button per Art showing its Sin, lit only at level 8 with that Sin, gold once evolved. Labels English only like F3; names through `tr`.
- Stress test (every Art evolved, every Sin maxed, now Siren's Dominion too): 1.30 ms mean tick on the new PC, same as the V17 code on it (the old 0.37 ms was the old PC).

**Built in V17d (S14b):**
- Arts 7–9 in `config/arts.rs` (10 now). `ArtStats.pierce` is used by the Scythe; `ArtDef::sloth()` says whose `duration` Sloth lengthens (not the flights of Hellflare, the Scythe or the Cross; the Sigil's wander yes). Piercing shots share `HitList<N>` (a ring of the last N `uid`s; Kiss hearts use it too, `PIERCE_MEMORY` 48 for the new ones). `World.view` (from `Game::set_view`) lets the Sigil bounce inside the view.
- **Reaper's Scythe** (Axe): every 2 s `amount` scythes leave her upwards at 300 px/s, always symmetric (user request): an odd count sends one straight up, the rest go in pairs to either side, each pair `SCYTHE_SPREAD` wider (`scythe_spread`); they fall at `SCYTHE_GRAVITY` for 1.6 s; 20 dmg, each enemy once, through `pierce` (2) more. Levels: +1 scythe (2, 5, 8), +10 dmg (3, 6, 8), +2 pierce (4, 7), −10% cooldown (7), +15% area (3, 5, 7: they grow with the levels, user request). **Midnight Harvest** (+ Night Wings): `HARVEST_EXTRA` +5 scythes evenly all round at `HARVEST_SPEED`, curling `HARVEST_CURL` rad/s, no gravity, unlimited pierce, `HARVEST_RADIUS`, `HARVEST_LIFE` 2.4 s.
- **Profaned Cross** (Cross): every 1.8 s the first at the nearest enemy within `CROSS_RANGE`, the rest at random ones (waits with nobody in range); starts at 260 px/s, slows `CROSS_DECEL` 240 px/s², turns back (reach v²/2a ≈ 140 px) and chases Veyra where she is now, speeding up by as much, until she catches it (`CROSS_CATCH`; `CROSS_MAX_LIFE` 10 s as a safety); 10 dmg, hits each enemy once each way. User request after the first try: crosses were hard to use when they flew back past her to the throw point, and too small (now ×1.2, `CROSS_RADIUS` 10). Its `duration` is unused. Levels: +10 dmg (2, 5, 8), +10% area +25% speed (3, 6), +1 cross (4, 7). **Heretic's Brand** (+ Wrath): +1 cross (`BRAND_EXTRA`), what it hits burns `BRAND_BURN` 2 s for `BRAND_SHARE` 30% of its damage every `BRAND_TICK` (`Enemies.burn_until`), drawn with a small `gate_flame` on the body; the crosses burn (`cross_burning` strip).
- **Wandering Sigil** (Runetracer): every 3 s `amount` sigils in random directions at 250 px/s for 2.25 s, bouncing off the view edges (`SIGIL_MARGIN` inside, its draw height on the top edge), 10 dmg, forgetting whom it hit at each bounce. Levels: +5 dmg +20% speed (2, 5), +5 dmg +0.3 s (3, 6), +1 sigil (4, 7), +0.5 s (8). Fades over its last `SIGIL_FADE`. **Broken Seal** (+ Infernal Hide): each bounce blasts `SEAL_BLAST_RADIUS` (× area) for `SEAL_BLAST_DAMAGE` 1.5× (`SealBlast`, a pink flash and a ring of 2×2 sparks).
- **Size grows with area** (user request): area scales the hit radius of scythes, crosses and sigils, and their sprites too. Sprites are never scaled at runtime: each is baked at `SIZE_TIERS` (6) sizes, tier k = 1 + k/4 times the base (`<name>_<k>` strips, `assets::size_tier(area)` picks the nearest, `SCYTHE_TIERS` etc.). The atlas grew to 1024×1024 for them.
- Art (`assets/arts.px.js`, a `spun()` helper bakes each rotation and size from a shape function so spins stay crisp; `tiered()` writes the sizes): `scythe_k` 21→45 px ×8, `cross_k` / `cross_burning_k` 23→47 px ×8, `sigil_k` 19→37 px ×8 (only the runes turn, the horned heart, now drawn from the heart curve, stays upright). Drawn mirrored when flying left, so they spin the way they go.
- Sounds (peak 50 ms RMS): `Sfx::Scythe` −29.0 (rising whoosh, thin steel ring on a high D and its fifth), `Cross` −28.1 (cracked bell, D with the tritone G#), `Sigil` −28.5 (glassy arpeggio D–F–G#–D), `SealBlast` about −28 (crack, low thump, a shard falling a tritone).
- Stress test: `STRESS_HP` 20000 → 100000 (the new Arts thinned the horde to a mean of 1,685); 1.06 ms mean tick, mean horde 2,348.

**Built in V17e (S14b, approved by the user 2026-10-22: "está bastante bien"):**
- Arts 10–12 in `config/arts.rs` (13 now). Behaviour constants of every Art and evolution moved from `config/mod.rs` to `config/art_tuning.rs` (mod.rs was about to pass 600 lines).
- **Bat Swarm** (Peachone/Ebony Wings): `amount` bats (2 → 5) circle her on an ellipse (`BAT_ORBIT` 44, half as deep) at `BAT_DRAW_HEIGHT` 30, `speed` 1.6 rad/s; every 2.5 s each lobs a blood bomb (`Bomb`, pool `MAX_BOMBS` 48) at a random enemy within `BAT_RANGE` (waits with nobody); it lands after `BAT_DROP_TIME` 0.45 s and bursts over `BAT_BLAST_RADIUS` 22 × area for 15 dmg (30 at 8). Levels: +1 bat (2, 5, 8), +5 dmg +15% area (3, 6), −10% cooldown (4, 7). **Crimson Host** (+ Gluttony): `HOST_EXTRA` +3 bats, blasts `HOST_BLAST` 1.4× wider, red bats with golden eyes; no healing.
- **Black Mass** (Pentagram): strikes every enemy within the view + `MASS_MARGIN` for 40 dmg every 12 s (140 every 8.4 s at 8: +20 dmg at 2, 4, 6, +40 at 8, −10% at 3, 5, 7); only when one is in view. Plain damage, so tough foes survive. Drawn: a ritual circle (`black_mass` 141×71 ×4, inverted pentagram on the ground) under her for `MASS_SHOW`, a red flash over the view for `MASS_FLASH`. **Witches' Sabbath** (+ Envy): cooldown × `SABBATH_COOLDOWN` 0.6, gold circle (`sabbath`), every soul on the map gets `Soul.called` and flies to her at `SABBATH_SOUL_SPEED`.
- **Gaze of Desire** (Clock Lancet): every 2 s `amount` golden beams (`Gaze`, `GAZE_REACH` 150 × area long, `GAZE_WIDTH` 6 × area), the first at the nearest enemy within `GAZE_RANGE`, the rest evenly around; whatever they touch is frozen for `duration` (1 s → 2 s, Sloth-scaled) in `Enemies.frozen_until`. No damage (decision). Frozen enemies skip `steer` (no move, no separation), deal no contact damage, hold their walk frame and tint `FROZEN_TINT` (gold). Levels: +0.25 s (2), +1 beam (3, 6), +20% area (4, 7), −10% +0.25 s (5), −10% +0.5 s (8). **Evil Eye** (+ Pride): `EVIL_EYE_EXTRA` +1 beam, frozen enemies take `EVIL_EYE_VULN` +50% from everything (`Enemies.frozen_vuln`, set by the Arts each tick, multiplies with the Scent's weakness in `strike`), golden `evil_eye` over their heads.
- Art (`assets/arts.px.js`): `bat` / `bat_host` 17×11 ×4, `blood_bomb` 6×6 ×2, `black_mass` / `sabbath` 141×71 ×4, `evil_eye` 9×7 ×2. Nothing area-grown needs size tiers: blasts and beams are drawn as shapes.
- Sounds (peak 50 ms RMS): `Sfx::Bats` −28.7 (wingbeat flurry, squeak D→Eb), `BatBlast` about −27 (wet splat, low thump), `BlackMass` −17.6, event tier (low bell, organ D against Eb, sine choir), `Gaze` −28.9 (glassy sine gliding up, tritone glint).
- Stress test: 0.33 ms mean tick, mean horde 2,295; lower than V17d's 1.06 ms mostly because many frozen enemies skip steering.

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
| Title "Requiem" | D minor, Phrygian Eb | 100 BPM 4/4, full from bar 1 | Funeral bell, organ-choir Dies Irae, 16th harp ostinato |
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
- Title "Requiem" (replaced "Dominion" on 2026-10-22; the user wanted it darker and gripping from the first second, so no slow intros on the title): crash + funeral bell (pulse, octave/twelfth arp clang) on bar 1, the Dies Irae with E made Eb on a hollow organ-choir wave over a 16th harp ostinato, octave bass and drums | B the leitmotif | C chromatic lament bass (D C# C B Bb A) under the Dies Irae up high, tremolo, drums doubling the second time | A' ending on Eb over A; 24 bars, 57.6 s. The old theme and three other candidates (Lilith's Lullaby, Heartbeat, Dominion March) are kept in `assets/music/drafts/`, not embedded.
- Old title "Dominion" (now `drafts/title_dominion.vtrk`): 84 is the dotted-quarter (6/8, 12 rows a bar). Leitmotif = D–Eb–D then the tritone fall to G# leaning into A. Form A (pulse lead over a D drone) | A' (her "voice" on the wave, the pulse whispering answers) | B (Gm Eb Cm D, swaying bass, wave pad) | A; 32 bars, 45.7 s.
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

