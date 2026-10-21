// Veyra, title-screen portrait (V14). Drawn natively at 190x178 and shown x2 (380x356), hips up,
// wings fully in frame. Drawn with the pxl DSL from the references in assets/refs (succubus
// empress sheet): front-facing, half-lidded smirk, one hand holding hellfire, the other on the hip,
// bat wings raised. No outlines: parts separate by value. Light: warm key from the upper left.
// Loop: wings, hair, tail, flame, one blink.
// Build: pxl build assets/veyra_pxl.px.js --only veyra_title,veyra_title_sheet --out assets/sprites/title

pal({
  k: "#12040c",
  // skin, crimson as on the sheet (dark -> light)
  A: "#3a0c1e", S: "#6c1c30", s: "#98303f", z: "#c44a4f", Z: "#dc6a63", X: "#f29484",
  // hair, deep muted violet
  H: "#12081c", h: "#231031", j: "#381a4e", J: "#552876", I: "#7a42a4", i: "#ae7ad2",
  // eyes, amber
  E: "#5a2008", e: "#b05c10", o: "#f0a420", O: "#ffe27a", w: "#fff4ec",
  // lips / mouth
  L: "#2a0612", l: "#5e0f24", m: "#8c2038", M: "#c4505e",
  // horns + tail, crimson
  R: "#2e0610", r: "#6a0e22", q: "#a21c32", Q: "#dc4652", U: "#f49090",
  // wings: membrane + bones
  W: "#3c0a1c", V: "#6e1430", v: "#9e2440", u: "#c43a52", B: "#1e0a22", b: "#3e1a40", n: "#7a3070",
  // suit, black-violet
  K: "#0e0610", D: "#24102c", d: "#3e1e4a", F: "#62366e",
  // gold + gem
  G: "#7a4a14", g: "#c99232", y: "#f6d27a", P: "#7e0a1e", x: "#e0183a", p: "#ff8a96",
  // hellfire
  C: "#a8164a", c: "#f0406e", T: "#ff9a9e", t: "#fff0e4",
})
ramps("kASszZX", "kHhjJIi", "kEeoO", "kLlmM", "kRrqQU", "kWVvu", "kBbn", "kKDdF", "kGgy", "kPxp", "kCcTt")

const W = 190, H = 178, TAU = Math.PI * 2

