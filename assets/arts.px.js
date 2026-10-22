// Arts (weapons) effects (PLAN.md 7.7, V15). Build:
//   node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/arts.px.js --out assets/sprites/arts
// Everything faces left like Veyra; the game flips by facing.

pal({
  k: "#16091a",                                            // outline, Veyra's plum ink
  R: "#5c0c1c", r: "#a8182e", q: "#e04454", Q: "#ff8a8a",  // tail crimson (+ hot edge)
  H: "#7a1238", h: "#d42a5c", i: "#ff6f9a", I: "#ffd0dc",  // heart pink
  x: "#c8301c", f: "#ff9a3c", F: "#ffe08a", X: "#6a1410",  // brimstone
})

// Smooth value noise in 0..1, for the grain of fire (its tongues and crust).
function fnoise(x, y) {
  const h = (i, j) => { const n = Math.sin(i * 127.1 + j * 311.7) * 43758.5453; return n - Math.floor(n) }
  const i = Math.floor(x), j = Math.floor(y), fx = x - i, fy = y - j
  const sx = fx * fx * (3 - 2 * fx), sy = fy * fy * (3 - 2 * fy)
  const a = h(i, j) + (h(i + 1, j) - h(i, j)) * sx, b = h(i, j + 1) + (h(i + 1, j + 1) - h(i, j + 1)) * sx
  return a + (b - a) * sy
}

// Tail Lash: a crescent swept in front of her. 56x20; the near end (right) is at her side.
const LW = 56, LH = 20
function lash(c, stage) {
  for (let y = 0; y < LH; y++) {
    for (let x = 0; x < LW; x++) {
      const ox = (x - 27.5) / 27.5, oy = (y - 19) / 17   // outer edge
      const ix = (x - 27.5) / 27.5, iy = (y - 19) / 11   // inner edge: meets the outer at the tips
      const out = ox * ox + oy * oy, inn = ix * ix + iy * iy
      if (out > 1 || inn < 1) continue
      const depth = (1 - out) / (inn - out + 1e-6)   // 0 at the outer edge, 1 at the inner
      // The swing starts at her side (right) and reaches round to the left tip.
      if (stage === 0 && x < 30) continue
      if (stage === 2) {
        if (depth > 0.5 || (x + y) % 2) continue
        c.px(x, y, Math.abs(x - 27.5) > 18 ? "R" : "r")
        continue
      }
      const end = Math.abs(x - 27.5) > 22 ? "R" : null
      c.px(x, y, end ?? (depth < 0.25 ? (stage === 1 ? "Q" : "q") : depth < 0.55 ? "q" : depth < 0.8 ? "r" : "R"))
    }
  }
}
anim("tail_lash", { w: LW, h: LH, frames: 3, fps: 16 }, (c) => lash(c, c.frame))

// Barbed Scourge: the same swing, darker and wet, with hooked bone barbs raking out of
// its outer edge (they lean back against the swing, which runs right to left).
pal({ B: "#2a0510", o: "#ecdcc0", O: "#8e7462" })           // dried blood, bone barbs
function barbedLash(c, stage) {
  for (let y = 0; y < LH; y++) {
    for (let x = 0; x < LW; x++) {
      const ox = (x - 27.5) / 27.5, oy = (y - 19) / 17
      const iy = (y - 19) / 11
      const out = ox * ox + oy * oy, inn = ox * ox + iy * iy
      if (out > 1 || inn < 1) continue
      const depth = (1 - out) / (inn - out + 1e-6)
      if (stage === 0 && x < 30) continue
      if (stage === 2) {
        if (depth > 0.5 || (x + y) % 2) continue
        c.px(x, y, "B")
        continue
      }
      const end = Math.abs(x - 27.5) > 22 ? "B" : null
      c.px(x, y, end ?? (depth < 0.12 && stage === 1 ? "q" : depth < 0.3 ? "r" : depth < 0.6 ? "R" : "B"))
    }
  }
  // Barbs: thorns 2 px thick at the root on the outer edge, a bone hook 4 px out.
  for (let th = 0.42; th < 2.8; th += 0.34) {
    const ex = 27.5 + 27.4 * Math.cos(th), ey = 19 - 16.9 * Math.sin(th)
    if (stage === 0 && ex < 30) continue
    let nx = Math.cos(th) / 27.5, ny = -Math.sin(th) / 17
    const nl = Math.hypot(nx, ny); nx /= nl; ny /= nl
    const tx = -ny, ty = nx   // along the edge, back against the swing (rightwards)
    const put = (x, y, k) => {
      x = Math.round(x); y = Math.round(y)
      if (x >= 0 && y >= 0 && x < LW && y < LH) c.px(x, y, k)
    }
    for (let t = 0; t <= 4; t++) {
      if (stage === 2 && t % 2) continue
      const bx = ex + nx * t + tx * t * 0.5, by = ey + ny * t + ty * t * 0.5
      put(bx, by, t === 0 ? "B" : t === 4 ? "O" : "o")
      if (t < 2) put(bx - tx, by - ty, t === 0 ? "B" : "O")   // thick root, shaded side
    }
  }
}
anim("tail_lash_barbed", { w: LW, h: LH, frames: 3, fps: 16 }, (c) => barbedLash(c, c.frame))


