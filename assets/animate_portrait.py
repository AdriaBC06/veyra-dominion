# Layered idle animation for the title Veyra, pixel-art safe.
# - wings and tail are cut into layers and swung as rigid parts with RotSprite (Scale2x x3, rotate,
#   sample back), so they keep clean pixel clusters at every angle
# - the wing layers are extended under the hair, so no holes open when they move
# - breathing lifts everything above the waist 1px (the seam hides in the corset)
# - a wave runs down the outer hair; embers and sparks drift up; gems glint; one blink
import sys, math
from PIL import Image, ImageDraw
import numpy as np

src, sheet_out, gif_out = sys.argv[1:4]
N, COLS, FPS = 48, 8, 12
PADX, PADT = 16, 16

orig = np.asarray(Image.open(src).convert("RGBA"))
H0, W0 = orig.shape[:2]
W, H = W0 + 2 * PADX, H0 + PADT
base = np.zeros((H, W, 4), np.uint8)
base[PADT:, PADX:PADX + W0] = orig

def P(pts):
    return [(x + PADX, y + PADT) for x, y in pts]

def poly_mask(pts):
    m = Image.new("L", (W, H), 0)
    ImageDraw.Draw(m).polygon(P(pts), fill=1)
    return np.asarray(m).astype(bool)

opaque = base[..., 3] > 0
r, g, b = (base[..., i].astype(int) for i in range(3))
purple = (b > r * 0.75) & (b > g)

# ---------------------------------------------------------------- layers
WING_L = poly_mask([(0, 20), (80, 20), (90, 60), (84, 85), (74, 110), (68, 135), (62, 160), (54, 185), (46, 216), (0, 216)]) & opaque
WING_R = poly_mask([(312, 20), (225, 20), (214, 60), (218, 95), (226, 120), (232, 150), (236, 185), (240, 216), (312, 216)]) & opaque
# where each wing continues hidden behind the hair/arm (filled so motion never opens holes)
WING_L_EXT = poly_mask([(60, 40), (100, 60), (104, 130), (76, 140)])
WING_R_EXT = poly_mask([(252, 40), (212, 60), (208, 130), (236, 140)])
# hair strands that stray into the wing outlines stay with the body
HAIRZONE = poly_mask([(36, 100), (92, 100), (92, 230), (36, 230)]) | poly_mask([(220, 100), (280, 100), (280, 230), (220, 230)])
# purple inside a wing outline is hair only if it connects to the hair outside the wing
def hair_inside(wing):
    from collections import deque
    cand = wing & HAIRZONE & purple
    seen = np.zeros_like(cand)
    q = deque()
    ys, xs = np.nonzero(purple & opaque & ~wing)
    for y, x in zip(ys, xs):
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < H and 0 <= nx < W and cand[ny, nx] and not seen[ny, nx]:
                seen[ny, nx] = True; q.append((ny, nx))
    while q:
        y, x = q.popleft()
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < H and 0 <= nx < W and cand[ny, nx] and not seen[ny, nx]:
                seen[ny, nx] = True; q.append((ny, nx))
    return seen
# gold below the wing claws is jewellery on the gloves, not wing
gold = (r > 150) & (g > 90) & (b < 110) & (r > b * 1.6)
low = np.zeros_like(opaque); low[80 + PADT:] = True
WING_L &= ~(gold & low); WING_R &= ~(gold & low)
FOOT_L, FOOT_R = WING_L.copy(), WING_R.copy()
WING_L &= ~hair_inside(WING_L)
WING_R &= ~hair_inside(WING_R)
# the tail, cut by hand: a tight outline around it that never reaches the hand (x < 234);
# inside it, everything that is not hair belongs to the tail (crimson body + its dark outline)
TAIL_POLY = poly_mask([(234, 219), (262, 221), (279, 227), (294, 246), (297, 268), (293, 289), (282, 301),
                       (264, 308), (249, 310), (249, 290), (251, 262), (250, 246), (240, 236), (234, 229)])
