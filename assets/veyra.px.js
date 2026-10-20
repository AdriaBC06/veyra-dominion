// Veyra, demoness / succubus (PLAN.md 7.6, V12).
// Lounging on the air as if sitting on an invisible throne: face to the viewer, legs crossed with
// the knees to the right, one hand resting behind her, the other open, wings spread on both sides.
// She faces left by default (the game flips by facing).
//
// Every exported layer is a 66x50 strip frame (the 64x48 drawing plus a 1px ink margin), so the
// layers stack at the same origin; pivot = (31, 36), under her hips. Back to front: wings, tail,
// hair, body. Body states: idle, move, cast, hurt; death is one composite strip of every layer.
// Parts ("_name") are drawn without ink; the exported strips add it with ink() below.

const W = 64, H = 48, TAU = Math.PI * 2

// Colours sampled from the reference sheet ("Paleta de colores") and the full-body image.
pal({
  k: "#16091a",                                            // outline, near-black plum
  S: "#7c2f2b", s: "#b44c3f", z: "#d8705a", Z: "#efa07e",  // skin: red-brown
  H: "#1d1035", h: "#36215e", j: "#5a3c8e",                // hair: deep violet
  R: "#5c0c1c", r: "#a8182e", q: "#e04454",                // horns + tail: crimson
  W: "#2e0c2a", w: "#5c1c4a", v: "#7e2c64",                // wings: wine membrane, dark bones
  B: "#120b18", b: "#271a33", n: "#43304f",                // suit: black leather, sheen
  P: "#2c1042", p: "#4c1e6a", m: "#74309a",                // skirt: purple
  U: "#2e1a3a", T: "#4a2a58", t: "#74508a",                // stockings, sleeves, boots: plum
  G: "#8a5a1c", g: "#d8a040", y: "#f8dc80",                // gold trim
  e: "#e81c40", o: "#f8b828", O: "#fff0a0",                // gem red, eyes gold
  x: "#ff5040", f: "#ff9a3c", F: "#ffe08a",                // hellfire rim / cast glow
})
ramps("kSszZ", "kHhj", "kRrq", "kWwv", "kBbn", "kUTt", "kPpm", "kGgy")

const lerp = (p, q, t) => [Math.round(p[0] + (q[0] - p[0]) * t), Math.round(p[1] + (q[1] - p[1]) * t)]
const mirror = (pts, cx) => pts.map(([x, y]) => [2 * cx - x, y])

// ------------------------------------------------------------------ hair
// One connected mass: crown, then a long curtain falling behind the shoulders to the hips, ending
// in pointed locks. Written as the right half and mirrored around the face centre (x = 26).
const CX = 26
const HAIR_R = [[26, 4], [30, 5], [33, 7], [35, 10], [35, 14], [36, 18], [37, 23], [38, 28], [38, 32],
  [39, 36], [36, 33], [36, 37], [33, 33], [26, 32]]
const STRANDS_R = [[[28, 6], [32, 11], [34, 20], [36, 30], [37, 34]], [[30, 9], [33, 17], [35, 26], [35, 35]]]

// sway -1..1 swings the locks; flow 0..1 streams them back (to the right) when she flies
function hairShape(sway = 0, flow = 0) {
  const bend = ([x, y]) => {
    const t = Math.max(0, (y - 14) / 22)
    return [Math.round(x + sway * 1.5 * t * t + flow * 5 * t), Math.round(y - flow * 2 * t)]
  }
  const right = HAIR_R.map(bend)
  const left = mirror(HAIR_R, CX).map(bend).reverse().slice(1, -1)
  const strands = [...STRANDS_R, ...STRANDS_R.map((s) => mirror(s, CX))].map((s) => s.map(bend))
  return { outline: [...right, ...left], strands }
}
function hair(c, sway = 0, flow = 0) {
  const { outline, strands } = hairShape(sway, flow)
  c.poly(outline.flat(), "h")
  strands.forEach((s, i) => { for (let k = 1; k < s.length; k++) c.line(...s[k - 1], ...s[k], i % 2 ? "j" : "H") })
  edges(c)
}