// ---------------------------------------------------------------- helpers
// Catmull-Rom through flat [x, y, ...] points
function cr(pts, closed = true, steps = 5) {
  const P = []
  for (let i = 0; i < pts.length; i += 2) P.push([pts[i], pts[i + 1]])
  const n = P.length, out = []
  const segs = closed ? n : n - 1
  for (let i = 0; i < segs; i++) {
    const p0 = P[closed ? (i - 1 + n) % n : Math.max(i - 1, 0)], p1 = P[i]
    const p2 = P[(i + 1) % n], p3 = P[closed ? (i + 2) % n : Math.min(i + 2, n - 1)]
    for (let s = 0; s < steps; s++) {
      const t = s / steps, t2 = t * t, t3 = t2 * t
      for (const k of [0, 1])
        out.push(0.5 * (2 * p1[k] + (-p0[k] + p2[k]) * t + (2 * p0[k] - 5 * p1[k] + 4 * p2[k] - p3[k]) * t2
          + (-p0[k] + 3 * p1[k] - 3 * p2[k] + p3[k]) * t3))
    }
  }
  if (!closed) out.push(P[n - 1][0], P[n - 1][1])
  return out
}
const R = (pts) => pts.map(Math.round)
const blob = (c, pts, col) => c.poly(R(cr(pts)), col)
const sharp = (c, pts, col) => c.poly(R(pts), col)
// a curve stroked with a brush
function strand(c, pts, col, w = 1) {
  const q = R(cr(pts, false, 6))
  for (let i = 0; i + 3 < q.length; i += 2) c.line(q[i], q[i + 1], q[i + 2], q[i + 3], col, w)
}
// outline polygon of a tapered ribbon along a curve: w0 at the root, w1 at the tip
function ribbonPoly(pts, w0, w1) {
  const q = cr(pts, false, 8), Lf = [], Rt = [], n = q.length / 2
  for (let i = 0; i < n; i++) {
    const a = Math.max(i - 1, 0), b = Math.min(i + 1, n - 1)
    const dx = q[b * 2] - q[a * 2], dy = q[b * 2 + 1] - q[a * 2 + 1]
    const d = Math.hypot(dx, dy) || 1
    const w = (w0 + (w1 - w0) * (i / (n - 1))) / 2
    Lf.push(q[i * 2] - dy / d * w, q[i * 2 + 1] + dx / d * w)
    Rt.unshift(q[i * 2] + dy / d * w, q[i * 2 + 1] - dx / d * w)
  }
  return R([...Lf, ...Rt])
}
const ribbon = (c, pts, w0, w1, col) => c.poly(ribbonPoly(pts, w0, w1), col)
// mirror flat points around the body axis (x = 95)
const mir = (pts) => pts.map((v, i) => (i % 2 ? v : W - v))
const shift = (pts, dx, dy = 0) => pts.map((v, i) => v + (i % 2 ? dy : dx))
// rotate flat points around (cx, cy)
const rot = (pts, cx, cy, a) => {
  const o = [], cs = Math.cos(a), sn = Math.sin(a)
  for (let i = 0; i < pts.length; i += 2) {
    const dx = pts[i] - cx, dy = pts[i + 1] - cy
    o.push(cx + dx * cs - dy * sn, cy + dx * sn + dy * cs)
  }
  return o
}
// quadratic bezier from a to b bent toward ctrl, without the end point
function quad(a, ctrl, b, n = 6) {
  const o = []
  for (let i = 0; i < n; i++) {
    const t = i / n, u = 1 - t
    o.push(u * u * a[0] + 2 * u * t * ctrl[0] + t * t * b[0], u * u * a[1] + 2 * u * t * ctrl[1] + t * t * b[1])
  }
  return o
}
// point in polygon (pixel centres)
function inPoly(poly, x, y) {
  let inside = false
  const px = x + 0.5, py = y + 0.5
  for (let i = 0, j = poly.length - 2; i < poly.length; j = i, i += 2) {
    const xi = poly[i], yi = poly[i + 1], xj = poly[j], yj = poly[j + 1]
    if (yi > py !== yj > py && px < ((xj - xi) * (py - yi)) / (yj - yi) + xi) inside = !inside
  }
  return inside
}
// sway grows down a lock (hair below y0 moves)
const sw = (pts, s, y0 = 40) => pts.map((v, i) => (i % 2 ? v : v + s * Math.max(0, pts[i + 1] - y0) / 130))

// ---------------------------------------------------------------- wings
// Bat wing: arm arches from the back up to the wrist, a gold claw on the wrist, four fingers fan
// out from the wrist and the membrane hangs between them in scallops. Flap = rotation about the root.
function wing(c, side, a) {
  const root = [74, 92], elbow = [46, 46], wrist = [30, 8]
  const tips = [[2, 42], [4, 88], [18, 122], [42, 140]], attach = [70, 124]
  const T = (pts) => { const o = rot(pts, root[0], root[1], a); return side < 0 ? o : mir(o) }
  const scallop = (p, q, k = 0.42) => quad(p, [(p[0] + q[0]) / 2 + (wrist[0] - (p[0] + q[0]) / 2) * k, (p[1] + q[1]) / 2 + (wrist[1] - (p[1] + q[1]) / 2) * k], q)
  const ends = [...tips, attach]
  // panels: wrist -> tip i -> scallop -> tip i+1
  const panels = []
  for (let i = 0; i < 4; i++) panels.push([...wrist, ...scallop(ends[i], ends[i + 1], i === 3 ? 0.2 : 0.42), ...ends[i + 1]])
  const toward = (pts, k) => pts.map((v, i) => wrist[i % 2] + (v - wrist[i % 2]) * k)
  // membrane: lit translucent toward the scalloped edge, dark toward the wrist
  for (const p of panels) sharp(c, T(p), "v")
  for (const p of panels) sharp(c, T(toward(p, 0.78)), "V")
  for (const p of panels) sharp(c, T(toward(p, 0.45)), "W")
  // inner panel between the last finger, the body and the arm
  sharp(c, T([...wrist, ...attach, ...root, ...quad(root, elbow, wrist, 6)]), "W")
  sharp(c, T([...toward([...attach], 0.9), ...root, ...elbow]), "V")
  // edge glow along the scallops
  for (let i = 0; i < 4; i++) {
    const e = scallop(ends[i], ends[i + 1], i === 3 ? 0.2 : 0.42)
    for (let j = 2; j + 3 < e.length; j += 2) {
      const p = R(T([e[j], e[j + 1] - 1, e[j + 2], e[j + 3] - 1]))
      c.line(p[0], p[1], p[2], p[3], "u")
    }
  }
  // fingers: thick at the wrist, thin at the tips
  for (const tp of tips) {
    // bowed slightly toward the body, like a real finger bone
    const mid = [wrist[0] + (tp[0] - wrist[0]) * 0.55 + 3, wrist[1] + (tp[1] - wrist[1]) * 0.55 - 2]
    ribbon(c, T([...wrist, ...mid, ...tp]), 3, 1, "B")
    strand(c, T([...shift(wrist, 0, 1), ...mid]), "b")
  }
  // arm: leading edge, purple-black with a lit top
  const arm = [...root, ...elbow, ...wrist]
  ribbon(c, T(arm), 6, 4, "B")
  strand(c, T(shift(arm, 0, -1)), "b", 2)
  strand(c, T(shift([root[0] - 6, root[1] - 14, ...elbow, ...wrist], 0, -2)), "n")
  c.circ(...R(T(wrist)), 2, "b")
  // gold claw growing out of the wrist, hooked outward
  const claw = [...wrist, wrist[0] - 3, wrist[1] - 6, wrist[0] - 1, wrist[1] - 12, wrist[0] + 4, wrist[1] - 15]
  ribbon(c, T(claw), 6, 0.5, "g")
  strand(c, T(shift(claw.slice(0, 4), -1)), "y")
  ribbon(c, T(shift([...wrist, wrist[0] - 2, wrist[1] - 6, wrist[0], wrist[1] - 11], 1.5)), 2, 0.5, "G")
}