TAIL = TAIL_POLY & opaque & ~purple
# where the tail leaves the hair at its root, the hair stays in front
TAIL_ROOT_FRONT = poly_mask([(236, 286), (251, 286), (251, 311), (236, 311)]) & opaque & purple

def layer(mask):
    out = np.zeros_like(base)
    out[mask] = base[mask]
    return out

def grow(img, region, steps):
    """Fill transparent pixels of region from opaque neighbours (nearest-colour inpainting)."""
    img = img.copy()
    for _ in range(steps):
        a = img[..., 3] > 0
        new = img.copy()
        for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
            sh = np.roll(np.roll(img, dy, 0), dx, 1)
            take = region & ~a & (sh[..., 3] > 0) & (new[..., 3] == 0)
            new[take] = sh[take]
        img = new
    return img

# only where the body/hair covers it, a few pixels in from its edge, so the fill never shows at rest
covered = opaque & ~(WING_L | WING_R)
inner = covered.copy()
for _ in range(2):
    inner &= np.roll(covered, 1, 0) & np.roll(covered, -1, 0) & np.roll(covered, 1, 1) & np.roll(covered, -1, 1)
# small islands: whatever is cut off from its own part belongs to the other one
def islands(mask, limit=60):
    from collections import deque
    seen = np.zeros_like(mask); out = []
    for y0, x0 in zip(*np.nonzero(mask)):
        if seen[y0, x0]:
            continue
        comp, q = [], deque([(y0, x0)]); seen[y0, x0] = True
        while q:
            y, x = q.popleft(); comp.append((y, x))
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    ny, nx = y + dy, x + dx
                    if 0 <= ny < H and 0 <= nx < W and mask[ny, nx] and not seen[ny, nx]:
                        seen[ny, nx] = True; q.append((ny, nx))
        if len(comp) < limit:
            out.append(comp)
    return out
body_mask = opaque & ~(WING_L | WING_R)
for comp in islands(body_mask):
    ys, xs = zip(*comp)
    (WING_L if np.mean(xs) < W / 2 else WING_R)[list(ys), list(xs)] = True
for wm in (WING_L, WING_R):
    for comp in islands(wm.copy()):
        ys, xs = zip(*comp)
        wm[list(ys), list(xs)] = False
covered = opaque & ~(WING_L | WING_R)
wingL = grow(layer(WING_L), WING_L_EXT & (covered | WING_L), 12)
wingR = grow(layer(WING_R), WING_R_EXT & (covered | WING_R), 12)
# close the pinholes the hair left inside each wing (only where the wing surrounds them)
wingL = grow(wingL, FOOT_L & ~WING_L, 3)
wingR = grow(wingR, FOOT_R & ~WING_R, 3)
tail = layer(TAIL)

tail = layer(TAIL)
body = base.copy()
body[WING_L | WING_R | TAIL] = 0
# rebuild the hair under the spade: clone the strands from just left of the hole and carry the
# hair's right edge (its outline) down across the gap
hole = TAIL & poly_mask([(232, 218), (264, 218), (264, 262), (232, 262)])
edge_of = {}
for y in range(H):
    xs = np.nonzero(body[y, :, 3] & purple[y])[0]
    xs = xs[(xs >= 230 + PADX) & (xs < 270 + PADX)]
    if len(xs):
        edge_of[y] = xs.max()
