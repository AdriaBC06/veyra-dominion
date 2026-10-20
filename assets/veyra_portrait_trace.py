#!/usr/bin/env python3
"""Trace the title-screen Veyra portrait from the reference (PLAN.md 7.5, V14 phase 2).

Cuts Veyra out of assets/refs/veyra_title_pose.png (horns to waist, no wings, no moon),
downscales it to the title size (x0.67), quantises it to a closed palette seeded from Veyra's
game palette and writes assets/veyra_portrait.trace.js: a pal() call plus the traced grid as
TRACE rows. assets/veyra_portrait.px.js includes it and does the hand cleanup on top.

    python3 assets/veyra_portrait_trace.py [--debug DIR]

Coordinates below are in reference pixels (1024x1536).
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage

ROOT = Path(__file__).resolve().parent.parent
REF = ROOT / "assets/refs/veyra_title_pose.png"
OUT = ROOT / "assets/veyra_portrait.trace.js"

SCALE = 0.67
CROP = (160, 40, 840, 577)          # ref box: horns at the top, waist at the bottom edge
N_COLOURS = 32

# Whole figure: hair, head, raised arm, torso. Gaps inside it (moon, wings, black) are removed
# by colour unless they fall in PROTECT.
FIGURE = [(500, 86), (530, 92), (548, 108), (575, 140), (598, 165), (612, 182), (630, 186),
          (619, 205), (614, 240), (632, 275), (657, 300), (692, 330), (732, 366), (767, 430),
          (797, 480), (817, 530), (830, 577), (182, 577), (192, 553), (216, 518), (238, 480),
          (246, 450), (240, 430), (250, 395), (268, 357), (295, 320), (330, 270), (352, 238),
          (368, 248), (396, 244), (402, 228), (409, 200), (422, 170), (445, 125), (470, 96)]
HORN_L = [(405, 84), (399, 92), (392, 105), (387, 120), (389, 136), (397, 148), (405, 158),
          (412, 165), (440, 160), (448, 140), (428, 136), (415, 132), (406, 122), (401, 110),
          (401, 98), (406, 88)]
HORN_R = [(575, 42), (591, 52), (604, 70), (611, 90), (609, 108), (601, 124), (591, 136),
          (580, 147), (565, 145), (548, 128), (544, 117), (548, 112), (560, 104), (574, 94),
          (584, 82), (588, 68), (585, 55)]
# Face, neck, chest, arm and hand: kept whole (skin, lips and gems are as red as the moon).
PROTECT = [(440, 150), (600, 150), (614, 200), (610, 300), (640, 340), (700, 380), (705, 577),
           (420, 577), (400, 500), (395, 420), (395, 385), (300, 420), (258, 446), (238, 430),
           (248, 395), (266, 357), (293, 320), (328, 270), (350, 236), (400, 246), (430, 250)]
# Moon seen between the raised forearm, the shoulder and the hair (inside PROTECT).
POCKET = [(345, 275), (380, 270), (410, 277), (435, 297), (440, 310), (402, 305), (392, 325),
          (385, 355), (340, 360), (330, 330)]

# Leather zones: violet sheen here is glove, sleeve, corset or choker, not hair.
ARM = [(352, 236), (400, 244), (412, 262), (405, 300), (400, 345), (410, 385), (395, 420),
       (300, 425), (258, 446), (238, 430), (248, 395), (266, 357), (293, 320), (328, 270)]
BODICE = [(455, 420), (480, 395), (530, 400), (600, 420), (650, 400), (690, 420), (705, 470),
          (700, 577), (455, 577)]
CHOKER = [(535, 300), (600, 300), (605, 335), (570, 350), (535, 335)]
# Face: no cluster smoothing here, eyes and mouth are too small for it.
FACE = [(468, 190), (560, 160), (600, 200), (590, 260), (560, 300), (520, 305), (478, 280)]

# Closed palette: one short ramp per material (dark -> light), from Veyra's game colours
# (assets/veyra.px.js) with the skin pushed redder as the reference.
RAMPS = {
    "ink": {"k": "#16091a"},
    "skin": {"a": "#521c1c", "b": "#7a2e28", "c": "#a04636", "d": "#c2604a", "e": "#e2876a"},
    "hair": {"f": "#1d0b2c", "g": "#2e1046", "h": "#47196c", "i": "#6a2a96", "j": "#9450c0",
             "l": "#b8467e"},
    "horn": {"m": "#3c0814", "n": "#6a0e20", "o": "#a8182e", "p": "#e04454", "q": "#f49a9a"},
    "leather": {"r": "#0f0812", "t": "#201226", "u": "#36223f", "U": "#58406a"},
    "gold": {"v": "#6a3e14", "w": "#b07428", "x": "#e0a844", "y": "#f8dc8a"},
    "red": {"A": "#6c0a20", "B": "#b81a36", "C": "#e85068", "D": "#ffb8c0"},
    "eye": {"E": "#d87818", "F": "#f8b828", "G": "#fff0d8"},
}
MATERIALS = list(RAMPS)
SMOOTH = {"skin", "hair", "leather", "horn"}     # big areas get their clusters cleaned


def poly_mask(size, pts):
    m = Image.new("L", size, 0)
    ImageDraw.Draw(m).polygon(pts, fill=255)
    return np.asarray(m) > 0


def srgb_to_lab(rgb):
    c = rgb / 255.0
    c = np.where(c > 0.04045, ((c + 0.055) / 1.055) ** 2.4, c / 12.92)
    xyz = c @ np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722],
                        [0.0193, 0.1192, 0.9505]]).T
    xyz /= np.array([0.95047, 1.0, 1.08883])
    f = np.where(xyz > 0.008856, np.cbrt(xyz), 7.787 * xyz + 16 / 116)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]),
                     200 * (f[..., 1] - f[..., 2])], -1)


def cutout(rgb):
    """Alpha mask (full ref resolution) of Veyra without moon, wings and background."""
    size = (rgb.shape[1], rgb.shape[0])
    fig = poly_mask(size, FIGURE)
    horns = poly_mask(size, HORN_L) | poly_mask(size, HORN_R)
    keep = poly_mask(size, PROTECT)
    f = rgb.astype(float)
    mx = f.max(-1) + 1e-6
    r, g, b = f[..., 0], f[..., 1], f[..., 2]
    moon = (g / mx < 0.33) & (b / mx < 0.42)
    wing = (r > 1.35 * b) & (r > 70)        # wing membrane is red-magenta, hair is violet
    black = mx < 28
    loose = (fig & ~keep) | poly_mask(size, POCKET)
    a = (fig & ~(loose & (moon | wing | black))) | horns
    return a


def shifts(a, r=1, fill=0):
    """Stack of a shifted by every offset in a (2r+1)^2 window (edges padded with fill)."""
    p = np.pad(a, r, constant_values=fill)
    h, w = a.shape
    return np.stack([p[r + dy:r + dy + h, r + dx:r + dx + w]
                     for dy in range(-r, r + 1) for dx in range(-r, r + 1)], -1)


def majority(lbl, n, r=1, valid=None):
    """Most common label (0..n-1) in each (2r+1)^2 window and its count; invalid pixels don't vote."""
    box = np.ones((2 * r + 1, 2 * r + 1), np.int16)
    ok = np.ones(lbl.shape, bool) if valid is None else valid
    cnt = np.stack([ndimage.convolve(((lbl == i) & ok).astype(np.int16), box, mode="constant")
                    for i in range(n)], -1)
    return cnt.argmax(-1), cnt.max(-1)


