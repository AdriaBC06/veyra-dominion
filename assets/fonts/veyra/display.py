"""Veyra Display: a small-caps pixel font drawn as 1 px skeletons, emboldened to the right
(thick verticals, thin horizontals, like a broad pen). Writes a TTF where 1 px = 100 units.

usage: python3 display.py <out.ttf>
"""
import sys
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

U = 100          # font units per pixel
CAP = 10         # cap height in px
ABOVE = 4        # px above the cap line (accents on capitals)
BELOW = 3        # px below the baseline
EM = CAP + ABOVE + BELOW

CAPS = {
'A': """
...#...
..#.#..
.#...#.
.#...#.
.#...#.
.#...#.
.#####.
.#...#.
.#...#.
.#.+.#.+""",
'B': """
+####.
.#...#
.#...#
.#..#.
.####.
.#...#
.#...#
.#...#
.#..#.
.###..""",
'C': """
...##.
..#..#
.#....
.#....
.#....
.#....
.#....
.#....
..#..#
...##.""",
'D': """
+###..
.#..#.
.#...#
.#...#
.#...#
.#...#
.#...#
.#...#
.#..#.
.###..""",
'E': """
+#####
.#....
.#....
.#....
.####.
.#....
.#....
.#....
.#....
.#####""",
'F': """
+#####
.#....
.#....
.#....
.####.
.#....
.#....
.#....
.#....
.#.+..""",
'G': """
...##.
..#..#
.#....
.#....
.#....
.#..##
.#...#
.#...#
..#..#
...##.""",
'H': """
+#...#
.#...#
.#...#
.#...#
.#####
.#...#
.#...#
.#...#
.#...#
.#.+.#.+""",
'I': """
+#
.#
.#
.#
.#
.#
.#
.#
.#
.#.+""",
'J': """
....+#
.....#
.....#
.....#
.....#
.....#
.....#
.....#
.#..#.
..##..""",
'K': """
+#...#
.#..#.
.#..#.
.#.#..
.##...
.#.#..
.#..#.
.#..#.
.#...#
.#.+.#.+""",
'L': """
+#....
.#....
.#....
.#....
.#....
.#....
.#....
.#....
.#....
.#####""",
'M': """
+#.....#
.##...##
.#.#.#.#
.#..#..#
.#.....#
.#.....#
.#.....#
.#.....#
.#.....#
.#.+...#.+""",
'N': """
+#....#
.##...#
.#.#..#
.#.#..#
.#..#.#
.#..#.#
.#...##
.#....#
.#....#
.#.+..#""",
'O': """
...##.
..#..#
.#...#
.#...#
.#...#
.#...#
.#...#
.#...#
.#..#.
..##..""",
'P': """
+####.
.#...#
.#...#
.#...#
.####.
.#....
.#....
.#....
.#....
.#.+..""",
'Q': """
...##.
..#..#
.#...#
.#...#
.#...#
.#...#
.#...#
.#.#.#
.#..#.
..##.#.+""",
'R': """
+####.
.#...#
.#...#
.#...#
.####.
.#.#..
.#..#.
.#..#.
.#...#
.#.+.#.+""",
'S': """
...##.
..#..#
.#....
.#....
..###.
.....#
.....#
.....#
.#..#.
..##..""",
'T': """
######
...#..
...#..
...#..
...#..
...#..
...#..
...#..
...#..
...#.+""",
'U': """
+#...#
.#...#
.#...#
.#...#
.#...#
.#...#
.#...#
.#...#
.#..#.
..##..""",
'V': """
+#...#
.#...#
.#...#
.#...#
..#.#.
..#.#.
..#.#.
..#.#.
...#..
...#..""",
'W': """
+#.....#
.#.....#
.#.....#
.#.....#
.#.....#
.#..#..#
.#..#..#
.#..#..#
.#.#.#.#
..#...#.""",
'X': """
+#...#
.#...#
..#.#.
..#.#.
...#..
...#..
..#.#.
..#.#.
.#...#
.#.+.#.+""",
'Y': """
+#...#
.#...#
..#.#.
..#.#.
...#..
...#..
...#..
...#..
...#..
...#.+""",
'Z': """
######
.....#
....#.
....#.
...#..
...#..
..#...
..#...
.#....
.#####""",
}

