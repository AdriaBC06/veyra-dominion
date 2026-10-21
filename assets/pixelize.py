#!/usr/bin/env python3
"""Redraw an image as pixel art with Qwen-Image-Edit-2511 (ComfyUI) + Pixel Art Fixer.

Pipeline (PLAN.md 7.5, V14 phase 2):
  1. crop the input (optional) and resize it to the generation size (~1 MP, a multiple of 16);
  2. ComfyUI API: Qwen-Image-Edit-2511 redraws it as pixel art on a flat green background;
  3. Pixel Art Fixer finds the pseudo-grid and rebuilds every cell as one real pixel;
  4. the green is keyed out and the colours snapped to a closed palette.

Writes into OUT_DIR, per seed: ai_<seed>.png (raw generation), native_<seed>.png (detected grid),
forced_<seed>.png (grid forced to --size) and *_x4.png previews of both.

    ~/.local/src/pixel-art-fixer/python/.venv/bin/python assets/pixelize.py \\
        assets/refs/veyra_title_pose.png OUT_DIR --crop veyra --size veyra --palette veyra [--fast] [--seeds 3] [--blocks 4]

Needs ComfyUI on localhost:8188 (~/comfyui/start-comfyui.sh) and the fixer checked out at
~/.local/src/pixel-art-fixer (python/ venv with `detector -> pixelfixer` symlink).
"""
import argparse
import json
import math
import random
import shutil
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

import numpy as np
from PIL import Image

HOST = "http://127.0.0.1:8188"
COMFY = Path.home() / "comfyui"
FIXER = Path.home() / ".local/src/pixel-art-fixer/python"
sys.path.insert(0, str(FIXER))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from detector import detect  # noqa: E402
from detector.reconstruct import reconstruct  # noqa: E402

KEY = (0, 255, 0)
PROMPT = (
    "Redraw this exact image as high-quality hand-made pixel art, like a detailed 16-bit "
    "JRPG character portrait. Keep exactly the same character, pose, framing, face, expression, "
    "eyes, hair, horns, outfit and proportions. Big chunky square pixels on a strict grid, each "
    "art pixel about {px} screen pixels wide. Clean dark outlines, a limited palette, flat cel "
    "shading with hard-edged clusters of colour, no anti-aliasing, no gradients, no blur, no "
    "noise, no dithering. Replace the whole background with flat solid pure green (#00FF00)."
)
# --blocks: the input is pre-reduced to a blocky draft, so the grid is given and the model only redraws.
# A clean anime input is otherwise returned almost unchanged (S11c).
BLOCKS_PROMPT = (
    "This is a rough blocky draft on a grid of {px}x{px}-pixel blocks. Redraw it as clean, high-quality "
    "hand-made pixel art, like a detailed 16-bit JRPG character portrait, on exactly the same grid: every "
    "art pixel is one solid {px}x{px} block, aligned with the draft's blocks. Keep exactly the same character, "
    "pose, framing, face, expression (closed-lip smirk showing two small fangs), eyes, hair, horns, wings, "
    "outfit and proportions. Clean dark outlines, a limited palette, flat cel shading with hard-edged clusters "
    "of colour, crisp gold trims, readable eyes. No anti-aliasing, no gradients, no blur, no noise, no "
    "dithering. Keep the background flat solid pure green (#00FF00)."
)
NEGATIVE = "blurry, photo, smooth gradients, anti-aliasing, jpeg artifacts, noise, text, watermark"


def call(path, data=None):
    req = urllib.request.Request(HOST + path, data=json.dumps(data).encode() if data else None,
                                 headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(req))


