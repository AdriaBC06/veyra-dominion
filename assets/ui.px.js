// UI kit (design.md 7.14, V19b): every menu and the HUD are built from these pieces, in the
// language of the title buttons (title.px.js): dark plum lacquer in chamfered gilt frames,
// rubies set into the gold, an ember glow when lit. Light comes from the top left.
// Build:
//   node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/ui.px.js --out assets/sprites/ui --no-preview
// The game draws them sliced (`kit.rs`): panels and slots as 9-slices (corner size noted on each),
// buttons, tabs and slider tracks as 3-slices of fixed height. Stretched middles are uniform rows
// or columns, so stretching never smears a detail.

pal({
  k: "#16091a",                                            // ink outline, Veyra's plum black
  z: "#06030a",                                            // groove / slot bed
  P: "#0f0712", Q: "#170b1a", S: "#2a0c1a", T: "#3c1222",  // lacquer: idle -> lit
  b: "#4a3320", B: "#7a5226", G: "#d8a040", Y: "#ffe08a",  // gilt: dim -> bright
  e: "#3a0a14", E: "#6a1420",                              // ember glow
  R: "#5c0c1c", r: "#b01c34", p: "#ff7088", w: "#ffe8ee",  // ruby: shade -> glint
  v: "#3a1450", V: "#7a3aa0", u: "#c08ae0",                // union violet
  s: "#2a2430", t: "#5a4e60",                              // tarnished (disabled, locked)
})

// Rings of a chamfered box: ring(x, y) = how deep (x, y) sits inside the box (x0, y0)-(x1, y1)
// with corner cut `ch`; -1 outside. Ring 0 is the outermost pixel row.
function ringOf(x0, y0, x1, y1, ch) {
  return (x, y) => {
    if (x < x0 || x > x1 || y < y0 || y > y1) return -1
    const dx = Math.min(x - x0, x1 - x), dy = Math.min(y - y0, y1 - y)
    return Math.min(dx, dy, dx + dy - ch)
  }
}
// Top/left faces catch the light (split along the anti-diagonal of a square corner).
const litSide = (x, y, x0, y0, x1, y1) => (x - x0) + (y - y0) < (x1 - x) + (y1 - y)

// The ember glow outside a lit frame: two solid rings, the outer one darker.
function glowRing(c, ring) {
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) {
    const d = ring(x, y)
    if (d === -1 && ringOuter(ring, x, y) === 1) c.px(x, y, "E")
    else if (d === -1 && ringOuter(ring, x, y) === 2) c.px(x, y, "e")
  }
}
// Distance (1, 2, ...) of an outside pixel from the box, by growing it.
function ringOuter(ring, x, y) {
  for (let i = 1; i <= 3; i++) {
    for (let oy = -i; oy <= i; oy++) for (let ox = -i; ox <= i; ox++)
      if (Math.abs(ox) + Math.abs(oy) <= i + 1 && ring(x + ox, y + oy) >= 0) return i
  }
  return 9
}

// A ruby cabochon centred at (cx, cy): 5 wide, 5 tall, glint top left.
function ruby(c, cx, cy, lit) {
  c.poly([cx - 2.5, cy + 0.5, cx + 0.5, cy - 2.5, cx + 3.5, cy + 0.5, cx + 0.5, cy + 3.5], lit ? "r" : "R")
  c.px(cx - 1, cy - 1, lit ? "w" : "r"); c.px(cx, cy - 1, lit ? "p" : "r")
  c.px(cx + 1, cy + 1, "R"); c.px(cx, cy + 2, lit ? "R" : "e")
}

