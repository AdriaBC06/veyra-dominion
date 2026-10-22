// Title backdrop (V14 phase 3, docs/design.md 7.5): night over razed Ashford, seen from a hill.
// A blood-red moon behind Veyra's head and shoulder, the village small in the valley right of
// centre (behind her), smoke rising only from it, and the crest with a leaning grave cross bottom
// left. Layers are drawn by `scenes/title.rs` with a subtle parallax:
//   title_sky, title_hills   880x360, right-anchored (canvas x = screen x + 872 - view width)
//   title_village            12-frame anim, placed at VILLAGE (canvas coords)
//   title_smoke              24 puff tiers (small, fire-lit -> big, thin); the game moves them up
//                            from the fires at SMOKE_SOURCES (canvas coords, kept in sync with Rust)
//   title_crest              left-anchored, bottom-aligned foreground
// Palette rule: the backdrop is cold and muted; only the moon and the fires are warm, and the moon
// is an orange blood red, lighter than Veyra's crimson wings so their silhouette reads over it.

const BW = 880, BH = 360
const rnd = (seed) => { let s = seed >>> 0; return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296) }
const B4 = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
const bayer = (x, y) => (B4[y & 3][x & 3] + 0.5) / 16
// Picks a key from `keys` (dark -> light) for a value v in 0..1. Only a narrow seam between two
// bands is dithered (steep = how narrow), so areas stay flat and read as pixel art, not noise.
function band(x, y, v, keys, steep = 3) {
  const n = keys.length - 1, f = Math.min(Math.max(v, 0), 1) * n
  const i = Math.min(Math.floor(f), n - 1), fr = Math.min(Math.max((f - i - 0.5) * steep + 0.5, 0), 1)
  return keys[fr > bayer(x, y) ? i + 1 : i]
}

pal({
  a: "#05050c", b: "#08081a", c: "#0d0c1f", e: "#141126", g: "#1c142a", h: "#27182b", i: "#341c2b", // sky
  "1": "#e6e0f0", "2": "#8c88a8", "3": "#4c4864",                                                     // stars
  X: "#1c0c18", Y: "#2a0f19",                                                                       // moon haze
  R: "#4e1018", Q: "#6e1a1e", m: "#8e2a24", n: "#aa3a2a", o: "#c25032", O: "#d86a3c",                // moon
  F: "#120f22", f: "#1e1630", G: "#0d0b1a", H: "#17112a",                                           // far / mid hills
  v: "#09080f", V: "#100c18", w: "#1c1019", W: "#2a1418",                                           // valley, fire glow
  u: "#100d17",                                                                                     // road dust
})
ramps("abcegh", "RQmnoO")

// The moon sits behind Veyra's head and left shoulder (the portrait's top-left is canvas (528, 33)).
const MX = 606, MY = 122, MR = 112

sprite("title_sky", { w: BW, h: BH }, (c) => {
  for (let y = 0; y < BH; y++) for (let x = 0; x < BW; x++) {
    // night overhead, a smoky warm haze over the horizon, strongest above the burning village
    const heat = Math.exp(-(((x - 540) / 260) ** 2)) * Math.max(0, (y - 120) / 140)
    c.px(x, y, band(x, y, y / 250 + heat * 0.35, "abceghi", 2.5))
  }
  // haze around the moon: two dithered rings
  c.clip([MX, MY, MR + 16], () => c.dither(MX - MR - 16, MY - MR - 16, 2 * MR + 32, 2 * MR + 32, ".", "X", 0.35))
  c.clip([MX, MY, MR + 7], () => c.dither(MX - MR - 7, MY - MR - 7, 2 * MR + 14, 2 * MR + 14, ".", "Y", 0.6))
  // stars: sparse, only high up and away from the moon
  const r = rnd(11)
  for (let k = 0; k < 46; k++) {
    const x = Math.floor(r() * BW), y = Math.floor(r() * 170) + 3, lum = r()
    if (Math.hypot(x - MX, y - MY) < MR + 26) continue
    c.px(x, y, lum > 0.85 ? "2" : "3")
    if (lum > 0.95) { c.px(x, y, "1"); for (const [i, j] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) c.px(x + i, y + j, "3") }
  }
  // moon body: limb darkening in bands, lit a little more toward the top left
  for (let y = MY - MR; y <= MY + MR; y++) for (let x = MX - MR; x <= MX + MR; x++) {
    const d = Math.hypot(x - MX, y - MY) / MR
    if (d > 1) continue
    const lit = Math.hypot(x - (MX - 30), y - (MY - 34)) / MR
    c.px(x, y, band(x, y, 1 - 0.55 * d ** 3 - 0.35 * lit ** 2 + 0.05, "RQmnoO", 3))
  }
  // maria: soft dark seas (lumpy blobs), and a few craters with a lit lower-right rim
  const seas = [[570, 94, 30, 22], [622, 140, 38, 24], [558, 158, 22, 16], [650, 88, 18, 14], [598, 186, 26, 10]]
  c.clip([MX, MY, MR - 2], () => {
    for (const [x, y, rx, ry] of seas) {
      const rr = rnd(x * 7 + y)
      for (let k = 0; k < 7; k++) {
        const ox = (rr() - 0.5) * rx, oy = (rr() - 0.5) * ry
        c.ellipse(Math.round(x + ox), Math.round(y + oy), Math.round(rx * (0.35 + rr() * 0.3)), Math.round(ry * (0.35 + rr() * 0.3)), "m")
      }
    }
    c.recolor({ m: "Q" }, (x, y) => Math.hypot(x - MX, y - MY) > MR * 0.82)
    for (const [x, y, rad] of [[538, 130, 6], [630, 60, 4], [664, 166, 5], [584, 212, 4], [610, 110, 3]]) {
      c.ring(x, y, rad, "Q"); c.clip((px, py) => px - x + py - y > 1, () => c.ring(x, y, rad, "O"))
    }
  })
})

