# Ashford's authored layout (V19d, design.md 7.14) -> assets/maps/ashford.map
# python3 docs/tools/ashford_layout.py                 writes the map
# python3 docs/tools/ashford_layout.py <S>             also renders <S>/ashford.png (needs
#   node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/village.px.js --out <S>/vil)
# Landmarks are placed by hand below; only the woods' trees and the meadow scatter are seeded.
import math, os, random, sys

COLS, ROWS, CELL = 160, 128, 16
START = (80, 62)
G = [['.'] * COLS for _ in range(ROWS)]
PROPS = []                       # (name, x, y) bottom-left in map pixels
SIZES = {'house_cottage': (52, 46, 2), 'house_long': (80, 60, 2), 'house_slate': (60, 58, 2),
         'chapel': (84, 104, 3)}  # w, h, footprint rows (the roof's depth on the ground)
rnd = random.Random(19)

def put(c, r, ch):
    if 0 <= c < COLS and 0 <= r < ROWS: G[r][c] = ch
def get(c, r): return G[r][c] if 0 <= c < COLS and 0 <= r < ROWS else 'T'
def prop(name, x, y): PROPS.append((name, int(x), int(y)))

# --- woods closing the border, thicker in the north-east
for r in range(ROWS):
    for c in range(COLS):
        d = min(c, r, COLS - 1 - c, ROWS - 1 - r)
        t = 6 + 2.2 * math.sin(c * 0.21) + 1.8 * math.sin(r * 0.17 + 1) + 1.2 * math.sin((c + r) * 0.09)
        if c > 120 and r < 36: t += 5 + 3 * math.sin(c * 0.3)
        if d < t: G[r][c] = 'T'

# --- river west to east across the south, flattened where the bridges cross
BRIDGES = [58, 118]
def river_row(c):
    for b in BRIDGES:
        if abs(c - b) <= 3: c = b
    return round(92 + 4 * math.sin(c / 17) + 2 * math.sin(c / 7))
for c in range(COLS):
    r0 = river_row(c)
    for r in range(r0, r0 + 3):
        if G[r][c] != 'T' or 8 < c < COLS - 9: G[r][c] = '~'
# pond in the north-east, by the woods
for r in range(40, 47):
    for c in range(128, 138):
        if ((c - 133) / 5.2) ** 2 + ((r - 43.2) / 3.4) ** 2 < 1: G[r][c] = '~'

# --- the ridge: a rocky outcrop across the north-west with a pass through it
for c in range(10, 58):
    mid = 24 + 6 * math.sin((c - 10) / 15) - (c - 10) * 0.12
    half = 1.6 + 0.9 * math.sin(c * 0.5)
    for r in range(int(mid - half), int(mid + half) + 1):
        if not 30 <= c <= 32: put(c, r, 'R')

# --- copses: tight stands of woods out in the meadows, and a second outcrop in the south-east;
# with the river and the ridge they make the lanes and pinch points the horde has to go round
def blob(cx, cy, rx, ry, ch, wob=0.25):
    for r in range(int(cy - ry - 2), int(cy + ry + 3)):
        for c in range(int(cx - rx - 2), int(cx + rx + 3)):
            a = math.atan2(r - cy, c - cx)
            k = 1 + wob * math.sin(3 * a + cx) + wob * 0.6 * math.sin(5 * a + cy)
            if ((c - cx) / rx) ** 2 + ((r - cy) / ry) ** 2 < k * k: put(c, r, ch)
for cx, cy, rx, ry in [(22, 92, 7, 5), (44, 30, 5, 3.5), (140, 82, 6, 8), (62, 16, 6, 3), (100, 86, 4, 3),
                       (16, 44, 4, 6), (138, 108, 5, 4), (76, 112, 3, 2.5), (120, 20, 4, 3)]:
    blob(cx, cy, rx, ry, 'T')
for c in range(124, 152):
    mid = 70 + 3 * math.sin(c / 6)
    for r in range(int(mid - 1.4), int(mid + 1.4) + 1):
        if c not in (136, 137): put(c, r, 'R')

# --- roads (2 cells wide) from the square to the lanes out
def road(*pts, w=2):
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        n = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        for k in range(n + 1):
            x, y = x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n
            for dx in range(w):
                for dy in range(w):
                    cc, rr = int(x) + dx, int(y) + dy
                    if get(cc, rr) == '.': put(cc, rr, ':')
r1, r2 = river_row(BRIDGES[0]), river_row(BRIDGES[1])
road((79, 46), (79, 52))                                           # chapel steps
road((70, 58), (52, 60), (34, 61), (20, 66), (4, 66))              # west lane
road((90, 58), (110, 60), (128, 59), (150, 57), (159, 57))         # east lane
road((80, 64), (72, 74), (62, 82), (BRIDGES[0], r1 - 2))           # south to the west bridge
road((BRIDGES[0], r1 + 3), (58, 104), (64, 116), (66, 127))        # on into the woods
road((116, 60), (118, r2 - 2))                                     # to the east bridge
road((BRIDGES[1], r2 + 3), (120, 108), (132, 116), (140, 127))
road((31, 66), (31, 20), (36, 4))                                  # north-west through the pass
road((88, 34), (92, 26), (100, 18), (104, 0))                      # north lane past the fields
road((86, 48), (88, 34))
for b, r0 in zip(BRIDGES, (r1, r2)):
    for r in range(r0, r0 + 3):
        put(b, r, '='); put(b + 1, r, '=')
    prop('bridge', b * CELL, r0 * CELL + 65)