// ------------------------------------------------------------------ panel (9-slice, corner 15)
// 40x40 with a 4 px glow margin; frame 0 idle, 1 lit (cards, the focused panel). Gilt rim two
// pixels thick over an ink line, a groove, lacquer with a lit top row and a thin gilt hairline
// inside. Each corner holds a gilt
// lozenge with a ruby, and short gilt brackets run along the edges from it.
const PM = 4, PW = 40
anim("ui_panel", { w: PW, h: PW, frames: 2, fps: 1 }, (c) => {
  const on = c.frame === 1, a = PM, z = PW - 1 - PM
  const ring = ringOf(a, a, z, z, 4)
  if (on) glowRing(c, ring)
  for (let y = 0; y < PW; y++) for (let x = 0; x < PW; x++) {
    const d = ring(x, y); if (d < 0) continue
    const lit = litSide(x, y, a, a, z, z)
    if (d === 0) c.px(x, y, "k")
    else if (d === 1) c.px(x, y, lit ? (on ? "Y" : "G") : (on ? "G" : "B"))
    else if (d === 2) c.px(x, y, lit ? (on ? "G" : "B") : "b")
    else if (d === 3) c.px(x, y, "z")
    else if (d === 4 && y === a + 4) c.px(x, y, on ? "T" : "Q")
    else if (d === 6) c.px(x, y, on ? "B" : "b")                                     // inner hairline
    else c.px(x, y, on ? "S" : "P")
  }
  // corner lozenges with rubies, and the brackets from them
  for (const [cx, cy] of [[a + 3, a + 3], [z - 3, a + 3], [a + 3, z - 3], [z - 3, z - 3]]) {
    c.poly([cx - 4.5, cy + 0.5, cx + 0.5, cy - 4.5, cx + 5.5, cy + 0.5, cx + 0.5, cy + 5.5], "k")
    c.poly([cx - 3.5, cy + 0.5, cx + 0.5, cy - 3.5, cx + 4.5, cy + 0.5, cx + 0.5, cy + 4.5], on ? "G" : "B")
    c.px(cx - 2, cy - 1, on ? "Y" : "G"); c.px(cx - 1, cy - 2, on ? "Y" : "G")
    ruby(c, cx, cy, on)
    const sx = cx < PW / 2 ? 1 : -1, sy = cy < PW / 2 ? 1 : -1
    for (let i = 5; i <= 8; i++) {
      c.px(cx + sx * i, cy - sy * 3 + sy * 1, on ? "G" : "B")
      c.px(cx - sx * 3 + sx * 1, cy + sy * i, on ? "G" : "B")
    }
  }
})

// ------------------------------------------------------------------ inset (9-slice, corner 4)
// A recessed well inside a panel: description boxes, list backgrounds, the slider groove.
sprite("ui_inset", { w: 12, h: 12 }, (c) => {
  const ring = ringOf(0, 0, 11, 11, 2)
  for (let y = 0; y < 12; y++) for (let x = 0; x < 12; x++) {
    const d = ring(x, y); if (d < 0) continue
    const lit = litSide(x, y, 0, 0, 11, 11)
    c.px(x, y, d === 0 ? (lit ? "z" : "B") : d === 1 && lit ? "k" : "z")
  }
})

// ------------------------------------------------------------------ button (3-slice, caps 14)
// 40x34 (26 tall + the 4 px glow margin). Frames: idle, lit, pressed (one pixel down, gold
// dimmed, no glow) and disabled (tarnished). A ruby sits in the frame at each end.
const BW = 40, BH = 26, BM = 4
anim("ui_button", { w: BW + 2 * BM, h: BH + 2 * BM, frames: 4, fps: 1 }, (c) => {
  const f = c.frame, on = f === 1, down = f === 2, off = f === 3
  const x0 = BM, x1 = BM + BW - 1, y0 = BM + (down ? 1 : 0), y1 = BM + BH - 1
  const ring = ringOf(x0, y0, x1, y1, 3)
  if (on) glowRing(c, ring)
  const gilt = off ? ["t", "s"] : on ? ["Y", "G"] : down ? ["B", "b"] : ["G", "B"]
  const lac = off ? ["P", "P"] : on ? ["T", "S"] : down ? ["S", "Q"] : ["Q", "P"]
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) {
    const d = ring(x, y); if (d < 0) continue
    if (d === 0) c.px(x, y, y < y0 + 3 || x < x0 + 2 ? gilt[0] : gilt[1])
    else if (d === 1) c.px(x, y, "z")
    else c.px(x, y, y < y0 + Math.floor((y1 - y0) * 0.42) ? lac[0] : lac[1])
  }
  const my = y0 + Math.floor((y1 - y0 + 1) / 2)
  for (const gx of [x0 + 1, x1 - 1]) {
    if (off) { c.poly([gx - 3, my, gx + 0.5, my - 3.5, gx + 4, my, gx + 0.5, my + 4], "s"); continue }
    c.poly([gx - 3, my, gx + 0.5, my - 3.5, gx + 4, my, gx + 0.5, my + 4], on ? "r" : "R")
    c.px(gx, my - 1, on ? "p" : "r"); c.px(gx, my + 2, "R")
  }
})

