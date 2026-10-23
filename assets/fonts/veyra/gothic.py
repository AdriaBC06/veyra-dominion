"""Veyra Gothic (titles): 18 px capitals and 13 px small caps built from pen primitives.
Horned stem heads, lancet (pointed) bowls, slanted feet and spade-tipped succubus tails.

usage: python3 gothic.py <out.ttf>
"""
import os
import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import display as vf

H = 18       # cap height
XH = 13      # small-cap height
ABOVE, BELOW = 5, 6
S = 3        # stem width


class G:
    """One glyph on a pixel canvas; y = 0 is the cap line, y = H - 1 the bottom row."""

    def __init__(self):
        self.px = set()

    def p(self, x, y):
        self.px.add((x, y))
        return self

    def rect(self, x0, y0, x1, y1):
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                self.px.add((x, y))
        return self

    def unp(self, x, y):
        self.px.discard((x, y))
        return self

    def stem(self, x, y0, y1, w=S):
        return self.rect(x, y0, x + w - 1, y1)

    def horns(self, x, y=0):
        """Two horns over a stem at x whose solid part starts at y + 2."""
        self.p(x - 1, y).p(x + S, y).p(x - 1, y + 1).p(x, y + 1).p(x + S - 1, y + 1).p(x + S, y + 1)
        return self

    def art(self, x, y, rows):
        """Blits hand-drawn rows ('#' on) with the top-left at (x, y)."""
        for j, row in enumerate(rows.strip('\n').split('\n')):
            for i, c in enumerate(row):
                if c == '#':
                    self.p(x + i, y + j)
        return self

    def head(self, x, y=0):
        """A slanted textura head: the top row of the stem shifted one pixel left."""
        return self.rect(x - 1, y, x + S - 2, y)

    def foot(self, x, y=H - 1, flick=True):
        """A slanted foot under a stem at x: the bottom row shifted right, a hairline flick."""
        self.rect(x + 1, y, x + S, y)
        if flick:
            self.p(x + S + 1, y - 1)
        return self

    def hline(self, x0, x1, y, h=1):
        return self.rect(x0, y, x1, y + h - 1)

    def diag(self, x0, y0, x1, y1, w=S):
        """A pen stroke from (x0, y0) to (x1, y1), each row w pixels wide."""
        n = max(abs(x1 - x0), abs(y1 - y0))
        for i in range(n + 1):
            x = round(x0 + (x1 - x0) * i / n)
            y = round(y0 + (y1 - y0) * i / n)
            self.rect(x, y, x + w - 1, y)
        return self

    def arch_top(self, x0, x1, y0, reach):
        """Pointed arch from the stems at x0 and x1 (outer edges) up to a tip at y0: the
        sides narrow to hairlines as they rise. `reach` rows from tip to full stems."""
        cx = (x0 + x1) / 2
        for i in range(reach):
            t = i / (reach - 1) if reach > 1 else 1
            half = (cx - x0) * t
            l = round(cx - half - 0.5)
            r = round(cx + half + 0.5)
            w = 1 if i < reach * 0.45 else (2 if i < reach * 0.8 else S)
            self.rect(l, y0 + i, l + w - 1, y0 + i)
            self.rect(r - w + 1, y0 + i, r, y0 + i)
        return self

    def arch_bot(self, x0, x1, y1, reach):
        """The same arch pointing down, its tip at y1."""
        g = G()
        g.arch_top(x0, x1, 0, reach)
        for x, y in g.px:
            self.p(x, y1 - y)
        return self

    def spade(self, x, y):
        """Veyra's tail tip, a lozenge centred on (x, y)."""
        return self.art(x - 2, y - 2, """
..#..
.###.
#####
.###.
..#..""")

    def done(self, adv_extra=1):
        m = min(x for x, _ in self.px)
        px = {(x - m, y) for x, y in self.px}
        return px


L = 1   # left stem x in most letters (room for a head/horn at x - 1)


def letter_A():
    g = G()
    # a pointed arch with a horned crossbar
    g.arch_top(L, L + 10, 0, 7)
    g.stem(L, 7, H - 2).stem(L + 8, 7, H - 2)
    g.hline(L + 3, L + 7, 10).hline(L + 3, L + 7, 11)
    g.unp(L + 5, 11)
    g.foot(L).foot(L + 8)
    return g


def letter_B():
    g = G().art(0, 0, """
#...#.......
##.###......
.#######....
.###..###...
.###...###..
.###...###..
.###..###...
.#######....
.###..###...
.###...###..
.###....###.
.###....###.
.###....###.
.###....###.
.###...###..
.###..###...
.#######....
#######.....""")
    return g