// ---------------------------------------------------------------- hills, valley
// Two ranges across the valley, the valley floor and the road climbing toward the viewer's hill.
const ridge = (x, base, a, s) => base - a * Math.sin(x / 97 + s) - a * 0.5 * Math.sin(x / 41 + s * 2) - a * 0.3 * Math.sin(x / 17 + s * 3)
const VILLAGE = [376, 208]       // top-left of title_village in canvas coords (Rust: VILLAGE)
const VW = 264, VH = 100

sprite("title_hills", { w: BW, h: BH }, (c) => {
  for (let x = 0; x < BW; x++) {
    const far = Math.round(ridge(x, 222, 7, 0.4)), mid = Math.round(ridge(x, 242, 5, 2.1))
    c.vline(x, far, BH, "F"); c.px(x, far, "f")
    c.vline(x, mid, BH, "G"); c.px(x, mid, "H")
    c.vline(x, Math.round(ridge(x, 258, 2, 4)), BH, "v")
  }
  // warm glow of the fires on the valley floor around the village
  const [vx, vy] = VILLAGE
  for (let y = 236; y < BH; y++) for (let x = 300; x < 800; x++) {
    if (!c.is(x, y, "vG")) continue
    const g = Math.hypot((x - (vx + 120)) / 150, (y - (vy + 60)) / 46)
    if (g < 1) c.px(x, y, band(x, y, 1 - g, c.is(x, y, "v") ? "vVwW" : "GGH", 2))
  }
  // the road out of the village, winding toward the bottom left and the viewer's hill
  for (let t = 0; t <= 0.7; t += 0.002) {
    const x = Math.round(vx + 40 - t * 330 + Math.sin(t * 7) * 14), y = Math.round(vy + 74 + t * 80)
    if (y < BH && c.is(x, y, "vVw")) c.hline(x - 1 - Math.round(t * 2), y, 2 + Math.round(t * 3), t < 0.25 ? "w" : "u")
  }
})

// ---------------------------------------------------------------- the razed village
// 3/4 view from the hill, back row to front: stone shells with coursed walls, timber-framed
// cottages, roof remnants with charred edges, a barn burnt to its skeleton, the church with
// buttresses, lancet windows lit from inside and a snapped tower; a well, an overturned cart,
// fences, fallen beams and a burnt tree. Fires of several kinds: three blazes (they feed the smoke
// columns), flames licking out of windows, burning beams, a burning haystack and smouldering heaps.
// Each fire lights the ground around it and the faces turned to it. 12 frames at 12 fps.
pal({
  k: "#050409", s: "#0c0a11", S: "#1b1520", t: "#272029",                     // inside, shade, wall, plaster
  g: "#100d15", G: "#2e2532",                                                 // mortar / timber shade, stone edge
  b: "#24171a", B: "#4a2219", L: "#7a3a1e", M: "#a8562a",                     // timber; fire-lit faces
  h: "#2a1d18", H: "#3e2a1c",                                                 // straw
  "1": "#160c14", "2": "#1e0e12", "3": "#29120f",                             // ground in firelight
  e: "#5e140e", E: "#a8301a",                                                 // embers
  r: "#6e1610", q: "#c23a16", y: "#f08a28", Y: "#ffd070", w: "#fff1c0",      // flame: rim -> white hot
})
const TAU = Math.PI * 2, FRAMES = 12
// The three blazes, x, base y, size (Rust: FIRES; they feed the smoke columns).
const FIRES = [[76, 60, 0.9], [124, 34, 1.2], [210, 62, 0.9]]