ys_known = sorted(edge_of)
for y in range(H):
    row = np.nonzero(hole[y])[0]
    if not len(row):
        continue
    # hair edge for this row: interpolate from the nearest rows above and below the spade
    above = [k for k in ys_known if k < 218 + PADT and k >= 205 + PADT]
    below = [k for k in ys_known if k > 262 + PADT and k <= 275 + PADT]
    e0 = np.mean([edge_of[k] for k in above]) if above else 252 + PADX
    e1 = np.mean([edge_of[k] for k in below]) if below else 258 + PADX
    t = (y - (218 + PADT)) / 44
    edge = round(e0 + (e1 - e0) * t)
    for x in row:
        if x > edge:
            continue
        if x == edge:
            body[y, x] = (0x1a, 0x04, 0x30, 255)           # the hair's dark rim
            continue
        for k in range(8, 40):                           # nearest real hair texture to the left
            sx = x - k
            if body[y, sx, 3] and purple[y, sx] and not hole[y, sx]:
                body[y, x] = body[y, sx]
                break
root_front = layer(TAIL_ROOT_FRONT)

# ---------------------------------------------------------------- RotSprite
def pack(a):
    return (a[..., 0].astype(np.uint32) << 24) | (a[..., 1].astype(np.uint32) << 16) | (a[..., 2].astype(np.uint32) << 8) | a[..., 3]

def scale2x(a):
    k = pack(a)
    A = np.pad(k, 1, mode="edge")
    up, down, left, right = A[:-2, 1:-1], A[2:, 1:-1], A[1:-1, :-2], A[1:-1, 2:]
    h, w = k.shape
    out = np.zeros((h * 2, w * 2, 4), np.uint8)
    def pick(cond, nb_rgba, i, j):
        out[i::2, j::2] = np.where(cond[..., None], nb_rgba, a)
    pa = np.pad(a, ((1, 1), (1, 1), (0, 0)), mode="edge")
    U, D, L, R = pa[:-2, 1:-1], pa[2:, 1:-1], pa[1:-1, :-2], pa[1:-1, 2:]
    pick((left == up) & (left != down) & (up != right), U, 0, 0)
    pick((up == right) & (up != left) & (right != down), R, 0, 1)
    pick((down == left) & (down != right) & (left != up), L, 1, 0)
    pick((right == down) & (right != up) & (down != left), D, 1, 1)
    return out

def bbox(img):
    ys, xs = np.nonzero(img[..., 3])
    return xs.min(), ys.min(), xs.max() + 1, ys.max() + 1

def rotsprite(img, pivot, angle, margin=40):
    """Rotate a layer about pivot (canvas coords) by angle (radians, screen space)."""
    x0, y0, x1, y1 = bbox(img)
    x0, y0 = max(0, x0 - 2), max(0, y0 - 2)
    crop = img[y0:y1 + 2, x0:x1 + 2]
    big = scale2x(scale2x(scale2x(crop)))
    S = 8
    out = np.zeros_like(img)
    oy0, oy1 = max(0, y0 - margin), min(H, y1 + margin)
    ox0, ox1 = max(0, x0 - margin), min(W, x1 + margin)
    ys, xs = np.mgrid[oy0:oy1, ox0:ox1]
    c, s = math.cos(-angle), math.sin(-angle)
    dx, dy = xs + 0.5 - pivot[0], ys + 0.5 - pivot[1]
    sx, sy = pivot[0] + dx * c - dy * s, pivot[1] + dx * s + dy * c
    bx = np.floor((sx - x0) * S).astype(int)
    by = np.floor((sy - y0) * S).astype(int)
    ok = (bx >= 0) & (by >= 0) & (bx < big.shape[1]) & (by < big.shape[0])
    vals = np.zeros((*xs.shape, 4), np.uint8)
    vals[ok] = big[by[ok], bx[ok]]
    out[oy0:oy1, ox0:ox1] = vals
    return tidy(out)

def tidy(img):
    """Rotation leaves pinholes and stray edge pixels: fill the holes, drop the strays."""
    for _ in range(2):
        o = img[..., 3] > 0
        nbs = [np.roll(img, s_, ax) for s_ in (1, -1) for ax in (0, 1)]
        n = sum((nb[..., 3] > 0).astype(int) for nb in nbs)
        hole = ~o & (n >= 3)
        for nb in nbs:
            take = hole & (nb[..., 3] > 0)
            img[take] = nb[take]
            hole &= ~take
        stray = o & (n <= 1)
        img[stray] = 0
    return img