// ---------------------------------------------------------------- hair
// a clump: base ribbon, a shadow down its right side and a sheen down its left (light from the left)
function clump(c, pts, w0, w1, base, dark, light, sheen = 0.6) {
  const poly = ribbonPoly(pts, w0, w1)
  c.poly(poly, base)
  c.clip(poly, () => {
    ribbon(c, shift(pts, w0 * 0.32), w0 * 0.5, w1 * 0.5, dark)
    if (light) {
      const n = Math.max(4, Math.round((pts.length / 2) * sheen) * 2)
      strand(c, shift(pts.slice(0, n), -w0 * 0.22), light, w0 > 10 ? 2 : 1)
    }
  })
}

function backHair(c, s) {
  // the whole mass, darkest
  blob(c, sw([95, 7, 120, 13, 128, 30, 134, 54, 140, 84, 144, 114, 148, 142, 152, 172, 38, 172, 42, 142, 46, 114, 50, 84, 56, 54, 62, 30, 70, 13], s), "H")
  // S-waves: every clump swings out and back as it falls, the waves grow toward the tips
  const wave = (x0, y0, x1, y1, amp, ph) => {
    const o = []
    for (let i = 0; i <= 8; i++) {
      const f = i / 8
      o.push(x0 + (x1 - x0) * f + Math.sin(f * Math.PI) * (x1 - x0) * 0.15 + Math.sin(f * 7 + ph) * amp * (0.3 + f), y0 + (y1 - y0) * f)
    }
    return o
  }
  const L = [
    // [points, w0, w1, base, dark, light]
    [wave(72, 14, 46, 172, 3, 0.0), 16, 6, "h", "H", "j"],
    [wave(78, 18, 60, 172, 2, 2.0), 14, 7, "h", "H", "j"],
    [wave(70, 18, 36, 168, 3, 1.0), 10, 1, "j", "h", "J"],
    [wave(66, 30, 40, 174, 3, 3.0), 9, 1, "j", "h", "J"],
  ]
  const Rt = L.map(([p, ...rest]) => [mir(p.map((v, i) => (i % 2 ? v : v + Math.sin(i) * 1.5))), ...rest])
  for (const [p, w0, w1, b, d, l] of [...L, ...Rt]) clump(c, sw(p, s), w0, w1, b, d, l)
}

// clean pixel noise in hair and wings: lone pixels take their neighbours' colour, stray edge
// pixels with a single neighbour are dropped
function despeckle(c, keys, x0, y0, x1, y1) {
  for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
    if (!c.is(x, y, keys)) continue
    const nb = [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]
    const solid = nb.filter(([a, b]) => c.is(a, b, "*"))
    if (solid.length <= 1) { c.px(x, y, "_"); continue }
    const k = c.key(x, y), others = solid.map(([a, b]) => c.key(a, b))
    if (others.length === 4 && others.every((o) => o === others[0] && o !== k)) c.px(x, y, others[0])
  }
}

