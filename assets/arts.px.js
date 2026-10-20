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

// Brimstone Halo: a burning ember that flickers.
anim("ember", { w: 7, h: 8, frames: 2, fps: 10 }, (c) => {
  c.circ(3, 4, 3, "X")
  c.circ(3, 4, 2, "x")
  c.circ(3, 4, 1, "f")
  c.px(3, 4, "F")
  if (c.frame === 0) { c.px(3, 1, "x"); c.px(3, 0, "f"); c.px(2, 2, "f") }
  else { c.px(4, 1, "x"); c.px(3, 2, "f"); c.px(4, 0, "x") }
})
