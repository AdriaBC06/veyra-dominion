// Ashford village props (V19d, design.md 7.14). Top-down 3/4 view like the ground: we see the
// front slope of every roof and the front wall below it. Sun from the upper left.
// Palette rule (7.6): muted, cold-leaning earth and stone so Veyra's reds and golds stay hers.
// Villagers are 20 px tall, so a door is ~13 px and a cottage ~3 villagers wide.

pal({
  k: "#10131c",                                            // outline, blue-black
  // plaster and timber (wattle and daub, Tudor frame)
  Q: "#5e5a4e", P: "#7a7464", p: "#8e8774",
  W: "#2c241e", w: "#463829", x: "#5e4c38",
  // thatch
  H: "#3e3220", T: "#5a4a2e", t: "#76623c", u: "#8e7a4c",
  // slate roof
  L: "#262c38", S: "#38404e", s: "#4c5566", v: "#636d80",
  // stone
  A: "#3a3a44", B: "#55535f", C: "#716f7c", D: "#8e8c98",
  // glass, iron, door
  g: "#161b26", G: "#2a3446", i: "#3a3e48", I: "#5c6270",
  // foliage and bark
  F: "#1c2a1e", f: "#283c28", E: "#36502f", e: "#48663a", h: "#5c7a46",
  b: "#2a211a", n: "#3e3024",
  // straw, cloth, earth
  y: "#6e5a36", Y: "#8c7446", z: "#a48c58",
  c: "#3e4a5c", o: "#5a3a3a", O: "#6e5f4a",
  M: "#1e2418",                                            // contact shadow on grass
  // water: deep, body, shallows, glint, foam (digits: every letter is taken)
  "1": "#16202c", "2": "#1e3040", "3": "#2a4454", "4": "#5a7a86", "5": "#8aa2a6",
  // ground (ground.png hues), flowers, crops, moss
  R: "#32402d", U: "#3c4c34", V: "#4a5a3e", X: "#5a6a46",
  d: "#463a2f", a: "#544638", l: "#625442",
  q: "#7e6e9c", r: "#b4b0a2", j: "#b09a4e", Z: "#c0a868", N: "#5e7a4e", m: "#4a5a34",
})
ramps("kRUVX", "kdal", "kmN",  "kWwx", "kHTtu", "kLSsv", "kABCD", "kQPp", "kFfEeh", "kbn", "kyYz", "kgG", "kiI")

// ------------------------------------------------------------------ materials

// Thatch: overlapping rows of straw bundles, each row's lower lip darker; a ragged eave.
function thatch(c, x0, y0, w, h, seed) {
  const r = c.rng(seed)
  c.rect(x0, y0, w, h, "t")
  for (let y = y0 + 2; y < y0 + h; y += 4) {
    c.hline(x0, y + 2, w, "T")
    for (let x = x0; x < x0 + w; x++) {
      const k = r()
      if (k < 0.18) c.px(x, y, "u")
      else if (k < 0.3) c.px(x, y + 1, "T")
    }
  }
  // ragged eave: straw ends hanging over the wall
  for (let x = x0; x < x0 + w; x++) if (r() < 0.55) c.px(x, y0 + h, "T")
  c.hline(x0, y0 + h - 1, w, "H")
}

// Slate: offset courses, light top edge of each course.
function slate(c, x0, y0, w, h) {
  c.rect(x0, y0, w, h, "s")
  for (let row = 0, y = y0; y < y0 + h; y += 4, row++) {
    c.hline(x0, y, w, "v")
    c.hline(x0, y + 3, w, "S")
    for (let x = x0 + (row % 2) * 3; x < x0 + w; x += 6) c.vline(x, y + 1, 2, "S")
  }
  c.hline(x0, y0 + h - 1, w, "L")
}

// Coursed stone with offset joints.
function stone(c, x0, y0, w, h) {
  c.rect(x0, y0, w, h, "C")
  for (let row = 0, y = y0; y < y0 + h; y += 4, row++) {
    c.hline(x0, y + 3, w, "B")
    for (let x = x0 + (row % 2) * 4; x < x0 + w; x += 8) c.vline(x, y, 3, "B")
    c.hline(x0, y, w, row ? "C" : "D")
  }
}

function windowAt(c, x, y) {               // 7x7: frame, mullion cross, one glint
  c.rect(x, y, 7, 7, "W")
  c.rect(x + 1, y + 1, 5, 5, "g")
  c.vline(x + 3, y + 1, 5, "W")
  c.hline(x + 1, y + 3, 5, "W")
  c.px(x + 1, y + 1, "G")
  c.px(x + 4, y + 1, "G")
  c.hline(x, y + 7, 7, "x")                // sill
}

function doorAt(c, x, y, w = 9, h = 13) {
  c.rect(x, y, w, h, "W")
  c.rect(x + 1, y + 1, w - 2, h - 1, "w")
  for (let i = x + 3; i < x + w - 1; i += 2) c.vline(i, y + 1, h - 1, "W")
  c.px(x + w - 3, y + 7, "I")              // iron ring
}

// ------------------------------------------------------------------ houses
// Roof band on top, plaster wall under it, eave shadow on the wall, plinth stones at the foot.
function house(name, o) {
  const { w, roof, wall, chimney, seed, slateRoof } = o
  const top = chimney ? 8 : 0
  sprite(name, { w, h: top + roof + wall, outline: "k" }, (c) => {
    if (chimney) {                          // stone stack rising behind the ridge
      const cx = chimney
      stone(c, cx, 0, 7, top + 8)
      c.rect(cx - 1, 0, 9, 2, "D")
    }
    const wy = top + roof
    // wall
    c.rect(2, wy, w - 4, wall, "P")
    c.rect(2, wy, 1, wall, "p")
    // timber frame: corner posts, rail, braces
    c.vline(2, wy, wall, "w"); c.vline(w - 3, wy, wall, "w")
    c.hline(2, wy + 9, w - 4, "w")
    // windows and door
    for (const wx of o.windows) {
      windowAt(c, wx, wy + 2)
      c.vline(wx - 2, wy, wall - 2, "w"); c.vline(wx + 8, wy, wall - 2, "w")
    }
    doorAt(c, o.door, wy + wall - 14)
    c.vline(o.door - 1, wy, wall - 1, "w"); c.vline(o.door + 9, wy, wall - 1, "w")
    // plinth stones and eave shadow
    c.rect(2, wy + wall - 2, w - 4, 2, "B")
    for (let x = 4; x < w - 4; x += 5) c.px(x, wy + wall - 2, "C")
    c.rect(o.door, wy + wall - 2, 9, 2, "w")
    c.shade([2, wy, w - 4, 2])
    // roof
    if (slateRoof) slate(c, 0, top, w, roof)
    else thatch(c, 0, top, w, roof, seed)
    // hipped ends: the left face catches the sun, the right one is in shade; hips and ridge lit
    const hip = Math.round(roof * 0.55), b = top + roof - 1
    c.light([0, top, hip, top, 0, b], 1)
    c.shade([w, top, w - hip, top, w, b], 2)
    c.line(0, b, hip, top, slateRoof ? "v" : "u")
    c.line(w - 1, b, w - 1 - hip, top, slateRoof ? "S" : "T")
    c.hline(hip, top, w - 2 * hip, slateRoof ? "v" : "u")
    c.shade([0, b - 1, w, 2])
    if (chimney) { c.rect(chimney, top - 1, 7, 2, "B") }
  })
}

