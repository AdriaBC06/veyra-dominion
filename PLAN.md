# Veyra: Dominion — Plan

Task order and progress. Working rules are in `CLAUDE.md`; design in `docs/design.md`; session notes in `docs/session-log.md`. Finished sessions and their task rows: `docs/archive/plan-done.md`.

## Progress

- [x] MVP T1–T15, CP1–CP3
- [x] V2 S1–S14b: V1–V17f (assets, fonts, resolution, input, i18n, saves, scenes, synth, SFX, music, Veyra, enemies, title, Arts, Sins, redesign), CP4–CP6
  - S14b review: Thralls / Infernal Court tested in game by the user, working (2026-10-24)
  - Note (user, 2026-10-24): fusions forced from F2 don't show up in the Grimoire. By design per 7.14 (`Run::forced` never unlocks); revisit only if the user wants otherwise

### V2 sessions

- **S14c — Icons** *(user approval)*
  - [x] Install the pxl pixel-art skill: https://github.com/crsolver/claude-pxl
  - [x] V17c Icons for every Art, evolution (own icon) and Sin, in cards and HUD
- **S15 — Enemy AI**
  - [x] V18 Enemy behaviours and elites (Ashford roster of 12, approved by the user)
  - [x] V17g Whip of Torment: Chain renamed, whip icon
  - [x] V17h Wandering Sigil stutter on 75 Hz (interpolation sign)
  - [x] V17i Black Mass / Witches' Sabbath nerf
  - [x] V17j Wandering Sigil sprite + matching icon *(user approval)*
  - [x] V17k Hellflare flames, Halo fireballs, flame hearts (Kiss of Ruin), Broken Seal look *(user approval)*
  - [x] V17l Bat Swarm / Crimson Host icons like their bats *(user approval)*
  - [x] V17m Thralls icon reads as a thrall *(user approval)*
  - [x] V17n Visible evolutions: Barbed Scourge, Infernal Crown, Abyssal Rift, Torment Eternal, Devouring Musk, Midnight Harvest *(user approval)*
  - [x] V18b Progression rebalance (10:00 Ashford timeline, stepped XP curve, soul tiers and drop chances, HP per minute; plan in design.md 7.8)
- **S16 — First boss**
  - [x] V19 Boss framework + first boss *(user approval)*
  - [x] **CP7 Combat depth**
- **S16b — Interface** *(user approval)*
  - [x] V19b Premium UI kit + every menu and the HUD redone (7.14)
  - [x] V19c Grimoire of Evolutions (title + pause)
  - [x] V19g First-time setup (language, blood, display, volume) + Blood setting
- **S16c — Ashford map** *(user approval)*
  - [ ] V19d Authored Ashford layout, village art, blocking grid, flow field, wall/projectile rules, y-sort
- **S16h — Living village** *(user approval)*
  - [ ] V19h Fleeing villagers, breakables + drops, building damage/collapse into rubble, fire
- **S16i — Nightfall** *(user approval)*
  - [ ] V19i Dusk-to-night cycle over the 10:00, light map (fire, torches, lamps, windows), building shadows
- **S16j — Minimap** *(user approval)*
  - [ ] V19e Minimap + full map (M)
- **S16d — Veyra sprite** *(user approval)*
  - [ ] V19f In-game Veyra sprite redone to match the title portrait
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
| V17c | Icons (pxl skill) for all Arts, evolutions and Sins, in cards and HUD | User approves | `feat: art and sin icons` |
| V18 | New behaviours (Ranged, Charger, Support, Swarm, Shielded, Flyer) + hostile shots + elites | Behaviour tests green | `feat: enemy behaviours and elites` |
| V17g | Chain of Torment → Whip of Torment / Látigo del tormento, whip icon | User approves | `feat: whip of torment (chain renamed, new icon)` |
| V17h | Sigil drawn along its real velocity (was extrapolated the wrong way when flying left) | Smooth on 75 Hz | `fix: sigil interpolation` |
| V17i | Black Mass 12 dmg / 20 s (42 / 14 s maxed), Sabbath ×0.8; `BALANCE_ONLY` probe | Probe: Mass alone no longer outlevels Hellflare alone | `balance: nerf black mass` |
| V17j | New Wandering Sigil (6 tiers) and the same design as its icon | User approves | `feat: new wandering sigil` |
| V17k | Swap looks: Hellflare throws flames, the Halo circles fireballs (centred on her); Kiss of Ruin flaming hearts; Broken Seal own sprite, throbbing cracks, animated burst + shockwave; icons to match | User approves | `feat: flames, fireballs and broken seal` |
| V17l | Bat Swarm / Crimson Host icons drawn from the in-game bats | User approves | `feat: bat icons` |
| V17m | Thralls icon: one of the summoned villagers | User approves | `feat: thrall icon` |
| V17n | Each of these evolutions changes how its Art looks in play, not only its icon | User approves | `feat: visible evolutions` |
| V18b | 10:00 Ashford timeline (retirements, pressure events, elites), stepped XP curve, soul tiers + phase drop chances, HP per minute; tuned with the balance probe | Probe hits the 7.8 level targets | `feat: progression rebalance` |
| V19 | Boss framework + Captain of the Watch | Boss beatable | `feat: boss framework and first boss` |
| **CP7** | **Checkpoint 7 — Combat depth** | | |
| V19b | UI kit (`assets/ui.px.js`: frames, buttons, sliders, tabs, slots) + motion/easing helpers; pause, level-up, Offering, game over, victory, settings, credits and HUD redone (7.14) | User approves captures of every screen | `feat: premium ui pass` |
| V19c | Grimoire of Evolutions: every evolution/union with recipe icons, silhouettes until fused once, `evolved=`/`seen=` in the save, from title and pause | Save tests green + user approves | `feat: grimoire of evolutions` |
| V19g | First-time setup on the first launch (no settings file): language, **blood** (full / reduced / off: UI blood, stained titles, in-game gore and the permanent blood layer), scale mode + fullscreen, master volume; skippable, each step reachable later in Settings, where Blood gets its own row | Save tests green + user approves | `feat: first-time setup and blood setting` |
| V19d | Finite authored Ashford (square, chapel, houses, river + bridges, ridge, woods border) drawn with pxl (sheet approved first); solid 16 px grid, push-out, flow-field chasers, spawns outside walls, bot navigation; thrown Arts and enemy shots stop at walls, sweeping ones pass; y-sort, roofs fade | Stress test on the map green + user approves | `feat: ashford map` |
| V19h | Civilians pool (flee, tiny souls), breakables + drop table, building structure HP → cracked → collapse into blocking rubble, heat from fire hits → burning → burnt (rubble can still burn) | Sim tests + stress green + user approves | `feat: living village` |
| V19i | Time-of-day over 0:00–10:00 (afternoon → golden → dusk → night), low-res light map (fires, fireballs, torches, lamps, windows, souls, Veyra), sun shadows that lengthen and fade; night stays readable | Stress tick unchanged + user approves | `feat: nightfall and lighting` |
| V19e | Minimap (baked landmarks, density-grid enemy dots, boss icon + edge arrow) and full map on M (pauses) | Stress tick unchanged + user approves | `feat: minimap and full map` |
| V19f | New in-game Veyra sprite (all layers and animations) matching the title portrait; method agreed with the user first (7.6) | User approves | `feat: new veyra sprite` |
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