// point in the resting hair silhouette (ray casting)
const HAIR_REST = hairShape().outline
function inHair(x, y) {
  let inside = false
  for (let i = 0, j = HAIR_REST.length - 1; i < HAIR_REST.length; j = i++) {
    const [xi, yi] = HAIR_REST[i], [xj, yj] = HAIR_REST[j]
    if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) inside = !inside
  }
  return inside
}

// ------------------------------------------------------------------ head
// Nearly frontal, proportioned to the body rather than detailed: small almond eyes (a dark lash
// line over gold), pointed ears sticking out sideways, a small smile. Left half mirrored, so the
// face is symmetric, and the eyes have no pupil to drift: she can't look cross-eyed.
// Horns grow from the sides of the crown, curve out and up.
sprite("_head", { mx: "c" }, `
  q.......
  rq......
  Rrq.....
  .Rrq.HHH
  ..RrHhhh
  ...Hhhjh
  ..Hhhjhh
  ..HhjhhH
  .HhhhHhh
  .HhHsHss
  zHhkkkss
  SzHsooss
  .HhszssS
  .Hhhssss
  .Hhhsssr
  ..HhSsss
  ..Hh..Ss`)
const HX = 19, HY = 1   // the head never moves between body frames: hair and horns stay aligned

// ------------------------------------------------------------------ body
// Slender build: pinched waist, bust and hips wider than it, long thin legs and arms.

// thigh skin to a gold garter, plum stocking over the knee and shin, heeled boot with a gem
function leg(c, hip, knee, ankle, toe, far = false) {
  const g = lerp(hip, knee, 0.5)
  c.line(...hip, ...g, "s", 4); c.line(...g, ...knee, "T", 3)
  c.line(...knee, ...ankle, "T", 2)
  c.line(g[0], g[1] - 2, g[0], g[1] + 1, "g"); c.px(g[0], g[1] + 1, "e")
  // boot: instep down to a pointed toe, stiletto heel under the ankle, gold band with a gem
  const mid = lerp(ankle, toe, 0.5)
  c.line(...ankle, ...mid, "T", 2); c.line(...mid, ...toe, "t")
  c.line(ankle[0] - 1, ankle[1] + 1, ankle[0] - 1, ankle[1] + 3, "B")
  c.px(ankle[0] - 1, ankle[1] - 1, "g"); c.px(ankle[0], ankle[1] - 1, "e")
  if (far) c.clip((x, y) => x > hip[0] + 2 || y > hip[1] + 2, () => c.recolor({ T: "U", t: "T", s: "S", B: "k" }))
}

// arm: bare shoulder, sheer plum sleeve with a gold band, gold cuff, hand
function arm(c, sh, el, hand) {
  const band = lerp(sh, el, 0.5)
  c.line(...sh, ...band, "s", 2); c.line(...band, ...el, "T", 2); c.line(...el, ...hand, "T", 1)
  c.px(...band, "g")
  const cuff = lerp(el, hand, 0.75); c.px(cuff[0], cuff[1], "g")
  c.rect(hand[0] - 1, hand[1] - 1, 2, 2, "s")
}

// torso: neck, sloping shoulders, bust, pinched waist, hips flaring to the seat
const TORSO = [25, 17, 28, 17, 29, 19, 32, 20, 34, 22, 34, 25, 31, 27, 30, 29, 32, 31, 36, 34,
  30, 36, 22, 35, 21, 32, 23, 29, 22, 27, 19, 25, 19, 22, 21, 20, 24, 19]
const SUIT = (x, y) => y >= 23 + (Math.abs(x - 26.5) < 1.5 ? 1 : 0)

// Legs at rest (crossed, heels hanging) and streamlined (trailing back while she flies). Both
// thighs come out of the leotard under the belt; the near one crosses over the skirt.
const LEGS = {
  far:  { hip: [29, 35], rest: [[44, 34], [47, 43], [49, 47]], lean: [[44, 36], [54, 40], [59, 41]] },
  near: { hip: [31, 33], rest: [[46, 27], [50, 38], [53, 44]], lean: [[46, 30], [56, 34], [61, 35]] },
}
const legAt = (l, lean, kick) => {
  const [k, a, t] = l.rest.map((p, i) => lerp(p, l.lean[i], lean))
  return [l.hip, k, [a[0] + Math.round(kick), a[1]], [t[0] + Math.round(kick * 1.5), t[1] - Math.round(kick * 0.5)]]
}

