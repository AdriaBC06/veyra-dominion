# Veyra: Dominion

Top-down survivors-like in Rust + macroquad (only dependency). Veyra, a succubus, razes a village. The MVP is done; V2 is in progress.

## Session start (keep it cheap)

1. Read `PLAN.md` (progress + task table) and the last entry of `docs/session-log.md` (`tail -n 12`).
2. The current session is the first one in `PLAN.md` → V2 sessions with unchecked tasks, unless the user picks another. Do only that session's tasks.
3. Read only the `docs/design.md` subsection (7.x) and the source files the task needs. No broad repo exploration. `docs/mvp.md` is history; its section 3 hard rules still apply (summarised below).

## Task loop

1. Implement → `cargo clippy -- -D warnings` and `cargo test` (trim output).
2. Tick the task in `PLAN.md`, record new decisions in the matching `docs/design.md` subsection.
3. Commit with the message from the task table, then `git push`. One task = one commit. **No tool attribution or Co-Authored-By trailers.**
4. *(user approval)* tasks: show the result (PNG/WAV path or capture), wait for the OK before committing. No OK → leave unchecked, note it in the log.
5. Before ending: append a 3–8 line entry to `docs/session-log.md` (date, done, decisions, half-done, gotchas), committed with the last task or as `docs: session N notes`.
6. Checkpoints (CPn): `cargo build --release`, log the binary size, tick it.

## Hard rules

- Simulation (`game.rs`, `enemies.rs`, `spatial.rs`, `upgrades.rs`, `arts.rs`) never calls drawing, input, time, window or audio. Input comes as `InputState`, time as `dt`; feedback leaves as `SimEvent`s. Keeps `cargo test` headless.
- Fixed 60 Hz timestep (dt clamped 0.25 s, max 8 ticks/frame).
- Hot path: SoA enemies, `swap_remove`, preallocated pools, zero allocations per tick. The stress test covers all Arts maxed + 2,500 enemies.
- All balance numbers and tables in `src/config/` (split per domain past ~600 lines).
- Every player-facing string goes through `i18n.rs` (EN + ES); the completeness test fails on empty strings.

## Map

- `src/main.rs` window + loop · `scenes/` (`App`, `Nav`: title, run, settings, credits) · `screen.rs` layout/render target · `ui.rs` widgets/focus
- Sim: `game.rs`, `enemies.rs`, `spatial.rs`, `upgrades.rs` (Sins), `arts.rs` + `config/arts.rs`, `blood.rs`
- `assets.rs` embedded PNG atlas (`assets/sprites/`), fonts in `assets/fonts/` · `audio/` own chip synth + tracker (`assets/music/*.vtrk`) · `save.rs` settings/save files · `i18n.rs`
- Art as code: `assets/*.px.js`; title portrait: `assets/depixelize.py` + `assets/animate_portrait.py` (see docs/design.md, V14 phase 2)
- `examples/render_audio.rs` writes WAVs to `target/audio/`

## Running and debug keys

- `cargo run` (dev profile is optimised enough). Use `XDG_DATA_HOME=<tmp dir>` for test runs so they don't land in the real lifetime stats.
- F2 Arts/Sins panel (levels, slots, force evolutions) · F3 debug overlay · F4 spawn one of each enemy · F5 make elites · F6 kill visible · F7 level up Arts · F8 drop an Offering · F9 EN/ES · F10 scale mode · F11 fullscreen.

## Gotchas

- Captures: `spectacle -a` grabs the *active* window (maybe another app). Use a temporary in-game capture (`VEYRA_CAPTURE` hack after `app.frame()`, before `next_frame`; never committed). Render-target `get_texture_data()` is vertically flipped and only has the UI after `present`; `get_screen_data()` is not flipped.
- New `TextStyle`s must be added to `TextStyle::ALL` (glyphs are pre-rasterised per size). `Key::ALL` is `cfg(test)` only; new strings just go in the `keys!` table.
- 1 px particles vanish on the grass at 360p, use 2×2; dark ends of enemy ramps are invisible on grass.
- Tracker: `#` starts a comment only at a word start (`F#4` is a note); tracks are `include_str!`'d, rebuild after editing a `.vtrk`.
- Audio silent → ALSA needs `pipewire-alsa`. Sound and the GPU (ComfyUI) need the sandbox off.
- `cargo fmt` reflows long calls; re-read before scripted edits. A scripted `str.replace` on a block that appears twice edits both; assert the count.
- Timing asserts (charm expiry etc.) need a 2-tick margin.
- pxl lives in `.claude/skills/pxl-pixel-art` (cloned from github.com/crsolver/claude-pxl, excluded in `.git/info/exclude`, not in the repo). Build with `--out assets/sprites/arts`, then delete the `_preview/` folder and the `.gif`s it writes next to the strips.
- Area-grown Arts (Scythe, Cross, Sigil) are never scaled at runtime: `tiered()` in `assets/arts.px.js` bakes 6 sizes (`<name>_0..5`), picked with `assets::size_tier(area)`. A new sized sprite needs its `SpriteId`s, `frame_ms`, `SHEETS` rows and json test entries per tier; the atlas is 1024×1024.
- Capture runs (sandboxed) segfault on exit after the PNGs are written; harmless so far.
- Stress test baseline on this PC: ~1.1 ms mean tick (`cargo test --release stress -- --nocapture`, `STRESS_HP` 100000 so the horde stays near full).