def letter_C():
    g = G().art(0, 0, """
.....##.....
....####....
...##..##...
..##....##..
.###....###.
###......#..
###.........
###.........
###.........
###.........
###.........
###.........
###......#..
.###....###.
..##....##..
...##..##...
....####....
.....##.....""")
    return g


def letter_D():
    g = G().art(0, 0, """
#...#.......
##.###......
.#######....
.###..###...
.###...###..
.###....###.
.###.....###
.###.....###
.###.....###
.###.....###
.###.....###
.###.....###
.###.....###
.###....###.
.###...###..
.###..###...
.#######....
#######.....""")
    return g


def letter_E():
    g = G()
    g.horns(L).stem(L, 2, H - 1)
    g.hline(L, L + 8, 2).p(L + 8, 3).p(L + 8, 4)
    g.hline(L, L + 6, 8).hline(L, L + 6, 9).p(L + 7, 7)
    g.hline(L - 1, L + 8, H - 1).p(L + 8, H - 2).p(L + 8, H - 3)
    return g


def letter_F():
    g = G()
    g.horns(L).stem(L, 2, H - 2).foot(L, flick=False)
    g.hline(L, L + 8, 2).p(L + 8, 3).p(L + 8, 4)
    g.hline(L, L + 6, 8).hline(L, L + 6, 9).p(L + 7, 7)
    return g


def letter_G():
    g = G().art(0, 0, """
.....##.....
....####....
...##..##...
..##....##..
.###....###.
###......#..
###.........
###.........
###...######
###......###
###......###
###......###
###......###
.###....###.
..##....##..
...##..##...
....####....
.....##.....""")
    return g


def letter_H():
    g = G()
    g.horns(L).stem(L, 2, H - 2).foot(L, flick=False)
    g.horns(L + 8).stem(L + 8, 2, H - 2).foot(L + 8)
    g.hline(L + 3, L + 7, 8).hline(L + 3, L + 7, 9)
    g.p(L + 5, 7).p(L + 5, 10)  # a lozenge on the bar
    return g


def letter_I():
    g = G()
    g.horns(L).stem(L, 2, H - 2).foot(L)
    return g


def letter_J():
    g = G().art(0, 0, """
.....#...#
.....##.##
......###.
......###.
......###.
......###.
......###.
......###.
......###.
......###.
......###.
......###.
......###.
......###.
......###.
.....###..
#...###...
.####.....""")
    g.spade(-1, H + 1)
    return g


def letter_K():
    g = G()
    g.horns(L).stem(L, 2, H - 2).foot(L, flick=False)
    g.diag(L + 9, 1, L + 3, 8, 2).p(L + 10, 0)
    g.diag(L + 3, 9, L + 8, H - 1, 3).diag(L + 9, H, L + 11, H + 2, 1)
    g.spade(L + 12, H + 3)
    return g


def letter_L():
    g = G()
    g.horns(L).stem(L, 2, H - 1)
    g.hline(L - 1, L + 8, H - 1).p(L + 8, H - 2).p(L + 8, H - 3)
    return g


def letter_M():
    g = G()
    g.horns(L).stem(L, 2, H - 2).foot(L, flick=False)
    g.horns(L + 11).stem(L + 11, 2, H - 2).foot(L + 11)
    g.diag(L + 3, 3, L + 6, 9, 2).diag(L + 10, 3, L + 7, 9, 2)
    return g


def letter_N():
    g = G()
    g.horns(L).stem(L, 2, H - 2).foot(L, flick=False)
    g.diag(L + 3, 2, L + 7, H - 2, 2)
    g.horns(L + 8).stem(L + 8, 2, H - 1)
    return g


def letter_O():
    g = G().art(0, 0, """
.....##.....
....####....
...##..##...
..##....##..
.###....###.
###......###
###......###
###......###
###......###
###......###
###......###
###......###
###......###
.###....###.
..##....##..
...##..##...
....####....
.....##.....""")
    return g


def letter_P():
    g = G().art(0, 0, """
#...#.......
##.###......
.#######....
.###..###...
.###...###..
.###...###..
.###...###..
.###..###...
.#######....
.###........
.###........
.###........
.###........
.###........
.###........
.###........
.###........
..###.......""")
    return g