DIGITS = {
'0': """
.###.
#...#
#...#
#..##
#.#.#
#.#.#
##..#
#...#
#...#
.###.""",
'1': """
.#.
##.
.#.
.#.
.#.
.#.
.#.
.#.
.#.
###""",
'2': """
.###.
#...#
....#
....#
...#.
..#..
.#...
#....
#....
#####""",
'3': """
.###.
#...#
....#
....#
..##.
....#
....#
....#
#...#
.###.""",
'4': """
...#.
..##.
..##.
.#.#.
.#.#.
#..#.
#####
...#.
...#.
...#.""",
'5': """
#####
#....
#....
####.
....#
....#
....#
....#
#...#
.###.""",
'6': """
.###.
#...#
#....
#....
####.
#...#
#...#
#...#
#...#
.###.""",
'7': """
#####
....#
....#
...#.
...#.
..#..
..#..
..#..
..#..
..#..""",
'8': """
.###.
#...#
#...#
#...#
.###.
#...#
#...#
#...#
#...#
.###.""",
'9': """
.###.
#...#
#...#
#...#
#...#
.####
....#
....#
#...#
.###.""",
}

# Punctuation, drawn on the 10-row cap box (rows past 10 fall into the descender).
PUNCT = {
'.': "\n.\n.\n.\n.\n.\n.\n.\n.\n.\n#",
',': "\n.\n.\n.\n.\n.\n.\n.\n.\n.\n#\n#",
':': "\n.\n.\n.\n#\n.\n.\n.\n.\n.\n#",
';': "\n.\n.\n.\n#\n.\n.\n.\n.\n.\n#\n#",
'!': "\n#\n#\n#\n#\n#\n#\n#\n.\n.\n#",
'¡': "\n.\n.\n#\n.\n.\n#\n#\n#\n#\n#\n#\n#",
'?': "\n.###.\n#...#\n....#\n...#.\n..#..\n..#..\n..#..\n.....\n.....\n..#..",
'¿': "\n.....\n.....\n..#..\n.....\n.....\n..#..\n..#..\n..#..\n.#...\n#....\n#...#\n.###.",
"'": "\n#\n#\n#",
'"': "\n#.#\n#.#\n#.#",
'-': "\n...\n...\n...\n...\n...\n###",
'+': "\n.....\n.....\n.....\n..#..\n..#..\n#####\n..#..\n..#..",
'%': "\n##..#\n##..#\n...#.\n...#.\n..#..\n..#..\n.#...\n.#...\n#..##\n#..##",
'/': "\n....#\n....#\n...#.\n...#.\n..#..\n..#..\n.#...\n.#...\n#....\n#....",
'(': "\n.#\n#.\n#.\n#.\n#.\n#.\n#.\n#.\n#.\n.#",
')': "\n#.\n.#\n.#\n.#\n.#\n.#\n.#\n.#\n.#\n#.",
'&': "\n.##..\n#..#.\n#..#.\n.##..\n.#...\n#.#.#\n#..#.\n#..#.\n#..##\n.##.#",
'×': "\n.....\n.....\n.....\n#...#\n.#.#.\n..#..\n.#.#.\n#...#",
'·': "\n.\n.\n.\n.\n.\n#",
}

# Small caps keep these rows of the capital (7 of 10); the rest are drawn by hand.
SMALL_ROWS = [0, 1, 3, 4, 6, 8, 9]
SMALL = {
'i': """
.#
.#
.#
.#
.#
.#
.#.+""",
'g': """
...##.
..#..#
.#....
.#..##
.#...#
..#..#
...##.""",
}

ACCENTS = {  # base, mark rows (drawn above the letter, centred)
'acute': ["..#", ".#."],
'diaer': ["#.#"],
'tilde': [".#.#", "#.#."],
}
COMPOSED = {
'Á': ('A', 'acute'), 'É': ('E', 'acute'), 'Í': ('I', 'acute'), 'Ó': ('O', 'acute'),
'Ú': ('U', 'acute'), 'Ü': ('U', 'diaer'), 'Ñ': ('N', 'tilde'),
'á': ('a', 'acute'), 'é': ('e', 'acute'), 'í': ('i', 'acute'), 'ó': ('o', 'acute'),
'ú': ('u', 'acute'), 'ü': ('u', 'diaer'), 'ñ': ('n', 'tilde'),
}


def rows_of(art):
    return [r for r in art.strip('\n').split('\n')]