// pose: lean 0..1 streamlines her for movement; kick swings the near heel (px); flutter pushes
// the skirt points back (px); cast 0..2 raises her open hand with a growing hellfire orb.
function body(c, pose = {}) {
  const { lean = 0, kick = 0, flutter = 0, cast } = pose
  // skirt flares below the hips toward the back, tattered points
  const f = Math.round(flutter), f2 = Math.round(flutter / 2)
  c.poly([21, 31, 33, 31, 37 + f, 35, 33 + f, 39, 30 + f, 44, 27 + f2, 40, 23 + f2, 45, 20 + f2, 40, 16, 43 - f2, 17, 37], "p")
  c.line(22, 33, 20 + f2, 41, "P"); c.line(28, 34, 27 + f2, 42, "P"); c.line(33, 33, 32 + f, 38, "m")
  // far leg, then torso, then the near leg crossed over it
  leg(c, ...legAt(LEGS.far, lean, 0), true)
  c.poly(TORSO, "s")
  c.clip(SUIT, () => { c.poly(TORSO, "b"); c.line(24, 26, 24, 30, "n"); c.line(29, 27, 29, 29, "n") })
  // corset: gold sweetheart neckline, gem at the centre, V belt, choker
  for (const [a, b] of [[[19, 23], [21, 22]], [[21, 22], [24, 22]], [[24, 22], [26, 24]], [[27, 24], [29, 22]], [[29, 22], [32, 22]], [[32, 22], [34, 23]]])
    c.line(...a, ...b, "g")
  c.px(26, 26, "e"); c.px(26, 27, "g")
  c.line(23, 30, 26, 33, "g"); c.line(31, 30, 28, 33, "g"); c.px(27, 33, "e")
  c.px(26, 20, "S"); c.px(26, 21, "S")                             // cleavage
  c.hline(25, 18, 3, "g"); c.px(26, 18, "e")                       // choker
  leg(c, ...legAt(LEGS.near, lean, kick))
  // arms: her right hand rests on the air behind her; the left one reaches out, palm open
  arm(c, [20, 21], [18, 28], [17, 34])
  if (cast === undefined) arm(c, [32, 21], [38, 24], [44, 19])
  else {
    const hand = [40, 11 - cast * 2]
    arm(c, [32, 21], [37, 18 - cast], hand)
    const r = cast + 1                                              // hellfire orb in her palm
    c.circ(hand[0] + 1, hand[1] - r - 1, r, "f"); c.circ(hand[0] + 1, hand[1] - r - 1, Math.max(0, r - 1), "F")
    if (cast === 2) for (const [dx, dy] of [[-4, -4], [5, -4], [-3, 1], [5, 0], [1, -8]])
      c.px(hand[0] + 1 + dx, hand[1] - r - 1 + dy, "f")             // sparks
  }
  c.stamp("_head", HX, HY)
  edges(c)
}

// Top edges catch the light; the bottom edge takes a subtle red rim from the hellfire below.
const TOP = { s: "z", b: "n", B: "b", T: "t", p: "m", P: "p", h: "j", H: "h", W: "w", w: "v" }
const RIM = { s: "S", z: "s", b: "R", B: "R", U: "R", p: "r", P: "R", h: "R", H: "R", w: "r", W: "R" }
function edges(c) {
  const top = [], bot = []
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) {
    if (!c.is(x, y, "*")) continue
    if (!c.is(x, y - 1, "*")) top.push([x, y])
    else if (!c.is(x, y + 1, "*")) bot.push([x, y])
  }
  for (const [x, y] of top) { const k = TOP[c.key(x, y)]; if (k) c.px(x, y, k) }
  for (const [x, y] of bot) { const k = RIM[c.key(x, y)]; if (k) c.px(x, y, k) }
}

