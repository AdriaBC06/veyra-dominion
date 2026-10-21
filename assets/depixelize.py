# Fake pixel art -> real pixel art: sample each grid cell's dominant colour, hard alpha,
# palette quantisation, then remove noise pixels that are close in colour to their surroundings.
import sys
from PIL import Image
import numpy as np

src, out = sys.argv[1], sys.argv[2]
PX, OX, PY, OY = 4.015, 1.25, 4.02, 2.5
NCOL = int(sys.argv[3]) if len(sys.argv) > 3 else 48

im = np.asarray(Image.open(src).convert("RGBA")).astype(np.int32)
H, W = im.shape[:2]
nx = int((W - OX) / PX); ny = int((H - OY) / PY)
cells = np.zeros((ny, nx, 4), np.uint8)
for j in range(ny):
    y0 = int(round(OY + j * PY)) + 1; y1 = int(round(OY + (j + 1) * PY))
    for i in range(nx):
        x0 = int(round(OX + i * PX)) + 1; x1 = int(round(OX + (i + 1) * PX))
        blk = im[y0:y1, x0:x1].reshape(-1, 4)
        if (blk[:, 3] > 128).sum() * 2 < len(blk):
            continue
        blk = blk[blk[:, 3] > 128]
        # dominant colour: the pixel closest to the block median (keeps a real colour, no blending)
        med = np.median(blk[:, :3], axis=0)
        k = np.argmin(((blk[:, :3] - med) ** 2).sum(1))
        cells[j, i, :3] = blk[k, :3]; cells[j, i, 3] = 255

img = Image.fromarray(cells, "RGBA")
# palette quantisation on the opaque pixels only
rgb = img.convert("RGB")
q = rgb.quantize(colors=NCOL, method=Image.Quantize.LIBIMAGEQUANT, dither=Image.Dither.NONE)
pal = np.array(q.getpalette()[: NCOL * 3]).reshape(-1, 3)
# small features would lose to the big areas: reserve their colours (eyes, pupils, gems)
FORCED = ["#f08a10", "#ffc830", "#fff0d8", "#1a0408", "#ff2040", "#ff9aa8", "#b8101e"]
pal = np.vstack([pal, [[int(h[i:i + 2], 16) for i in (1, 3, 5)] for h in FORCED]])
alpha = cells[..., 3] > 0

def lab(c):
    c = c / 255.0
    c = np.where(c > 0.04045, ((c + 0.055) / 1.055) ** 2.4, c / 12.92)
    M = np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]])
    xyz = c @ M.T / np.array([0.9505, 1.0, 1.089])
    f = np.where(xyz > 0.008856, np.cbrt(xyz), 7.787 * xyz + 16 / 116)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])], -1)
pl = lab(pal.astype(float))
cl = lab(cells[..., :3].astype(float))
idx = np.argmin(((cl[..., None, :] - pl[None, None]) ** 2).sum(-1), axis=-1)
dist = np.sqrt(((pl[:, None] - pl[None]) ** 2).sum(-1))

# despeckle: a pixel unlike all 4 neighbours, whose 8-neighbourhood is dominated by one similar colour
for _ in range(2):
    new = idx.copy()
    for y in range(1, ny - 1):
        for x in range(1, nx - 1):
            if not alpha[y, x]:
                continue
            c = idx[y, x]
            n4 = [(y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)]
            if any(idx[a, b] == c and alpha[a, b] for a, b in n4):
                continue
            n8 = [idx[a, b] for a in (y - 1, y, y + 1) for b in (x - 1, x, x + 1) if (a, b) != (y, x) and alpha[a, b]]
            if len(n8) < 6:
                continue
            vals, cnt = np.unique(n8, return_counts=True)
            m = vals[np.argmax(cnt)]
            if cnt.max() >= 5 and dist[c, m] < 18:
                new[y, x] = m
    idx = new
# lone opaque pixels on the silhouette edge
for y in range(1, ny - 1):
    for x in range(1, nx - 1):
        if alpha[y, x] and sum(alpha[a, b] for a, b in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1))) <= 1:
            alpha[y, x] = False

outa = np.zeros((ny, nx, 4), np.uint8)
outa[..., :3] = pal[idx]; outa[..., 3] = np.where(alpha, 255, 0)
Image.fromarray(outa, "RGBA").save(out)
print(nx, ny, "colours", len(np.unique(idx[alpha])))