// Brimstone Halo: a fireball (V17k, was a flame): a ball of molten brimstone, its
// swirls turning, crusted dark at the rim, flames licking up from its crown. 13x17;
// its ground point is the bottom centre.
const FW = 13, FH = 17
anim("ember", { w: FW, h: FH, frames: 4, fps: 10, outline: "X" }, (c) => {
  const cx = 6, cy = 10.5, R = 5.4, spin = c.frame * Math.PI / 2
  for (let y = 0; y < FH; y++) {
    for (let x = 0; x < FW; x++) {
      const u = x - cx, v = y - cy, r = Math.hypot(u, v)
      let heat = null
      if (r <= R + 0.2) {
        // Hot heart, cooler rim; swirls spiral round it as it rolls.
        const swirl = Math.sin(3 * Math.atan2(v, u) + r * 1.3 - spin)
        heat = 1.0 - r / R * 0.8 + swirl * 0.22 * (r / R)
        if (r > R - 1.1 && fnoise(x * 0.9, y * 0.9 + c.frame * 3) > 0.62) heat = 0.1   // crust
      } else if (v < 0) {
        // The crown: three tongues rising off the top, flickering.
        for (const [tx, base] of [[-2.5, 1.5], [0, 3], [2.5, 1.5]]) {
          const h = base + [0, 1, 2, 1][(c.frame + Math.round(tx) + 4) & 3]
          const k = (-v - Math.sqrt(Math.max(0, R * R - u * u))) / h   // 0 at the ball, 1 at the tip
          if (k >= 0 && k <= 1 && Math.abs(u - tx - Math.sin(k * 3 + spin) * 0.6) <= 1.3 * (1 - k)) heat = Math.max(heat ?? 0, 0.65 - k * 0.5)
        }
      }
      if (heat !== null) c.px(x, y, heat > 0.78 ? "F" : heat > 0.52 ? "f" : heat > 0.28 ? "x" : "X")
    }
  }
})

// Infernal Crown: the Halo's fireball crowned, burning white-hot to its heart in gold
// and white, a taller crown of five flames. 15x21; the ball's centre is (7, 14.5).
pal({ y: "#fffbe6", g: "#e8b04a", G: "#8a5a1c" })   // white heat, gold (as the trim)
anim("ember_crown", { w: 15, h: 21, frames: 4, fps: 10, outline: "X" }, (c) => {
  const cx = 7, cy = 14.5, R = 5.4, spin = c.frame * Math.PI / 2
  for (let y = 0; y < 21; y++) {
    for (let x = 0; x < 15; x++) {
      const u = x - cx, v = y - cy, r = Math.hypot(u, v)
      let heat = null
      if (r <= R + 0.2) {
        const swirl = Math.sin(3 * Math.atan2(v, u) + r * 1.3 - spin)
        heat = 1.08 - r / R * 0.7 + swirl * 0.18 * (r / R)
        if (r > R - 1.1 && fnoise(x * 0.9, y * 0.9 + c.frame * 3) > 0.7) heat = 0.3
      } else if (v < 0) {
        // The crown: five points, the middle one tallest, like a king's.
        for (const [tx, base] of [[-4.2, 2.5], [-2.2, 4], [0, 6], [2.2, 4], [4.2, 2.5]]) {
          const h = base + [0, 1, 2, 1][(c.frame + Math.round(tx) + 4) & 3]
          const k = (-v - Math.sqrt(Math.max(0, R * R - u * u))) / h
          if (k >= -0.2 && k <= 1 && Math.abs(u - tx - Math.sin(k * 3 + spin) * 0.5) <= 1.2 * (1 - k)) heat = Math.max(heat ?? 0, 0.85 - k * 0.5)
        }
      }
      if (heat !== null) c.px(x, y, heat > 0.9 ? "y" : heat > 0.68 ? "F" : heat > 0.48 ? "g" : heat > 0.3 ? "f" : "x")
    }
  }
})