function crownAndBangs(c) {
  // crown over the skull
  // its lower edge is the top of the M: two arches over the forehead triangles
  blob(c, [95, 6, 114, 10, 124, 22, 126, 38, 118, 30, 107, 18, 101, 16, 95, 20, 89, 16, 83, 18, 72, 30, 64, 38, 66, 22, 76, 10], "j")
  // the shine band across the crown
  ribbon(c, [72, 22, 82, 15, 95, 12, 108, 14, 118, 21], 4, 3, "J")
  strand(c, [76, 19, 84, 14, 95, 12, 104, 13], "I")
  c.px(85, 14, "i"); c.px(86, 13, "i"); c.px(87, 13, "i")
  // bangs as on the sheet, an open M: a centre lock down to the nose bridge, a bare triangle of
  // forehead over each eye, long locks sweeping across the outer corners to the cheeks
  const locks = [
    // [root, mid, tip, width, sheen]
    [[94, 8], [95, 24], [95, 48], 10, "J"],
    [[88, 10], [84, 22], [80, 46], 14, "J"],
    [[79, 14], [73, 30], [71, 57], 9, "J"],
    [[100, 10], [106, 22], [110, 46], 14, null],
    [[109, 14], [117, 30], [119, 57], 9, null],
  ]
  for (const [r, m, t, w, sh] of locks) clump(c, [...r, ...m, ...t], w, 1, "j", "h", sh, 0.4)
  // the shine band carries on across the bangs, broken by the lock edges
  for (let x = 74; x < 117; x++) {
    const y = Math.round(16 + ((x - 95) / 22) ** 2 * 5)
    if (c.is(x, y, "j")) c.px(x, y, x < 95 ? "I" : "J")
    if (c.is(x, y + 1, "j")) c.px(x, y + 1, "J")
  }
  // parting
  strand(c, [94, 8, 93, 14, 92, 20], "h")
}

function sideLocks(c, s) {
  // they come out from behind the ears, hug the jaw and fall over the shoulders to the chest
  clump(c, sw([76, 42, 75, 54, 72, 66, 66, 82, 62, 102, 60, 122, 57, 140], s), 7, 1, "j", "h", "J")
  clump(c, sw([114, 42, 115, 54, 118, 66, 124, 82, 128, 102, 130, 122, 134, 142], s), 7, 1, "j", "h", null)
  clump(c, sw([71, 46, 68, 62, 62, 80, 58, 100, 56, 118], s), 6, 1, "j", "H", "J")
  clump(c, sw([119, 46, 122, 62, 128, 80, 132, 100, 134, 118], s), 6, 1, "h", "H", null)
}

// ---------------------------------------------------------------- head
function horn(c, side) {
  const M = side < 0 ? (p) => p : mir
  // thick root in the hair, sweeping far out, then up, the tip hooking back in
  const spine = [83, 22, 72, 17, 62, 10, 55, 1, 54, -6, 57, -10]
  const body = ribbonPoly(M(spine), 13, 1)
  c.poly(body, "q")
  c.clip(body, () => {
    ribbon(c, M(shift(spine, 3, 3)), 8, 0.5, "r")               // underside
    ribbon(c, M(shift(spine, 5, 5)), 4, 0.5, "R")
    strand(c, M(shift(spine.slice(0, 8), -2, -2)), "Q")         // lit top edge
    strand(c, M(shift(spine.slice(0, 6), -3, -3)), "U")
    for (const [x, y, dx, dy] of [[73, 16, 2, 5], [64, 9, 4, 4], [57, 1, 5, 2]]) {   // ridges across
      const p = M([x - dx, y - dy, x + dx, y + dy])
      c.line(...R(p), "r")
    }
  })
}

function ears(c) {
  for (const M of [(p) => p, mir]) {
    blob(c, M([76, 34, 68, 31, 60, 30, 67, 36, 75, 44]), "z")
    blob(c, M([75, 36, 67, 32, 69, 36, 74, 41]), "s")
  }
  c.line(62, 30, 69, 32, "Z")
}