house("house_cottage", { w: 52, roof: 26, wall: 20, door: 30, windows: [10], seed: 3 })
house("house_long", { w: 80, roof: 30, wall: 22, door: 36, windows: [10, 58], chimney: 62, seed: 7 })
house("house_slate", { w: 60, roof: 28, wall: 22, door: 12, windows: [30, 44], chimney: 8, slateRoof: true })

// ------------------------------------------------------------------ chapel
// Stone nave, slate roof, bell tower on the left with a spire; arched door and a round window.
sprite("chapel", { w: 84, h: 104, outline: "k" }, (c) => {
  const nx = 22, nw = 62, ry = 40, wy = 70
  slate(c, nx, ry, nw, wy - ry)
  c.hline(nx, ry, nw, "v")
  stone(c, nx + 2, wy, nw - 4, 34)
  // arched door
  c.rect(nx + 24, wy + 16, 15, 18, "W")
  c.circ(nx + 31, wy + 16, 7, "W")
  c.rect(nx + 25, wy + 16, 13, 18, "w")
  c.circ(nx + 31, wy + 16, 6, "w")
  for (let x = nx + 27; x < nx + 37; x += 3) c.vline(x, wy + 12, 22, "W")
  c.px(nx + 33, wy + 24, "I")
  // round window above the door
  c.circ(nx + 31, wy + 6, 4, "B"); c.circ(nx + 31, wy + 6, 3, "g")
  c.vline(nx + 31, wy + 3, 7, "B"); c.hline(nx + 28, wy + 6, 7, "B")
  // tall windows
  for (const x of [nx + 8, nx + 48]) { c.rect(x, wy + 8, 5, 12, "g"); c.circ(x + 2, wy + 8, 2, "g"); c.px(x + 1, wy + 8, "G") }
  c.shade([nx + 2, wy, nw - 4, 2])
  // tower
  stone(c, 2, 34, 22, 70)
  c.rect(7, 44, 12, 14, "A"); c.circ(12, 44, 6, "A")       // belfry opening
  c.rect(10, 46, 5, 6, "i"); c.hline(9, 52, 7, "I")        // bell
  c.rect(10, 70, 6, 9, "g"); c.circ(12, 70, 3, "g")
  c.shade([18, 34, 6, 70], 1, 0.5)
  // spire
  c.poly([0, 34, 26, 34, 13, 0], "s")
  c.poly([13, 0, 26, 34, 13, 34], "S")
  for (let y = 8; y < 34; y += 5) c.line(13 - y * 13 / 34, y, 13 + y * 13 / 34, y, "L")
  c.vline(13, -1, 1, "I")
})

// ------------------------------------------------------------------ animation helpers

// Bends everything above `pivot` sideways with the wind: rows shift more the higher they are.
// Pixels are moved as raw colours, so it works after shading and on any palette. Swaying
// anims are drawn 2 px narrower than their canvas; the whole drawing moves 1 px right so the
// lean never runs off the edge.
function sway(c, amp, pivot, phase = c.phase) {
  const s = Math.sin(phase * Math.PI * 2) * amp
  for (let y = 0; y < c.h; y++) {
    const off = 1 + (y < pivot ? Math.round(s * (pivot - y) / pivot) : 0)
    const row = []
    for (let x = 0; x < c.w; x++) row.push(c.get(x, y))
    for (let x = 0; x < c.w; x++) {
      const v = row[x - off]
      c.px(x, y, v && (v & 0xff) ? "#" + (v >>> 0).toString(16).padStart(8, "0") : "_")
    }
  }
}
const WIND = { frames: 8, fps: 5, outline: "k" }

// ------------------------------------------------------------------ ground (32x32, seamless)
// Grass: blade tufts with lit tips, the odd pebble and daisy. No round colour patches: the user
// found them ugly. Everything wraps at the edges.
function wrapPx(c, x, y, k) { c.px(((x % 32) + 32) % 32, ((y % 32) + 32) % 32, k) }
function pebble(c, x, y) { wrapPx(c, x, y, "B"); wrapPx(c, x + 1, y, "C"); wrapPx(c, x, y + 1, "R") }
function tuft(c, x, y, r) {
  wrapPx(c, x, y, "R"); wrapPx(c, x - 1, y, "R"); wrapPx(c, x + 1, y, "R")
  wrapPx(c, x - 1, y - 1, "V"); wrapPx(c, x + 1, y - 1, "V"); wrapPx(c, x, y - 1, "V")
  wrapPx(c, x - (r() < 0.5 ? 1 : 0), y - 2, "X")
  if (r() < 0.5) wrapPx(c, x + 1, y - 2, "V")
}