// ---- V16 ------------------------------------------------------------------

// Allure: a small heart that beats. It rings the charm pulse and floats over the
// heads of charmed enemies.
const CHARM = `
  .h.h.
  hIhhh
  hihhh
  .hhH.
  ..H..`
anim("charm_heart", { outline: "k", fps: 6 }, CHARM, frame(), frame(grid(1, 1, "Ii\nii"), grid(3, 1, "i")))

// Hellgate (V17b): the pool itself is drawn by the game as an orange area of any size;
// these small flames burn inside it, each on its own beat. 7x10, standing on the
// bottom centre.
anim("gate_flame", { w: 7, h: 10, frames: 4, fps: 10 }, (c) => {
  const sway = [0, 1, 0, -1][c.frame]
  const tall = [10, 9, 8, 9][c.frame]
  for (let y = 10 - tall; y < 10; y++) {
    const t = (y - (10 - tall)) / (tall - 1)     // 0 at the tip, 1 at the base
    const half = t < 0.75 ? 3 * Math.pow(t / 0.75, 0.9) : 3 * Math.sqrt(1 - Math.pow((t - 0.75) / 0.25, 2) * 0.5)
    const shift = Math.round(sway * (1 - t))
    for (let x = 0; x < 7; x++) {
      const d = Math.abs(x - 3 - shift)
      if (d > half + 0.3) continue
      const core = (d / Math.max(half, 0.5)) * 0.8 + (1 - t) * 0.7
      c.px(x, y, core > 1.05 ? "X" : core > 0.75 ? "x" : core > 0.45 ? "f" : "F")
    }
  }
})

// Whip of Torment: the spark where the lash bites into an enemy.
anim("chain_spark", { w: 9, h: 9, frames: 2, fps: 16 }, (c) => {
  if (c.frame === 0) {
    c.hline(0, 4, 9, "q"); c.vline(4, 0, 9, "q")
    c.hline(2, 4, 5, "Q"); c.vline(4, 2, 5, "Q")
    c.px(4, 4, "I")
  } else {
    for (const [x, y] of [[2, 2], [6, 2], [2, 6], [6, 6]]) c.px(x, y, "q")
    for (const [x, y] of [[3, 3], [5, 3], [3, 5], [5, 5]]) c.px(x, y, "Q")
    c.px(4, 4, "I"); c.px(4, 3, "Q"); c.px(3, 4, "Q"); c.px(5, 4, "Q"); c.px(4, 5, "Q")
  }
})

pal({ W: "#2e0c2a", w: "#5c1c4a", v: "#7e2c64" })   // her wings (the Offering uses them)

// ---- V17 ------------------------------------------------------------------

// Abyssal Offering: a horned reliquary of dark plum and brimstone gold, dropped by
// elites. Veyra's sigil (a heart crowned with horns) pulses on its face.
pal({ g: "#e8b04a", G: "#8a5a1c" })                       // brimstone gold trim
anim("offering", { w: 11, h: 12, frames: 4, fps: 6, outline: "k" }, (c) => {
  const sym = (x, y, col) => { c.px(x, y, col); c.px(10 - x, y, col) }
  sym(1, 0, "G"); sym(1, 1, "W"); sym(2, 1, "G")          // horns on the lid
  c.hline(2, 2, 7, "G")
  c.hline(1, 3, 9, "W"); sym(1, 3, "G")                   // lid
  c.hline(0, 4, 11, "g"); sym(0, 4, "G")                  // gold band
  c.rect(0, 5, 11, 6, "w")                                // body
  c.vline(1, 5, 6, "v"); c.vline(0, 5, 6, "W"); c.vline(10, 5, 6, "W")
  c.hline(0, 10, 11, "W")
  sym(0, 11, "G"); sym(1, 11, "G")                        // clawed feet
  // The sigil: dim, lit, blazing, lit.
  const [rim, core, hot] = [["R", "r", "q"], ["r", "q", "Q"], ["q", "Q", "I"], ["r", "q", "Q"]][c.frame]
  sym(3, 5, rim); sym(4, 6, rim)                          // its horns
  sym(4, 7, core); c.px(5, 7, hot)                        // heart
  sym(4, 8, rim); c.px(5, 8, core); c.px(5, 9, rim)
  if (c.frame === 2) { sym(3, 7, "r"); c.px(5, 6, "q") }  // the glow spills over
})