// ------------------------------------------------------------------ ink
// 1px outline into the margin. `skip` leaves some edges unlined: the body skips its head where it
// meets the hair layer, so the head hair and the curtain behind read as one mass with no seam.
function ink(c, skip) {
  const out = []
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++)
    if (!c.is(x, y, "*") && (c.is(x - 1, y, "*") || c.is(x + 1, y, "*") || c.is(x, y - 1, "*") || c.is(x, y + 1, "*")) && !(skip && skip(x, y)))
      out.push([x, y])
  for (const [x, y] of out) c.px(x, y, "k")
}
const HEAD_SEAM = (x, y) => y <= HY + 18 && inHair(x - 1, y - 1)

// Export a part anim as an inked strip: one frame per duration, drawing shifted into the margin.
function strip(name, part, ms, { loop = true, skip } = {}) {
  anim(name, { w: W + 2, h: H + 2, loop },
    ...ms.map((d, i) => frame({ ms: d }, (c) => { c.stamp(part, 1, 1, { frame: i }); ink(c, skip) })))
}

// ------------------------------------------------------------------ body states
sprite("_body_rest", { w: W, h: H }, (c) => body(c))

// Idle float: the crossed heel dangles and swings while the skirt points sway.
const IDLE = [[0, 0], [1, 0.5], [2, 1], [1, 0.5]]   // kick px, flutter px
anim("_idle", { w: W, h: H, frames: 4 }, (c) => body(c, { kick: IDLE[c.frame][0], flutter: IDLE[c.frame][1] }))
strip("veyra_idle", "_idle", [200, 200, 200, 200], { skip: HEAD_SEAM })

// Move lean: legs trail back, the skirt streams behind, the heel kicks with each wing beat.
anim("_move", { w: W, h: H, frames: 4 }, (c) =>
  body(c, { lean: 1, kick: [0, 1, 2, 1][c.frame], flutter: [3, 4, 5, 4][c.frame] }))
strip("veyra_move", "_move", [125, 125, 125, 125], { skip: HEAD_SEAM })

// Cast: the open hand rises and a hellfire orb grows in her palm, then flares.
anim("_cast", { w: W, h: H, frames: 3 }, (c) => body(c, { cast: c.frame }), glow("fF", { r: 4, strength: 0.6, bands: 3 }))
strip("veyra_cast", "_cast", [80, 80, 160], { loop: false, skip: HEAD_SEAM })

// Hurt: patches over the resting sprite. Frame 0 flashes hot (two steps lighter) with the eyes
// squeezed shut (gold covered, lash line kept); frame 1 keeps the squint at normal colours.
const SHUT = grid(HX + 4, HY + 11, "ss...ss")
anim("_hurt", { base: "_body_rest" }, frame(light(undefined, 2), SHUT), frame(SHUT))
strip("veyra_hurt", "_hurt", [60, 120], { loop: false, skip: HEAD_SEAM })

// ------------------------------------------------------------------ hair, tail, wings
anim("_hair", { w: W, h: H, frames: 4 }, (c) => hair(c, Math.sin(c.phase * TAU)))
strip("veyra_hair", "_hair", [200, 200, 200, 200])
anim("_hair_move", { w: W, h: H, frames: 4 }, (c) => hair(c, Math.sin(c.phase * TAU), 1))
strip("veyra_hair_move", "_hair_move", [125, 125, 125, 125])

// Crimson tail from the small of her back, arching up the right side over the wing to a spade.
sprite("_spade", { mx: "c" }, `
  ...q
  ..rr
  .rrr
  Rrrr
  .R.r`)
function tail(c, wag = 0) {
  const pts = []
  for (let i = 0; i <= 12; i++) {
    const t = i / 12
    pts.push([34 + 12 * t + Math.sin(t * Math.PI) * 5 + wag * t * t * 2, 32 - 18 * t + Math.sin(t * Math.PI * 2) * 1.5])
  }
  for (let i = 1; i < pts.length; i++)
    c.line(...pts[i - 1].map(Math.round), ...pts[i].map(Math.round), "r", i < 7 ? 2 : 1)
  const end = pts[pts.length - 1].map(Math.round)
  c.stamp("_spade", end[0] - 3, end[1] - 4)
  edges(c)
}
anim("_tail", { w: W, h: H, frames: 4 }, (c) => tail(c, Math.sin(c.phase * TAU)))
strip("veyra_tail", "_tail", [200, 200, 200, 200])