// ------------------------------------------------------------------ title rule
// Under every screen title: a gilt rule drawn by the game between two finials, with a ruby in a
// gilt lozenge at its centre (like DOMINION on the title). `ui_rule_end` faces left (the right
// end is the same, flipped).
sprite("ui_rule_end", { w: 9, h: 7 }, `
  ....k....
  ...kGk...
  kkkGYGkkk
  .kGYGBBkG
  kkkBBbkkk
  ...kbk...
  ....k....`)
sprite("ui_rule_gem", { w: 15, h: 11 }, (c) => {
  c.poly([0, 5.5, 7.5, -2, 15, 5.5, 7.5, 13], "k")
  c.poly([1.5, 5.5, 7.5, -0.5, 13.5, 5.5, 7.5, 11.5], "B")
  c.poly([1.5, 5.5, 7.5, -0.5, 7.5, 5.5], "G")
  c.px(7, 1, "Y"); c.px(6, 2, "Y"); c.px(3, 5, "Y")
  ruby(c, 7, 5, true)
})

// ------------------------------------------------------------------ icon slots (9-slice, corner 5)
// 20x20 for a 16 px icon. Rims: 0 plain gilt, 1 ruby (new), 2 bright gold (evolution),
// 3 violet (union), 4 tarnished (locked / empty), 5 lit (focused).
const RIMS = [["G", "B"], ["r", "R"], ["Y", "G"], ["u", "V"], ["t", "s"], ["Y", "G"]]
anim("ui_slot", { w: 20, h: 20, frames: RIMS.length, fps: 1 }, (c) => {
  const [hi, lo] = RIMS[c.frame], ring = ringOf(0, 0, 19, 19, 2)
  for (let y = 0; y < 20; y++) for (let x = 0; x < 20; x++) {
    const d = ring(x, y); if (d < 0) continue
    const lit = litSide(x, y, 0, 0, 19, 19)
    if (d === 0) c.px(x, y, "k")
    else if (d === 1) c.px(x, y, lit ? hi : lo)
    else c.px(x, y, c.frame === 5 ? (y < 8 ? "S" : "Q") : y < 8 ? "Q" : "z")
  }
  // rivets at the corners
  for (const [x, y] of [[2, 2], [17, 2], [2, 17], [17, 17]]) c.px(x, y, hi)
})