def graph(image, prompt, seed, fast):
    steps, cfg = (8, 1.0) if fast else (40, 4.0)
    model = ["37", 0]
    g = {
        "37": {"class_type": "UNETLoader", "inputs": {"unet_name": "qwen_image_edit_2511_fp8mixed.safetensors", "weight_dtype": "default"}},
        "38": {"class_type": "CLIPLoader", "inputs": {"clip_name": "qwen_2.5_vl_7b_fp8_scaled.safetensors", "type": "qwen_image", "device": "default"}},
        "39": {"class_type": "VAELoader", "inputs": {"vae_name": "qwen_image_vae.safetensors"}},
        "41": {"class_type": "LoadImage", "inputs": {"image": image}},
    }
    if fast:
        g["61"] = {"class_type": "LoraLoaderModelOnly", "inputs": {"model": model, "lora_name": "Qwen-Image-Edit-2511-Lightning-8steps-V1.0-bf16.safetensors", "strength_model": 1.0}}
        model = ["61", 0]
    g.update({
        "66": {"class_type": "ModelSamplingAuraFlow", "inputs": {"model": model, "shift": 3.0}},
        "75": {"class_type": "CFGNorm", "inputs": {"model": ["66", 0], "strength": 1.0}},
        "76": {"class_type": "TextEncodeQwenImageEditPlus", "inputs": {"clip": ["38", 0], "prompt": prompt, "vae": ["39", 0], "image1": ["41", 0]}},
        "77": {"class_type": "TextEncodeQwenImageEditPlus", "inputs": {"clip": ["38", 0], "prompt": NEGATIVE, "vae": ["39", 0], "image1": ["41", 0]}},
        "70": {"class_type": "FluxKontextMultiReferenceLatentMethod", "inputs": {"conditioning": ["76", 0], "reference_latents_method": "index_timestep_zero"}},
        "78": {"class_type": "FluxKontextMultiReferenceLatentMethod", "inputs": {"conditioning": ["77", 0], "reference_latents_method": "index_timestep_zero"}},
        "88": {"class_type": "VAEEncode", "inputs": {"pixels": ["41", 0], "vae": ["39", 0]}},
        "3": {"class_type": "KSampler", "inputs": {
            "model": ["75", 0], "positive": ["70", 0], "negative": ["78", 0], "latent_image": ["88", 0],
            "seed": seed, "steps": steps, "cfg": cfg, "sampler_name": "euler", "scheduler": "simple", "denoise": 1.0}},
        "8": {"class_type": "VAEDecode", "inputs": {"samples": ["3", 0], "vae": ["39", 0]}},
        "9": {"class_type": "SaveImage", "inputs": {"images": ["8", 0], "filename_prefix": "pixelize"}},
    })
    return g


def generate(image, prompt, seed, fast):
    pid = call("/prompt", {"prompt": graph(image, prompt, seed, fast)})["prompt_id"]
    t0 = time.time()
    while pid not in (h := call(f"/history/{pid}")):
        time.sleep(2)
    st = h[pid]["status"]
    if st["status_str"] != "success":
        raise SystemExit(json.dumps(st["messages"][-1], indent=1))
    im = next(i for o in h[pid]["outputs"].values() for i in o.get("images", []))
    q = urllib.parse.urlencode({"filename": im["filename"], "subfolder": im.get("subfolder", ""), "type": "output"})
    print(f"  seed {seed}: generated in {time.time() - t0:.0f} s")
    return Image.open(urllib.request.urlopen(f"{HOST}/view?{q}")).convert("RGB")


def key_out(rgba):
    """Green background -> transparent (green is not in Veyra's colours)."""
    c = rgba[..., :3].astype(int)
    bg = (c[..., 1] > 150) & (c[..., 1] > c[..., 0] + 60) & (c[..., 1] > c[..., 2] + 60)
    rgba[bg, 3] = 0
    return rgba


def snap(rgba, palette):
    if palette is None:
        return rgba
    c = rgba[..., :3].reshape(-1, 1, 3).astype(int)
    idx = ((c - palette[None]) ** 2).sum(-1).argmin(1)
    rgba[..., :3] = palette[idx].reshape(rgba.shape[:2] + (3,))
    return rgba