def letter_Q():
    g = G().art(0, 0, """
.....##.....
....####....
...##..##...
..##....##..
.###....###.
###......###
###......###
###......###
###......###
###......###
###......###
###......###
###......###
.###....###.
..##....##..
...##..##...
....####....
.....##.....""")
    g.diag(6, H - 4, 9, H + 1, 2)
    g.spade(11, H + 2)
    return g


def letter_R():
    g = G().art(0, 0, """
#...#.......
##.###......
.#######....
.###..###...
.###...###..
.###...###..
.###...###..
.###..###...
.#######....
.###........
.###........
.###........
.###........
.###........
.###........
.###........
..###.......
............""")
    g.diag(5, 9, 9, H - 1, 3).diag(10, H, 11, H + 1, 1)
    g.spade(12, H + 2)
    return g


def letter_S():
    g = G().art(0, 0, """
.....##.....
....####....
...##..##...
..##....##..
.###....###.
###......#..
.###........
..####......
....####....
......####..
........###.
.........###
..#......###
.###....###.
..##....##..
...##..##...
....####....
.....##.....""")
    return g


def letter_T():
    g = G()
    g.hline(L - 1, L + 10, 1).hline(L - 1, L + 10, 2)
    g.p(L - 1, 0).p(L + 10, 0).p(L - 1, 3).p(L + 10, 3)
    g.stem(L + 4, 3, H - 2).foot(L + 4)
    return g


def letter_U():
    g = G().art(0, 0, """
#...#..#...#
##.###.##.##
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
.###....###.
..##....###.
...##..###..
....#####.#.
.....##.....""")
    return g


def letter_V():
    g = G()
    g.horns(L).stem(L, 2, 6).horns(L + 8).stem(L + 8, 2, 6)
    g.diag(L, 7, L + 4, H - 1, 3).diag(L + 8, 7, L + 5, H - 2, 2)
    return g


def letter_W():
    g = G()
    g.horns(L).stem(L, 2, H - 2).horns(L + 11).stem(L + 11, 2, H - 2)
    g.diag(L + 3, H - 2, L + 6, 8, 2).diag(L + 10, H - 2, L + 7, 8, 2)
    g.p(L + 3, H - 1).p(L + 11, H - 1).p(L + 12, H - 1)
    return g


def letter_X():
    g = G()
    g.horns(L).horns(L + 9)
    g.diag(L, 2, L + 9, H - 2, 3).diag(L + 9, 2, L + 1, H - 2, 1).foot(L + 9)
    g.rect(L, H - 1, L + 2, H - 1)
    return g


def letter_Y():
    g = G()
    g.horns(L).horns(L + 9)
    g.diag(L, 2, L + 4, 8, 3).diag(L + 10, 2, L + 6, 8, 1)
    g.stem(L + 4, 9, H - 2).foot(L + 4)
    return g


def letter_Z():
    g = G()
    g.hline(L - 1, L + 9, 1).hline(L - 1, L + 9, 2).p(L - 1, 0).p(L - 1, 3)
    g.diag(L + 7, 3, L, H - 3, 3)
    g.hline(L, L + 10, H - 2).hline(L, L + 10, H - 1).p(L + 10, H).p(L + 10, H - 3)
    return g


CAPS = {chr(c): globals()[f'letter_{chr(c)}']() for c in range(ord('A'), ord('Z') + 1)}