// Bat wing: arm bone from the shoulder up to the wrist, three fingers fanning out, scalloped
// membrane. One definition (her right wing, our left); the other is its mirror around x = 28.
const WING = { root: [22, 19], wrist: [12, 6], tips: [[1, 4], [0, 15], [7, 25]], attach: [19, 26] }
function wing(c, w, deg, mirrorX = 0) {
  const a = (deg * Math.PI) / 180, [ox, oy] = w.root
  const rot = ([x, y]) => {
    const p = [ox + (x - ox) * Math.cos(a) - (y - oy) * Math.sin(a), oy + (x - ox) * Math.sin(a) + (y - oy) * Math.cos(a)]
    return [Math.round(mirrorX ? 2 * mirrorX - p[0] : p[0]), Math.round(p[1])]
  }
  const R = rot(w.root), Wr = rot(w.wrist), T = w.tips.map(rot), A = rot(w.attach)
  const scallop = (p, q) => lerp(lerp(p, q, 0.5), Wr, 0.3)
  const edge = [T[0], scallop(T[0], T[1]), T[1], scallop(T[1], T[2]), T[2], scallop(T[2], A), A]
  c.poly([...R, ...Wr, ...edge.flat()], "w")
  c.poly([...Wr, ...T[0], ...scallop(T[0], T[1]), ...T[1]], "v")     // lit outer panel
  c.line(...R, ...Wr, "W", 2)
  for (const t of T) c.line(...Wr, ...t, "W")
  c.px(Wr[0], Wr[1] - 1, "R")                                         // thumb claw
}
const FLAP = [0, 10, 18, 9]   // degrees: a slow, lazy beat
anim("_wings", { w: W, h: H, frames: 4 }, (c) => {
  const f = FLAP[c.frame]
  wing(c, WING, -f, 28)        // far wing (her left), in her shadow
  shade(); shade()
  wing(c, WING, f)             // near wing
  edges(c)
})
strip("veyra_wings", "_wings", [200, 200, 200, 200])

// ------------------------------------------------------------------ death
// Death dissolve: every layer at rest, burning away from the bottom up. Pixels on the burning
// front turn to embers, burnt ones are erased, and a few sparks rise from them.
const noise = (x, y) => { const v = Math.sin(x * 12.9898 + y * 78.233) * 43758.5453; return v - Math.floor(v) }
function dissolve(c, i) {
  const th = (i / 5) * 1.15, burnt = [], embers = [], sparks = []
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) {
    if (!c.is(x, y, "*")) continue
    const v = 0.35 * noise(x, y) + 0.65 * (1 - y / c.h)
    if (v < th - 0.12) { burnt.push([x, y]); if (noise(y, x) < 0.05) sparks.push([x, y - i * 3]) }
    else if (v < th) embers.push([x, y, v < th - 0.06 ? "f" : "x"])
  }
  for (const [x, y] of burnt) c.px(x, y, "_")
  for (const [x, y, k] of embers) c.px(x, y, k)
  for (const [x, y] of sparks) if (!c.is(x, y, "*")) c.px(x, y, noise(x, i) < 0.5 ? "F" : "f")
}
anim("veyra_death", { w: W + 2, h: H + 2, frames: 6, fps: 7, loop: false },
  ...["veyra_wings", "veyra_tail", "veyra_hair", "veyra_hurt"].map((name) => stamp(name, 0, 0, { frame: name === "veyra_hurt" ? 1 : 0 })),
  (c) => dissolve(c, c.frame))

// ------------------------------------------------------------------ review
// Every state stacked with its layers, as the game draws them.
const STACK = (state, hairName = "veyra_hair", x = 0) =>
  ["veyra_wings", "veyra_tail", hairName, state].map((name) => stamp(name, x, 0))
scene("veyra_composite", { w: W + 2, h: H + 2 }, ...STACK("veyra_idle"))
