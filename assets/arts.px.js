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
