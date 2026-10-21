// Arts (weapons) effects (PLAN.md 7.7, V15). Build:
//   node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/arts.px.js --out assets/sprites/arts
// Everything faces left like Veyra; the game flips by facing.

pal({
  k: "#16091a",                                            // outline, Veyra's plum ink
  R: "#5c0c1c", r: "#a8182e", q: "#e04454", Q: "#ff8a8a",  // tail crimson (+ hot edge)
  H: "#7a1238", h: "#d42a5c", i: "#ff6f9a", I: "#ffd0dc",  // heart pink
  x: "#c8301c", f: "#ff9a3c", F: "#ffe08a", X: "#6a1410",  // brimstone
})

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

// Kiss of Ruin: a heart bolt that pulses.
const HEART = `
  .hh.hh.
  hiIhhhh
  hiihhhh
  hhhhhhH
  .hhhhH.
  ..hhH..
  ...H...`
anim("heart_bolt", { outline: "k", fps: 8 }, HEART, frame(), frame(grid(1, 1, "iI\nii"), grid(5, 1, "i")))

// Brimstone Halo: one big flame (V17b), a teardrop of brimstone that licks and
// sways. 13x17; its ground point is the bottom centre.
const FW = 13, FH = 17
anim("ember", { w: FW, h: FH, frames: 4, fps: 10 }, (c) => {
  const sway = [0, 1, 0, -1][c.frame]
  const lick = [0, 1, 2, 1][c.frame]
  for (let y = 0; y < FH; y++) {
    const t = y / (FH - 1)                      // 0 at the tip, 1 at the base
    // Narrow tip, widest two thirds down, rounded base.
    const half = t < 0.7 ? 6 * Math.pow(t / 0.7, 0.8) : 6 * Math.sqrt(1 - Math.pow((t - 0.7) / 0.3, 2) * 0.55)
    const shift = Math.round(sway * (1 - t) * 1.5)
    for (let x = 0; x < FW; x++) {
      const d = Math.abs(x - 6 - shift)
      if (d > half + 0.25 || y < lick) continue
      const core = (d / Math.max(half, 0.5)) * 0.8 + (1 - t) * 0.6
      c.px(x, y, core > 1.05 ? "X" : core > 0.75 ? "x" : core > 0.45 ? "f" : "F")
    }
  }
  // Sparks breaking off the tip.
  if (c.frame === 1) c.px(8, 1, "f")
  if (c.frame === 3) { c.px(4, 0, "x"); c.px(3, 2, "f") }
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

// Chain of Torment: the spark where the chain bites into an enemy.
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

// Wandering Sigil: a ring of 4 runes turning around her mark (a horned heart), which
// stays upright. 17x17 at tier 0, 8 frames (a quarter turn: the runes repeat).
function mark(u, v) {
  // Horns: two short strokes rising out of the heart's lobes.
  for (const side of [-1, 1]) {
    const hx = u * side - 1.6, hy = v + 2.2
    if (hx >= 0 && hx <= 1.4 && Math.abs(hy + hx * 1.3) <= 0.6) return "h"
  }
  // The heart: (x² + y² - 1)³ ≤ x² y³, y up.
  const x = u / 2.6, y = -(v - 0.2) / 2.6
  const f = (x * x + y * y - 1) ** 3 - x * x * y ** 3
  if (f > 0) return null
  if (x < -0.15 && y > 0.25 && y < 0.75) return "I"
  if (x + y < -0.55) return "H"
  return x < 0 && y > 0 ? "i" : "h"
}
tiered("sigil", 7.2, { fps: 20, outline: "k" }, (c, size, scale) => {
  const m = (size - 1) / 2, turn = c.phase * Math.PI * 2 / 4
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const dx = (x - m) / scale, dy = (y - m) / scale, r = Math.hypot(dx, dy)
      if (r <= 7.2 && r >= 5.6) {
        const a = Math.atan2(dy, dx) - turn
        const rune = ((a / (Math.PI / 2)) % 1 + 1) % 1
        c.px(x, y, rune < 0.18 ? "I" : rune < 0.3 ? "i" : "H")
        continue
      }
      const key = mark(dx, dy)
      if (key) c.px(x, y, key)
    }
  }
})