anim("title_village", { w: VW, h: VH, frames: FRAMES, fps: 12 }, (c) => {
  const ph = c.frame / FRAMES, R = (seed) => rnd(seed)
  const lights = []                                   // [x, y, radius] for the face lighting pass
  const fl = (n, seed) => Math.sin(ph * TAU * n + seed)  // loops over the 12 frames

  // ---- flames
  // One tongue, painted row by row: it narrows, sways more toward the tip and leans with the wind.
  function tongue(cx, base, H, W, n, seed, lean = 1) {
    const h = Math.max(2, Math.round(H * (0.78 + 0.22 * fl(n, seed))))
    for (let i = 0; i < h; i++) {
      const s = i / h, mid = cx + lean * s * s * h * 0.25 + 1.6 * s * fl(n + 1, seed + s * 3)
      const half = Math.max(0.5, (W / 2) * (1 - s) ** 0.7)
      for (let x = Math.round(mid - half); x <= Math.round(mid + half); x++) {
        const u = Math.abs(x + 0.5 - mid) / (half + 0.5)
        let k = u > 0.75 || s > 0.82 ? "r" : u > 0.45 || s > 0.6 ? "q" : s > 0.35 ? "y" : "Y"
        if (s < 0.15 && u < 0.3 && W >= 4) k = "w"
        c.px(x, base - i, k)
      }
    }
    // a lick that breaks off the tip and fades as it rises
    const t = (ph * n + seed) % 1
    if (H > 6 && t < 0.5) c.px(Math.round(cx + lean * 2 + t * 3), base - h - 1 - Math.round(t * 6), t < 0.25 ? "q" : "r")
  }
  function blaze(x, y, sz, seed) {
    const r = R(seed), n = 7
    for (let j = 0; j < n; j++) {
      const t = j / (n - 1) - 0.5, tall = 1 - Math.abs(t) * 1.4
      tongue(x + Math.round(t * 16 * sz), y, sz * (7 + 13 * tall + 5 * r()), sz * (3 + 2 * r()), 1 + (j % 3), seed + j * 1.7)
    }
    c.hline(x - Math.round(5 * sz), y, Math.round(10 * sz) + 1, "y"); c.hline(x - Math.round(3 * sz), y, Math.round(6 * sz) + 1, "w")
    lights.push([x, y - 4, 16 * sz])
  }
  const windowFire = (x, y, seed) => { tongue(x, y, 6, 2, 2, seed); tongue(x + 1, y, 4, 2, 3, seed + 2); lights.push([x, y - 2, 7]) }
  function beamFire(x0, y0, x1, y1, seed) {
    c.line(x0, y0, x1, y1, "b")
    const len = Math.hypot(x1 - x0, y1 - y0)
    for (let d = 1; d < len; d += 3) {
      const x = Math.round(x0 + ((x1 - x0) * d) / len), y = Math.round(y0 + ((y1 - y0) * d) / len)
      tongue(x, y - 1, 2 + 3 * (0.5 + 0.5 * Math.sin(d + seed)), 2, 1 + (d % 3), seed + d)
    }
    lights.push([(x0 + x1) / 2, (y0 + y1) / 2, len * 0.5 + 4])
  }
  function smoulder(x, y, w, seed) {                  // a heap glowing without much flame
    const r = R(seed)
    for (let i = 0; i < w; i++) {
      const hh = Math.round((1 - ((i / (w - 1)) * 2 - 1) ** 2) * 3)
      c.vline(x + i, y - hh, hh + 1, "s")
      for (let j = 0; j <= hh; j++) if (r() < 0.35) {
        const g = 0.5 + 0.5 * fl(1 + (i % 2), seed + i * 2.1 + j)
        c.px(x + i, y - j, g > 0.75 ? "y" : g > 0.4 ? "E" : "e")
      }
    }
    if (fl(1, seed) > 0.2) tongue(x + Math.floor(w / 2), y - 2, 3, 2, 2, seed)
  }

  // ---- ground lit by each fire (drawn first, under everything), flickering with it
  const glow = (x, y, rad) => {
    const k = rad * (0.94 + 0.06 * fl(2, x))
    for (let yy = Math.round(y - k * 0.4); yy <= y + k * 0.4; yy++) for (let xx = Math.round(x - k); xx <= x + k; xx++) {
      const d = Math.hypot((xx - x) / k, (yy - y) / (k * 0.4))
      if (d < 1) c.px(xx, yy, band(xx, yy, 1 - d, ".123", 2.5))
    }
  }
  // the street winding through, then the firelit ground
  for (let y = 40; y < VH; y++) { const x = Math.round(120 - (y - 40) * 1.6 + 8 * Math.sin(y / 9)); c.hline(x, y, 5 + Math.round((y - 40) / 12), "1") }
  c.recolor({ "1": "." }, (x, y) => x > 255)
  for (const [x, y, sz] of FIRES) glow(x, y + 4, 26 * sz)
  glow(150, 92, 16); glow(98, 72, 9); glow(56, 46, 8)

  // ---- buildings
  // Walls: "stone" (courses with offset joints) or "timber" (plaster panels in a charred frame).
  function wall(x, top, w, h, mat, seed) {
    const r = R(seed)
    // collapse: the middle of the wall is knocked lower than the corners
    const dip = 2 + Math.floor(r() * (h - 3)), at = 4 + Math.floor(r() * (w - 8)), span = 3 + Math.floor(r() * 5)
    for (let i = 0; i < w; i++) {
      const cut = Math.max(0, Math.round(dip * (1 - Math.abs(i - at) / span))) + (r() < 0.25 ? 1 : 0)
      for (let y = top + cut; y < top + h; y++) {
        let k
        if (mat === "stone") {
          const row = y - top, course = Math.floor(row / 3)
          k = row % 3 === 2 || (i + course * 3) % 6 === 0 ? "g" : "S"
        } else {
          const row = y - top
          k = i % 6 === 0 || row === Math.floor(h / 2) || (i % 6 === row % 6 && i < w / 2) ? "b" : "t"
        }
        c.px(x + i, y, k)
      }
      c.px(x + i, top + cut, mat === "stone" ? "G" : "b")
    }
    if (mat === "stone") for (let i = 0; i < w; i++) c.px(x + i, top + h - 1, "g")
  }
  // A roofless shell: inside seen from above, back wall top, front wall, side in shade, openings.
  function house(o) {
    const { x, gy, w, d = 7, h = 9, mat = "stone", roof = 0, chimney = -1, win = [], seed } = o
    const top = gy - h
    c.rect(x + 1, top - d, w - 1, d + 1, "k"); c.hline(x + 1, top - d, w - 1, "s")
    if (roof) {                                        // remnant of the front roof slope, tiles in rows
      const rw = Math.round(w * roof), r = R(seed + 9)
      for (let y = top - d - 3; y <= top; y++) {
        const edge = rw - Math.round(r() * 3), row = y - top + d + 3
        for (let i = 0; i < edge; i++) c.px(x + i, y, row === 0 ? "G" : row % 3 === 2 && (i + row) % 4 < 2 ? "k" : row % 3 === 0 ? "g" : "s")
        c.px(x + edge, y, "B")                         // the burnt edge still glows
      }
      c.hline(x - 1, top, rw, "g")                     // eaves shadow
      for (let j = rw + 3; j < w - 1; j += 4) c.line(x + j, top, x + j, top - d, "b")   // bare rafters
    } else {
      for (let j = 3 + (seed % 3); j < w - 2; j += 6) c.line(x + j, top - 1, x + j + 2, top - d + 1, "b")
    }
    wall(x, top, w, h, mat, seed)
    c.vline(x + w, top - d + 1, h + d - 1, "k"); c.vline(x + w + 1, top - d + 3, h + d - 3, "s")
    if (chimney >= 0) {
      const cx = x + 1 + Math.round(chimney * (w - 5))
      c.rect(cx, top - d - 8, 3, d + 9, "g"); c.vline(cx, top - d - 8, d + 9, "G"); c.hline(cx - 1, top - d - 8, 5, "G")
    }
    for (const [wx, kind] of win) {                     // openings: dark, or lit by a fire inside
      const lit = kind === "fire", X = x + wx
      if (kind === "door") c.rect(X, gy - 6, 3, 6, "k")
      else c.rect(X, top + 3, 2, 3, lit ? (fl(3, X) > 0 ? "y" : "q") : "k")
      if (lit) windowFire(X, top + 3, X * 0.7)
    }
  }
  function church(x, gy) {
    const w = 40, h = 14, d = 8, top = gy - h
    c.rect(x + 1, top - d, w - 1, d + 1, "k"); c.hline(x + 1, top - d, w - 1, "s")
    for (const j of [6, 13, 27, 34]) c.line(x + j, top, x + j + 2, top - d + 1, "b")
    wall(x, top, w, h, "stone", 41)
    c.vline(x + w, top - d + 1, h + d - 1, "k")
    for (const bx of [0, 10, 20, 30, 39]) { c.rect(x + bx, top + 2, 2, h - 2, "g"); c.px(x + bx, top + 2, "G") }  // buttresses
    for (const lx of [5, 15, 25, 34]) {                // lancet windows, the nave burning inside
      c.rect(x + lx, top + 4, 2, 5, fl(2, lx) > -0.3 ? "q" : "y"); c.px(x + lx, top + 3, "q"); c.px(x + lx + 1, top + 3, "q")
    }
    c.rect(x + 18, gy - 7, 4, 7, "k"); c.px(x + 19, gy - 8, "k"); c.px(x + 20, gy - 8, "k")
    // the bell tower, snapped: jagged top, a lancet, the spire's beams sticking out
    const tx = x - 9, th = 40
    for (let i = 0; i < 11; i++) {
      const brk = [12, 9, 6, 4, 5, 8, 11, 15, 17, 19, 20][i]
      for (let y = gy - th + brk; y < gy; y++) c.px(tx + i, y, (y - gy) % 3 === 0 || i === 10 ? "g" : i < 2 ? "G" : "S")
      c.px(tx + i, gy - th + brk, "G")
    }
    c.rect(tx + 4, gy - 30, 2, 5, "k"); c.px(tx + 4, gy - 31, "k")
    c.line(tx + 6, gy - 35, tx + 15, gy - 44, "b"); c.line(tx + 3, gy - 34, tx - 2, gy - 41, "b"); c.line(tx + 5, gy - 34, tx + 9, gy - 47, "b")
  }
  function barn(x, gy) {                               // burnt down to its posts and rafters
    const w = 32, h = 12
    c.rect(x, gy - 3, w, 3, "s")
    for (let i = 0; i <= w; i += 8) c.vline(x + i, gy - h + (i === 16 ? 0 : 2), h - (i === 16 ? 0 : 2), "b")
    c.line(x, gy - h + 2, x + 16, gy - h - 7, "b"); c.line(x + 16, gy - h - 7, x + 26, gy - h - 1, "b")
    c.hline(x, gy - h + 2, w + 1, "b")
    beamFire(x + 26, gy - h - 1, x + 33, gy - 1, 5)
    smoulder(x + 4, gy, 12, 6)
  }
  const well = (x, y) => {
    c.ellipse(x, y, 5, 2, "g"); c.ellipse(x, y, 3, 1, "k"); c.hline(x - 5, y, 11, "G")
    c.vline(x - 4, y - 7, 7, "b"); c.vline(x + 4, y - 7, 7, "b"); c.hline(x - 5, y - 7, 11, "b")
  }
  const cart = (x, y) => {                             // overturned, one wheel in the air
    c.rect(x, y - 4, 12, 4, "b"); c.hline(x, y - 4, 12, "G"); c.hline(x + 1, y - 2, 10, "g")      // bed on its side
    c.line(x + 12, y - 2, x + 18, y + 1, "b")                                                 // shaft
    c.ring(x + 3, y - 7, 3, "G"); c.px(x + 3, y - 7, "b"); c.px(x + 3, y - 9, "g"); c.px(x + 1, y - 7, "g")  // wheel
  }
  const fence = (x0, y0, x1, y1) => {
    const n = Math.round(Math.hypot(x1 - x0, y1 - y0) / 4)
    for (let i = 0; i <= n; i++) {
      const x = Math.round(x0 + ((x1 - x0) * i) / n), y = Math.round(y0 + ((y1 - y0) * i) / n)
      if (i % 5 !== 3) c.vline(x, y - 3, 4, "b")      // a few posts burnt away
    }
    c.line(x0, y0 - 2, x1, y1 - 2, "s")
  }
  const tree = (x, y) => {                             // burnt, bare
    c.line(x, y, x, y - 14, "s"); c.line(x + 1, y, x + 1, y - 9, "k")
    c.line(x, y - 8, x - 5, y - 13, "s"); c.line(x, y - 11, x + 4, y - 17, "s"); c.line(x - 3, y - 11, x - 4, y - 15, "s")
    c.line(x + 2, y - 14, x + 6, y - 15, "s")
  }
  function haystack(x, y) {
    c.ellipse(x, y - 3, 7, 4, "h"); c.clip((px, py) => py > y - 2, () => c.ellipse(x, y - 3, 7, 4, "H"))
    c.hline(x - 7, y, 15, "s")
    for (let j = -5; j <= 5; j += 2) tongue(x + j, y - 4 - Math.round(2 * Math.cos(j / 4)), 5 + 4 * Math.cos(j / 3), 3, 1 + (Math.abs(j) % 3), j + 30)
    lights.push([x, y - 5, 12])
  }

  // back row
  tree(250, 46)
  house({ x: 12, gy: 50, w: 22, h: 10, mat: "stone", chimney: 0, win: [[5, "fire"], [14, "door"]], seed: 1 })
  house({ x: 40, gy: 48, w: 26, h: 10, mat: "timber", roof: 0.4, win: [[18, "fire"]], seed: 2 })
  church(96, 48)
  barn(150, 50)
  house({ x: 190, gy: 46, w: 20, h: 9, mat: "stone", chimney: 1, win: [[4, "dark"]], seed: 3 })
  house({ x: 216, gy: 50, w: 24, h: 10, mat: "timber", roof: 0.5, win: [[16, "fire"]], seed: 4 })
  blaze(...FIRES[1], 23)
  // middle row
  fence(0, 60, 22, 58)
  house({ x: 6, gy: 72, w: 24, h: 9, mat: "timber", roof: 0.6, win: [[18, "fire"]], seed: 5 })
  house({ x: 66, gy: 66, w: 20, h: 9, mat: "stone", win: [[12, "door"]], seed: 6 })
  blaze(...FIRES[0], 11)
  smoulder(92, 70, 9, 12)
  house({ x: 120, gy: 68, w: 26, h: 10, mat: "timber", roof: 0.45, chimney: 1, seed: 7 })
  house({ x: 196, gy: 72, w: 26, h: 10, mat: "stone", win: [[20, "dark"]], seed: 8 })
  blaze(...FIRES[2], 37)
  // front row
  well(60, 84)
  house({ x: 26, gy: 94, w: 18, h: 7, mat: "stone", win: [[6, "dark"]], seed: 9 })
  house({ x: 88, gy: 94, w: 24, h: 8, mat: "timber", roof: 0.55, win: [[4, "fire"], [16, "door"]], seed: 10 })
  cart(126, 88)
  haystack(150, 92)
  beamFire(170, 96, 184, 90, 13)
  fence(186, 96, 236, 90)
  smoulder(232, 84, 8, 14)

  // ---- light: faces turned toward a fire step up, in hard bands; each fire breathes a little
  const lit = []
  for (const [fx, fy, rad] of lights) {
    const R2 = rad * (0.95 + 0.05 * fl(2, fx))
    for (let y = Math.floor(fy - R2); y <= fy + R2; y++) for (let x = Math.floor(fx - R2); x <= fx + R2; x++) {
      const d = Math.hypot(x - fx, y - fy)
      if (d > R2 || !c.is(x, y, "sStgGb")) continue
      if (c.is(x + Math.sign(Math.round(fx - x)), y + Math.sign(Math.round(fy - y)), ".k123")) lit.push([x, y, d / R2])
    }
  }
  lit.forEach(([x, y, d]) => c.px(x, y, d < 0.3 ? "M" : d < 0.6 ? "L" : "B"))
})