for (let i = 0; i < 4; i++) sprite("ground_grass_" + i, { w: 32, h: 32 }, (c) => {
  const r = c.rng(11 + i * 7)
  c.rect(0, 0, 32, 32, "U")
  for (let n = 0; n < 9; n++) tuft(c, r() * 32 | 0, r() * 32 | 0, r)
  if (i === 1) pebble(c, r() * 32 | 0, r() * 32 | 0)
  if (i === 3) for (let n = 0; n < 3; n++) wrapPx(c, r() * 32 | 0, r() * 32 | 0, "r")   // daisies
})
for (let i = 0; i < 2; i++) sprite("ground_dirt_" + i, { w: 32, h: 32 }, (c) => {
  const r = c.rng(51 + i * 13)
  c.rect(0, 0, 32, 32, "a")
  for (let n = 0; n < 5; n++) {                       // pebbles: lit top, shadow under
    const x = r() * 32 | 0, y = r() * 32 | 0
    wrapPx(c, x, y, "l"); wrapPx(c, x + 1, y, "l"); wrapPx(c, x, y + 1, "d"); wrapPx(c, x + 1, y + 1, "d")
  }
  for (let n = 0; n < 2; n++) tuft(c, r() * 32 | 0, r() * 32 | 0, r)
})

// ------------------------------------------------------------------ village props

anim("well", { frames: 8, fps: 4, outline: "k", w: 22, h: 26 }, (c) => {
  c.vline(2, 0, 16, "w"); c.vline(19, 0, 16, "w")          // posts
  c.poly([0, 3, 11, -2, 22, 3, 22, 6, 0, 6], "T")           // little roof
  c.hline(0, 3, 22, "t")
  c.hline(4, 9, 14, "W")                                   // windlass
  const sw = Math.round(Math.sin(c.phase * Math.PI * 2) * 0.6)   // bucket sways on its rope
  c.line(11, 9, 11 + sw, 12, "n")
  c.rect(10 + sw, 13, 3, 3, "x"); c.px(10 + sw, 13, "z")
  c.ellipse(11, 18, 10, 3, "B")
  c.rect(1, 18, 21, 7, "C")
  for (let x = 3; x < 21; x += 4) c.vline(x, 18, 7, "B")
  c.hline(1, 21, 21, "B")
  c.ellipse(11, 18, 8, 2, "g")
})

// Street lamp, symmetric: finial, hipped cap, four-pane lantern (unlit by day), post, base.
sprite("lamp", { mx: "c", outline: "k" }, `
  ....I
  ...iI
  ..iiI
  .iiII
  iiiii
  .iGGz
  .iGzY
  .iGGz
  .iiii
  ..iiI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ...iI
  ..iiI
  ..iiI
  .iiiI
  iiiiI`)

sprite("brazier", { w: 11, h: 12, outline: "k" }, (c) => {
  c.poly([0, 0, 11, 0, 8, 5, 3, 5], "i")
  c.hline(0, 0, 11, "I")
  c.line(3, 5, 1, 11, "i"); c.line(7, 5, 9, 11, "i"); c.vline(5, 5, 6, "i")
  c.rect(2, 0, 7, 1, "W")
})

sprite("barrel", { w: 11, h: 14, outline: "k" }, (c) => {
  c.rect(0, 2, 11, 11, "w")
  c.rect(1, 1, 9, 13, "w")
  c.vline(2, 2, 11, "x"); c.vline(8, 2, 11, "W")
  c.hline(0, 4, 11, "i"); c.hline(0, 10, 11, "i")
  c.ellipse(5, 1, 4, 1, "x"); c.hline(3, 1, 5, "n")
})

sprite("crate", { w: 13, h: 13, outline: "k" }, (c) => {
  c.rect(0, 0, 13, 4, "x")                                 // lid seen from above
  c.rect(0, 4, 13, 9, "w")
  c.box(0, 4, 13, 9, "W")
  c.line(1, 5, 11, 11, "W")
  c.hline(0, 3, 13, "W")
})

sprite("hay_bale", { w: 16, h: 12, outline: "k" }, (c) => {
  c.rect(0, 0, 16, 4, "z"); c.rect(0, 4, 16, 8, "Y")
  for (let x = 1; x < 16; x += 2) c.vline(x, 5, 6, "y")
  c.vline(4, 0, 12, "n"); c.vline(11, 0, 12, "n")
  c.hline(0, 4, 16, "y")
})

// Round bale lying on its side: the cut end shows the rolled spiral, the body runs to the right.
sprite("hay_roll", { w: 19, h: 14, outline: "k" }, (c) => {
  c.rect(6, 1, 12, 13, "Y"); c.rect(18, 2, 1, 11, "Y")    // body, rounded far end
  c.hline(6, 1, 12, "z"); c.hline(6, 2, 13, "z")           // sun on the top of the roll
  c.shade((x, y) => x > 14 && y > 2 && c.is(x, y, "Y"), 1)
  for (const x of [10, 13, 16]) for (let y = 4; y < 13; y += 2) if (c.is(x, y, "Y")) c.px(x, y, "y")
  c.vline(14, 1, 13, "n")                                  // twine
  c.circ(6, 7, 6, "Y"); c.circ(6, 7, 4, "z"); c.ring(6, 7, 3, "y"); c.ring(6, 7, 1, "Y")
  c.px(6, 7, "y"); c.px(3, 4, "Z")
})

sprite("cart", { w: 30, h: 18, outline: "k" }, (c) => {
  c.rect(4, 2, 22, 4, "x")                                 // bed seen from above
  c.rect(4, 6, 22, 6, "w")
  for (let x = 8; x < 26; x += 5) c.vline(x, 6, 6, "W")
  c.line(0, 10, 5, 8, "W", 2)                              // shaft
  c.circ(9, 13, 4, "W"); c.circ(9, 13, 2, "n"); c.px(9, 13, "W")
  c.circ(22, 13, 4, "W"); c.circ(22, 13, 2, "n"); c.px(22, 13, "W")
  c.rect(6, 2, 5, 3, "Y"); c.rect(13, 1, 6, 4, "z")        // sacks of straw
})

anim("stall", { frames: 8, fps: 5, outline: "k", w: 34, h: 30 }, (c) => {
  c.vline(2, 8, 20, "w"); c.vline(31, 8, 20, "w")
  // striped awning, muted blue and linen; its scalloped hem lifts in the wind
  for (let x = 0; x < 34; x++) c.vline(x, 0, 9, (x >> 2) % 2 ? "c" : "O")
  for (let x = 0; x < 34; x += 4) {
    const lift = Math.sin(c.phase * Math.PI * 2 + x * 0.3) > 0.3 ? 1 : 0
    c.hline(x, 9 - lift, 3, (x >> 2) % 2 ? "c" : "O")
    if (lift) c.hline(x, 9, 3, "_")
  }
  c.shade([0, 6, 34, 3])
  c.rect(1, 18, 32, 4, "x"); c.rect(1, 22, 32, 6, "w")     // counter
  c.hline(1, 22, 32, "W")
  c.circ(8, 17, 2, "o"); c.circ(13, 17, 2, "o"); c.circ(11, 16, 2, "o")   // apples
  c.rect(20, 15, 8, 3, "Y")                                // bread loaves
  c.hline(20, 15, 8, "z")
})