// ---- V17d -----------------------------------------------------------------

pal({ S: "#6e6680", s: "#b8b0c8", e: "#f4eefc" })          // scythe steel (+ edge glint)

// Draws `shape(u, v)` (centre-relative coordinates, y down) turned by `angle` and
// grown by `scale`, sampled at pixel centres, so every frame of a spin is crisp at
// every size.
function spun(c, size, angle, scale, shape) {
  const m = (size - 1) / 2, cos = Math.cos(-angle), sin = Math.sin(-angle)
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const dx = (x - m) / scale, dy = (y - m) / scale
      const key = shape(dx * cos - dy * sin, dx * sin + dy * cos)
      if (key) c.px(x, y, key)
    }
  }
}

// Area grows these Arts: each is baked at SIZE_TIERS sizes (`<name>_<tier>`, tier k
// is 1 + k/4 times the base), so the game never scales a sprite. `reach` is the
// shape's half size at scale 1; the outline adds a pixel around.
const SIZE_TIERS = 6
function tiered(name, reach, opts, draw) {
  for (let k = 0; k < SIZE_TIERS; k++) {
    const scale = 1 + k / 4, size = 2 * Math.ceil(reach * scale) + 1
    anim(`${name}_${k}`, { w: size, h: size, frames: 8, ...opts }, (c) => draw(c, size, scale))
  }
}

// Reaper's Scythe: a plum shaft with a crescent of pale steel, spinning (8 turns of
// 45 degrees). 21x21 at tier 0.
function scythe(u, v) {
  // Along the shaft (t, its top end at +7.5) and across it (b, the blade's side).
  const t = (u - v) / Math.SQRT2, b = (-u - v) / Math.SQRT2
  // The blade: a crescent off the top end, thick at the shaft and curling to a point.
  const outer = (t / 7.5) ** 2 + (b / 10) ** 2, inner = ((t + 2) / 7) ** 2 + (b / 10.5) ** 2
  if (b >= 0 && t > -1 && outer <= 1 && inner > 1) {
    if (inner < 1.25) return "e"                      // the cutting edge, inside the curve
    return outer > 0.8 ? "S" : "s"
  }
  if (Math.abs(b) <= 0.75 && t >= -8 && t <= 7.5) return t > 6 ? "G" : "w"
  return null
}
tiered("scythe", 9, { fps: 20, outline: "k" }, (c, size, scale) =>
  spun(c, size, c.phase * Math.PI * 2, scale, scythe))

// Midnight Harvest: the same scythe forged of night, a blade of midnight blue whose
// edge shines like the moon, on a violet shaft tipped in moonlight.
pal({ N: "#141c48", n: "#2e44a0", l: "#dcecff", p: "#5a3a8a" })
const NIGHT = { S: "N", s: "n", e: "l", w: "p", G: "l" }
tiered("scythe_harvest", 9, { fps: 20, outline: "k" }, (c, size, scale) =>
  spun(c, size, c.phase * Math.PI * 2, scale, (u, v) => NIGHT[scythe(u, v)] ?? null))

