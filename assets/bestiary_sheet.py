import json, os, sys, glob
from PIL import Image, ImageDraw, ImageFont
# Labelled contact sheet of the enemy roster and enemy effects (docs/design.md 7.8).
# Usage: python3 assets/bestiary_sheet.py [out.png]  (default docs/art/ashford_bestiary.png)
ROOT=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
R=ROOT+'/assets/sprites/'
EN=R+'enemies'; FX=R+'enemy_fx'
OUT=sys.argv[1] if len(sys.argv)>1 else ROOT+'/docs/art/ashford_bestiary.png'
S=4; BG=(60,76,52,255)
fonts=glob.glob(ROOT+'/assets/fonts/*.ttf')
try: font=ImageFont.truetype(ROOT+'/assets/fonts/Jersey15-Regular.ttf',26)
except Exception: font=ImageFont.load_default(size=18)
def frames(png):
    im=Image.open(png).convert('RGBA'); js=png[:-4]+'.json'
    n=len(json.load(open(js))['frames']) if os.path.exists(js) else 1
    w=im.width//n
    return [im.crop((i*w,0,(i+1)*w,im.height)) for i in range(n)]
enemies=[('Rat','rat'),('Villager','villager'),('Crow','crow'),('Hound','hound'),('Torchbearer','torchbearer'),('Shepherd','shepherd'),
 ('Barricader','barricader'),('Curate','curate'),('Ratcatcher','ratcatcher'),('Knight','knight'),('Zealot','zealot'),('Brute','brute'),
 ('Captain of the Watch (boss)','captain'),('Captain charging','captain_charge')]
fx=[('stone',FX+'/sprites/stone.png'),('torch_spin',FX+'/anims/torch_spin.png'),('fire_patch',FX+'/anims/fire_patch.png'),
 ('soul_faint',FX+'/sprites/soul_faint.png'),('soul',R+'soul.png'),('soul_radiant',FX+'/anims/soul_radiant.png'),('bless',FX+'/anims/bless.png'),
 ('zealot_blast',FX+'/anims/zealot_blast.png')]
def row(items, maxw):
    rows=[[]]; x=0
    for name,fr in items:
        w=sum(f.width*S+6 for f in fr)+30
        w=max(w, int(font.getlength(name))+30)
        if x+w>maxw and rows[-1]: rows.append([]); x=0
        rows[-1].append((name,fr,w)); x+=w
    return rows
items=[]
for name,k in enemies:
    old=k in ('rat','villager','thrall','knight','brute')
    p=(R+'enemies/anims/' if old else EN+'/anims/')+k+'.png'
    items.append((name,frames(p)))
fitems=[(n,frames(p)) for n,p in fx]
MAXW=1500
rows=row(items,MAXW)+[None]+row(fitems,MAXW)
H=20
heights=[]
for r in rows:
    h=40 if r is None else max(f.height for _,fr,_ in r for f in fr)*S+40
    heights.append(h); H+=h+20
img=Image.new('RGBA',(MAXW,H),BG); d=ImageDraw.Draw(img); y=20
for r,h in zip(rows,heights):
    if r is None:
        d.text((20,y+10),'EFFECTS (not flipped, no flash copies)',font=font,fill=(230,230,220)); y+=h+20; continue
    x=20
    for name,fr,w in r:
        fx_=x
        for f in fr:
            big=f.resize((f.width*S,f.height*S),Image.NEAREST)
            img.alpha_composite(big,(fx_,y+h-40-big.height)); fx_+=big.width+6
        d.text((x,y+h-32),name,font=font,fill=(235,235,225))
        x+=w
    y+=h+20
img.save(OUT); print(img.size)