// Gravestones: weathered, moss creeping up from the earth, a low mound in front.
function mound(c, x, y, w) { c.ellipse(x, y, w, 2, "d"); c.hline(x - w + 2, y - 2, w * 2 - 3, "a") }
sprite("grave_round", { w: 13, h: 18, outline: "k" }, (c) => {
  mound(c, 6, 15, 6)
  c.rect(2, 4, 9, 11, "C"); c.circ(6, 4, 4, "C"); c.rect(3, 4, 7, 1, "C")
  c.light((x, y) => c.is(x, y, "C") && x < 4, 1); c.shade((x, y) => c.is(x, y, "C") && x > 8, 1)
  c.hline(4, 6, 5, "B"); c.hline(4, 8, 5, "B"); c.hline(5, 10, 3, "B")   // worn lettering
  c.line(9, 1, 7, 4, "A")                                  // chip and crack from the shoulder
  c.px(2, 14, "m"); c.px(3, 14, "m"); c.px(2, 13, "m"); c.px(10, 14, "m")
})
sprite("grave_cross", { w: 13, h: 22, outline: "k" }, (c) => {
  mound(c, 6, 19, 6)
  c.rect(5, 0, 3, 19, "C"); c.rect(1, 4, 11, 3, "C")
  c.ring(6, 5, 3, "B"); c.px(6, 5, "C")                    // celtic ring
  c.vline(5, 0, 19, "D"); c.hline(1, 4, 11, "D")
  c.vline(7, 7, 12, "B"); c.hline(1, 6, 11, "B")
  c.px(5, 17, "m"); c.px(5, 18, "m"); c.px(7, 18, "m")
})
sprite("grave_slab", { w: 13, h: 16, outline: "k" }, (c) => {   // sunk and leaning
  mound(c, 6, 13, 6)
  c.poly([2, 13, 3, 2, 10, 0, 11, 12], "B")
  c.poly([3, 2, 10, 0, 10, 2, 3, 4], "D")
  c.line(5, 6, 8, 5, "A"); c.line(5, 8, 8, 7, "A")
  c.px(3, 12, "m"); c.px(4, 12, "m"); c.px(4, 11, "m")
})

// Fences link up on one rule: every post sits at x+1..x+5 of its piece. fence_h repeats every
// 16 px (post on its left), fence_v every 12 px going down the screen under a post, and
// fence_post closes a run at the next 16 px step (its x 0 carries the rails' last pixel).
// Outlines are drawn by hand so joins stay seamless.
sprite("fence_h", { w: 16, h: 14 }, (c) => {
  for (const y of [3, 8]) { c.hline(0, y, 16, "k"); c.hline(0, y + 1, 16, "x"); c.hline(0, y + 2, 16, "W"); c.hline(0, y + 3, 16, "k") }
  c.rect(1, 0, 5, 14, "k"); c.rect(2, 1, 3, 12, "w"); c.vline(2, 1, 12, "x"); c.px(3, 1, "x")
})
sprite("fence_post", { w: 7, h: 14 }, (c) => {
  for (const y of [3, 8]) { c.px(0, y, "k"); c.px(0, y + 1, "x"); c.px(0, y + 2, "W"); c.px(0, y + 3, "k") }
  c.rect(1, 0, 5, 14, "k"); c.rect(2, 1, 3, 12, "w"); c.vline(2, 1, 12, "x"); c.px(3, 1, "x")
})
sprite("fence_v", { w: 7, h: 12 }, (c) => {               // rails seen end-on from above
  c.rect(1, 0, 5, 12, "k"); c.rect(2, 0, 3, 12, "W"); c.vline(2, 0, 12, "x")
  c.rect(0, 2, 7, 10, "k"); c.rect(1, 3, 5, 8, "w"); c.vline(1, 3, 8, "x"); c.hline(1, 3, 5, "x")
})

// Banner: cloth ripples in a wave travelling down its length.
anim("banner", { frames: 8, fps: 6, outline: "k", w: 10, h: 28 }, (c) => {
  c.vline(1, 0, 28, "w"); c.px(1, 0, "x")
  for (let y = 2; y < 21; y++) {
    const o = Math.round(Math.sin(c.phase * Math.PI * 2 - y * 0.35) * (y - 2) / 18)
    const w = y < 16 ? 7 : 7 - (y - 15) * 2 + 1
    if (w <= 0) continue
    const x0 = y < 16 ? 2 : 2 + (y - 15)
    c.hline(x0 + o, y, Math.min(w, 7), "c")
    if (o > 0 && x0 === 2) c.px(2, y, "c")
  }
  c.vline(5, 4, 10, "D"); c.hline(3, 7, 5, "D")
  c.shade((x, y) => c.is(x, y, "c") && x >= 7, 1)
})