// Profaned Cross: the Church's cross turned upside down, tarnished gold with a
// crimson heart, spinning. 23x23 at tier 0. The burning one (Heretic's Brand) is the
// same cross wreathed in brimstone.
const CROSS_SCALE = 1.2
function cross(u, v) {
  const bar = Math.abs(v - 2.5) <= 1.0 && Math.abs(u) <= 4.5
  const post = Math.abs(u) <= 1.0 && v >= -6.5 && v <= 5.5
  if (!bar && !post) return null
  if (Math.abs(u) < 0.6 && Math.abs(v - 2.5) < 0.6) return "q"
  return u + v < 0 ? "g" : "G"
}
tiered("cross", 8 * CROSS_SCALE, { fps: 25, outline: "k" }, (c, size, scale) =>
  spun(c, size, c.phase * Math.PI * 2, scale * CROSS_SCALE, cross))
tiered("cross_burning", 8 * CROSS_SCALE, { fps: 25, outline: "X" }, (c, size, scale) => {
  const flick = c.frame % 2
  spun(c, size, c.phase * Math.PI * 2, scale * CROSS_SCALE, (u, v) => {
    const key = cross(u, v)
    if (key) return key === "q" ? "F" : key === "g" ? "f" : "x"
    // A tongue of flame licks off each arm.
    const bar = Math.abs(v - 2.5) <= 1.6 + flick * 0.4 && Math.abs(u) <= 5.6
    const post = Math.abs(u) <= 1.6 + flick * 0.4 && v >= -7.4 && v <= 6.4
    return bar || post ? "x" : null
  })
})

// Wandering Sigil: Lilith's glyph (a crescent, horns up, over a cross) inside a ring
// whose 4 runes turn around it; the glyph stays upright. 17x17 at tier 0, 8 frames (a
// quarter turn: the runes repeat). The icon (`icons.px.js`) is the same seal.
function lilith(u, v) {
  // The crescent: the lower half of a ring centred 3 above the middle, and its horn tips.
  const d = Math.hypot(u, v + 3)
  if (v >= -4 && d <= 3.6 && d > 2.4) return "I"
  if (Math.abs(Math.abs(u) - 3) <= 0.5 && Math.abs(v + 5) <= 0.5) return "i"
  if (Math.abs(u) <= 0.5 && Math.abs(v - 1) <= 0.5) return "e"       // where they meet
  if (Math.abs(u) <= 0.5 && v >= 1 && v <= 5.5) return "I"           // the stem
  if (Math.abs(v - 3) <= 0.5 && Math.abs(u) <= 2.5) return "I"       // the bar
  return null
}
tiered("sigil", 7.2, { fps: 20, outline: "k" }, (c, size, scale) => {
  const m = (size - 1) / 2, turn = c.phase * Math.PI / 2, band = Math.round(scale)
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const d = Math.hypot(x - m, y - m)
      // A rune every quarter, turning: `off` is how far round the ring (in pixels).
      const a = Math.atan2(y - m, x - m) - turn
      const off = m * Math.abs((((a + Math.PI / 4) % (Math.PI / 2)) + Math.PI / 2) % (Math.PI / 2) - Math.PI / 4)
      if (d <= m + 0.5 && d > m + 0.5 - band) {
        c.px(x, y, off < 0.6 * scale ? "e" : off < 1.6 * scale ? "I" : "i")
        continue
      }
      if (d <= m + 0.5 - band && d > m + 0.5 - 2 * band) {
        c.px(x, y, off < 0.6 * scale ? "I" : "H")     // the rune bites into the ring
        continue
      }
      const key = lilith((x - m) / scale, (y - m) / scale)
      if (key) c.px(x, y, key)
    }
  }
})

