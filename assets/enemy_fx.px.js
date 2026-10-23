// Ashford enemy effects (V18): sling stone, thrown torch, burning patch, zealot blast, soul tiers,
// curate blessing. Not flipped, no flash copies; anims are equal-width horizontal strips.
// Same palette rule as enemies.px.js: enemy fire is dull ochre / pale yellow, never Veyra's
// crimson or magenta; the blessing is a cold white-blue (gold is hers).

pal({
  k: "#10131c",                                            // outline, blue-black
  B: "#55535f", C: "#7d7b88", D: "#a6a4ae", j: "#c8d2da",  // stone greys
  W: "#4a3a2a", w: "#6e5a44", e: "#8a7458",                // torch wood
  K: "#7a5e34", y: "#b8924a", Z: "#e8dca4", Y: "#f6f0d0",  // dull fire: ochre to pale core
  a: "#2a2622",                                            // scorched earth
  g: "#8a8a88", G: "#6a6a6c", h: "#b4b4b0",                // smoke
  s: "#60a8e8", S: "#c4ecff", T: "#f2fbff",                // soul (soul.png hues)
  f: "#62d896", F: "#dcfae6",                              // faint soul: green
  R: "#e0a030", V: "#ffe08a", X: "#fffaea",                // radiant soul: gold
  H: "#e0a03060", I: "#e0a03030",                          // radiant halo
  b: "#a8cce8", c: "#eaf6ff",                              // blessing: cold white-blue
  o: "#b8924a70",                                          // fire glow on the ground
  m: "#8a8a8890", n: "#b4b4b080",                          // thin smoke
})
ramps("kBCDj", "kWwe", "kKyZY", "kGgh")

// ------------------------------------------------------------------ stone (5x5)
sprite("stone", { outline: "k" }, `
  jj.
  jDC
  DCB`)

// ------------------------------------------------------------------ torch_spin (9x9, 4 frames)
// A torch tumbling end over end: the same 7x7 drawing turned 90 degrees clockwise per frame.
const TORCH = [
  "....Zy.",
  "...yZZy",
  "...KyyK",
  "...w...",
  "..w....",
  ".W.....",
  "W......",
]
function rotCW(rows, n) {
  let r = rows
  for (let i = 0; i < n; i++) {
    const h = r.length
    r = r[0].split("").map((_, x) => r.map((_, y) => r[h - 1 - y][x]).join(""))
  }
  return r.join("\n")
}
anim("torch_spin", { w: 7, h: 7, outline: "k", fps: 12 },
  frame(grid(0, 0, rotCW(TORCH, 0))), frame(grid(0, 0, rotCW(TORCH, 1))),
  frame(grid(0, 0, rotCW(TORCH, 2))), frame(grid(0, 0, rotCW(TORCH, 3))))

// ------------------------------------------------------------------ fire_patch (26x12, 3 frames)
// Burning oil on the ground seen at an angle: a scorched ellipse, flame tongues that shift per
// frame, smoke wisps. No outline; the rim is broken so the edge feels soft.
const TONGUES = [
  [[5, 4, -1], [9, 6, 1], [13, 8, 0], [17, 5, -1], [21, 3, 1]],
  [[6, 5, 1], [10, 4, -1], [14, 7, 1], [18, 6, 0], [20, 3, -1]],
  [[4, 3, 0], [8, 5, -1], [12, 6, 1], [16, 8, -1], [21, 4, 0]],
]
anim("fire_patch", { w: 26, h: 12, fps: 8, frames: 3 }, (c) => {
  const f = c.frame
  c.ellipse(13, 9, 12, 2, "o")                 // soft glow on the ground
  c.ellipse(13, 9, 10, 1, "K")
  for (const [x, h, lean] of TONGUES[f]) {
    c.poly([x - 2, 10, x + 2, 10, x + lean, 10 - h], "y")
    if (h > 3) c.poly([x - 1, 10, x + 1, 10, x, 12 - h], "Z")
    c.px(x, 10, "Y")
  }
  // smoke wisps over the tallest tongues, drifting with the frame
  const smoke = [[[13, 1], [14, 0], [9, 3]], [[14, 2], [15, 1], [10, 5]], [[16, 1], [17, 0], [12, 4]]][f]
  for (const [x, y] of smoke) c.px(x, y, "m")
  c.px(1, 9, "_"); c.px(25, 9, "_")
})

