# Terrain mockup (V19d): dual-grid water/cobble/path tiles, bridge and walls on the new ground.
# node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/village.px.js --out <S>/vil
# python3 docs/tools/terrain_mock.py <S> .   -> <S>/terrain.png + terrain.gif
import sys, json, os, random
from PIL import Image
S, R = sys.argv[1], sys.argv[2]; V = S + '/vil'
SC, N = 2, 16
random.seed(5)
# 16 px cells: ~ water, # cobbles, : path, W wall, . grass
MAP = """\
RRRRRRR...............TTTTTTTT
RRRRRR...:...........TTTTTTTTT
RRRR.....:.......WWWWWWWW.TTTT
RR.......:.......W......W..TTT
.....#####.......W......W...TT
....#######::::::W......W.....
....#######......W............
....#######...........::::::::
.....#####......:::::::.......
.........:......:.............
.........::::::::.............
...........:..................
~~~~~~~~~~~:~~~~~~~~~~~~~~~~~~
~~~~~~~~~~~:~~~~~~~~~~~~~~~~~~
~~~~~~~~~~~:~~~~~~~~~~~~~~~~~~
..~~.......:..................
...........:..................
"""
G = [list(r) for r in MAP.splitlines()]
H, W = len(G), len(G[0])
def load(n):
    if os.path.exists(f'{V}/anims/{n}.png'):
        strip = Image.open(f'{V}/anims/{n}.png').convert('RGBA'); js = json.load(open(f'{V}/anims/{n}.json'))
        return [(strip.crop((f['frame']['x'], f['frame']['y'], f['frame']['x']+f['frame']['w'], f['frame']['y']+f['frame']['h'])), f['duration']) for f in js['frames'].values()]
    return [(Image.open(f'{V}/sprites/{n}.png').convert('RGBA'), 1000)]
cache = {}
def frame(n, t):
    fr = cache.setdefault(n, load(n)); tot = sum(d for _, d in fr); t %= tot
    for im, d in fr:
        if t < d: return im
        t -= d
    return fr[-1][0]
gr = [Image.open(f'{V}/sprites/ground_grass_{i}.png').convert('RGBA') for i in range(4)]
base = Image.new('RGBA', (W*16, H*16))
for ty in range(0, H*16, 32):
    for tx in range(0, W*16, 32): base.paste(gr[random.choice([0,0,1,2,2,3])], (tx, ty))
def cell(x, y, ch):
    x = min(max(x, 0), W-1); y = min(max(y, 0), H-1)
    c = G[y][x]
    return 1 if (c == ch or (ch == ':' and c == '#')) else 0   # paths run under the plaza
def dual(ch, name):
    out = []
    for j in range(-1, H):
        for i in range(-1, W):
            k = cell(i, j, ch) + 2*cell(i+1, j, ch) + 4*cell(i, j+1, ch) + 8*cell(i+1, j+1, ch)
            if k: out.append((f'{name}_{k}_{(i & 1) + 2 * (j & 1)}', i*16+8, j*16+8))
    return out
# the bridge column is water for the terrain, deck drawn on top
for y in range(12, 15): G[y][11] = '~'
T = dual(':', 'path') + dual('#', 'cobble') + dual('~', 'water') + dual('T', 'woods') + dual('R', 'ridge')
def wallpiece(x, y):
    w = lambda a, b: 0 <= b < H and 0 <= a < W and G[b][a] == 'W'
    l, r, u, d = w(x-1, y), w(x+1, y), w(x, y-1), w(x, y+1)
    out = []
    if l or r: out.append('wall_h' if l and r else 'wall_h_l' if r else 'wall_h_r')
    if d or (u and not (l or r)): out.append('wall_v' if d else 'wall_v_end')
    if u and not d and (l or r): pass
    return out
P = []
for y in range(H):
    for x in range(W):
        if G[y][x] == 'W':
            for n in wallpiece(x, y): P.append((n, x*16, y*16+16, 0))
def add(n, x, y): P.append((n, x, y, random.randrange(0, 1600)))
add('bridge', 11*16-8, 15*16+10)
add('well', 6*16+4, 6*16+14); add('lamp', 4*16+6, 5*16); add('lamp', 10*16, 5*16)
add('stall', 2*16, 9*16); add('house_cottage', 0, 4*16)
for n,x,y in [('grave_round',19*16,4*16+4),('grave_cross',21*16,5*16),('grave_slab',23*16,4*16+6),('grave_round',20*16,6*16+8),('grave_cross',23*16,6*16+10)]: add(n,x,y)
for n,x,y in [('reeds',3*16,12*16+2),('reeds',20*16,12*16+4),('reeds',25*16,15*16+14),('tree_pine',14*16,3*16),
              ('bush',16*16,11*16+12),('flowers_lav',6*16,11*16+10),('rock',27*16,11*16+12),('barrel',9*16,8*16+6)]: add(n,x,y)
for y in range(H):
    for x in range(W):
        if G[y][x] == 'T':
            for k in range(2): add(random.choice(['tree_pine','tree_pine','tree_oak','tree_oak_small']), x*16-8+random.randrange(12), y*16+8+k*8+random.randrange(6))
P.sort(key=lambda p: p[2])
frames = []
for k in range(N):
    t = k * 125; im = base.copy()
    for n, x, y in T: im.alpha_composite(frame(n, t), (x, y))
    for n, x, y, dl in P:
        f = frame(n, t + dl); im.alpha_composite(f, (x, y - f.height))
    frames.append(im.crop((0, 0, 480, 270)).resize((480*SC, 270*SC), Image.NEAREST).convert('RGB'))
frames[0].save(f'{S}/terrain.png')
frames[0].save(f'{S}/terrain.gif', save_all=True, append_images=frames[1:], duration=125, loop=0)
