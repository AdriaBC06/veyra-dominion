# Village art mockup (V19d): composes village.px.js output on the new ground with real villagers.
# node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/village.px.js --out <S>/vil
# python3 docs/tools/village_mock.py <S> .   -> <S>/mock2.png + mock2.gif (fence anchors are top-left)
import sys, json, os, random
from PIL import Image, ImageDraw
S, R = sys.argv[1], sys.argv[2]; V = S + '/vil'
W, H, SC, N = 480, 270, 2, 32
random.seed(3)
def load(n):
    if os.path.exists(f'{V}/anims/{n}.png'):
        strip = Image.open(f'{V}/anims/{n}.png').convert('RGBA'); js = json.load(open(f'{V}/anims/{n}.json'))
        fr = [(strip.crop((f['frame']['x'], f['frame']['y'], f['frame']['x']+f['frame']['w'], f['frame']['y']+f['frame']['h'])), f['duration']) for f in js['frames'].values()]
        return fr
    return [(Image.open(f'{V}/sprites/{n}.png').convert('RGBA'), 1000)]
cache = {}
def frame(n, t):
    fr = cache.setdefault(n, load(n)); tot = sum(d for _, d in fr); t %= tot
    for im, d in fr:
        if t < d: return im
        t -= d
    return fr[-1][0]
gr = [Image.open(f'{V}/sprites/ground_grass_{i}.png').convert('RGBA') for i in range(4)]
di = [Image.open(f'{V}/sprites/ground_dirt_{i}.png').convert('RGBA') for i in range(2)]
base = Image.new('RGBA', (W, H))
for ty in range(0, H, 32):
    for tx in range(0, W, 32):
        road = 120 <= ty < 160 or (200 <= tx < 232 and ty < 140)
        base.paste(random.choice(di) if road else gr[random.choice([0,0,1,2,2,3])], (tx, ty))
vil = Image.open(R+'/assets/sprites/enemies/anims/villager.png').convert('RGBA').crop((0,0,14,20))
P = []  # (name, x, y_bottom, delay, shadow)
def add(n, x, y, sh=True): P.append((n, x, y, random.randrange(0, 1600), sh))
# village row
for n,x,y in [('chapel',20,122),('house_long',110,108),('house_cottage',240,104),('house_slate',300,108)]: add(n,x,y)
for n,x,y in [('tree_pine',2,70),('tree_pine',380,60),('tree_oak',400,110),('tree_birch',446,96),('tree_oak_small',372,128),
              ('well',170,150),('lamp',96,140),('lamp',282,140),('banner',150,116),('barrel',196,152),('crate',206,154),('stall',330,160),('cart',380,166)]: add(n,x,y)
# graveyard
for n,x,y in [('grave_round',6,150),('grave_cross',24,156),('grave_slab',42,152),('grave_round',12,172),('grave_cross',36,176),('bush',58,170)]: add(n,x,y)
# farm: corral with crops, scarecrow, hay, trough
fx, fy, fw, fh = 150, 176, 9, 5   # fence ring, top-left anchors: fence_h is 14 tall, fence_v 12
def tl(n, x, y): P.append((n, x, y + 14 if n != 'fence_v' else y + 12, 0, False))
by = fy + 12 + fh * 12
for j in range(fh): tl('fence_v', fx, fy + 12 + j*12); tl('fence_v', fx + fw*16, fy + 12 + j*12)
for i in range(fw): tl('fence_h', fx + i*16, fy); tl('fence_h', fx + i*16, by)
tl('fence_post', fx + fw*16, fy); tl('fence_post', fx + fw*16, by)
for row in range(3):
    for i in range(6): add('crop_wheat' if row < 2 else 'crop_cabbage', fx + 8 + i*16, fy + 22 + row*14, False)
add('scarecrow', fx + 106, fy + 50); add('hay_bale', fx + 112, fy + 64); add('trough', 112, 222); add('hay_roll', 112, 246)
add('house_cottage', 40, 246); add('hay_roll', 10, 266); add('hay_bale', 100, 262)
# vegetation
for n,x,y in [('bush',330,200),('bush_berry',360,196),('flowers_lav',300,210),('flowers_white',312,224),('flowers_yellow',420,200),
              ('grass_tall',345,236),('fern',400,240),('reeds',450,250),('reeds',438,262),('rock',380,262),('rock_small',300,250),
              ('flowers_lav',70,200),('grass_tall',92,196),('fern',4,210),('bush_berry',440,150),('flowers_yellow',260,190),('tree_oak',300,300),('tree_birch',448,290)]: add(n,x,y)
P.sort(key=lambda p: p[2])
frames = []
for k in range(N):
    t = k * 100; im = base.copy()
    sh = Image.new('RGBA', im.size); d = ImageDraw.Draw(sh)
    for n,x,y,dl,s in P:
        if s:
            w = frame(n, 0).width; d.ellipse((x+1, y-4, x+w-1, y+3), fill=(16,22,14,110))
    im.alpha_composite(sh)
    for n,x,y,dl,s in P:
        f = frame(n, t + dl); im.alpha_composite(f, (x, y - f.height))
    for x,y in [(140,170),(186,166),(262,156),(230,232)]: im.alpha_composite(vil, (x, y - 20))
    frames.append(im.resize((W*SC, H*SC), Image.NEAREST).convert('RGB'))
frames[0].save(f'{S}/mock2.png')
frames[0].save(f'{S}/mock2.gif', save_all=True, append_images=frames[1:], duration=100, loop=0)