// Broken Seal (the Sigil evolved): the same seal cracked open and running hot, gold
// and brimstone instead of pink; its cracks throb brighter and dimmer as if about to
// burst (it does, on every bounce), and a chip is missing from its ring.
function segDist(u, v, [x0, y0], [x1, y1]) {
  const dx = x1 - x0, dy = y1 - y0
  const t = Math.max(0, Math.min(1, ((u - x0) * dx + (v - y0) * dy) / (dx * dx + dy * dy)))
  return Math.hypot(u - x0 - t * dx, v - y0 - t * dy)
}
const CRACKS = [
  [[5.6, -5.6], [3, -3], [3.6, -0.6], [1, 1.6]],
  [[-7.4, 1.5], [-4.6, 0.4], [-3.4, 2.8]],
  [[-1.6, 7.2], [-0.6, 4.6]],
]
function cracked(u, v, width) {
  return CRACKS.some((line) => line.slice(1).some((b, i) => segDist(u, v, line[i], b) <= width))
}
tiered("sigil_broken", 7.2, { fps: 20, outline: "k" }, (c, size, scale) => {
  const m = (size - 1) / 2, turn = c.phase * Math.PI / 2, band = Math.round(scale)
  const glow = ["x", "f", "F", "e", "e", "F", "f", "x"][c.frame]   // the throb of the cracks
  const crackW = 0.45 + 0.1 * (scale - 1)
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const d = Math.hypot(x - m, y - m), u = (x - m) / scale, v = (y - m) / scale
      const ang = Math.atan2(y - m, x - m)
      // The chip knocked out of the ring, up and to the right.
      if (d > m + 0.5 - 2 * band && Math.abs(ang + Math.PI / 4) < 0.22) continue
      if (cracked(u, v, crackW)) { c.px(x, y, glow); continue }
      const a = ang - turn
      const off = m * Math.abs((((a + Math.PI / 4) % (Math.PI / 2)) + Math.PI / 2) % (Math.PI / 2) - Math.PI / 4)
      if (d <= m + 0.5 && d > m + 0.5 - band) {
        c.px(x, y, off < 0.6 * scale ? "e" : off < 1.6 * scale ? "F" : "g")
        continue
      }
      if (d <= m + 0.5 - band && d > m + 0.5 - 2 * band) {
        c.px(x, y, off < 0.6 * scale ? "F" : "G")
        continue
      }
      const key = lilith(u, v)
      if (key) c.px(x, y, key === "e" ? "e" : key === "i" ? "f" : "F")
    }
  }
})

// Broken Seal's burst where a sigil bounces: a white-hot flash, rays of gold and
// pink, shards of the seal flung out, then embers. 33x33, 6 frames over 0.4 s. The
// game adds the shockwave ring at the blast's true radius.
anim("seal_burst", { w: 33, h: 33, frames: 6, fps: 15 }, (c) => {
  const m = 16, f = c.frame
  const ray = (len, width, key, rot) => {
    for (let k = 0; k < 8; k++) {
      const a = rot + k * Math.PI / 4
      for (let r = 0; r <= len; r += 0.5) {
        const w = width * (1 - r / (len + 1))
        for (let s = -w; s <= w; s += 0.5) {
          c.px(Math.round(m + Math.cos(a) * r - Math.sin(a) * s), Math.round(m + Math.sin(a) * r + Math.cos(a) * s), key)
        }
      }
    }
  }
  const disc = (r, key) => c.circ(m, m, r, key)
  if (f === 0) { disc(5, "f"); disc(4, "F"); disc(2, "e") }
  if (f === 1) { ray(12, 1.6, "f", 0); ray(9, 1.0, "F", 0); ray(7, 1.2, "i", Math.PI / 8); disc(5, "F"); disc(3, "e") }
  if (f === 2) { ray(15, 1.2, "x", 0); ray(12, 0.8, "f", 0); ray(10, 0.8, "h", Math.PI / 8); c.ring(m, m, 6, "F"); disc(3, "f") }
  // Shards: little pieces of the seal's ring flying out and spinning.
  if (f >= 2) {
    for (let k = 0; k < 6; k++) {
      const a = k * Math.PI / 3 + 0.4, r = 5 + f * 2.6
      const x = Math.round(m + Math.cos(a) * r), y = Math.round(m + Math.sin(a) * r - (f - 2) * 0.6)
      c.rect(x, y, 2, 2, f < 5 ? (k % 2 ? "i" : "g") : "G")
    }
  }
  if (f === 3) { c.ring(m, m, 9, "x"); c.ring(m, m, 8, "f"); disc(2, "x") }
  // Embers drifting up as it dies down.
  if (f >= 3) {
    for (let k = 0; k < 7; k++) {
      const a = k * 2.4, r = 4 + (k % 3) * 3
      c.rect(Math.round(m + Math.cos(a) * r), Math.round(m + Math.sin(a) * r * 0.6 - (f - 2) * 2), 2, 2, f < 5 ? "f" : "x")
    }
  }
})

// ---- V17e -----------------------------------------------------------------

pal({ m: "#b04a8a" })                                      // wing rim catching the light

