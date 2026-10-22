# Veyra: Dominion — Plan

Task order and progress. Working rules are in `CLAUDE.md`; design in `docs/design.md`; session notes in `docs/session-log.md`. Finished sessions and their task rows: `docs/archive/plan-done.md`.

## Progress

- [x] MVP T1–T15, CP1–CP3
- [x] V2 S1–S14b: V1–V17f (assets, fonts, resolution, input, i18n, saves, scenes, synth, SFX, music, Veyra, enemies, title, Arts, Sins, redesign), CP4–CP6
  - Pending from S14b: in-game review of Thralls / Infernal Court (V17f, F2/F7/F8)

### V2 sessions

- **S14c — Icons** *(user approval)*
  - [x] Install the pxl pixel-art skill: https://github.com/crsolver/claude-pxl
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

## V2 tasks (7.12)

| # | Task | Done when | Commit |
|---|---|---|---|
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
