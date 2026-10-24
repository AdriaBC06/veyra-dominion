# Village art (V19d) into the game: packs pxl's `terrain` and `village` sheets as PNG + index.
# node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/village.px.js --out <tmp>
# python3 assets/village_sheets.py <tmp>      -> assets/sprites/village/{terrain,village}.{png,idx}
# Index: one sprite per line, `name w h ms x,y x,y ...` (one x,y per frame; ms 0 = still).
import json, os, shutil, sys
src = sys.argv[1]
out = os.path.join(os.path.dirname(__file__), 'sprites', 'village')
os.makedirs(out, exist_ok=True)
for sheet in ('terrain', 'village'):
    js = json.load(open(f'{src}/{sheet}.json'))
    frames = list(js['frames'].items())
    tags = {t['name']: (t['from'], t['to']) for t in js['meta'].get('frameTags', [])}
    in_tag = {i for a, b in tags.values() for i in range(a, b + 1)}
    lines = []
    for i, (name, f) in enumerate(frames):
        if i not in in_tag:
            r = f['frame']; lines.append(f"{name} {r['w']} {r['h']} 0 {r['x']},{r['y']}")
    for name, (a, b) in tags.items():
        fr = [frames[i][1] for i in range(a, b + 1)]
        r0 = fr[0]['frame']
        lines.append(f"{name} {r0['w']} {r0['h']} {fr[0].get('duration', 100)} " +
                     ' '.join(f"{f['frame']['x']},{f['frame']['y']}" for f in fr))
    with open(f'{out}/{sheet}.idx', 'w') as f: f.write('\n'.join(sorted(lines)) + '\n')
    shutil.copy(f'{src}/{sheet}.png', f'{out}/{sheet}.png')
    print(sheet, len(lines), 'sprites')
