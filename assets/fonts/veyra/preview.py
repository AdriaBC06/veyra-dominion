"""Glyph sheet of one Veyra font straight from its pixel sets (exact; Pillow's TTF hinting
shifts rows, the game's rasteriser doesn't).

usage: python3 preview.py <display|gothic13|gothic|titling> <out.png> 'line one|line two' [row step]
"""
import importlib
import os
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
glyphs = importlib.import_module(sys.argv[1]).glyphs
lines = sys.argv[3].split('|')
step = int(sys.argv[4]) if len(sys.argv) > 4 else 30


def paint(text):
    x, pts = 0, []
    for ch in text:
        px, adv = glyphs.get(ch, glyphs['?'])
        pts += [(x + a, b) for a, b in px]
        x += adv
    return pts, x


img = Image.new('RGB', (max(paint(l)[1] for l in lines) + 16, step * len(lines) + 24), (20, 10, 12))
for i, line in enumerate(lines):
    col = (240, 138, 44) if i % 2 else (209, 196, 178)
    for a, b in paint(line)[0]:
        img.putpixel((8 + a, 14 + step * i + b), col)
img.resize((img.width * 3, img.height * 3), Image.NEAREST).save(sys.argv[2])