// ------------------------------------------------------------------ slider
// Track: 3-slice, caps 4, 9 tall: a recessed groove the game fills with ember (rows R, r, p).
// Knob: a gilt lozenge with a ruby; frame 1 lit.
anim("ui_slider_track", { w: 12, h: 9, frames: 2, fps: 1 }, (c) => {
  const on = c.frame === 1, ring = ringOf(0, 0, 11, 8, 2)
  for (let y = 0; y < 9; y++) for (let x = 0; x < 12; x++) {
    const d = ring(x, y); if (d < 0) continue
    const lit = litSide(x, y, 0, 0, 11, 8)
    c.px(x, y, d === 0 ? (lit ? (on ? "G" : "B") : "b") : d === 1 ? "k" : "z")
  }
})
anim("ui_slider_knob", { w: 11, h: 15, frames: 2, fps: 1 }, (c) => {
  const on = c.frame === 1
  c.poly([0, 7.5, 5.5, -0.5, 11, 7.5, 5.5, 15.5], "k")
  c.poly([1.5, 7.5, 5.5, 1.5, 9.5, 7.5, 5.5, 13.5], on ? "G" : "B")
  c.poly([1.5, 7.5, 5.5, 1.5, 5.5, 7.5], on ? "Y" : "G")
  ruby(c, 5, 7, on)
})

// ------------------------------------------------------------------ toggle
// A gem socket: empty (dark glass) when off, a lit ruby with a glint when on.
anim("ui_toggle", { w: 13, h: 13, frames: 2, fps: 1 }, (c) => {
  const on = c.frame === 1, ring = ringOf(0, 0, 12, 12, 3)
  for (let y = 0; y < 13; y++) for (let x = 0; x < 13; x++) {
    const d = ring(x, y); if (d < 0) continue
    const lit = litSide(x, y, 0, 0, 12, 12)
    c.px(x, y, d === 0 ? "k" : d === 1 ? (lit ? "G" : "B") : d === 2 ? "k" : on ? (lit ? "r" : "R") : "z")
  }
  if (on) { c.px(4, 4, "w"); c.px(5, 4, "p"); c.px(4, 5, "p"); c.px(8, 8, "e") }
  else { c.px(4, 4, "s") }
})

// ------------------------------------------------------------------ tabs (3-slice, caps 6)
// 24x20. Frames: idle (sunken, dim gilt), lit (hover/focus) and active (raised, joined to the
// panel below: no bottom rim).
anim("ui_tab", { w: 24, h: 20, frames: 3, fps: 1 }, (c) => {
  const f = c.frame, active = f === 2, lit = f === 1
  const y0 = active ? 0 : 3, x0 = 0, x1 = 23, y1 = 19
  for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) {
    const dx = Math.min(x - x0, x1 - x), dy = y - y0
    if (dx + dy < 3) continue
    const edge = dx === 0 || dy === 0 || dx + dy === 3
    const rim = dx === 1 || dy === 1 || dx + dy === 4
    if (edge) c.px(x, y, "k")
    else if (rim) c.px(x, y, active ? (dy === 1 || x < 12 ? "Y" : "G") : lit ? "G" : "b")
    else c.px(x, y, active ? (dy < 8 ? "T" : "S") : lit ? "Q" : "P")
  }
  if (!active) for (let x = x0; x <= x1; x++) c.px(x, y1, "k")
})

// ------------------------------------------------------------------ focus cursor + spark
// A gilt arrowhead pointing at the focused item, its ruby flickering like an ember (4 frames).
anim("ui_cursor", { w: 8, h: 11, frames: 4, fps: 8 }, (c) => {
  c.poly([0, 0, 8, 5.5, 0, 11], "k")
  c.poly([1, 2, 6, 5.5, 1, 9], "G")
  c.line(1, 2, 5, 5, "Y")
  const glow = ["r", "p", "r", "R"][c.frame]
  c.px(2, 5, glow); c.px(3, 5, glow); c.px(2, 4, c.frame === 1 ? "w" : "r"); c.px(2, 6, "R")
})
// A four-pointed twinkle drifting off the focused item: grow, flash, fade.
anim("ui_spark", { w: 5, h: 5, frames: 4, fps: 10, loop: false }, (c) => {
  const f = c.frame
  if (f === 0) c.px(2, 2, "G")
  if (f === 1) { c.px(2, 2, "Y"); c.px(1, 2, "G"); c.px(3, 2, "G"); c.px(2, 1, "G"); c.px(2, 3, "G") }
  if (f === 2) { c.px(2, 2, "w"); for (const [x, y] of [[0, 2], [4, 2], [2, 0], [2, 4]]) c.px(x, y, "B");
    for (const [x, y] of [[1, 2], [3, 2], [2, 1], [2, 3]]) c.px(x, y, "Y") }
  if (f === 3) { c.px(2, 2, "G"); c.px(1, 2, "b"); c.px(3, 2, "b"); c.px(2, 1, "b"); c.px(2, 3, "b") }
})