// ------------------------------------------------------------------ farm
// Crop rows tile every 16 px; they ripple in the wind with a different phase per column.
anim("crop_wheat", { frames: 8, fps: 5, w: 16, h: 16 }, (c) => {
  c.rect(0, 12, 16, 4, "d"); c.hline(0, 12, 16, "a")       // furrow
  for (let x = 0; x < 16; x += 2) {
    const o = Math.round(Math.sin(c.phase * Math.PI * 2 + x * 0.4))
    const top = 2 + (x * 7 % 3)
    c.vline(x, top + 2, 11 - top, "y")
    c.vline(x + o, top, 3, "z"); c.px(x + o, top, "Z")
    c.px(x + 1, top + 5, "Y")
  }
  for (let x = 0; x < 16; x++) c.px(x, 15, "k")
})
sprite("crop_cabbage", { w: 16, h: 12 }, (c) => {
  c.rect(0, 8, 16, 4, "d"); c.hline(0, 8, 16, "a")
  for (const x of [4, 12]) {
    c.circ(x, 7, 3, "k"); c.circ(x, 7, 2, "N"); c.px(x - 1, 6, "Z"); c.px(x + 1, 8, "E")
    c.px(x - 3, 8, "E"); c.px(x + 3, 8, "E")
  }
})
anim("scarecrow", { frames: 8, fps: 4, outline: "k", w: 19, h: 28 }, (c) => {
  c.vline(9, 6, 22, "n"); c.hline(1, 10, 17, "n")          // cross of poles
  const f = Math.sin(c.phase * Math.PI * 2) > 0.2 ? 1 : 0  // coat tails flap
  c.rect(5, 9, 9, 10, "O"); c.rect(4, 10, 11, 3, "O")
  c.px(4, 19 - f, "O"); c.px(14, 19 - f, "O"); c.vline(5, 19, 1 + f, "O")
  c.vline(9, 9, 10, "w")
  c.px(1, 11, "Y"); c.px(2, 12, "Y"); c.px(17, 11, "Y"); c.px(16, 12, "Y")   // straw hands
  c.circ(9, 5, 3, "z"); c.px(8, 5, "k"); c.px(10, 5, "k")  // sack head
  c.rect(5, 1, 9, 2, "W"); c.rect(7, 0, 5, 1, "W")         // hat
  c.shade((x, y) => c.is(x, y, "O") && x > 11, 1)
})
sprite("trough", { w: 22, h: 10, outline: "k" }, (c) => {
  c.rect(0, 0, 22, 3, "x"); c.rect(1, 1, 20, 2, "G")       // water
  c.px(4, 1, "I"); c.px(12, 2, "I")
  c.rect(0, 3, 22, 5, "w"); c.hline(0, 3, 22, "W")
  c.rect(1, 8, 2, 2, "W"); c.rect(19, 8, 2, 2, "W")
})
// ------------------------------------------------------------------ plants (all burn in V19h)
sprite("bush", { outline: "k", w: 22, h: 16 }, (c) => {
  const r = c.rng(21)
  for (const [x, y, rr] of [[6, 9, 5], [15, 9, 5], [11, 6, 6], [11, 11, 5]]) c.circ(x, y, rr, "f")
  for (let i = 0; i < 14; i++) c.stamp("_clump", 2 + (r() * 15 | 0), 2 + (r() * 9 | 0))
  for (let i = 0; i < 4; i++) c.stamp("_lit", 3 + (r() * 9 | 0), 1 + (r() * 5 | 0))
  c.shade((x, y) => c.is(x, y, "eEh") && y > 11, 1)
  c.recolor({ e: "f", E: "f", h: "f" }, (x, y) => !c.is(x, y, "*"))
})
sprite("bush_berry", { outline: "k", w: 18, h: 14 }, (c) => {
  const r = c.rng(33)
  for (const [x, y, rr] of [[5, 8, 4], [12, 8, 4], [9, 5, 5]]) c.circ(x, y, rr, "f")
  for (let i = 0; i < 10; i++) c.stamp("_clump", 1 + (r() * 12 | 0), 1 + (r() * 8 | 0))
  for (let i = 0; i < 6; i++) { const x = 2 + (r() * 13 | 0), y = 3 + (r() * 8 | 0); if (c.is(x, y, "fEe")) c.px(x, y, "q") }
  c.shade((x, y) => c.is(x, y, "eEh") && y > 9, 1)
})
function flowers(name, petal, seed) {
  anim(name, { frames: 8, fps: 5, w: 16, h: 10 }, (c) => {
    const r = c.rng(seed)
    for (let i = 0; i < 7; i++) {
      const x = 1 + (r() * 12 | 0), h = 3 + (r() * 5 | 0)
      c.vline(x, 10 - h, h, r() < 0.5 ? "E" : "e")
      if (r() < 0.6) c.px(x + (r() < 0.5 ? -1 : 1), 9, "E")   // leaf
      c.px(x, 9 - h, petal); c.px(x - 1, 10 - h, petal); c.px(x + 1, 10 - h, petal); c.px(x, 10 - h, "j")
    }
    sway(c, 1, 6)
  })
}
flowers("flowers_lav", "q", 41)
flowers("flowers_white", "r", 43)
flowers("flowers_yellow", "j", 47)
anim("grass_tall", { frames: 8, fps: 5, w: 14, h: 12 }, (c) => {
  for (let x = 1; x < 11; x++) {
    const h = 5 + ((x * 5) % 6), lean = x < 6 ? -1 : 1
    c.line(x, 11, x + (h > 8 ? lean : 0), 12 - h, x % 3 ? "e" : "E")
    c.px(x + (h > 8 ? lean : 0), 12 - h, "X")
  }
  sway(c, 1, 9)
})
anim("fern", { ...WIND, w: 18, h: 11 }, (c) => {
  for (const [dx, dy] of [[-7, -2], [-5, -6], [0, -8], [5, -6], [7, -2]]) {
    c.line(8, 10, 8 + dx, 10 + dy, "E")
    const n = Math.max(Math.abs(dx), Math.abs(dy))
    for (let i = 2; i < n; i += 2) {                        // leaflets along each frond
      const x = 8 + Math.round(dx * i / n), y = 10 + Math.round(dy * i / n)
      c.px(x - (dy ? 1 : 0), y - (dx ? 1 : 0), "e"); c.px(x + (dy ? 1 : 0), y + (dx ? 0 : 0), "e")
    }
  }
  c.light((x, y) => c.is(x, y, "e") && x < 7 && y < 6, 1)
  sway(c, 1, 8)
})
anim("reeds", { frames: 8, fps: 5, w: 14, h: 18 }, (c) => {
  for (const [x, h] of [[1, 12], [3, 16], [5, 10], [7, 15], [9, 13], [10, 8]]) {
    c.vline(x, 18 - h, h, x % 2 ? "E" : "e")
    if (h > 12) { c.vline(x, 18 - h, 3, "n"); c.px(x, 18 - h, "b") }   // cattail heads
  }
  sway(c, 1, 14)
})

// ------------------------------------------------------------------ trees
// Canopy built from leaf clumps: dark mass, mid clumps, lit clumps towards the upper left.
// Trees and bushes stay still (user, 2026-10-24); only small plants, crops and cloth move.
sprite("_clump", "  .ee.\n  eeee\n  eEEe\n  .EE.")
sprite("_lit", " .h.\n hhe\n .e.")