def grid(art, top):
    """Pixels (x, y) with y counted down from the cap line; `top` shifts the art down.
    '#' is a pen pixel (emboldened to the right), '+' a hairline pixel kept as is."""
    px = set()
    for y, row in enumerate(rows_of(art)):
        for x, c in enumerate(row):
            if c == '#':
                px.add((x, y + top))
            elif c == '+':
                px.add((-1 - x, y + top))  # tagged: negative x until bold() runs
    return px


def bold(px):
    pen = {(x, y) for x, y in px if x >= 0}
    hair = {(-1 - x, y) for x, y in px if x < 0}
    out = pen | {(x + 1, y) for x, y in pen} | hair
    m = min((x for x, _ in out), default=0)
    return {(x - m, y) for x, y in out}


def width(px):
    return max((x for x, _ in px), default=-1) + 1


def make(caps, small, small_rows, digits, punct, cap, xh, space=4):
    """Glyph pixel sets and advances: capitals, small caps (`xh` rows), digits, punctuation
    and the Spanish accented letters."""
    g = {}

    def add(ch, px, adv=None):
        g[ch] = (px, adv if adv is not None else width(px) + 1)

    for ch, art in caps.items():
        add(ch, bold(grid(art, 0)))
        low = ch.lower()
        if low in small:
            art_s = small[low]
        else:
            rows = rows_of(art)
            art_s = '\n'.join(rows[i] for i in small_rows)
        add(low, bold(grid(art_s, cap - xh)))
    for ch, art in digits.items():
        add(ch, bold(grid(art, 0)))
    for ch, art in punct.items():
        add(ch, bold(grid(art.lstrip('\n'), 0)))
    add(' ', set(), space)
    for ch, (base, mark) in COMPOSED.items():
        px, adv = g[base]
        w = width(px)
        rows = ACCENTS[mark]
        mw = len(rows[0]) + 1
        top = (cap - xh if base.islower() else 0) - len(rows) - 1
        ox = (w - mw) // 2
        if base in 'iI':  # over the stem, not the flag
            ox = w - 2 - (mw - 2) // 2 - 1
        m = {(x + ox, y + top) for x, y in bold(grid('\n'.join(rows), 0))}
        add(ch, px | m, adv)
    return g


glyphs = make(CAPS, SMALL, SMALL_ROWS, DIGITS, PUNCT, CAP, 7)


def build(out, glyphs=glyphs, family='Veyra Display', cap=CAP, above=ABOVE, below=BELOW, xh=7):
    em = cap + above + below
    order = ['.notdef'] + [f'u{ord(c):04X}' for c in glyphs]
    fb = FontBuilder(em * U, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap({ord(c): f'u{ord(c):04X}' for c in glyphs})
    shapes, metrics = {}, {}
    shapes['.notdef'] = TTGlyphPen(None).glyph()
    metrics['.notdef'] = (5 * U, 0)
    for c, (px, adv) in glyphs.items():
        pen = TTGlyphPen(None)
        # one rectangle per horizontal run, in font units (y up from the baseline)
        rows = {}
        for x, y in px:
            rows.setdefault(y, []).append(x)
        for y, xs in rows.items():
            xs.sort()
            start = prev = xs[0]
            for x in xs[1:] + [None]:
                if x is not None and x == prev + 1:
                    prev = x
                    continue
                y0 = (cap - 1 - y) * U
                x0, x1 = start * U, (prev + 1) * U
                pen.moveTo((x0, y0)); pen.lineTo((x0, y0 + U)); pen.lineTo((x1, y0 + U)); pen.lineTo((x1, y0)); pen.closePath()
                if x is not None:
                    start = prev = x
        shapes[f'u{ord(c):04X}'] = pen.glyph()
        metrics[f'u{ord(c):04X}'] = (adv * U, 0)
    fb.setupGlyf(shapes)
    fb.setupHorizontalMetrics(metrics)
    asc, desc = (cap + above) * U, -below * U
    fb.setupHorizontalHeader(ascent=asc, descent=desc)
    fb.setupNameTable({'familyName': family, 'styleName': 'Regular'})
    fb.setupOS2(sTypoAscender=asc, sTypoDescender=desc, usWinAscent=asc, usWinDescent=-desc,
                sCapHeight=cap * U, sxHeight=xh * U)
    fb.setupPost()
    fb.save(out)
    print(out, len(glyphs), 'glyphs, native size', em)


if __name__ == "__main__":
    build(sys.argv[1])