# --- the square: cobbles round the well, stalls and lamps
for r in range(48, 70):
    for c in range(66, 96):
        if ((c - 80.5) / 10.5) ** 2 + ((r - 58) / 7.5) ** 2 < 1: G[r][c] = '#'
prop('well', 80 * CELL - 3, 57 * CELL + 6)
for name, c, r, dx in [('stall', 72, 55, 0), ('stall', 86, 55, 4), ('stall', 73, 63, 6), ('stall', 85, 63, 0)]:
    prop(name, c * CELL + dx, r * CELL + 14)
for c, r in [(70, 52), (91, 52), (69, 64), (92, 64), (79, 47), (60, 59), (100, 59)]:
    prop('lamp', c * CELL + 4, r * CELL + 2)
for name, x, y in [('banner', 76, 51), ('banner', 84, 51), ('barrel', 71, 57), ('crate', 72, 57),
                   ('barrel', 89, 61), ('cart', 93, 60), ('hay_bale', 94, 62), ('brazier', 78, 60), ('brazier', 83, 60)]:
    prop(name, x * CELL + 2, y * CELL + 12)

# --- buildings: bottom-left at a cell corner, footprint blocks walking and shots
def building(name, c, r):
    w, h, depth = SIZES[name]
    prop(name, c * CELL, r * CELL)
    for rr in range(r - depth, r):
        for cc in range(c, c + math.ceil(w / CELL)):
            put(cc, rr, 'H')
building('chapel', 74, 47)
for name, c, r in [('house_slate', 64, 51), ('house_cottage', 89, 51),
                   ('house_long', 54, 57), ('house_cottage', 46, 57), ('house_slate', 38, 58),
                   ('house_slate', 48, 68), ('house_cottage', 58, 68), ('house_long', 38, 70),
                   ('house_long', 96, 57), ('house_cottage', 104, 57), ('house_slate', 110, 57),
                   ('house_cottage', 98, 68), ('house_long', 104, 68), ('house_slate', 122, 57),
                   ('house_cottage', 66, 74), ('house_long', 84, 74), ('house_cottage', 92, 80),
                   ('house_cottage', 24, 63), ('house_cottage', 36, 46),
                   ('house_long', 88, 108), ('house_cottage', 42, 106), ('house_slate', 126, 105),
                   ('house_cottage', 110, 40)]:
    building(name, c, r)

# --- churchyard east of the chapel: dry-stone wall with a gate on the south side
for c in range(84, 98):
    if c not in (90, 91): put(c, 46, 'W')
    put(c, 34, 'W')
for r in range(34, 47):
    put(84, r, 'W'); put(97, r, 'W')
for k, (c, r) in enumerate([(86, 37), (89, 37), (92, 36), (95, 37), (87, 41), (90, 40), (93, 41), (86, 44), (94, 44)]):
    prop(['grave_round', 'grave_cross', 'grave_slab'][k % 3], c * CELL + 2, r * CELL + 14)
prop('tree_oak', 95 * CELL - 6, 43 * CELL + 10)