def guide():
    """Veyra cut out and downscaled: colour, coverage and the region masks at sprite scale."""
    rgb = np.asarray(Image.open(REF).convert("RGB"))
    a = cutout(rgb)
    x0, y0, x1, y1 = CROP
    rgb, a = rgb[y0:y1, x0:x1], a[y0:y1, x0:x1]
    w, h = round((x1 - x0) * SCALE), round((y1 - y0) * SCALE)
    pre = np.dstack([rgb * a[..., None], a * 255.0]).astype(np.float32)
    small = np.stack([np.asarray(Image.fromarray(pre[..., i]).resize((w, h), Image.BOX))
                      for i in range(4)], -1)
    alpha = small[..., 3] / 255.0
    col = small[..., :3] / np.maximum(alpha[..., None], 1e-6)
    region = lambda pts: poly_mask((w, h), [((x - x0) * SCALE, (y - y0) * SCALE) for x, y in pts])
    regions = {k: region(v) for k, v in dict(horn=HORN_L, horn2=HORN_R, arm=ARM, bodice=BODICE,
               choker=CHOKER, face=FACE, protect=PROTECT).items()}
    regions["horn"] |= regions.pop("horn2")
    return col, alpha > 0.5, regions


def clean_alpha(solid, col):
    """Fill pinholes, drop specks and smooth the silhouette; new pixels take a neighbour's colour."""
    s = solid.copy()
    holes, n = ndimage.label(~s)
    sizes = ndimage.sum(np.ones_like(holes), holes, range(n + 1))
    s |= (sizes[holes] <= 6) & (holes > 0)
    s = shifts(s.astype(np.int8)).sum(-1) >= 5
    parts, n = ndimage.label(s)
    sizes = ndimage.sum(np.ones_like(parts), parts, range(n + 1))
    s &= sizes[parts] >= 12
    _, idx = ndimage.distance_transform_edt(~solid, return_indices=True)
    col = np.where((s & ~solid)[..., None], col[idx[0], idx[1]], col)
    return s, col


