# Veyra: Dominion

A small top-down survivors-like written in Rust with [macroquad](https://github.com/not-fl3/macroquad).
Veyra, a red-skinned succubus villain, has risen to raze a village. Survive 5:00 against animals and humans while absorbing their souls.

All art is pixel art defined in code; there are no image files.

## Run

```sh
cargo run --release
```

On Arch Linux you need `libx11 libxi mesa alsa-lib libxcursor libxrandr`.

## Controls

| Key | Action |
|---|---|
| WASD / arrows | Move |
| Esc | Pause / resume |
| Enter | Start / confirm |
| 1 / 2 / 3, or ← / → + Enter | Pick upgrade |
| R | Restart (game over / victory) |
| E | Endless mode (after victory) |
| T | Back to title (pause / game over / victory) |
| F3 | Debug overlay |

Veyra fires Hellflare on her own at the nearest enemy; you only steer.

## Balance notes

All numbers live in `src/config.rs`.

- **Veyra**: 100 HP, speed 180, 0.5 s invulnerability after a hit (the strongest touching enemy deals its damage).
- **Hellflare**: 10 damage every 0.6 s, range 340, speed 320.
- **Enemies**: Rat 6 HP / 5 dmg / fast, in packs of 2–4. Villager 14 HP / 8 dmg, 1–2. Knight 45 HP / 15 dmg / slow, from 2:00 on.
- **Waves**: spawn rate ramps linearly from 1 to 25 enemies/s over 5:00, up to 2,500 alive. Spawn weights drift from rats toward villagers and knights over time.
- **Souls**: Rat 1, Villager 2, Knight 5 XP. Level *n* → *n*+1 needs `8 + 6n` XP. Souls within 48 px are pulled in.
- **Upgrades** (pick 1 of 3, capped): Fury +25% damage (×5), Frenzy +15% fire rate (×5), Ember Legion +1 fireball (×4, up to 5), Light Wings +10% speed (×5), Blood Pact +20 max HP and heal 30 (×5), Soul Pull +40% pickup radius (×5).
- **Victory** at 5:00. Endless mode keeps going with enemy HP +50% per extra minute.
- **Performance**: the headless stress test (`cargo test`) runs 20,000 ticks at 2,500 enemies; mean tick is about 0.6 ms in the dev profile.

## Development

See [PLAN.md](PLAN.md) for architecture, tasks and progress.

## Licence

GPL-3.0. See [LICENSE](LICENSE).

Fonts, embedded in the executable under the SIL Open Font License 1.1 (licence texts in `assets/fonts/`):

- **Jacquarda Bastarda 9** by Sarah Cadigan-Fried (The Soft Type Project Authors)
- **Silkscreen** by Jason Kottke (The Silkscreen Project Authors)

## En español

Survivors-like en Rust + macroquad: Veyra, una súcubo villana, debe sobrevivir 5 minutos arrasando un pueblo. Ejecuta con `cargo run --release`.