def over(dst, top):
    m = top[..., 3] > 0
    dst[m] = top[m]
    return dst

def shift(img, dx, dy):
    out = np.zeros_like(img)
    ys, xs = slice(max(0, dy), H + min(0, dy)), slice(max(0, dx), W + min(0, dx))
    yd, xd = slice(max(0, -dy), H + min(0, -dy)), slice(max(0, -dx), W + min(0, -dx))
    out[ys, xs] = img[yd, xd]
    return out

# ---------------------------------------------------------------- body motion
SEAM = 186 + PADT                          # breathing seam, inside the corset
HAIR_L = (28, 75, 150, 311)                # x0, x1, y0, y1 of the swaying outer hair
HAIR_R = (232, 270, 120, 220)

def breathe(img, lift):
    if not lift:
        return img
    out = img.copy()
    out[:SEAM - 1] = img[1:SEAM]
    return out

def sway(img, box, amp, ph, inner_right):
    x0, x1, y0, y1 = (box[0] + PADX, box[1] + PADX, box[2] + PADT, box[3] + PADT)
    out = img.copy()
    hair = (img[..., 2].astype(int) > img[..., 0].astype(int) * 0.75) & (img[..., 3] > 0)
    for y in range(y0, min(y1, H)):
        fy = (y - y0) / (y1 - y0)
        for x in range(x0, x1):
            # 0 at the inner edge (stays attached), full at the outer edge
            fx = (x1 - x) / (x1 - x0) if inner_right else (x - x0) / (x1 - x0)
            d = round(amp * fx * (0.3 + 0.7 * fy) * math.sin(ph - y / 18))
            if d == 0:
                continue
            sx = x - d
            src_hair = 0 <= sx < W and hair[y, sx]
            if src_hair:
                out[y, x] = img[y, sx]
            elif hair[y, x]:
                out[y, x] = 0
    return out

# ---------------------------------------------------------------- details
SKIN = np.array([0xcd, 0x4d, 0x52])
EYES = [(128, 63, 145, 69), (155, 57, 178, 64)]
gems = [(x, y) for y in range(H) for x in range(W) if tuple(base[y, x, :3]) == (0xff, 0x9a, 0xa8) and opaque[y, x]]

def blink(img, lift):
    for x0, y0, x1, y1 in EYES:
        for y in range(y0 + PADT - lift, y1 + PADT - lift):
            for x in range(x0 + PADX, x1 + PADX):
                c = img[y, x, :3].astype(int)
                if img[y, x, 3] and not (c[2] > c[0]) and c.sum() >= 40:
                    img[y, x, :3] = SKIN

EMBER = [(0xff, 0xf0, 0xd8), (0xff, 0xb0, 0x8a), (0xf0, 0x40, 0x6e), (0xa8, 0x16, 0x4a)]
rng = np.random.default_rng(7)
embers = [dict(x=rng.uniform(20, W - 20), y0=rng.uniform(0, H), loops=int(rng.integers(1, 3)),
               wob=rng.uniform(1, 3), ph=rng.uniform(0, 6.28), big=rng.random() < 0.55,
               front=rng.random() < 0.35) for _ in range(34)]

def draw_embers(img, f, front):
    for e in embers:
        if e["front"] != front:
            continue
        t = f / N
        y = (e["y0"] - t * e["loops"] * (H + 20)) % (H + 20) - 10
        life = 1 - y / H                                    # brightest low, fading as it rises
        x = e["x"] + math.sin(t * 6.283 * e["loops"] + e["ph"]) * e["wob"]
        xi, yi = int(round(x)), int(round(y))
        col = EMBER[min(3, max(0, int(life * 4)))]
        pts = [(0, 0)] + ([(1, 0), (-1, 0), (0, 1), (0, -1)] if e["big"] and life < 0.75 else [])
        for dx, dy in pts:
            if 0 <= xi + dx < W and 0 <= yi + dy < H:
                img[yi + dy, xi + dx] = (*(col if (dx, dy) == (0, 0) else EMBER[min(3, int(life * 4) + 1)]), 255)