function face(c) {
  // a longer oval like the sheet: narrow cheeks, pointed chin
  blob(c, [76, 24, 114, 24, 116, 36, 115, 46, 112, 56, 106, 65, 99, 71, 95, 73, 91, 71, 84, 65, 78, 56, 75, 46, 74, 36], "z")
  // shadow down the right side and under the jaw
  blob(c, [113, 34, 116, 37, 115, 46, 112, 56, 106, 65, 100, 70, 107, 60, 111, 48], "s")
  // smoky lids above the lashes
  for (const [x0, x1] of [[77, 89], [100, 112]]) for (let x = x0; x <= x1; x++) c.px(x, 32, "s")
  // nose
  c.px(96, 54, "s"); c.px(96, 55, "s"); c.px(95, 56, "S")
}

// eyes: almond, outer corners swept up (tsurime), heavy lid over the iris, slit pupils
const EYE = `
  .kkkkkkkkkkkk..
  kkkkkkkkkkkkkkk
  kk.wwEEEEEEkk..
  ...wweeokeew...
  ....woookoow...
  ....wOOOOOOw...
  .....OOOOOO....
  ....SSSSSSS....`
const EYE_SHUT = `
  ...............
  ...............
  ...............
  ...............
  ...............
  kk...........k.
  .kkkkkkkkkkkk..
  ....SSSSSSS....`
const flipRows = (g) => g.split("\n").map((r) => r.trim()).filter(Boolean).map((r) => [...r].reverse().join("")).join("\n")
function eyes(c, blink) {
  const g = blink ? EYE_SHUT : EYE
  c.grid(75, 33, g)
  c.grid(100, 33, flipRows(g))
  // brows: thin, low, slanting down to the nose: dominant
  for (const M of [(p) => p, mir]) {
    const b = M([76, 28, 83, 28, 89, 31])
    strand(c, b, "A")
    const i = R(M([88, 31, 89, 32]))
    c.px(i[0], i[1], "A"); c.px(i[2], i[3], "A")
  }
  // wicked grin: one corner hooked up, mouth just open, a fang over the lower lip
  c.grid(90, 61, `
    .........l
    ll.....ll.
    .lLLLLLl..
    ..wLLl....
    ...mmm....`)
}

// ---------------------------------------------------------------- body
// regions shared by the body and the corset
const LB = [79, 111], RB = [111, 111], BRX = 19, BRY = 18
const inEll = (p, rx, ry, x, y) => ((x + 0.5 - p[0]) / rx) ** 2 + ((y + 0.5 - p[1]) / ry) ** 2 <= 1
const inCircle = (p, r, x, y) => inEll(p, r, r, x, y)
const inBreast = (x, y, g = 0.5) => inEll(LB, BRX + g, BRY + g, x, y) || inEll(RB, BRX + g, BRY + g, x, y)
// neckline: deep V at the cleavage, rising over each breast, a small lace scallop along it
const cupTop = (x) => {
  const d = x < 95 ? 95 - x : x - 94
  const base = d < 1 ? 117 : 117 - Math.min(d, 14) * 0.8 + Math.max(0, d - 20) * 0.7
  return base + (Math.abs(Math.sin(x * 0.7)) < 0.4 ? 1 : 0)
}
const TORSO = R(cr([89, 58, 101, 58, 101, 76, 114, 80, 126, 81, 132, 88, 128, 98, 127, 110, 124, 126, 116, 142,
  126, 156, 130, 166, 132, 172, 58, 172, 60, 166, 64, 156, 74, 142, 66, 126, 63, 110, 62, 98, 58, 88, 64, 81, 76, 80, 89, 76], true, 3))
const inCups = (x, y) => inBreast(x, y) && y >= cupTop(x)
// high-cut leg openings that hug the hips: the edge eases from the waist down to the crotch
const legEdge = (y) => 17 * Math.pow(Math.max(0, y - 138) / 32, 1.4)
const inLegs = (x, y) => y > 138 && (x < 72 + legEdge(y) || x > 118 - legEdge(y))
const inCorset = (x, y) => !inLegs(x, y) && (inCups(x, y) || (y >= 118 && inPoly(TORSO, x, y)))

