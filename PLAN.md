# Veyra: Dominion — Plan

Task order and progress. Working rules are in `CLAUDE.md`; design in `docs/design.md`; MVP history in `docs/mvp.md`; session notes in `docs/session-log.md`.

## Progress

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
  - [x] V8 Chip synthesizer core
- **S6 — SFX**
  - [x] V9 Sound effects and mixer
- **S7 — First music** *(user approval)*
  - [x] V10 Tracker format + first themes
- **S8 — Soundtrack** *(user approval)*
  - [x] V11 Remaining music + jingles
  - [x] **CP5 Audio**
- **S9 — Veyra** *(user approval; ask for the reference images at the start)*
  - [x] V12 Veyra redesign
    - [x] Reference images received (`assets/refs/`) and pose agreed (seated lounge, see 7.6)
    - [x] Sprite set drawn as code (`assets/veyra.px.js`): body states idle 4 / move 4 / cast 3 / hurt 2 / death 6, wings, tail and hair layers, red rim light
    - [x] User approves the look (2026-10-20: "está buenísima, perfecta")
    - [x] In game: `SpriteId` + `SHEETS` rows for the new strips, layered drawing, pivot, bigger shadow
- **S10 — Enemies look** *(user approval)*
  - [x] V13 Enemy restyle with variable sizes
    - [x] Art drawn as code (`assets/enemies.px.js`): rat, villager, knight + brute farmer (40 px), 2 walk frames each
    - [x] User approves the look (2026-10-20: "está muy bien")
    - [x] In game: sprites embedded (`SpriteId::Brute` added), drawn bottom-centre by sprite size; Brute `EnemyDef` (chase, unlocks at 3:00); F4 debug spawn
    - [x] `EnemyDef.size`, radius derived from the sprite, spatial queries use the largest radius *in play*
    - [x] Death bursts in each enemy's palette (short white flash ~0.12 s + 6–10 pooled bits that fall and fade)
    - [x] Blood (asked for in S10): drops in the burst and permanent pools that grow where many die
    - [x] Elite outline: `elite` flag on enemies + debug key (F5); real elites come in V18
    - [x] Checked in game by the user (F4/F5/F6) and with scratch captures; stress test green
- **S11 — Title screen** *(user approval)*
  - [x] V14 Animated title scene
    - [x] Phase 1 composition approved
    - [x] Phase 2 Veyra portrait: animated 1:1 pixel-art portrait on the title screen (approved)
    - [x] Phases 3–5 (backdrop, logo + buttons, integration), approved in game
  - [x] **CP6 Look & feel**
- **S12 — Arts I**
  - [x] V15 Arts framework + first Arts
- **S13 — Arts II**
  - [x] V16 Remaining Arts
- **S14 — Sins**
  - [x] V17 Sins and evolutions
- **S14b — Arts & Sins redesign** *(user approval; see 7.7 "Redesign direction"; one commit per task)*
  - [x] Review the evolution pairs with the user (7.7, agreed 2026-10-21)
  - [x] V17b Rework the 7 existing Arts (Kiss → Hellflare evolution, Wing Gust → Scent of Sin), 12 Sins, Offering sacrifice (+ F2 debug panel, Veyra centred on her hitbox)
  - [x] V17d Reaper's Scythe, Profaned Cross, Wandering Sigil (+ evolutions)
  - [x] V17e Bat Swarm, Black Mass, Gaze of Desire (+ evolutions)
  - [x] V17f Thralls + Allure/Thralls union (committed at the user's request, in-game review pending)
- **S14c — Icons** *(user approval)*
  - [ ] Install the pxl pixel-art skill: https://github.com/crsolver/claude-pxl
  - [ ] V17c Icons for every Art, evolution (own icon) and Sin, in cards and HUD
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


## V2 tasks (7.12)

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
| V17b | Rework existing Arts, 12 Sins, Offering sacrifice (7.7 "Redesign direction") | Tests green, user approves in game | `feat: arts and sins redesign` |
| V17d | Reaper's Scythe, Profaned Cross, Wandering Sigil | Tests + user approves | `feat: scythe, cross and sigil arts` |
| V17e | Bat Swarm, Black Mass, Gaze of Desire | Tests + user approves | `feat: bats, black mass and gaze arts` |
| V17f | Thralls + Allure/Thralls union | Tests + user approves | `feat: thralls and first union` |
| V17c | Icons (pxl skill) for all Arts, evolutions and Sins, in cards and HUD | User approves | `feat: art and sin icons` |
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
| V30 | Release: Windows + Linux builds, icon, `strip = true`, licences bundle next to the executable (`LICENSE`, EULA, `THIRD-PARTY-NOTICES.md` and the `licenses/` folder: OFL texts + every crate licence; the credits point to it; regenerate if dependencies changed), README/press kit | Builds run on both | `chore: release packaging` |
| **CP10** | **Checkpoint 10 — Release candidate** | | |