SMALL_ART = {'o': '\n....##....\n...####...\n..##..##..\n.###..###.\n###....###\n###....###\n###....###\n###....###\n###....###\n.###..###.\n..##..##..\n...####...\n....##....', 'c': '\n....##....\n...####...\n..##..##..\n.###..###.\n###....#..\n###.......\n###.......\n###.......\n###....#..\n.###..###.\n..##..##..\n...####...\n....##....', 'g': '\n....##....\n...####...\n..##..##..\n.###..###.\n###....#..\n###.......\n###..#####\n###....###\n###....###\n.###..###.\n..##..##..\n...####...\n....##....', 's': '\n....##....\n...####...\n..##..##..\n.###..###.\n###....#..\n.####.....\n...####...\n.....####.\n..#....###\n.###..###.\n..##..##..\n...####...\n....##....', 'd': '\n#...#......\n##.###.....\n.#######...\n.###..###..\n.###...###.\n.###....###\n.###....###\n.###....###\n.###....###\n.###...###.\n.###..###..\n.#######...\n#######....', 'b': '\n#...#.....\n##.###....\n.#######..\n.###..###.\n.###..###.\n.#######..\n.###..###.\n.###...###\n.###...###\n.###...###\n.###..###.\n.#######..\n#######...', 'p': '\n#...#.....\n##.###....\n.#######..\n.###..###.\n.###..###.\n.###..###.\n.#######..\n.###......\n.###......\n.###......\n.###......\n.###......\n..###.....', 'r': '\n#...#.....\n##.###....\n.#######..\n.###..###.\n.###..###.\n.###..###.\n.#######..\n.###.###..\n.###..###.\n.###..###.\n.###...###\n.###...###\n..###...###', 'u': '\n#...#..#...#\n##.###.##.##\n.###....###.\n.###....###.\n.###....###.\n.###....###.\n.###....###.\n.###....###.\n.###....###.\n.###....###.\n..##....###.\n...##..###..\n....#####.#.', 'j': '\n.....#...#\n.....##.##\n......###.\n......###.\n......###.\n......###.\n......###.\n......###.\n......###.\n......###.\n......###.\n.....###..\n#...###...\n.####.....'}
SMALL_ART['q'] = SMALL_ART['o']
SMALL_EXTRA = {'r': 'g.diag(10, 13, 11, 14, 1).spade(12, 15)', 'q': 'g.diag(5, 9, 8, 13, 2).spade(10, 14)', 'j': 'g.spade(-1, 14)'}


def small_hand(ch):
    """A hand-drawn small cap, moved down onto the baseline."""
    g = G().art(0, 0, SMALL_ART[ch])
    if ch in SMALL_EXTRA:
        exec(SMALL_EXTRA[ch], {'g': g})
    return {(x, y + H - XH) for x, y in g.px}


def small(g):
    """Small cap: the capital squeezed to XH rows by dropping rows that repeat the row above,
    nearest the middle first, then other middle rows."""
    rows = {}
    for x, y in g.px:
        rows.setdefault(y, set()).add(x)
    keep = list(range(H))
    mid = H / 2

    def same(y):
        return rows.get(y, set()) == rows.get(y - 1, set())

    while len(keep) > XH:
        cands = [y for y in keep[3:-3] if same(y)] or keep[4:-4]
        y = min(cands, key=lambda y: abs(y - mid))
        keep.remove(y)
    remap = {y: i for i, y in enumerate(keep)}
    out = set()
    for x, y in g.px:
        if y in remap:
            out.add((x, remap[y] + (H - XH)))
        elif y >= H:  # tails below the line move up with the bottom
            out.add((x, y - (H - XH) + (H - XH)))
    return out


def make():
    gl = {}

    def add(ch, px, adv=None):
        m = min(x for x, _ in px)
        px = {(x - m, y) for x, y in px}
        # tails under the line may run under the next letter
        w = max((x for x, y in px if y < H), default=max(x for x, _ in px)) + 1
        gl[ch] = (px, adv if adv is not None else w + 2)

    for ch, g in CAPS.items():
        add(ch, g.px)
        low = ch.lower()
        add(low, small_hand(low) if low in SMALL_ART else small(g))
    # figures and punctuation: Veyra Display's, doubled in weight to sit with the title
    for ch, (px, adv) in vf.glyphs.items():
        if ch.isdigit() or not ch.isalpha():
            if ch == ' ':
                gl[' '] = (set(), 7)
                continue
            # scale 10-row art to the cap height: x by 1, rows stretched 18/10
            out = set()
            for x, y in px:
                for yy in range(round(y * H / 10), round((y + 1) * H / 10)):
                    out.add((x, yy))
                    out.add((x + 1, yy))
            add(ch, out)
    # accents
    marks = {'acute': [(2, 0), (3, 0), (1, 1), (2, 1), (0, 2), (1, 2)],
             'diaer': [(0, 1), (1, 1), (0, 2), (1, 2), (4, 1), (5, 1), (4, 2), (5, 2)],
             'tilde': [(1, 0), (2, 0), (0, 1), (3, 1), (6, 1), (4, 2), (5, 2), (7, 0)]}
    for ch, (base, mark) in vf.COMPOSED.items():
        px, adv = gl[base]
        w = max(x for x, _ in px) + 1
        top = min(y for _, y in px)
        mk = marks[mark]
        mw = max(x for x, _ in mk) + 1
        ox = (w - mw) // 2
        gl[ch] = (px | {(x + ox, y + top - 4) for x, y in mk}, adv)
    return gl


glyphs = make()

if __name__ == '__main__':
    vf.build(sys.argv[1], glyphs, 'Veyra Gothic', H, ABOVE, BELOW, XH)