function torso(c) {
  sharp(c, TORSO, "z")
  // form light: a rim of light down the left edge, shadow wrapping the right edge
  const inT = (dx, dy) => (x, y) => inPoly(TORSO, x + dx, y + dy)
  c.clip((x, y) => inT(0, 0)(x, y) && !inT(-3, 0)(x, y) && y > 76, () => c.rect(40, 70, 110, 110, "Z"))
  c.clip((x, y) => inT(0, 0)(x, y) && !inT(7, 0)(x, y) && y > 84, () => c.rect(40, 70, 110, 110, "s"))
  // neck shadow under the chin, a hint of collarbones
  blob(c, [89, 66, 101, 66, 101, 75, 95, 78, 89, 75], "s")
  c.line(81, 84, 87, 85, "s"); c.line(109, 84, 103, 85, "s")
  // thighs: round tops, inner shadow toward the crotch
  c.clip((x, y) => inPoly(TORSO, x, y) && inLegs(x, y), () => {
    c.ellipse(68, 172, 7, 14, "Z")
    c.ellipse(86, 178, 6, 10, "s")
    c.ellipse(104, 178, 6, 10, "s")
  })
}

function breasts(c) {
  for (const [p, lit] of [[LB, true], [RB, false]]) {
    c.ellipse(p[0], p[1], BRX, BRY, "z")
    // light rims the upper left of each, shadow wraps the lower right
    c.clip((x, y) => inEll(p, BRX, BRY, x, y) && !inEll([p[0] + 2, p[1] + 3], BRX, BRY, x, y), () => c.rect(p[0] - BRX, p[1] - BRY, BRX * 2, BRY * 2, lit ? "Z" : "z"))
    // shadow only on the outer side, away from the cleavage
    c.clip((x, y) => inEll(p, BRX, BRY, x, y) && !inEll([p[0] + (lit ? 3 : -3), p[1] - 2], BRX, BRY, x, y) && (lit ? x < p[0] : x > p[0]),
      () => c.rect(p[0] - BRX, p[1] - BRY, BRX * 2, BRY * 2, lit ? "z" : "s"))
    if (lit) { c.px(p[0] - 9, p[1] - 11, "X"); c.px(p[0] - 8, p[1] - 12, "X") }
  }
  // cleavage: the right breast's inner edge
  c.clip((x, y) => inEll(RB, BRX, BRY, x, y) && !inEll([RB[0] + 1, RB[1]], BRX, BRY, x, y) && y > 101, () => c.rect(88, 92, 12, 30, "s"))
}

function corset(c) {
  c.clip(inCorset, () => {
    c.rect(50, 90, 90, 84, "D")
    // cups: lit upper left of each, dark underneath and outside
    // cups: the same light as the skin, a sheen on the upper left of each
    for (const [p, lit] of [[LB, true], [RB, false]])
      c.clip((x, y) => inEll([p[0] - 3, p[1] + 1], BRX - 6, BRY - 6, x, y) && !inEll([p[0] + 1, p[1] + 4], BRX - 6, BRY - 6, x, y),
        () => c.rect(p[0] - BRX, p[1] - BRY, BRX * 2, BRY * 2, lit ? "F" : "d"))
    // under-bust crease
    c.clip((x, y) => inBreast(x, y, 1.5) && !inBreast(x, y, -1) && y > LB[1] + 4, () => c.rect(50, 90, 90, 40, "K"))
    // panels: seams curve with the body, the right side turns into shadow
    strand(c, [84, 124, 82, 136, 85, 150, 89, 168], "K")
    strand(c, [106, 124, 108, 136, 105, 150, 101, 168], "K")
    blob(c, [114, 122, 128, 118, 120, 140, 106, 172, 101, 172, 110, 140], "K")
    strand(c, [74, 126, 77, 138, 82, 152], "d", 2)
    // gold piping along the neckline and the leg openings
    for (let x = 50; x < 140; x++) {
      let y = 85
      while (y < 135 && !inCups(x, y)) y++
      if (y < 135) c.px(x, y, "g")
    }
    for (let y = 139; y < 172; y++) {
      c.px(Math.ceil(72 + legEdge(y)), y, "g"); c.px(Math.floor(118 - legEdge(y)), y, "g")
    }
    // centre gem under the bust, gold drop to the lower gem
    sharp(c, [95, 118, 100, 125, 95, 135, 90, 125], "g")
    sharp(c, [95, 121, 98, 125, 95, 131, 92, 125], "x")
    c.px(94, 123, "p"); c.px(95, 122, "p")
    c.line(95, 135, 95, 154, "G")
    sharp(c, [95, 154, 98, 159, 95, 166, 92, 159], "g"); sharp(c, [95, 156, 96, 159, 95, 163, 94, 159], "x")
  })
  // high collar with a sheer V panel down the sternum, gold edged, gem at the cleavage
  c.rect(89, 73, 13, 4, "K"); c.hline(89, 73, 13, "g"); c.hline(89, 76, 13, "G")
  sharp(c, [89, 77, 101, 77, 96, 86, 95, 88, 94, 86], "D")
  strand(c, [89, 77, 92, 82, 95, 88], "g"); strand(c, [101, 77, 98, 82, 95, 88], "g")
  sharp(c, [95, 77, 97, 80, 95, 84, 93, 80], "x"); c.px(94, 79, "p")
}

