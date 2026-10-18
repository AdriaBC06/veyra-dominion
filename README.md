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
| Esc | Pause |
| Enter | Start / confirm |
| 1 / 2 / 3 | Pick upgrade |
| R | Restart |
| E | Endless mode (after victory) |
| T | Back to title |
| F3 | Debug overlay |

## Development

See [PLAN.md](PLAN.md) for architecture, tasks and progress.

## Licence

GPL-3.0. See [LICENSE](LICENSE).

## En español

Survivors-like en Rust + macroquad: Veyra, una súcubo villana, debe sobrevivir 5 minutos arrasando un pueblo. Ejecuta con `cargo run --release`.