def glint(img, f, lift):
    for i, (gx, gy) in enumerate(gems):
        k = (f - i * 5) % N
        gy -= lift if gy < SEAM else 0
        arms = {0: 1, 1: 2, 2: 1}.get(k)
        if arms is None:
            continue
        img[gy, gx, :3] = (0xff, 0xff, 0xf0)
        for a in range(1, arms + 1):
            for dx, dy in ((a, 0), (-a, 0), (0, a), (0, -a)):
                if img[gy + dy, gx + dx, 3]:
                    img[gy + dy, gx + dx, :3] = (0xff, 0xe8, 0xb0) if a == 1 else (0xf0, 0x9a, 0x80)

# ---------------------------------------------------------------- frames
_n = sum(np.roll(opaque, s_, ax).astype(int) for s_ in (1, -1) for ax in (0, 1))
BASE_HOLES = ~opaque & (_n >= 3)
PIV_L, PIV_R = (88 + PADX, 112 + PADT), (224 + PADX, 112 + PADT)
PIV_T = (252 + PADX, 300 + PADT)
frames = []
for f in range(N):
    ph = f / N * 2 * math.pi
    lift = 1 if math.sin(ph - 0.6) > 0.15 else 0
    flap = math.radians(7.5) * math.sin(ph)
    img = np.zeros((H, W, 4), np.uint8)
    draw_embers(img, f, front=False)
    over(img, shift(rotsprite(wingL, PIV_L, flap), 0, -lift))
    over(img, shift(rotsprite(wingR, PIV_R, -flap), 0, -lift))
    bd = breathe(body, lift)
    bd = sway(bd, HAIR_L, 3.0, ph * 2, inner_right=True)
    bd = sway(bd, HAIR_R, 2.0, ph * 2 + 1.5, inner_right=False)
    over(img, bd)
    # tail: lags the wings, swings from its root behind the thigh
    over(img, rotsprite(tail, PIV_T, math.radians(7) * math.sin(ph * 2 - 1.2)))
    over(img, root_front)
    if f in (36, 37):
        blink(img, lift)
    glint(img, f, lift)
    # pinholes the motion opened between strands (gaps the original already had are kept)
    for _ in range(2):
        o = img[..., 3] > 0
        nbs = [np.roll(img, s_, ax) for s_ in (1, -1) for ax in (0, 1)]
        n = sum((nb[..., 3] > 0).astype(int) for nb in nbs)
        hole = ~o & (n >= 3) & ~BASE_HOLES
        for nb in nbs:
            take = hole & (nb[..., 3] > 0)
            img[take] = nb[take]
            hole &= ~take
    draw_embers(img, f, front=True)
    frames.append(img)

rows = math.ceil(N / COLS)
sheet = np.zeros((rows * H, COLS * W, 4), np.uint8)
for i, fr in enumerate(frames):
    sheet[(i // COLS) * H:(i // COLS + 1) * H, (i % COLS) * W:(i % COLS + 1) * W] = fr
Image.fromarray(sheet, "RGBA").save(sheet_out)
gif = []
for fr in frames:
    bg = Image.new("RGBA", (640, 360), (30, 14, 30, 255)); bg.alpha_composite(Image.fromarray(fr, "RGBA"), (640 - W, 360 - H))
    gif.append(bg.resize((1280, 720), Image.NEAREST).convert("P", palette=Image.ADAPTIVE))
gif[0].save(gif_out, save_all=True, append_images=gif[1:], duration=round(1000 / FPS), loop=0)
print("frame", W, H, "sheet", sheet.shape, "gems", len(gems))