// ---------------------------------------------------------------- arms
function armRight(c) {
  // viewer's right: elbow out, hand on the hip
  c.circ(126, 88, 7,  "z")
  ribbon(c, [127, 88, 133, 104, 137, 120], 12, 9, "z")
  c.clip(ribbonPoly([127, 88, 133, 104, 137, 120], 12, 9), () => ribbon(c, [131, 90, 137, 104, 141, 120], 7, 6, "s"))
  c.clip((x, y) => inCircle([126, 88], 7, x, y), () => c.circ(123, 85, 3,  "Z"))
  // glove from mid upper arm
  ribbon(c, [134, 108, 137, 120], 11, 9, "D")
  ribbon(c, [137, 120, 132, 136, 122, 150], 9, 7, "D")
  strand(c, [139, 116, 139, 128, 133, 142, 125, 151], "K", 2)
  strand(c, [133, 120, 129, 134, 121, 145], "d")
  c.line(129, 108, 141, 104,  "g", 2); c.px(131, 105,  "y"); c.px(135, 104,  "y"); c.px(139, 102,  "y"); c.px(135, 107,  "x")
  c.line(119, 147, 126, 152,  "g", 2); c.px(122, 149,  "x")
  strand(c, [122, 98, 126, 106, 129, 112], "S")             // crease against the body
  // hand on the hip, fingers spread down over the curve
  blob(c, [111, 149, 119, 147, 123, 153, 119, 161, 112, 160, 109, 154], "z")
  c.line(111, 154, 110, 160,  "s"); c.line(114, 155, 113, 161,  "s"); c.line(117, 155, 117, 160,  "s")
  c.line(113, 150, 118, 149,  "Z")
  strand(c, [108, 154, 111, 161, 119, 162], "S")
}

function armLeft(c) {
  // viewer's left: elbow down, forearm out, palm up holding the flame
  c.circ(64, 88, 7,  "z")
  ribbon(c, [63, 88, 57, 104, 53, 118], 12, 9, "z")
  c.clip(ribbonPoly([63, 88, 57, 104, 53, 118], 12, 9), () => ribbon(c, [68, 90, 62, 104, 58, 118], 6, 5, "s"))
  c.clip((x, y) => inCircle([64, 88], 7, x, y), () => c.circ(61, 85, 4,  "Z"))
  // glove
  ribbon(c, [55, 108, 53, 118], 11, 9, "D")
  ribbon(c, [53, 118, 45, 113, 39, 104], 9, 6, "D")
  strand(c, [55, 121, 46, 117, 40, 108], "K", 2)
  strand(c, [51, 113, 44, 109, 40, 102], "d")
  c.line(48, 106, 62, 110,  "g", 2); c.px(50, 105,  "y"); c.px(54, 105,  "y"); c.px(58, 107,  "y"); c.px(55, 108,  "x")
  c.line(36, 102, 41, 107,  "g", 2); c.px(38, 104,  "x")
  strand(c, [68, 98, 64, 106, 61, 112], "S")                // crease against the body
  // palm up: fingers curl up on the left, thumb on the right
  blob(c, [25, 98, 29, 101, 35, 101, 39, 98, 40, 103, 35, 107, 28, 106, 24, 102], "z")
  sharp(c, [24, 97, 26, 94, 28, 99, 26, 101], "z")
  sharp(c, [28, 96, 30, 94, 31, 99, 29, 100], "Z")
  sharp(c, [38, 96, 40, 93, 41, 99, 39, 100], "z")
  c.line(26, 104, 36, 105,  "s"); c.line(28, 106, 34, 106,  "S")
}