function oak(name, r, seed, birch) {
  const s = r * 2 + 2
  sprite(name, { outline: "k", w: s, h: s + 10 }, (c) => {
    const bark = birch ? "r" : "n", dark = birch ? "k" : "b"
    c.rect(r - 2, 2 * r - 6, 5, 16, bark); c.vline(r - 2, 2 * r - 6, 16, birch ? "D" : "b")
    if (birch) for (const y of [2 * r - 2, 2 * r + 3, 2 * r + 6]) c.hline(r - 1, y, 2, dark)
    c.px(r - 3, 2 * r + 9, bark); c.px(r + 3, 2 * r + 9, bark)  // root flare
    const rnd = c.rng(seed)
    for (let i = 0; i < 7; i++) {                           // lumpy silhouette from blobs
      const a = i / 7 * Math.PI * 2 + rnd(), d = r * 0.45
      c.circ(Math.round(r + 1 + Math.cos(a) * d), Math.round(r + 1 + Math.sin(a) * d), Math.round(r * 0.55), "f")
    }
    c.circ(r - 1, r - 1, r - 4, "E")
    for (let i = 0; i < r * 3; i++) {
      const a = rnd() * Math.PI * 2, d = rnd() * (r - 3)
      c.stamp("_clump", Math.round(r + Math.cos(a) * d) - 2, Math.round(r + Math.sin(a) * d) - 2)
    }
    for (let i = 0; i < r; i++) {
      const a = Math.PI * (1.0 + rnd() * 0.6), d = rnd() * (r - 4)
      c.stamp("_lit", Math.round(r - 2 + Math.cos(a) * d), Math.round(r - 3 + Math.sin(a) * d))
    }
    c.shade((x, y) => (x - r) + (y - r) > r * 0.6 && c.is(x, y, "eEh"), 1, 0.6)
    if (birch) c.recolor({ f: "E", E: "e", e: "h", h: "X" })
  })
}
oak("tree_oak", 15, 2)
oak("tree_oak_small", 11, 9)
oak("tree_birch", 10, 17, true)

sprite("tree_pine", { outline: "k", w: 32, h: 56 }, (c) => {
  c.rect(14, 46, 4, 10, "n"); c.vline(14, 46, 10, "b")
  for (const [y, hw, h] of [[0, 6, 14], [8, 9, 15], [17, 12, 16], [27, 15, 19]]) {
    c.poly([16 - hw, y + h, 16 + hw, y + h, 16, y], "f")
    c.poly([16 - hw, y + h, 16, y + h, 16, y], "E")
    for (let x = 16 - hw + 1; x < 16 + hw; x += 3) c.px(x, y + h - 1, "F")   // drooping tips
    c.line(16 - hw + 2, y + h - 2, 15, y + 3, "e")
  }
})

// Rocks: three faces (lit top, front, shadow side), cracks along the facets, moss on top.
sprite("rock", { w: 30, h: 22, outline: "k" }, (c) => {
  c.poly([1, 10, 6, 3, 14, 0, 24, 1, 29, 8, 28, 18, 20, 21, 6, 21, 1, 16], "B")
  c.poly([1, 10, 6, 3, 14, 0, 24, 1, 29, 8, 20, 11, 9, 12], "D")       // top face
  c.poly([20, 11, 29, 8, 28, 18, 20, 21], "A")                          // shadow side
  c.poly([6, 3, 14, 0, 13, 5, 7, 7], "C")                               // second facet on top
  c.line(9, 12, 20, 11, "C")                                            // lit edge
  c.line(13, 13, 11, 20, "A"); c.line(11, 16, 8, 18, "A")               // crack, forking
  c.line(23, 12, 25, 17, "k")
  for (const [x, y] of [[8, 4], [9, 4], [10, 3], [16, 2], [17, 2], [17, 3], [4, 9], [5, 8]]) c.px(x, y, "m")
  c.px(3, 21, "C"); c.px(26, 21, "B")                                   // pebbles at the foot
})
sprite("rock_small", { w: 14, h: 10, outline: "k" }, (c) => {
  c.poly([0, 5, 4, 1, 10, 0, 14, 4, 13, 9, 1, 10], "B")
  c.poly([0, 5, 4, 1, 10, 0, 14, 4, 8, 6], "D")
  c.poly([8, 6, 14, 4, 13, 9, 9, 10], "A")
  c.px(5, 2, "m"); c.px(6, 2, "m")
})

// ------------------------------------------------------------------ terrain (dual grid)
// Each 16 px tile sits on the shared corner of four map cells, offset half a cell: its index is
// TL + 2 TR + 4 BL + 8 BR (bit set = the terrain is in that cell). The edge is the 0.5 contour of
// the corner values blended across the tile plus a wobble that repeats every 64 px, so every
// index comes in variants `<name>_<index>_<variant>`, variant = (tile x & 3) + 4 (tile y & 3)
// + 16 * the two cells past the tile (see `dual`), and a tile always meets its neighbours.
// Tiles are drawn over the ground; outside stays clear.
const TAU = Math.PI * 2
function wobble(x, y, s) {
  return 0.11 * Math.sin(TAU * (2 * x + 2 * y) / 64 + s) + 0.07 * Math.sin(TAU * (6 * x - 2 * y) / 64 + 2 * s)
    + 0.04 * Math.sin(TAU * (4 * x + 10 * y) / 64 + 3 * s) + 0.07 * Math.sin(TAU * (3 * x + y) / 64 + 4 * s)
    + 0.04 * Math.sin(TAU * (x - 5 * y) / 64 + 5 * s)
}
const ox = (n) => (n & 3) * 16, oy = (n) => (n >> 2 & 3) * 16
// The field at tile pixel (x, y). Rows above or below the tile read the cells past it (the
// variant's top bits `n >> 4`), so a face that reaches over the tile's edge meets the next tile.
function field(i, n, x, y, s) {
  let tl = i & 1, tr = i >> 1 & 1, bl = i >> 2 & 1, br = i >> 3 & 1, ty = y
  const far = n >> 4
  if (y < 0) { bl = tl; br = tr; tl = far & 1; tr = far >> 1 & 1; ty = y + 16 }
  else if (y >= 16) { tl = bl; tr = br; bl = far & 1; br = far >> 1 & 1; ty = y - 16 }
  const u = (x + 0.5) / 16, v = (ty + 0.5) / 16
  return (tl * (1 - u) + tr * u) * (1 - v) + (bl * (1 - u) + br * u) * v + wobble(x + ox(n), y + oy(n), s)
}
// `far`: 4 when the tiles read the two cells above them (water) or below them (ridge).
function dual(name, fn, far = 1) {
  for (let i = 1; i < 16; i++) for (let n = 0; n < 16 * far; n++) fn(name + "_" + i + "_" + n, i, n)
}
const hash = (a, b) => (((a * 73856093) ^ (b * 19349663)) >>> 0) % 997