# --- farms: fenced fields of wheat and cabbage, scarecrows, hay; one past the river
def field(c0, r0, cols, rows, crops):
    x0, y0 = c0 * CELL, r0 * CELL
    for j in range(rows):
        for i in range(cols):
            prop(crops if j % 3 else 'crop_cabbage', x0 + 8 + i * 16, y0 + 30 + j * 14)
    by = y0 + 12 + (rows * 14 // 12 + 2) * 12
    for i in range(cols + 1):
        prop('fence_h', x0 + i * 16, y0 + 14); prop('fence_h', x0 + i * 16, by + 14)
    for y in range(y0 + 12, by, 12):
        prop('fence_v', x0, y + 12); prop('fence_v', x0 + (cols + 1) * 16, y + 12)
    prop('fence_post', x0 + (cols + 1) * 16, y0 + 14); prop('fence_post', x0 + (cols + 1) * 16, by + 14)
field(66, 100, 14, 7, 'crop_wheat'); field(104, 28, 12, 6, 'crop_wheat'); field(40, 76, 8, 5, 'crop_wheat')
for name, x, y in [('scarecrow', 74, 104), ('scarecrow', 110, 33), ('hay_roll', 84, 99), ('hay_bale', 86, 99),
                   ('hay_roll', 100, 36), ('trough', 96, 110), ('hay_bale', 46, 82), ('cart', 52, 82)]:
    prop(name, x * CELL, y * CELL)

# --- meadow scatter: flowers, tall grass, bushes, rocks, lone trees on open grass; reeds by water
def open_cell(c, r, ring=1):
    return all(get(c + dx, r + dy) == '.' for dx in range(-ring, ring + 1) for dy in range(-ring, ring + 1))
taken = set((x // CELL, y // CELL) for _, x, y in PROPS)
for r in range(ROWS):
    for c in range(COLS):
        if (c, r) in taken: continue
        k = rnd.random()
        if open_cell(c, r) and k < 0.05:
            prop(rnd.choice(['flowers_lav', 'flowers_white', 'flowers_yellow', 'grass_tall', 'grass_tall', 'fern']),
                 c * CELL + rnd.randrange(4), r * CELL + 8 + rnd.randrange(8))
        elif open_cell(c, r, 2) and k < 0.065:
            prop(rnd.choice(['bush', 'bush_berry', 'rock_small', 'rock', 'tree_oak', 'tree_oak_small', 'tree_birch', 'tree_pine']),
                 c * CELL - 4, r * CELL + 14)
        elif G[r][c] == '.' and get(c, r + 1) == '~' and k < 0.3:
            prop('reeds', c * CELL + rnd.randrange(4), r * CELL + 18)

# --- trees packing the woods: dense along the edge, the odd gap deeper in
for r in range(ROWS):
    for c in range(COLS):
        if G[r][c] != 'T': continue
        edge = any(get(c + dx, r + dy) != 'T' for dx in (-2, 0, 2) for dy in (-2, 0, 2))
        if rnd.random() < (0.9 if edge else 0.55):
            name = rnd.choice(['tree_pine', 'tree_pine', 'tree_oak', 'tree_oak_small', 'tree_birch'])
            prop(name, c * CELL - 8 + rnd.randrange(8), r * CELL + 12 + rnd.randrange(6))

assert G[START[1]][START[0]] in '.:#', 'start must be open'
PROPS.sort(key=lambda p: p[2])
root = os.path.join(os.path.dirname(__file__), '..', '..')
with open(os.path.join(root, 'assets', 'maps', 'ashford.map'), 'w') as f:
    f.write('# Ashford (V19d). Generated by docs/tools/ashford_layout.py: edit that, not this.\n')
    f.write('# . grass  : path  # cobbles  ~ water  = bridge  W wall  H building  R ridge  T woods\n')
    f.write(f'size {COLS} {ROWS}\nstart {START[0]} {START[1]}\ngrid\n')
    for row in G: f.write(''.join(row) + '\n')
    f.write('props\n')
    for n, x, y in PROPS: f.write(f'{n} {x} {y}\n')
print(f'{len(PROPS)} props')

if len(sys.argv) > 1:
    from PIL import Image
    import json
    S = sys.argv[1]; V = S + '/vil'
    def first(n):
        p = f'{V}/anims/{n}.png'
        if os.path.exists(p):
            js = json.load(open(f'{V}/anims/{n}.json')); f0 = next(iter(js['frames'].values()))['frame']
            return Image.open(p).convert('RGBA').crop((f0['x'], f0['y'], f0['x'] + f0['w'], f0['y'] + f0['h']))
        return Image.open(f'{V}/sprites/{n}.png').convert('RGBA')
    cache = {}
    def spr(n): return cache.setdefault(n, first(n))
    gr = [spr(f'ground_grass_{i}') for i in range(4)]
    im = Image.new('RGBA', (COLS * CELL, ROWS * CELL))
    for ty in range(0, ROWS * CELL, 32):
        for tx in range(0, COLS * CELL, 32): im.paste(gr[rnd.randrange(4)], (tx, ty))
    def has(c, r, chars): return 1 if get(c, r) in chars else 0
    for chars, name in [(':#', 'path'), ('#', 'cobble'), ('~=', 'water'), ('T', 'woods'), ('R', 'ridge')]:
        for r in range(-1, ROWS):
            for c in range(-1, COLS):
                k = has(c, r, chars) + 2 * has(c + 1, r, chars) + 4 * has(c, r + 1, chars) + 8 * has(c + 1, r + 1, chars)
                if k: im.alpha_composite(spr(f'{name}_{k}_{(c & 1) + 2 * (r & 1)}'), (c * CELL + 8, r * CELL + 8))
    items = list(PROPS)
    for r in range(ROWS):
        for c in range(COLS):
            if G[r][c] != 'W': continue
            l, rt, u, d = get(c - 1, r) == 'W', get(c + 1, r) == 'W', get(c, r - 1) == 'W', get(c, r + 1) == 'W'
            if l or rt: items.append(('wall_h' if l and rt else 'wall_h_l' if rt else 'wall_h_r', c * CELL, r * CELL + 16))
            if d: items.append(('wall_v' if u else 'wall_v_top', c * CELL, r * CELL + 16))
            elif u: items.append(('wall_v_end', c * CELL, r * CELL + 16))
    items.sort(key=lambda p: p[2])
    for n, x, y in items:
        s = spr(n); im.alpha_composite(s, (x, y - s.height))
    im.convert('RGB').save(f'{S}/ashford.png')
    im.convert('RGB').resize((COLS * 4, ROWS * 4), Image.BOX).save(f'{S}/ashford_small.png')