// Bat Swarm: a plum bat with red eyes, flapping (up, level, down, level). 15x9. The
// Crimson Host's bats are the same flock in blood red with golden eyes.
const BAT_UP = `
  .m...........m.
  .vw.........wv.
  ..vw..W.W..wv..
  ..wvw.WWW.wvw..
  ...wwwqWqwww...
  ......WWW......
  .......W.......
  ...............
  ...............`
const BAT_MID = `
  ...............
  ...............
  ......W.W......
  .mm...WWW...mm.
  mvvwwwqWqwwwvvm
  .wvwvwWWWwvwvw.
  ..w.w..W..w.w..
  ...............
  ...............`
const BAT_DOWN = `
  ...............
  ...............
  ......W.W......
  ......WWW......
  ....mmqWqmm....
  ...mvwWWWwvm...
  ..wvw..W..wvw..
  .wvw.......wvw.
  .w...........w.`
function batAnim(name, swap) {
  const g = (s) => grid(0, 0, s.replace(/[a-zA-Z]/g, (k) => swap[k] ?? k))
  anim(name, { w: 15, h: 9, fps: 12, outline: "k" },
    frame(g(BAT_UP)), frame(g(BAT_MID)), frame(g(BAT_DOWN)), frame(g(BAT_MID)))
}
batAnim("bat", {})
batAnim("bat_host", { W: "R", w: "r", v: "q", m: "Q", q: "F" })

// A blood bomb the bats drop: a crimson orb with a glint that winks.
anim("blood_bomb", { outline: "k", fps: 8 }, `
  .rr.
  rQqr
  rqrR
  .RR.`, frame(), frame(grid(1, 1, "qQ")))

// Black Mass: the ritual circle under her, seen on the ground (an ellipse twice as
// wide as deep): two rings with runes between them around an inverted pentagram (one
// point toward the viewer). 141x71, 4 frames: the runes flicker round the ring.
// Witches' Sabbath draws it in brimstone gold with pink runes.
const MASS_RX = 70, MASS_RY = 35
function massCircle(c, line, dark, rune, hot) {
  const cx = MASS_RX, cy = MASS_RY
  // The star's points on the inner ring, projected onto the ground.
  const pts = []
  for (let k = 0; k < 5; k++) {
    const a = Math.PI / 2 + k * 2 * Math.PI / 5           // first point down (toward us)
    pts.push([cx + Math.cos(a) * (MASS_RX - 12), cy + Math.sin(a) * (MASS_RY - 6)])
  }
  const segDist = (x, y, [ax, ay], [bx, by]) => {
    const dx = bx - ax, dy = by - ay
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)))
    return Math.hypot(x - ax - t * dx, y - ay - t * dy)
  }
  for (let y = 0; y <= 2 * MASS_RY; y++) {
    for (let x = 0; x <= 2 * MASS_RX; x++) {
      const e = (rx, ry) => Math.hypot((x - cx) / rx, (y - cy) / ry)
      if (e(MASS_RX, MASS_RY) <= 1 && e(MASS_RX - 2, MASS_RY - 1.2) > 1) { c.px(x, y, line); continue }
      if (e(MASS_RX - 12, MASS_RY - 6) <= 1 && e(MASS_RX - 13.5, MASS_RY - 6.8) > 1) { c.px(x, y, dark); continue }
      // Runes in the band between the rings: short strokes, one lit at a time.
      const band = e(MASS_RX - 4, MASS_RY - 2) <= 1 && e(MASS_RX - 10, MASS_RY - 5) > 1
      if (band) {
        const a = (Math.atan2((y - cy) / MASS_RY, (x - cx) / MASS_RX) / (2 * Math.PI) + 1) % 1
        const slot = Math.floor(a * 20), within = a * 20 - slot
        if (within > 0.35 && within < 0.65 && (slot + y) % 3 !== 0) {
          c.px(x, y, slot % 4 === c.frame ? hot : rune)
        }
        continue
      }
      for (let k = 0; k < 5; k++) {
        if (segDist(x, y, pts[k], pts[(k + 2) % 5]) <= 0.72) { c.px(x, y, line); break }
      }
    }
  }
  for (const [px, py] of pts) c.px(Math.round(px), Math.round(py), hot)
}
anim("black_mass", { w: 2 * MASS_RX + 1, h: 2 * MASS_RY + 1, frames: 4, fps: 8 },
  (c) => massCircle(c, "q", "r", "R", "Q"))