function flame(c, t) {
  const k = t * TAU, x = 32, y = 100
  const tongue = (dx, h, w, ph, cols) => {
    cols.forEach((col, i) => {
      const f = 1 - i * 0.24, hh = h * f + Math.sin(k * 2 + ph + i) * 2, ww = w * f
      const tip = Math.sin(k + ph) * 3 * (1 - i * 0.2)
      blob(c, [x + dx - ww, y, x + dx - ww * 0.9, y - hh * 0.3, x + dx - ww * 0.3 + tip * 0.5, y - hh * 0.7,
        x + dx + tip, y - hh, x + dx + ww * 0.6 + tip * 0.5, y - hh * 0.6, x + dx + ww * 0.9, y - hh * 0.25, x + dx + ww, y], col)
    })
  }
  tongue(-5, 18, 5, 2.1, ["C", "c"])
  tongue(5, 20, 5, 4.2, ["C", "c"])
  tongue(0, 34, 9, 0, ["C", "c", "T", "t"])
  for (let i = 0; i < 4; i++) {
    const p = (t + i / 4) % 1
    c.px(Math.round(x - 8 + i * 5 + Math.sin((p * 2 + i) * TAU) * 2), Math.round(y - 32 - p * 26), p < 0.5 ? "T" : p < 0.8 ? "c" : "C")
  }
}

// ---------------------------------------------------------------- tail
function tail(c, t) {
  // an S from behind the hip; each joint lags the one before, so the motion travels to the tip
  const base = [118, 160, 136, 160, 152, 150, 160, 134, 158, 118, 162, 104]
  const pts = base.map((v, i) => (i % 2 ? v : v + Math.sin(t * TAU - i * 0.35) * (i / 2) * 0.7))
  ribbon(c, pts, 7, 3, "r")
  c.clip(ribbonPoly(pts, 7, 3), () => { strand(c, shift(pts, -2, -1), "q", 2); strand(c, shift(pts, -2, -2), "Q") })
  // spade: rounded barbs, a point, its notch sitting over the tail end
  const q = cr(pts, false, 8), n = q.length
  const ex = q[n - 2], ey = q[n - 1]
  let dx = ex - q[n - 6], dy = ey - q[n - 5]
  const d = Math.hypot(dx, dy); dx /= d; dy /= d
  const px = -dy, py = dx
  const P = (f, s) => [ex + dx * f + px * s, ey + dy * f + py * s]
  const spade = [...P(14, 0), ...P(8, 3.5), ...P(2, 7), ...P(-1, 6.5), ...P(1, 2.5), ...P(-1, 0), ...P(1, -2.5), ...P(-1, -6.5), ...P(2, -7), ...P(8, -3.5)]
  blob(c, spade, "q")
  c.clip(R(cr(spade)), () => {
    blob(c, [...P(12, 0), ...P(6, 3), ...P(2, 5), ...P(3, 1)], "Q")
    blob(c, [...P(12, 0), ...P(3, -6), ...P(0, -5), ...P(3, -1)], "r")
  })
}

// ---------------------------------------------------------------- the portrait
function veyra(c, t) {
  const flap = Math.sin(t * TAU) * 0.05
  const sway = Math.sin(t * TAU + 1) * 3
  const blink = c.frame === 26
  wing(c, -1, flap); wing(c, 1, flap)
  backHair(c, sway)
  despeckle(c, "HhjJIiWVvuBbn", 0, -10, W, H)
  tail(c, t)
  torso(c)
  breasts(c)
  corset(c)
  armRight(c)
  face(c)
  horn(c, -1); horn(c, 1)
  crownAndBangs(c)
  // bangs cast a thin shadow on the forehead
  for (let y = 20; y < 50; y++) for (let x = 70; x < 120; x++)
    if (c.is(x, y, "zZ") && c.is(x, y - 1, "HhjJIi")) c.px(x, y, "s")
  armLeft(c)
  sideLocks(c, sway)
  despeckle(c, "HhjJIi", 0, -10, 70, H); despeckle(c, "HhjJIi", 120, -10, W, H)
  ears(c)
  eyes(c, blink)
  flame(c, (t * 3) % 1)
}

sprite("veyra_still", { w: W, h: H }, layer("all", { y: 10 }, (c) => { c.frame = 0; veyra(c, 0) }), glow("cTt", { r: 14, strength: 0.35 }))
anim("veyra_title", { w: W, h: H, frames: 32, fps: 10 }, layer("all", { y: 10 }, (c) => veyra(c, c.phase)), glow("cTt", { r: 14, strength: 0.35 }))
sheet("veyra_title_sheet", ["veyra_title"], { cols: 8 })