// River and ponds sit below the grass: the bank on their far side shows an earth face (3/4
// view), a dark band of its shadow lies on the water, shallows line the other shores and faint
// ripples cross the deep water. Glints (`glint`) are scattered over it by the game.
dual("water", (name, i, n) => sprite(name, { w: 16, h: 16 }, (c) => {
  const f = (x, y) => field(i, n, x, y, 0.7)
  for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
    const v = f(x, y), wx = x + ox(n), wy = y + oy(n)
    if (v < 0.5) { if (v > 0.45) c.px(x, y, "R"); continue }   // grass lip
    let up = 0
    while (up < 7 && f(x, y - up - 1) >= 0.5) up++
    if (up < 4) c.px(x, y, ["a", "a", "d", "d"][up])       // earth face of the near bank
    else if (up < 6) c.px(x, y, "1")                       // its shadow on the water
    else if (v < 0.52) c.px(x, y, "1")                     // wet rim under the far shore
    else if (v < 0.6) c.px(x, y, "3")                      // shallows
    else c.px(x, y, (wy & 1) && hash((wx + 5 * wy) >> 2, wy) % 13 === 0 ? "1" : "2")
  }
  for (let x = 0; x < 16; x++) for (let y = 0; y < 16; y++)  // roots and stones in the face
    if (c.is(x, y, "a") && hash(x + ox(n), y + oy(n)) % 9 === 0) c.px(x, y, hash(x + ox(n), 5) & 1 ? "l" : "d")
}), 4)

// A glint on the current: a dash that brightens, drifts east and fades.
anim("glint", { frames: 8, fps: 5, w: 12, h: 1 }, (c) => {
  const [x, len] = [[0, 0], [1, 1], [2, 2], [3, 3], [5, 4], [7, 3], [9, 2], [11, 1]][c.frame]
  for (let k = 0; k < len; k++) c.px(x + k, 0, k === len - 1 && len > 2 ? "5" : "4")
})

// Plaza cobbles: rounded setts in offset courses (4 px, repeats every 16), a kerb of darker
// setts along the edge and an earth bed just outside it.
dual("cobble", (name, i, n) => sprite(name, { w: 16, h: 16 }, (c) => {
  for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
    const v = field(i, n, x, y, 2.1)
    if (v < 0.5) { if (v > 0.45) c.px(x, y, "d"); continue }
    const row = y >> 2, o = (row & 1) * 2, col = ((x + o) & 15) >> 2, cx = (x + o) & 3, cy = y & 3
    if (cx === 3 || cy === 3) { c.px(x, y, "A"); continue }  // mortar joints
    const kerb = v < 0.58, tone = hash(col + (n & 3) * 4, row + (n >> 2 & 3) * 4) % 7
    let k = kerb || tone === 0 ? "B" : "C"
    if (cx === 0 && cy === 0 && tone === 2 && !kerb) k = "D"   // lit upper-left of the odd sett
    else if (cx === 2 && cy === 2) k = kerb ? "A" : "B"     // shaded lower-right
    c.px(x, y, k)
  }
}))

// Dirt paths and lanes: packed earth with a darker rut line inside the edge, pebbles, and the
// grass fraying over the edge in tufts.
dual("path", (name, i, n) => sprite(name, { w: 16, h: 16 }, (c) => {
  for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
    const v = field(i, n, x, y, 4.3)
    if (v < 0.5) continue
    c.px(x, y, v < 0.55 ? "d" : "a")
  }
  for (let y = 0; y < 15; y++) for (let x = 0; x < 15; x++) {
    const h = hash(x + ox(n), y + oy(n))
    if (h % 41 === 0 && c.is(x, y, "a") && c.is(x + 1, y + 1, "a")) { c.px(x, y, "l"); c.px(x + 1, y, "l"); c.px(x, y + 1, "d") }
    else if (h % 23 === 1 && y > 0 && c.is(x, y, "d")) { c.px(x, y, "V"); c.px(x, y - 1, "X") }   // tufts on the rim
  }
}))

// Ridge: a raised outcrop that blocks. Its top is darker grass (tufts only, no speckle) with a
// lit grass lip; along its south edge a 9 px rock face of blocks with vertical joints (offset
// per course, lit tops) drops to the grass, with a contact shadow at its foot.
dual("ridge", (name, i, n) => sprite(name, { w: 16, h: 16 }, (c) => {
  const f = (x, y) => field(i, n, x, y, 5.9), wx = (x) => x + ox(n)
  for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
    const v = f(x, y)
    if (v < 0.5) {                                        // foot shadow, under the south face
      const rising = y > 0 ? f(x, y - 1) > v : v > f(x, y + 1)
      if (v > 0.43 && rising) c.px(x, y, "M")
      continue
    }
    let down = 0
    while (down < 10 && f(x, y + down + 1) >= 0.5) down++
    if (down < 9) {                                       // rock face, lip on top
      const fy = 8 - down, blk = (wx(x) + (fy > 4 ? 3 : 0)) >> 2, top = fy === 1 || fy === 5
      const joint = (wx(x) + (fy > 4 ? 3 : 0)) % 4 === 0 && hash(blk, fy > 4) % 3 !== 0
      c.px(x, y, fy === 0 ? "X" : fy >= 7 ? "A" : joint ? "A" : top ? (hash(blk, fy) & 1 ? "D" : "C") : hash(blk, fy > 4) % 3 ? "B" : "C")
    } else if (down < 10) c.px(x, y, "X")                 // grass lip
    else c.px(x, y, (hash(wx(x), y + oy(n)) % 17) === 0 ? "U" : "R")
    if (v < 0.52 && down >= 9) c.px(x, y, "k")            // outline on the top's other sides
  }
}), 4)