def materials(col, solid, R):
    """Material index per pixel (MATERIALS order), -1 outside."""
    c = col / 255.0
    mx, mn = c.max(-1), c.min(-1)
    sat = (mx - mn) / (mx + 1e-6)
    r, g, b = c[..., 0], c[..., 1], c[..., 2]
    hue = np.degrees(np.arctan2(np.sqrt(3) * (g - b), 2 * r - g - b)) % 360
    gr = g / (r + 1e-6)
    I = MATERIALS.index
    dark = mx < 0.16
    violet = (hue > 250) & (hue < 335)
    gold = (hue > 22) & (hue < 60) & (sat > 0.45) & (mx > 0.4)
    warm = ~violet & ((hue < 40) | (hue > 345)) & (mx >= 0.16)
    m = np.full(solid.shape, -1)
    m[violet] = I("hair")
    m[warm & (gr > 0.28)] = I("skin")
    red = warm & (gr <= 0.28)
    m[red & R["protect"] & (mx > 0.45)] = I("red")
    m[red & R["protect"] & (mx <= 0.45)] = I("skin")
    m[red & ~R["protect"]] = I("hair")          # moonlit rim on the hair
    m[gold] = I("gold")
    m[gold & R["face"]] = I("eye")
    leather = R["arm"] | R["bodice"] | R["choker"]
    m[leather & (violet | dark)] = I("leather")
    m[R["horn"] & ~dark & ~gold] = I("horn")
    m[R["horn"] & dark] = I("horn")
    m[dark & R["face"] & (m < 0)] = I("ink")
    # Remaining dark pixels join the material around them (hair shadow, leather, ...).
    todo = (m < 0) & solid
    known = (m >= 0) & solid & (m != I("ink"))
    for rad in (3, 6):
        best, cnt = majority(np.where(known, m, 0), len(MATERIALS), rad, known)
        fill = todo & (cnt > 0)
        m[fill] = best[fill]
        todo &= ~fill
    m[todo] = I("leather")
    big = [I(x) for x in SMOOTH]
    for _ in range(2):                         # material areas as clean shapes, not grain
        sel = np.isin(m, big) & ~R["face"] & solid
        best, cnt = majority(np.where(sel, m, 0), len(MATERIALS), 2, sel)
        upd = sel & (cnt >= 9)
        m[upd] = best[upd]
    m[~solid] = -1
    return m


def shade(col, m, solid, R):
    """Key per pixel: nearest ramp colour (Lab) of the pixel's material, then cluster cleanup."""
    keys, idx = [], {}
    for mat in MATERIALS:
        for k in RAMPS[mat]:
            idx[k] = len(keys)
            keys.append(k)
    key_mat = np.array([i for i, mat in enumerate(MATERIALS) for _ in RAMPS[mat]])
    out = np.full(m.shape, -1)
    lab = srgb_to_lab(np.clip(col, 0, 255))
    for i, mat in enumerate(MATERIALS):
        sel = m == i
        if not sel.any():
            continue
        if mat in SMOOTH:                      # soften the photo grain before picking shades
            w = sel.astype(float)
            blur = np.stack([ndimage.gaussian_filter(lab[..., j] * w, 1.6) for j in range(3)], -1)
            lab_m = blur / np.maximum(ndimage.gaussian_filter(w, 1.6), 1e-6)[..., None]
        else:
            lab_m = lab
        ramp = list(RAMPS[mat])
        rl = srgb_to_lab(np.array([[int(RAMPS[mat][k][j:j + 2], 16) for j in (1, 3, 5)]
                                   for k in ramp], float))
        d = ((lab_m[sel][:, None] - rl[None]) ** 2).sum(-1)
        out[sel] = [idx[ramp[j]] for j in d.argmin(1)]
    smooth = np.isin(m, [MATERIALS.index(x) for x in SMOOTH]) & ~R["face"]
    for _ in range(2):                         # majority within the same material: clean clusters
        best, cnt = majority(np.where(solid, out, 0), len(keys), 1, solid)
        same = key_mat[best] == m
        upd = smooth & same & (cnt >= 4)
        out[upd] = best[upd]
    # Orphans: a pixel with no equal neighbour takes the most common one.
    eq = (shifts(out, 1, -2) == out[..., None]).sum(-1) - 1
    best, _ = majority(np.where(solid, out, 0), len(keys), 1, solid)
    orphan = solid & (eq == 0) & smooth
    out[orphan] = best[orphan]
    return out, keys