anim("sabbath", { w: 2 * MASS_RX + 1, h: 2 * MASS_RY + 1, frames: 4, fps: 8 },
  (c) => massCircle(c, "g", "G", "i", "I"))

// Evil Eye: a golden eye that floats over every frozen foe, its iris smouldering.
anim("evil_eye", { outline: "k", fps: 4 }, `
  ..GGG..
  .GIqIG.
  GIqkqIG
  .GIqIG.
  ..GGG..`, frame(), frame(grid(2, 2, "Q.Q")))

// ---- V17k: flames in flight -----------------------------------------------

// Hellflare flames and Kiss of Ruin hearts fly head first, a tail of fire streaming
// behind. 9 headings for the right half (straight up to straight down, 22.5° apart;
// the game mirrors them for the left), 4 flickers each: frame = heading * 4 + flicker.
const HEADINGS = 9, FLICKERS = 4
function flying(c, size, len, width, head, keys) {
  const m = (size - 1) / 2
  const d = Math.floor(c.frame / FLICKERS), flick = c.frame % FLICKERS
  const a = -Math.PI / 2 + d * Math.PI / (HEADINGS - 1)
  const dx = Math.cos(a), dy = Math.sin(a)
  const key = (heat) => heat > 0.74 ? keys[3] : heat > 0.5 ? keys[2] : heat > 0.27 ? keys[1] : keys[0]
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const u = x - m, v = y - m
      const along = u * dx + v * dy, across = -u * dy + v * dx
      let heat = head(u, v, along, across, flick)
      // The tail: a long middle tongue and two shorter ones splaying out, each
      // narrowing to a point and licking from side to side as it flickers.
      for (const [spread, reach, phase] of [[0, 1, 0], [1, 0.6, 1.7], [-1, 0.6, 3.4]]) {
        const l = len * reach
        if (along >= 0.5 || along <= -l) continue
        const t = Math.max(0, -along) / l
        const wob = spread * t * width * 0.8 + Math.sin(t * 5 - flick * Math.PI / 2 + phase) * 1.1 * t
        const w = (spread ? 0.5 : 1) * width * (1 - t) ** 0.8 + 0.3
        const off = Math.abs(across - wob)
        if (off > w) continue
        const grain = fnoise(-along * 0.7 + flick * 1.6, across * 1.1 + 5)
        const fire = (1 - t) * (1 - 0.6 * off / w) * (spread ? 0.8 : 0.9) + (grain - 0.5) * 0.4
        heat = Math.max(heat ?? 0, fire)
      }
      if (heat !== null && heat > 0) c.px(x, y, key(heat))
    }
  }
  // Sparks shed behind it, a different pair each flicker.
  for (const [k, side] of [[len + 1.5 + flick % 2, flick < 2 ? 1 : -1], [len - 1 + (flick + 1) % 3, flick % 2 ? 2 : -2]]) {
    const x = Math.round(m - dx * k - dy * side), y = Math.round(m - dy * k + dx * side)
    if (x >= 0 && y >= 0 && x < size && y < size) c.px(x, y, keys[2])
  }
}
// Hellflare: a tongue of brimstone, its round head leading.
anim("fireball", { w: 25, h: 25, frames: HEADINGS * FLICKERS, fps: 12, outline: "X" }, (c) =>
  flying(c, 25, 12, 3.0, (u, v, along, across) => {
    const r = Math.hypot(along - 0.8, across)
    return r <= 3.3 ? 1.05 - r / 4.2 : null
  }, "XxfF"))
// Kiss of Ruin: an upright heart of pink fire, white-hot inside, trailing flame.
anim("heart_bolt", { w: 23, h: 23, frames: HEADINGS * FLICKERS, fps: 12, outline: "k" }, (c) =>
  flying(c, 23, 10, 2.8, (u, v, along, across, flick) => {
    const x = u / 4.2, y = -(v - 0.5) / 4.2
    const f = (x * x + y * y - 1) ** 3 - x * x * y ** 3
    if (f > 0) return null
    return 0.3 + Math.min(0.6, -f * 2.2) + (fnoise(u * 0.8 + flick * 2, v * 0.8) - 0.5) * 0.3
  }, "HhiI"))