// Woods floor: the dense border forest's ground (dark mould, leaf litter, ferny tufts) under the
// trees the layout packs on it; its rim fades into darker grass.
dual("woods", (name, i, n) => sprite(name, { w: 16, h: 16 }, (c) => {
  for (let y = 0; y < 16; y++) for (let x = 0; x < 16; x++) {
    const v = field(i, n, x, y, 1.3)
    if (v < 0.5) { if (v > 0.42) c.px(x, y, "R"); continue }
    const h = hash(x + ox(n), y + oy(n)) % 11
    c.px(x, y, v < 0.56 ? "f" : h === 0 ? "n" : h === 1 ? "E" : h < 4 ? "f" : "F")
  }
}))

// ------------------------------------------------------------------ bridge
// Plank bridge across a river running east-west: deck of cross planks, side beams with posts
// every 16 px, the south end's face and two piles into the water. Its length spans a 48 px
// river plus both banks.
sprite("bridge", { w: 32, h: 76 }, (c) => {
  const r = c.rng(5)
  c.rect(0, 0, 32, 70, "k")
  c.rect(4, 1, 24, 68, "w")
  for (let y = 1; y < 69; y += 4) {                         // planks, uneven ends, lit tops
    const l = 4 - (r() < 0.3 ? 1 : 0), w = 24 + (r() < 0.3 ? 1 : 0) + (4 - l)
    c.rect(l, y, w, 3, "w"); c.hline(l, y, w, "x"); c.hline(4, y + 3, 24, "W")
    if (r() < 0.5) c.px(6 + (r() * 18 | 0), y + 1, "W")      // knot
  }
  for (const x of [1, 28]) {                                // side beams seen from above
    c.rect(x, 0, 3, 70, "W"); c.vline(x, 0, 70, "x")
    for (let y = 2; y < 70; y += 16) { c.rect(x, y, 3, 4, "x"); c.hline(x, y, 3, "z"); c.hline(x, y + 4, 3, "k") }
  }
  c.rect(1, 70, 30, 4, "W"); c.hline(1, 70, 30, "x")        // south face
  c.rect(0, 70, 1, 5, "k"); c.rect(31, 70, 1, 5, "k"); c.hline(0, 74, 32, "k")
  for (const x of [4, 25]) { c.rect(x, 74, 3, 2, "b"); c.px(x, 74, "n") }   // piles
  c.shade([27, 1, 1, 68], 1)                               // deck edge under the east beam
})

// ------------------------------------------------------------------ dry-stone walls
// Village and churchyard walls on the 16 px grid. A horizontal piece shows its cap (top face,
// 6 px) and its front face; a vertical run shows only its cap, 8 px wide at x 4..11, drawn over
// the horizontal piece it leaves from so the face stops there. `wall_v_end` closes a vertical
// run with its face; `wall_h_l` / `wall_h_r` start and end a horizontal run at a corner.
// Fieldstones of uneven length (3-6 px) per course; widths come from a hash of the course so
// pieces repeat every 16 px and still join.
function course(row, seed) {
  const cuts = []
  for (let x = 0; x < 16;) { cuts.push(x); x += 3 + hash(row * 5 + x, seed) % 4; if (x > 13) break }
  return cuts
}
function wallCap(c, x0, x1, y0, y1) {
  for (let y = y0; y < y1; y++) {
    const row = y >> 2, cy = y & 3, cuts = course(row, 3)
    for (let x = x0; x < x1; x++) {
      const joint = cy === 3 || cuts.includes(x & 15)
      const lit = cuts.includes((x - 1) & 15) && cy === 0
      c.px(x, y, joint ? "A" : lit ? "D" : hash(cuts.filter((q) => q <= (x & 15)).length, row) % 3 ? "C" : "B")
    }
  }
}
function wallFace(c, x0, x1, y0, y1) {
  for (let y = y0; y < y1; y++) {
    const row = (y - y0) / 3 | 0, cy = (y - y0) % 3, cuts = course(row, 11)
    for (let x = x0; x < x1; x++) c.px(x, y, cy === 2 || cuts.includes(x & 15) ? "k" : cy === 0 ? "B" : "A")
  }
  c.hline(x0, y0, x1 - x0, "D")                             // lit lip of the cap
  for (let x = x0; x < x1; x += 5) if (hash(x, 7) % 3 === 0) c.px(x, y1 - 1, "m")   // moss at the foot
}
function wall(name, x0, x1, opts) {
  sprite(name, { w: 16, h: 16 }, (c) => {
    if (opts.h) {
      wallCap(c, x0, x1, 1, 6); wallFace(c, x0, x1, 6, 15)
      c.hline(x0, 0, x1 - x0, "k"); c.hline(x0, 15, x1 - x0, "k")
      if (x0 > 0) c.vline(x0 - 1, 0, 16, "k")
      if (x1 < 16) c.vline(x1, 0, 16, "k")
    }
    if (opts.v) {
      const end = opts.end ? 6 : 16
      wallCap(c, 4, 12, 0, end)
      if (opts.end) { wallFace(c, 4, 12, 6, 15); c.hline(4, 15, 8, "k") }
      c.vline(3, 0, 16, "k"); c.vline(12, 0, 16, "k")
      if (opts.top) c.hline(4, 0, 8, "k")
    }
  })
}
wall("wall_h", 0, 16, { h: true })
wall("wall_h_l", 4, 16, { h: true })
wall("wall_h_r", 0, 12, { h: true })
wall("wall_v", 0, 0, { v: true })
wall("wall_v_top", 0, 0, { v: true, top: true })
wall("wall_v_end", 0, 0, { v: true, end: true })

sheet("terrain", ["ground_*", "glint", "water_*", "cobble_*", "path_*", "ridge_*", "woods_*"], { pad: 1 })
sheet("village", ["bridge", "wall_*", "house_*", "chapel", "well", "lamp", "brazier", "barrel", "crate", "hay_*", "cart",
  "stall", "grave_*", "fence_*", "banner", "crop_*", "scarecrow", "trough", "bush*",
  "flowers_*", "grass_tall", "fern", "reeds", "tree_*", "rock*"], { pad: 1 })
