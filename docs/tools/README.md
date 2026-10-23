# Dev tools

## `capture-hack.patch`: screen captures without a human

A temporary patch that makes the game capture one screen by itself and quit, used to show UI work to the user for approval. **Never commit it applied**: apply it, take captures, revert it before any commit.

```sh
git apply docs/tools/capture-hack.patch      # on
git apply -R docs/tools/capture-hack.patch   # off (do this before committing)
```

What it adds (`main.rs`, `scenes/mod.rs`, `scenes/run.rs`, `assets.rs`):

- `VEYRA_CAPTURE=<kind>:<out.png>`: resizes the window to 1280×720, drives the game to `<kind>` and writes the screen at frame 640, then exits. Kinds: `pause`, `levelup`, `offering`, `gameover`, `victory` (a run with the autopilot and Arts levelled), `title`, `settings`, `credits`.
- `VEYRA_SKIN=stone|gilt`: UI kit skin (F1 switches it in game).
- `VEYRA_BLOOD=<0..1>`: forces the UI blood level (normally `1 - e^(-kills/900)`).
- `VEYRA_NOTEXT=1`: draws no text, for lettering font mock-ups over the real panels.

Run it with the sandbox off (needs the GPU window) and a scratch data dir, so the run doesn't touch the real lifetime stats:

```sh
XDG_DATA_HOME=/tmp/veyra-xdg VEYRA_BLOOD=0.5 VEYRA_CAPTURE=pause:/tmp/pause.png timeout 60 cargo run -q
```

Dark captures look lighter in most image viewers than in game: sample pixels (e.g. with PIL) before judging contrast.

If the patch stops applying after code changes, re-create it by hand from the description above and refresh this file with `git diff > docs/tools/capture-hack.patch` (only the hack hunks).

## `assets/fonts/veyra/`: Veyra's own pixel fonts (work in progress)

Fonts drawn as code (V19b, pending review). See design.md 7.14. Build a TTF with `python3 assets/fonts/veyra/<font>.py <out.ttf>` (needs `fontTools`), preview with `python3 assets/fonts/veyra/preview.py <font> <out.png> 'Line one|Line two' [row step]` (needs Pillow).