// ------------------------------------------------------------------ blood (both skins)
// Fresh red blood the game throws on frames and buttons (clipped to them) as the run gets
// bloodier (`kit::blood`): splatters with spikes and thrown droplets, runs that pour from the top
// edge and end in a bead, red flecks, and drips that fall off the bottom once a box is soaked.
pal({
  K: "#1c0000", D: "#300000", d: "#440000", c: "#580000", L: "#6c0202", g: "#880808",
})
function rand(seed) {
  let a = seed >>> 0
  return () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}
function blob(cells, cx, cy, rx, ry, rng, rough = 0.6) {
  const p1 = rng() * 6, p2 = rng() * 6
  for (let y = Math.floor(cy - ry * 1.4 - 1); y <= cy + ry * 1.4 + 1; y++)
    for (let x = Math.floor(cx - rx * 1.4 - 1); x <= cx + rx * 1.4 + 1; x++) {
      const a = Math.atan2(y - cy, x - cx)
      const wob = 1 + rough * 0.18 * (Math.sin(a * 3 + p1) + Math.sin(a * 7 + p2))
      if (Math.hypot((x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry) <= wob) cells.add(x + "," + y)
    }
}
// A tapering stroke from (x0, y0) to (x1, y1), r0 thick at the start, r1 at the end.
function stroke(cells, x0, y0, x1, y1, r0, r1) {
  const n = Math.ceil(Math.hypot(x1 - x0, y1 - y0) * 2)
  for (let i = 0; i <= n; i++) {
    const u = i / n, x = x0 + (x1 - x0) * u, y = y0 + (y1 - y0) * u, r = r0 + (r1 - r0) * u
    for (let yy = Math.floor(y - r); yy <= y + r; yy++) for (let xx = Math.floor(x - r); xx <= x + r; xx++)
      if (Math.hypot(xx + 0.5 - x, yy + 0.5 - y) <= Math.max(r, 0.5)) cells.add(xx + "," + yy)
  }
}
// Paints cells as wet blood: a bright rim where it thins out, a darker clotted body mottled in
// clusters, a few wet highlights.
function paint(c, cells, rng) {
  const has = (x, y) => cells.has(x + "," + y)
  const depth = (x, y) => {
    for (let r = 1; r <= 4; r++)
      for (const [dx, dy] of [[r, 0], [-r, 0], [0, r], [0, -r]]) if (!has(x + dx, y + dy)) return r - 1
    return 4
  }
  const mott = new Map()
  const clump = (x, y) => {
    const k = ((x + (y >> 1)) >> 1) + "," + ((y + (x >> 2)) >> 1)
    if (!mott.has(k)) mott.set(k, rng())
    return mott.get(k)
  }
  for (const key of cells) {
    const [x, y] = key.split(",").map(Number)
    if (x < 0 || y < 0 || x >= c.w || y >= c.h) continue
    const d = depth(x, y), m = clump(x, y)
    let k
    if (d === 0) k = m < 0.5 ? "L" : "c"
    else if (d === 1) k = m < 0.3 ? "L" : "c"
    else k = m < 0.25 ? "D" : m < 0.6 ? "d" : "c"
    if (d >= 1 && !has(x - 1, y - 1) && rng() < 0.5) k = "g"
    c.px(x, y, k)
  }
}

// Splatters, 72x48: a body thrown at the wall, spikes radiating off it, droplets flung
// further out. Six of them.
const SPW = 72, SPH = 48
anim("ui_blood_splat", { w: SPW, h: SPH, frames: 6, fps: 1 }, (c) => {
  const rng = rand(4021 + c.frame * 7919), cells = new Set()
  const cx = 36 + (rng() - 0.5) * 6, cy = 24 + (rng() - 0.5) * 4
  const big = 9 + rng() * 4
  blob(cells, cx, cy, big, big * 0.75, rng, 1.2)
  // lobes make it splash, not a disc
  for (let k = 0; k < 3; k++) {
    const la = rng() * Math.PI * 2, r = big * (0.4 + rng() * 0.3)
    blob(cells, cx + Math.cos(la) * big * 0.9, cy + Math.sin(la) * big * 0.65, r, r * 0.8, rng, 0.8)
  }
  const spikes = 6 + Math.floor(rng() * 4)
  for (let k = 0; k < spikes; k++) {
    const a = (k / spikes) * Math.PI * 2 + rng() * 0.6
    const len = big + 3 + rng() * 11
    const ex = cx + Math.cos(a) * len, ey = cy + Math.sin(a) * len * 0.66
    stroke(cells, cx + Math.cos(a) * big * 0.6, cy + Math.sin(a) * big * 0.5, ex, ey, 2.2 + rng() * 1.2, 0.4)
    if (rng() < 0.6) blob(cells, ex + Math.cos(a) * 2.5, ey + Math.sin(a) * 2, 1 + rng() * 0.9, 1 + rng() * 0.8, rng, 0.2)
  }
  // flung droplets, more and smaller further out
  for (let k = 0; k < 26; k++) {
    const a = rng() * Math.PI * 2, r = big + 8 + rng() * 22
    const x = cx + Math.cos(a) * r, y = cy + Math.sin(a) * r * 0.62
    const s = r > big + 20 ? 0.6 : 0.6 + rng() * 1.1
    blob(cells, x, y, s, s, rng, 0)
  }
  paint(c, cells, rng)
})

// Runs, 12x64: blood pooled on the top edge pouring down in a thin stream that ends in a heavy
// bead, sometimes a drop fallen below it. Six lengths.
anim("ui_blood_run", { w: 12, h: 64, frames: 6, fps: 1 }, (c) => {
  const rng = rand(733 + c.frame * 104729), cells = new Set()
  const len = [22, 34, 46, 28, 54, 40][c.frame]
  const x = 6
  blob(cells, x, 0.5, 5.5, 2.2, rng, 1.2)
  blob(cells, x + (rng() - 0.5) * 3, 2.5, 2.6, 2.2, rng, 0.6)
  let xx = x
  for (let y = 3; y <= len; y++) {
    if (rng() < 0.1) xx += rng() < 0.5 ? -0.5 : 0.5
    const w = y < 8 ? 1.6 - (y - 3) * 0.1 : 0.8 + 0.35 * (Math.sin(y * 0.45 + c.frame) > 0.6 ? 1 : 0)
    stroke(cells, xx, y, xx, y + 1, w, w)
  }
  blob(cells, xx, len + 1, 1.9, 2.5, rng, 0.1)
  if (c.frame % 2 === 1) blob(cells, xx, len + 7, 1.2, 1.6, rng, 0)
  paint(c, cells, rng)
  // the bead catches the light
  c.px(Math.round(xx) - 1, len, "g")
})

// Flecks, 16x16: a few specks of blood, the fine spray that lands everywhere. Six patterns.
anim("ui_blood_fleck", { w: 16, h: 16, frames: 6, fps: 1 }, (c) => {
  const rng = rand(55 + c.frame * 31337)
  const n = 4 + Math.floor(rng() * 4)
  for (let k = 0; k < n; k++) {
    const x = Math.floor(1 + rng() * 13), y = Math.floor(1 + rng() * 13)
    const col = rng() < 0.5 ? "c" : "L"
    c.px(x, y, col)
    if (rng() < 0.45) { c.px(x + 1, y, "d"); c.px(x, y + 1, "d"); c.px(x + 1, y + 1, col) }
  }
})

// A drip: a bead swells under an edge, stretches, falls and is gone. 5x24, 12 frames.
anim("ui_blood_drip", { w: 5, h: 24, frames: 12, fps: 8 }, (c) => {
  const f = c.frame, cells = new Set(), still = () => 0.5
  if (f <= 5) {
    for (let y = 0; y <= 1 + f * 0.6; y++) cells.add("2," + y)
    blob(cells, 2.5, 1.6 + f * 0.7, 0.8 + f * 0.2, 0.9 + f * 0.25, still, 0)
  } else if (f <= 9) {
    const y = [7, 10, 15, 21][f - 6]
    cells.add("2,0"); cells.add("2,1")
    blob(cells, 2.5, y, 1.2, 1.5, still, 0); cells.add("2," + (y - 2))
  } else {
    cells.add("2,0"); cells.add("2,1")
  }
  for (const key of cells) {
    const [x, y] = key.split(",").map(Number)
    if (x < 0 || y < 0 || x >= c.w || y >= c.h) continue
    const edge = !cells.has((x - 1) + "," + y) || !cells.has((x + 1) + "," + y)
    c.px(x, y, edge ? "d" : "L")
  }
  if (f >= 3 && f <= 9) {
    const y = f <= 5 ? Math.round(1.6 + f * 0.7) : [7, 10, 15, 21][f - 6]
    c.px(1, y - 1, "g")
  }
})

// ------------------------------------------------------------------ stone skin pieces
// The Stone skin's frames are generated by `stone.rs`; these are its carved ornaments, in the
// same stone and hellfire ramps. Every glow is animated (4 frames) like the cracks.
pal({
  k: "#0a0608", "0": "#151014", "1": "#221b20", "2": "#30272b", "3": "#3f3538", "4": "#54484a",
  "5": "#6c5e5d", A: "#3a0a0a", B: "#7a1a12", C: "#c8401c", F: "#f08a2c", W: "#ffd27a",
})
const FIRE = ["A", "B", "C", "F", "W"]
const heat = (f, p, base) => FIRE[Math.max(0, Math.min(4, Math.round(base + 1.2 * Math.cos((f / 4 - p) * Math.PI * 2))))]

// Keystone: a horned skull carved in the stone, eyes burning; its horns rise and hook inward
// like Veyra's. 27x24 plus the ink outline, 4 frames.
anim("ui_skull", { w: 27, h: 24, frames: 4, fps: 6, outline: "k" }, (c) => {
  const f = c.frame, cx = 13, oy = 4
  const solid = new Set(), put = (x, y) => solid.add(x + "," + y)
  // cranium and jaw
  for (let y = 0; y < 24; y++) for (let x = 0; x < 27; x++) {
    const dx = (x - cx) / 6.6, dy = (y - (oy + 7)) / 6.2
    if (dx * dx + dy * dy <= 1) put(x, y)
    if (y >= oy + 10 && y <= oy + 15 && Math.abs(x - cx) <= 4 - (y >= oy + 14 ? 1 : 0)) put(x, y)
  }
  // horns: out from the temples, up, the tips hooking in
  for (const s of [-1, 1]) {
    for (let i = 0; i <= 16; i++) {
      const u = i / 16, r = 2.0 * (1 - u) + 0.45
      const x = cx + s * (5 + 6 * Math.sin(u * Math.PI * 0.55) - 2.5 * u * u * u)
      const y = oy + 6 - u * 10
      for (let yy = Math.floor(y - r); yy <= y + r; yy++) for (let xx = Math.floor(x - r); xx <= x + r; xx++)
        if (Math.hypot(xx - x, yy - y) <= r) put(xx, yy)
    }
  }
  const has = (x, y) => solid.has(x + "," + y)
  for (const key of solid) {
    const [x, y] = key.split(",").map(Number)
    if (x < 0 || x > 26 || y < 0 || y > 23) continue
    const lit = !has(x - 1, y - 1) || !has(x, y - 1) || !has(x - 1, y)
    const dark = !has(x + 1, y + 1) || !has(x + 1, y)
    const horn = y < oy + 4 || Math.abs(x - cx) > 6
    c.px(x, y, lit ? (horn ? "5" : "4") : dark ? "1" : horn ? "2" : "3")
  }
  // eye sockets with burning eyes, the nose and the teeth
  for (const s of [-1, 1]) {
    const ex = cx + s * 3, ey = oy + 7
    for (let y = ey - 1; y <= ey + 1; y++) for (let x = ex - 1; x <= ex + 1; x++) c.px(x, y, "k")
    c.px(ex - s * 2, ey, "k"); c.px(ex, ey + 2, "k")
    c.px(ex, ey, heat(f, s * 0.15, 3.2)); c.px(ex, ey + 1, heat(f, s * 0.15 + 0.3, 2.2))
  }
  c.px(cx, oy + 10, "k"); c.px(cx - 1, oy + 11, "k"); c.px(cx + 1, oy + 11, "k")
  for (let x = cx - 3; x <= cx + 3; x++) c.px(x, oy + 13, x % 2 ? "k" : "4")
  for (let x = cx - 2; x <= cx + 2; x++) c.px(x, oy + 14, "1")
  // a crack down the brow that smoulders
  for (const [x, y, p] of [[cx + 1, oy + 1, 0.1], [cx + 1, oy + 2, 0.2], [cx, oy + 3, 0.3], [cx, oy + 4, 0.4]]) c.px(x, y, heat(f, p, 1.6))
})
// The title rule's centre: a demonic eye in a carved diamond, the slit pupil in hellfire. 17x11.
anim("ui_eye", { w: 17, h: 11, frames: 4, fps: 6 }, (c) => {
  const f = c.frame
  c.poly([0, 5.5, 8.5, -1, 17, 5.5, 8.5, 12], "k")
  c.poly([1.5, 5.5, 8.5, 0.5, 15.5, 5.5, 8.5, 10.5], "3")
  c.poly([1.5, 5.5, 8.5, 0.5, 8.5, 5.5], "4")
  c.poly([8.5, 5.5, 15.5, 5.5, 8.5, 10.5], "1")
  // the eye: almond of fire, black slit
  for (let x = 4; x <= 12; x++) {
    const half = Math.round(2.2 * Math.sin(((x - 4) / 8) * Math.PI))
    for (let y = 5 - half; y <= 5 + half; y++) c.px(x, y, heat(f, (x - 8) * 0.06, 3.0 - Math.abs(y - 5) * 0.9))
    c.px(x, 5 - half - 1, "k"); c.px(x, 5 + half + 1, "k")
  }
  c.px(8, 4, "k"); c.px(8, 5, "k"); c.px(8, 6, "k")
})
// The rule's end: a carved spike. 9x7 (left end; the right is flipped).
sprite("ui_stone_rule_end", { w: 9, h: 7 }, `
  ....kkkkk
  ..kk44444
  kk4433333
  .kk222222
  ...kk1111
  .....kkkk
  .........`)
// Focus cursor: a black claw with a hellfire vein, pointing right. 9x11, 4 frames.
anim("ui_claw", { w: 9, h: 11, frames: 4, fps: 8 }, (c) => {
  const f = c.frame
  c.poly([0, 1, 4, 3, 9, 5.5, 4, 8, 0, 10], "k")
  c.poly([1, 2.5, 4, 4, 7.5, 5.5, 4, 7, 1, 8.5], "2")
  c.line(1, 3, 6, 5, "4")
  c.px(2, 5, heat(f, 0, 2.6)); c.px(3, 5, heat(f, 0.2, 3.2)); c.px(4, 5, heat(f, 0.4, 2.4)); c.px(5, 5, heat(f, 0.6, 1.4))
})