// ---------------------------------------------------------------- smoke puffs
// 3 x 24 frames: shade, rim and body of a puff at height n/23 of its climb (round and fire-lit low,
// wider and thinning out high up). The game draws every puff's shade (offset down-left), then every
// rim (offset up-right), then every body, so the light only shows on the column's outer edge.
pal({ s: "#0f0c15", S: "#1c1622", T: "#2e2532", w: "#3a1e22", W: "#5e2e20", b: "#1e1820", B: "#241a22", C: "#2c1c22" })
anim("title_smoke", { w: 56, h: 56, frames: 72, fps: 1 }, (c) => {
  const v = Math.floor(c.frame / 24), t = (c.frame % 24) / 23, r = 3 + 22 * t ** 0.85, cx = 28, cy = 28
  const lobes = [[0, 0, 1], [-0.45, 0.25, 0.72], [0.45, 0.2, 0.66], [0.1, -0.32, 0.6]]
  const key = v === 0 ? (t < 0.12 ? "W" : t < 0.3 ? "w" : "s") : v === 1 ? "T" : t < 0.15 ? "C" : t < 0.4 ? "B" : t < 0.7 ? "b" : "S"
  for (let y = 0; y < 56; y++) for (let x = 0; x < 56; x++)
    if (lobes.some(([ox, oy, s]) => Math.hypot(x - (cx + ox * r), y - (cy + oy * r)) <= r * s + 0.3)) c.px(x, y, key)
  if (t > 0.4) c.dither(0, 0, 56, 56, ".", "_", Math.min(0.88, ((t - 0.4) / 0.5) * 0.95))
})