// ------------------------------------------------------------------ zealot_blast (44x28, 5 frames)
// Oil explosion, played once: pale flash -> lumpy ochre fireball -> grey smoke ring that breaks up.
function puffs(c, cx, cy, n, rx, ry, r, col, seed, out = 0) {
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2 + seed
    const k = 0.75 + 0.25 * Math.sin(i * 2.3 + seed * 5)
    c.circ(Math.round(cx + Math.cos(a) * rx * k), Math.round(cy + Math.sin(a) * ry * k - out), Math.max(1, Math.round(r * k)), col)
  }
}
anim("zealot_blast", { w: 44, h: 28, fps: 14, frames: 5, loop: false }, (c) => {
  const cx = 22, cy = 15
  switch (c.frame) {
    case 0: // flash
      c.ellipse(cx, cy, 9, 6, "y")
      c.line(cx - 14, cy, cx + 14, cy, "Z"); c.line(cx, cy - 10, cx, cy + 8, "Z")
      c.line(cx - 8, cy - 6, cx + 8, cy + 5, "Z"); c.line(cx + 8, cy - 6, cx - 8, cy + 5, "Z")
      c.ellipse(cx, cy, 7, 5, "Z"); c.ellipse(cx, cy, 4, 3, "Y")
      break
    case 1: // fireball
      puffs(c, cx, cy, 8, 8, 6, 5, "K", 0.3)
      puffs(c, cx, cy - 1, 8, 7, 5, 4, "y", 0.3)
      c.ellipse(cx, cy - 1, 7, 5, "y"); c.ellipse(cx, cy - 2, 5, 3, "Z"); c.ellipse(cx - 1, cy - 2, 2, 1, "Y")
      for (const [x, y] of [[5, 14], [39, 13], [9, 5], [35, 6], [23, 2], [14, 25], [31, 24]]) c.px(x, y, "Z")
      break
    case 2: // fireball wrapped in smoke
      puffs(c, cx, cy, 10, 14, 8, 5, "G", 1.1)
      puffs(c, cx, cy - 1, 10, 13, 7, 4, "g", 1.1)
      puffs(c, cx, cy - 1, 8, 8, 5, 4, "K", 0.5)
      c.ellipse(cx, cy - 2, 8, 5, "y"); c.ellipse(cx, cy - 3, 4, 2, "Z")
      for (const [x, y] of [[1, 12], [42, 16], [5, 3], [38, 3], [22, 0], [11, 26], [33, 26]]) c.px(x, y, "y")
      break
    case 3: // smoke ring, embers
      puffs(c, cx, cy, 12, 16, 9, 4, "G", 0.2, 1)
      puffs(c, cx, cy - 1, 12, 16, 9, 3, "g", 0.2, 1)
      puffs(c, cx, cy - 2, 12, 15, 8, 2, "h", 0.2, 1)
      c.ellipse(cx, cy - 1, 11, 5, "_")
      c.ellipse(cx, cy, 4, 2, "m")
      for (const [x, y] of [[9, 9], [35, 10], [15, 22], [29, 21], [22, 12]]) c.px(x, y, "y")
      break
    case 4: // fading
      puffs(c, cx, cy - 2, 12, 18, 10, 3, "m", 0.2, 1)
      puffs(c, cx, cy - 3, 12, 18, 10, 2, "n", 0.2, 1)
      c.ellipse(cx, cy - 2, 14, 7, "_")
      break
  }
})

// ------------------------------------------------------------------ souls
// soul.png (8x8, blue) is the middle tier. The colour tells the worth: faint is smaller and
// green, radiant is bigger and gold, with a white core and a halo.
sprite("soul_faint", { w: 6, h: 6 }, `
  ..f...
  .fFf..
  .fFf..
  ..f...`)

anim("soul_radiant", { w: 11, h: 13, fps: 3, frames: 2 }, (c) => {
  const big = c.frame === 1
  c.ellipse(5, 5, big ? 5 : 4, big ? 6 : 5, "I")
  c.ellipse(5, 5, big ? 4 : 3, big ? 5 : 4, "H")
  c.poly([5, 0, 9, 5, 5, 10, 1, 5], "R")
  c.poly([5, 2, 7, 5, 5, 8, 3, 5], "V")
  c.vline(5, 3, 5, "X"); c.hline(4, 5, 3, "X")
  if (big) { c.px(5, 0, "V"); c.px(9, 5, "V"); c.px(1, 5, "V") }
})

// ------------------------------------------------------------------ bless (5x7, 2 frames)
// Small cold cross floating over a healed enemy; it twinkles and rises a pixel.
anim("bless", { w: 5, h: 7, fps: 4 },
  frame(grid(0, 1, "..b..\n..c..\nbcccb\n..c..\n..c..\n..b..")),
  frame(grid(0, 0, "b.c.b\n..c..\nccccc\n..c..\n..c..\n..b..")))