def save(rgba, path):
    img = Image.fromarray(rgba, "RGBA")
    img.save(path)
    img.resize((img.width * 4, img.height * 4), Image.NEAREST).save(path.with_name(path.stem + "_x4.png"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("out_dir")
    ap.add_argument("--crop", help="x0,y0,x1,y1 in input pixels, or 'veyra' (portrait CROP)")
    ap.add_argument("--size", required=True, help="WxH of the final sprite, or 'veyra' (CROP x SCALE)")
    ap.add_argument("--palette", default="veyra", help="'veyra', 'none' or a number of colours")
    ap.add_argument("--gen-mp", type=float, default=1.0, help="generation size in megapixels")
    ap.add_argument("--prompt")
    ap.add_argument("--blocks", type=int, help="pre-reduce the input to a draft of NxN blocks (art pixel size)")
    ap.add_argument("--seeds", type=int, default=1)
    ap.add_argument("--seed", type=int)
    ap.add_argument("--fast", action="store_true", help="Lightning LoRA, 8 steps")
    a = ap.parse_args()

    import veyra_portrait_trace as vt
    src = Image.open(a.image).convert("RGB")
    if a.crop:
        src = src.crop(vt.CROP if a.crop == "veyra" else tuple(map(int, a.crop.split(","))))
    if a.size == "veyra":
        tw, th = round(src.width * vt.SCALE), round(src.height * vt.SCALE)
    else:
        tw, th = map(int, a.size.split("x"))
    k = (a.gen_mp * 1e6 / (tw * th)) ** 0.5
    unit = math.lcm(16, a.blocks or 1)
    gw, gh = round(tw * k / unit) * unit, round(th * k / unit) * unit
    print(f"target {tw}x{th}, generation {gw}x{gh} (~{gw / tw:.1f} px per art pixel)")

    palette = None
    if a.palette == "veyra":
        palette = np.array([[int(v[i:i + 2], 16) for i in (1, 3, 5)]
                            for ramp in vt.RAMPS.values() for v in ramp.values()])
    out = Path(a.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    name = f"pixelize_{out.name}.png"     # per run: queued runs must not share an input file
    src = src.resize((gw, gh), Image.LANCZOS)
    if a.blocks:
        src = src.resize((gw // a.blocks, gh // a.blocks), Image.BOX).resize((gw, gh), Image.NEAREST)
    src.save(COMFY / "input" / name)
    shutil.copy(COMFY / "input" / name, out / "input.png")

    prompt = (a.prompt or (BLOCKS_PROMPT if a.blocks else PROMPT)).format(px=a.blocks or round(gw / tw))
    seeds = [a.seed] if a.seed is not None else [random.randint(0, 2**31) for _ in range(a.seeds)]
    for seed in seeds:
        ai = generate(name, prompt, seed, a.fast)
        ai.save(out / f"ai_{seed}.png")
        rgba = np.asarray(ai.convert("RGBA")).copy()
        r = detect(rgba)
        print(f"  detected {r['cols']}x{r['rows']} (step {r['step_x']:.2f}x{r['step_y']:.2f}, {r.get('consensus')})")
        for tag, (sx, sy, cols, rows) in {
            "native": (r["step_x"], r["step_y"], r["cols"], r["rows"]),
            "forced": (ai.width / tw, ai.height / th, tw, th),
        }.items():
            px = reconstruct(rgba, sx, sy, cols, rows, color="mode", palette_snap=False)
            px = np.asarray(Image.fromarray(np.asarray(px)).convert("RGBA")).copy()
            if a.palette.isdigit():
                q = Image.fromarray(px).convert("RGB").quantize(int(a.palette), method=Image.KMEANS)
                px[..., :3] = np.asarray(q.convert("RGB"))
            px = snap(key_out(px), palette)
            save(px, out / f"{tag}_{seed}.png")
    print(f"done: {out}")


if __name__ == "__main__":
    main()