// ---------------------------------------------------------------- foreground crest
// The viewer's hill: near-black, its ridge caught by the fire light from the valley on the right,
// dry grass, a small grave mound and a leaning wooden cross. Bottom-left of the screen.
pal({
  k: "#050409", K: "#0a0810", s: "#130e18", r: "#2c1418", R: "#4a1e1a",       // ground, rim light
  g: "#1a121c", G: "#33181a",                                                 // grass
  w: "#0c090d", x: "#1e1418", X: "#3e1e1a", Z: "#6a2e20",                     // cross wood: dark -> fire-lit
  p: "#2a2026",                                                               // rope
})
const CW = 300, CH = 170
const crest = (x) => 100 + 0.0012 * x * x + 1.5 * Math.sin(x / 9) + Math.sin(x / 4.3)
sprite("title_crest", { w: CW, h: CH }, (c) => {
  for (let x = 0; x < CW; x++) {
    const y = Math.round(crest(x)); if (y >= CH) continue
    c.vline(x, y, CH - y, "k"); c.vline(x, y + 1, 3, "K")
    c.px(x, y, x > 110 ? "R" : x > 60 ? "r" : "s")
  }
  // grave mound under the cross and a broken headstone further right
  c.ellipse(36, 104, 16, 5, "K"); c.hline(25, 99, 22, "s"); c.hline(40, 100, 8, "r")
  c.rect(92, 104, 5, 7, "K"); c.px(92, 104, "."); c.vline(96, 105, 6, "r"); c.hline(93, 104, 3, "s")
  // dry grass tufts along the ridge, a few blades catching the light
  const rr = rnd(9)
  for (let k = 0; k < 150; k++) {
    const x = Math.floor(rr() * CW), y = Math.round(crest(x)); if (y >= CH - 2) continue
    const h = 2 + Math.floor(rr() * 5), lean = rr() < 0.5 ? -1 : 1
    c.line(x, y, x + lean * Math.round(h / 3), y - h, x > 70 && rr() < 0.5 ? "G" : "g")
  }
  // leaning cross: a post and a bar as rotated boxes, lit on the right edges by the fire
  const ang = -0.17, ca = Math.cos(ang), sa = Math.sin(ang), bx = 34, by = 104
  const at = (u, v) => [bx + u * ca - v * sa, by + u * sa + v * ca]           // u across, v up (negative)
  const quad = (u0, v0, u1, v1, k) => c.poly([...at(u0, v0), ...at(u1, v0), ...at(u1, v1), ...at(u0, v1)], k)
  quad(-3, 6, 3, -84, "w"); quad(-18, -56, 18, -62, "w")
  quad(1, 6, 3, -84, "x"); quad(-18, -56, 18, -57.5, "x"); quad(16, -56, 18, -62, "X")
  quad(2, -30, 3, -84, "X"); quad(2, -79, 3, -84, "Z"); quad(10, -61, 18, -62, "X")
  quad(-4, -54, 4, -64, "p"); quad(-4, -58, 4, -59, "w")
  c.clip((x, y) => y > 98, () => c.recolor({ x: "w", X: "x" }))               // buried end in shade
})