def thin(img):
    """Zhang-Suen thinning to 1px lines."""
    img = img.astype(np.uint8)
    while True:
        changed = False
        for step in (0, 1):
            p = np.pad(img, 1)
            P = [p[1:-1, 1:-1], p[:-2, 1:-1], p[:-2, 2:], p[1:-1, 2:], p[2:, 2:], p[2:, 1:-1],
                 p[2:, :-2], p[1:-1, :-2], p[:-2, :-2]]
            n = sum(x.astype(int) for x in P[1:])
            seq = P[1:] + [P[1]]
            A = sum(((seq[i] == 0) & (seq[i + 1] == 1)).astype(int) for i in range(8))
            if step == 0:
                c = (P[1] * P[3] * P[5] == 0) & (P[3] * P[5] * P[7] == 0)
            else:
                c = (P[1] * P[3] * P[7] == 0) & (P[1] * P[5] * P[7] == 0)
            d = (img == 1) & (n >= 2) & (n <= 6) & (A == 1) & c
            if d.any():
                img[d] = 0
                changed = True
        if not changed:
            return img.astype(bool)


def gold_lines(shape):
    rgb = np.asarray(Image.open(REF).convert("RGB")).astype(float) / 255
    mx, mn = rgb.max(-1), rgb.min(-1)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    hue = np.degrees(np.arctan2(np.sqrt(3) * (g - b), 2 * r - g - b)) % 360
    gold = (hue > 22) & (hue < 60) & ((mx - mn) / (mx + 1e-6) > 0.45) & (mx > 0.45)
    x0, y0, x1, y1 = CROP
    f = np.asarray(Image.fromarray(gold[y0:y1, x0:x1].astype(np.float32)).resize(shape[::-1], Image.BOX))
    return thin(f > 0.15)


def main():
    col, solid, R = guide()
    solid, col = clean_alpha(solid, col)
    m = materials(col, solid, R)
    out, keys = shade(col, m, solid, R)
    ink = keys.index("k")
    # Ink line where an area meets a lighter area of another material.
    big = np.isin(m, [MATERIALS.index(x) for x in SMOOTH])
    lum = srgb_to_lab(np.clip(col, 0, 255))[..., 0]
    lum_s = ndimage.gaussian_filter(lum, 1.5)
    edge = np.zeros_like(solid)
    for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        nm = np.roll(m, (dy, dx), (0, 1))
        nl = np.roll(lum_s, (dy, dx), (0, 1))
        edge |= big & (nm >= 0) & (nm != m) & np.isin(nm, [MATERIALS.index(x) for x in SMOOTH]) & (nl > lum_s)
    out[edge & ~R["face"]] = ink
    # Gold filigree as 1px lines (it is thinner than a pixel at this scale).
    gl = gold_lines(solid.shape)
    gl &= solid & ~R["face"]
    out[gl] = np.where(lum[gl] > 62, keys.index("y"), keys.index("x"))
    h, w = out.shape
    lut = np.array(keys + ["."])
    grid = lut[np.where(solid, out, len(keys))]
    with open(OUT, "w") as f:
        f.write("// Generated by assets/veyra_portrait_trace.py; do not edit by hand.\n")
        f.write("pal({\n")
        for mat in MATERIALS:
            f.write("  " + ", ".join(f'{k}: "{v}"' for k, v in RAMPS[mat].items()) + f",  // {mat}\n")
        f.write("})\n")
        f.write(f"const TRACE_W = {w}, TRACE_H = {h}\n")
        f.write("const TRACE = [\n")
        for row in grid:
            f.write(f'  "{"".join(row)}",\n')
        f.write("].join(\"\\n\")\n")
    print(f"{OUT.relative_to(ROOT)}: {w}x{h}, {len(np.unique(out[solid]))} colours")


if __name__ == "__main__":
    main()
