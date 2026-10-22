// Icons for every Art, evolution, union and Sin (design.md 7.7, V17c), shown on the
// level-up / Offering cards (2x) and the HUD (1x). Build:
//   node .claude/skills/pxl-pixel-art/bin/pxl.js build assets/icons.px.js --out assets/sprites/icons --no-preview
// Each icon is 14x14 plus the outline (16x16). The `icons` sheet is one strip in the
// order `assets::Icon::frame` expects: Arts, evolutions (Arts that have one), unions, Sins.

pal({
  k: "#16091a",                                            // outline, Veyra's plum ink
  X: "#6a1410", x: "#c8301c", f: "#ff9a3c", F: "#ffe08a",  // brimstone
  R: "#5c0c1c", r: "#a8182e", q: "#e04454", Q: "#ff8a8a",  // crimson
  H: "#7a1238", h: "#d42a5c", i: "#ff6f9a", I: "#ffd0dc",  // heart pink
  o: "#7a4a10", y: "#d89a28", Y: "#ffd65a",                // gold
  s: "#3a3f52", S: "#7a8296", w: "#d8dce8", W: "#ffffff",  // steel, bone
  p: "#2e1238", P: "#6a2a7a", v: "#a85ac8", V: "#e0a8f0",  // abyss purple
  d: "#2a1020",                                            // deep shadow
  e: "#1e5a3a", E: "#3fae6a", n: "#a8f0c0",                // envy green
  b: "#4a2a18", B: "#8a5a30",                              // wood, leather
  u: "#1f4aa8", c: "#6cc8ff", C: "#dff6ff",                // soul blue (the XP souls)
  m: "#5a1a4e", M: "#8c3478", j: "#c860a8",                // Veyra's wings
})
ramps("XxfF", "RrqQ", "HhiI", "oyY", "sSwW", "pPvV", "ebB", "ucC", "mMj")

const N = 14
const icon = (name, ...items) => sprite("ic_" + name, { w: N, h: N, outline: "k" }, ...items)
const half = (name, ...items) => sprite("ic_" + name, { w: N, h: N, outline: "k", mx: true }, ...items)
// Symmetric about one centre column: drawn 13 wide (write the left half and column 6),
// then `extras` (light from the top left, details that break the mirror) on the icon.
function sym(name, items, extras = []) {
  sprite("_sym_" + name, { w: N - 1, h: N, mx: "c" }, ...items)
  icon(name, stamp("_sym_" + name, 0, 0), ...extras)
}
// Light comes from the top left on every icon: darken one step right of column `x`.
const shadeRight = (x) => shade((px) => px > x)

// --- Shared shapes ----------------------------------------------------------

// Devil tail: an S curve from the bottom left up to a spade on the top right.
function tail(c, dark, mid, light, barbs) {
  c.grid(0, 0, `
    ..........q...
    .........qQq..
    ........qQqqq.
    .......qqqqqqq
    .......qqq.qqq
    ..........rr..
    .........rr...
    ....rrrrrr....
    ...rrrrrr.....
    ..rr..........
    ..rr..........
    ..rrr.........
    ...rrrrr......
    .....RRRR.....`.replaceAll("q", mid).replaceAll("Q", light).replaceAll("r", mid).replaceAll("R", dark))
  if (barbs) for (const [x, y] of [[1, 9], [1, 11], [5, 6], [8, 9], [9, 12], [11, 5]]) c.px(x, y, barbs)
}

// Teardrop flame, tip up, base centred on (cx, base).
function flame(c, cx, base, h, w, cols) {
  for (let y = 0; y < h; y++) {
    const t = y / (h - 1)
    const hw = t < 0.65 ? w * Math.pow(t / 0.65, 0.9) : w * Math.sqrt(1 - Math.pow((t - 0.65) / 0.35, 2) * 0.6)
    for (let x = Math.floor(cx - hw); x <= Math.ceil(cx + hw); x++) {
      const d = Math.abs(x - cx) / Math.max(hw, 0.5)
      if (d > 1.05) continue
      const k = d > 0.7 ? cols[0] : t > 0.45 && d < 0.4 ? cols[2] : cols[1]
      c.px(x, base - h + 1 + y, k)
    }
  }
}

const HEART9 = `
  .hhh.hhh.
  hiIhhhhhh
  hIihhhhhh
  hhhhhhhhH
  .hhhhhhH.
  ..hhhhH..
  ...hhH...
  ....H....`

// Perfume bottle with a haze curling off it.
function bottle(c, glass, liquid, shine, haze) {
  c.rect(5, 0, 4, 2, "y")
  c.px(5, 0, "o")
  c.rect(6, 2, 2, 2, "w")
  c.poly([3, 6, 5, 4, 9, 4, 11, 6, 11, 12, 10, 14, 4, 14, 3, 12], glass)
  c.poly([4, 8, 10, 8, 10, 12, 9, 13, 5, 13, 4, 12], liquid)
  c.vline(4, 6, 2, shine)
  c.px(5, 5, shine)
  for (const [x, y] of [[11, 2], [12, 1], [13, 3], [10, 0]]) c.px(x, y, haze)
}

// Scythe: a long handle from the bottom left, the blade curling left from its top.
function scythe(c, blade, edge, handle) {
  c.line(1, 13, 10, 1, handle, 2)
  c.poly([10, 0, 4, 0, 1, 2, 0, 6, 2, 3, 5, 2, 10, 2], blade)
  c.line(4, 0, 9, 0, edge)
  c.line(1, 2, 0, 5, edge)
}

// Inverted cross: long bar down, short arms near the bottom.
function cross(c, body, light, dark) {
  c.rect(6, 0, 3, 14, body)
  c.rect(2, 8, 11, 3, body)
  c.vline(6, 0, 8, light)
  c.hline(2, 8, 4, light)
  c.vline(8, 11, 3, dark)
  c.hline(9, 10, 4, dark)
}

// Ring of runes with a horned heart inside.
// Wandering Sigil: the same seal as the Art (`arts.px.js`), Lilith's glyph in a ring
// with four runes; Broken Seal cracks it, sparks flying.
function sigil(c, broken) {
  // Broken Seal runs hot: gold ring, white-hot glyph, burning cracks.
  const [ring, rune, glyph, core] = broken ? ["y", "Y", "F", "W"] : ["i", "I", "I", "W"]
  for (let y = 0; y < N; y++) for (let x = 0; x < N; x++) {
    const d = Math.hypot(x - 6.5, y - 6.5)
    if (d <= 7 && d > 6) c.px(x, y, ring)
    else if (broken && d <= 6 && d > 5) c.px(x, y, "o")
  }
  for (const [x, y] of [[6, 0], [7, 0], [13, 6], [13, 7], [6, 13], [7, 13], [0, 6], [0, 7]]) c.px(x, y, rune)
  c.grid(0, 2, `
    ...I......I...
    ...II....II...
    ....II..II....
    .....IIII.....
    ......WW......
    ....IIIIII....
    ......II......
    ......II......`.replaceAll("I", glyph).replaceAll("W", core))
  if (broken) {
    // A chip knocked out of the ring, and cracks glowing across it.
    for (const [x, y] of [[10, 1], [11, 1], [11, 2], [12, 2], [12, 3]]) c.px(x, y, "_")
    c.line(10, 3, 8, 5, "x"); c.line(8, 5, 9, 7, "x"); c.px(9, 4, "f")
    c.line(0, 8, 3, 7, "x"); c.line(3, 7, 4, 9, "x"); c.px(2, 8, "f")
    c.line(5, 13, 6, 11, "x")
    for (const [x, y] of [[13, 0], [12, 0]]) c.px(x, y, "F")
  }
}

const BAT_HALF = `
  .......
  .....P.
  P....PP
  PP..PPP
  PPPpPPP
  .PPpPPP
  .P..PPP
  ......P`

// Inverted pentagram in a ritual circle, left half of a 13 px symmetric icon.
function pentagram(ring, star) {
  return `
    ....rrr
    ..rr...
    .r.Q...
    .r.QQQ.
    r...Q.Q
    r...QQ.
    r...Q..
    r.QQ.Q.
    rQQQQQQ
    .r...Q.
    .r...Q.
    ..rr..Q
    ....rrr`.replaceAll("r", ring).replaceAll("Q", star)
}

// Diamond soul, like the XP souls on the ground, `r` px from its centre to a tip.
function soul(c, cx, cy, r) {
  for (let y = -r; y <= r; y++) {
    for (let x = -r; x <= r; x++) {
      const d = Math.abs(x) + Math.abs(y)
      if (d > r) continue
      c.px(cx + x, cy + y, d === r ? (x + y < 0 ? "c" : "u") : x + y < 0 ? "C" : "c")
    }
  }
}

// --- Arts -------------------------------------------------------------------

// Hellflare: a flame flying up and right, its tail splitting into tongues.
icon("hellflare", (c) => {
  c.poly([7, 3, 0, 13, 11, 8], "x")
  c.poly([6, 4, 0, 7, 8, 6], "x")
  c.poly([9, 7, 5, 13, 11, 8], "x")
  c.poly([7, 4, 2, 11, 10, 8], "f")
  c.circ(9, 5, 3, "x")
  c.circ(9, 5, 2, "f")
  c.px(10, 4, "F"); c.px(9, 4, "F"); c.px(10, 5, "F"); c.px(8, 6, "F")
  c.px(1, 5, "f"); c.px(4, 13, "f")
})

icon("tail_lash", (c) => tail(c, "R", "r", "Q"))

// Brimstone Halo: fireballs circling on their orbit, the nearest one biggest.
function fireball(c, cx, cy, r) {
  c.circ(cx, cy, r, "x")
  c.circ(cx - 0.5, cy - 0.5, r - 1, "f")
  c.px(cx - 1, cy - 1, "F")
  if (r > 2) { c.px(cx, cy - 1, "F"); c.px(cx - 1, cy, "F") }
  c.px(cx - 1, cy - r - 1, "x"); c.px(cx + 1, cy - r - 1, "f")
}
icon("brimstone_halo", (c) => {
  c.ellipse(7, 7, 6, 3, "X", true)
  fireball(c, 2, 5, 1.5)
  fireball(c, 12, 5, 1.5)
  fireball(c, 7, 10, 3)
})

// Allure: a pair of lips.
half("allure", (c) => c.grid(0, 3, `
  ...hhh.
  ..hiihh
  .hiIihh
  hhhhhhH
  .HHHHHH
  .hiiihh
  ..hhiih
  ...hhhh`))

// Hellgate: scorched pit with flames rising.
icon("hellgate", (c) => {
  c.ellipse(7, 10, 6, 3, "X")
  c.ellipse(7, 10, 4, 2, "d")
  flame(c, 4, 10, 7, 1.6, ["x", "f", "F"])
  flame(c, 9, 10, 10, 2, ["x", "f", "F"])
})

// Whip of Torment: a whip whose lash is crimson current, cracking at the tip.
icon("whip_of_torment", (c) => {
  // Leather grip with a gold pommel.
  c.line(1, 12, 4, 9, "b", 2)
  c.line(1, 12, 3, 10, "B")
  c.px(0, 13, "y"); c.px(1, 13, "o"); c.px(0, 12, "Y")
  const lash = [[5, 8], [5, 6], [7, 5], [7, 3], [10, 2], [11, 1], [13, 3], [12, 5], [13, 7], [12, 9]]
  for (let i = 1; i < lash.length; i++) {
    const [[x0, y0], [x1, y1]] = [lash[i - 1], lash[i]]
    c.line(x0, y0, x1, y1, i < 4 ? "q" : "Q")
  }
  for (const [x, y] of lash.slice(2, -1)) c.px(x, y, "I")
  // Crackle leaping off the lash.
  for (const [x, y] of [[3, 6], [5, 1], [9, 0], [10, 5]]) c.px(x, y, "q")
  // The crack at the tip.
  const [tx, ty] = lash[lash.length - 1]
  c.hline(tx - 2, ty + 2, 5, "q"); c.vline(tx, ty, 5, "q")
  c.px(tx - 1, ty + 2, "Q"); c.px(tx + 1, ty + 2, "Q"); c.px(tx, ty + 1, "Q"); c.px(tx, ty + 3, "Q")
  c.px(tx, ty + 2, "I")
})

icon("scent_of_sin", (c) => bottle(c, "I", "i", "W", "i"))

icon("reapers_scythe", (c) => scythe(c, "S", "w", "B"))

icon("profaned_cross", (c) => cross(c, "y", "Y", "o"))

icon("wandering_sigil", (c) => sigil(c, false))

half("bat_swarm", (c) => {
  c.grid(0, 3, BAT_HALF)
  c.px(5, 6, "Q")
})

// Black Mass: an inverted pentagram in a ritual circle.
sym("black_mass", [pentagram("r", "Q")])

// Gaze of Desire: a golden eye with a slit pupil casting its beams.
sym("gaze_of_desire", [`
  ..o.o.o
  ...oooo
  .ooWWyy
  oWWWyyk
  oWWWyyk
  .ooWWyy
  ...oooo
  .......
  ......Y
  ...y..Y
  ..y...Y
  .y....Y
  y.....Y`], [grid(5, 3, "Y")])

// Thralls: a seduced villager, a heart over the hood and hearts in the eyes.
sym("thralls", [`
  ....qq.
  ....qqq
  .....qq
  ......q
  ...HHHH
  ..HHIII
  ..HIqIq
  ..HHIII
  ...HIII
  ....hhh
  ..hhhhh
  .hhhhhh
  .hhhhhh
  .hhhhhh`], [grid(4, 0, "Q"), recolor({ h: "H", I: "i" }, (x) => x > 7)])

// --- Evolutions (Arts with one, in ARTS order) --------------------------------

// Kiss of Ruin (Hellflare): a heart of pink fire flying up and right, trailing flame.
icon("kiss_of_ruin", (c) => {
  c.poly([6, 6, 0, 13, 8, 9], "h")
  c.line(5, 6, 1, 8, "h"); c.line(7, 9, 4, 13, "h")
  c.line(6, 7, 1, 12, "i")
  c.px(0, 7, "I"); c.px(3, 13, "I")
  c.grid(4, 1, HEART9)
})

icon("barbed_scourge", (c) => tail(c, "X", "R", "q", "w"))

// Infernal Crown (Brimstone Halo): a gold crown burning.
icon("infernal_crown", (c) => {
  c.rect(1, 9, 12, 4, "y")
  c.hline(1, 9, 12, "Y")
  c.hline(1, 12, 12, "o")
  c.poly([1, 9, 1, 5, 4, 9], "y")
  c.poly([13, 9, 13, 5, 10, 9], "y")
  c.poly([4, 9, 7, 4, 10, 9], "y")
  c.px(6, 10, "q"); c.px(7, 10, "q")
  flame(c, 1, 5, 5, 1.2, ["x", "f", "F"])
  flame(c, 12, 5, 5, 1.2, ["x", "f", "F"])
  flame(c, 6.5, 4, 5, 1.4, ["x", "f", "F"])
})

// Siren's Dominion (Allure): a charmed heart bursting apart.
sym("sirens_dominion", [`
  ......I
  .I....I
  ..I....
  ..hhh..
  .hiIhh.
  .hIihh.
  .hhhhh.
  ..hhhh.
  ...hhh.
  ....hh.
  .....h.
  ..I....
  .I....I`], [recolor({ i: "h", I: "h" }, [7, 3, 6, 4]), recolor({ h: "H" }, (x, y) => x > 9 && y > 3 && y < 11)])

// Abyssal Rift (Hellgate): a burning tear in the ground.
icon("abyssal_rift", (c) => {
  c.poly([7, 0, 11, 6, 10, 11, 7, 14, 4, 11, 3, 6], "x")
  c.poly([7, 2, 9, 6, 9, 10, 7, 12, 5, 10, 5, 6], "p")
  c.poly([7, 4, 8, 7, 7, 10, 6, 7], "d")
  for (const [x, y] of [[1, 3], [12, 9], [2, 12], [13, 2]]) c.px(x, y, "f")
})

// Torment Eternal (Chain): from its first victim the chain bites everyone around.
sym("torment_eternal", [`
  q.....q
  .w....w
  ..S...S
  ...w..w
  .....ww
  ....wWw
  qSwSwqw
  ....www
  .....w.
  ...w..S
  ..S...w
  .w....S
  q.....q`], [recolor({ w: "S" }, (x, y) => x > 7 && y > 3 && y < 9)])

// Devouring Musk (Scent of Sin): the haze feeds her, a heart in the bottle.
icon("devouring_musk", (c) => {
  bottle(c, "Q", "r", "W", "q")
  c.grid(5, 8, "Q.Q\nQQQ\n.Q.")
})

// Midnight Harvest (Scythe): three moonlit blades spiralling out from her.
icon("midnight_harvest", (c) => {
  const cx = 6.5, cy = 6.5
  for (let y = 0; y < N; y++) {
    for (let x = 0; x < N; x++) {
      const dx = x + 0.5 - cx - 0.5, dy = y + 0.5 - cy - 0.5
      const r = Math.hypot(dx, dy)
      if (r < 2.2 || r > 7) continue
      for (let k = 0; k < 3; k++) {
        let t = Math.atan2(dy, dx) - (k * 2 * Math.PI) / 3 - r * 0.42
        t = ((t % (2 * Math.PI)) + 3 * Math.PI) % (2 * Math.PI) - Math.PI
        const width = 0.95 * Math.sin(((r - 2.2) / 4.8) * Math.PI) + 0.15
        if (t < 0 || t > width) continue
        c.px(x, y, t < 0.3 ? "W" : r > 5.4 ? "v" : "V")
      }
    }
  }
  c.circ(6, 6, 1, "P")
  c.px(6, 6, "v")
})

icon("heretics_brand", (c) => {
  cross(c, "x", "f", "X")
  flame(c, 7, 4, 5, 1.6, ["x", "f", "F"])
  flame(c, 3, 8, 4, 1, ["x", "f", "F"])
  flame(c, 11, 8, 4, 1, ["x", "f", "F"])
})

icon("broken_seal", (c) => sigil(c, true))

// Crimson Host (Bat Swarm): red bats with golden eyes.
half("crimson_host", (c) => {
  c.grid(0, 3, BAT_HALF.replaceAll("P", "r").replaceAll("p", "R"))
  c.px(5, 6, "Y")
})

// Witches' Sabbath (Black Mass): a golden circle, every soul flying to it.
sym("witches_sabbath", [pentagram("y", "Y")],
  [(c) => { for (const [x, y] of [[1, 1], [11, 1], [1, 12], [12, 12]]) soul(c, x, y, 1) }])

// Evil Eye (Gaze of Desire): the nazar amulet, "mal de ojo".
sym("evil_eye", [(c) => {
  c.circ(6, 6, 6, "y")
  c.circ(6, 6, 5, "u")
  c.circ(6, 6, 4, "W")
  c.circ(6, 6, 3, "c")
  c.circ(6, 6, 1, "k")
}], [px(4, 4, "C"), px(3, 5, "C"), recolor({ W: "w", u: "p" }, (x, y) => x + y > 15)])

// Infernal Court (Allure + Thralls): a crowned heart.
sym("infernal_court", [`
  ....y.y
  ....yyy
  ....yyq
  ....ooo
  ..hhh..
  .hiIhhh
  .hIihhh
  .hhhhhh
  ..hhhhh
  ...hhhh
  ....hhh
  .....hh
  ......h`], [recolor({ i: "h", I: "h" }, (x) => x > 6), recolor({ h: "H" }, (x, y) => x > 9 && y > 5)])

// --- Sins: the deadly sin's own imagery, drawn as what it gives -------------------

// Wrath (+damage): a burning sword.
sym("wrath", [`
  ......W
  .....wW
  ...f.wW
  ..fxfwW
  ..xfxwW
  ...xfwW
  ..f.xwW
  ..xfxwW
  ...xxwW
  .yyyyyy
  ......B
  .....bB
  .....bB
  ....yyY`], [recolor({ W: "w", w: "S" }, (x, y) => x > 6 && y < 9)])

// Bloodlust (faster recharge): a blood drop inside a turning arrow.
icon("bloodlust", (c) => {
  const cx = 6.5, cy = 6.5
  for (let y = 0; y < N; y++) {
    for (let x = 0; x < N; x++) {
      const dx = x - cx, dy = y - cy, r = Math.hypot(dx, dy)
      const a = Math.atan2(dy, dx)
      if (r < 4.6 || r > 6.6 || (a > -1.7 && a < -0.7)) continue
      c.px(x, y, x + y < 11 ? "q" : "r")
    }
  }
  c.poly([9, 0, 13, 3, 9, 5], "q")
  c.grid(5, 3, `
    .Q.
    .q.
    qQq
    qqr
    .r.`)
})

// Legion (+amount): a crowd of demons, one stepping to the front.
sym("legion", [`
  P...P..
  PP.PP..
  PPPPP..
  PqPqr..
  PPPPrr.
  .PPPrrr
  .PP.rFr
  .PP.rrr
  .PPrrrr
  ...rrrr
  ..rrrrr
  .rrrrrr
  .rrrrrr
  .rrrrrr`], [recolor({ r: "q" }, (x, y) => x > 3 && x < 6 && y > 3 && y < 6), recolor({ r: "R", P: "p" }, (x, y) => x > 9 && y > 3)])

// Night Wings (+speed): Veyra's bat wings spread. Light
// top edge from the claw to the root, dark bones down to each point, red scalloped rim.
sym("night_wings", [(c) => c.grid(0, 2, `
  j......
  jj.....
  MjjM...
  MmMjj..
  MmMmj..
  MmMmq..
  MmMq...
  MqMq...
  q.q....`)], [recolor({ M: "m", j: "M" }, (x, y) => x > 6 && y > 4)])

// Gluttony (+max HP, regen): a heart with a bite taken out of it.
sym("gluttony", [`
  .......
  ..qqq..
  .qQQqq.
  qQQqqqq
  qQqqqqq
  qqqqqqq
  qqqqqqq
  .qqqqqq
  ..qqqqq
  ...qqqq
  ....qqq
  .....qq
  ......q`], [recolor({ q: "r" }, (x, y) => x > 7 && y > 4), circ(11, 2, 3, "_"), circ(12, 6, 1, "_"), px(8, 2, "R"), px(9, 5, "R"), px(10, 6, "R"), px(11, 8, "R")])

// Lust (+soul pickup): the female (pink) and male (blue) symbols interlocked.
icon("lust", (c) => {
  c.ring(5, 8, 3, "i")
  c.line(3, 10, 0, 13, "i")
  c.line(0, 11, 2, 13, "i")
  c.ring(8, 5, 3, "c")
  c.line(10, 3, 12, 1, "c")
  c.grid(11, 0, "ccc\n..c\n..c")
  // Interlocked: blue over pink at the top crossing, pink over blue at the bottom.
  c.px(8, 5 + 3, "i")
  c.recolor({ i: "I", c: "C" }, (x, y) => x + y < 8 || (x + y < 12 && y < 3))
  c.recolor({ i: "h", c: "u" }, (x, y) => x + y > 16)
})

// Sloth (+duration): an hourglass, and she naps while it runs.
sprite("_hourglass", { w: 9, h: 12, mx: "c" }, `
  PPPPP
  .wYYY
  .wYYY
  ..wYY
  ...wY
  ....Y
  ...wY
  ..w..
  .w.YY
  .wYYY
  .wYYY
  PPPPP`)
icon("sloth", stamp("_hourglass", 0, 2), recolor({ Y: "y", P: "p" }, (x) => x > 5), grid(9, 0, `
  VVVVV
  ...V.
  ..V..
  .V...
  VVVVV`), grid(11, 6, "vvv\n.v.\nvvv"))

// Dark Sigil (+area): a rune ring pushing its reach outward.
sym("dark_sigil", [(c) => {
  c.ring(6, 6, 3, "v")
  c.circ(6, 6, 1, "V")
  for (const [x, y, ax, ay] of [[6, 0, 0, 1], [6, 12, 0, -1], [0, 6, 1, 0], [12, 6, -1, 0]]) {
    c.px(x, y, "V")
    c.px(x + ay, y + ax, "V")
    c.px(x - ay, y - ax, "V")
    c.px(x + ax, y + ay, "P")
  }
  for (const [x, y] of [[2, 2], [10, 2], [2, 10], [10, 10]]) c.px(x, y, "P")
}], [recolor({ v: "P", V: "v" }, (x, y) => x + y > 14)])

// Infernal Hide (+armour): a horned shield.
sym("infernal_hide", [`
  w......
  ww.....
  .wSSSSS
  .Sqqrrw
  .Sqrrrw
  .Sqrrrw
  .Srrrrw
  .Srrrrw
  ..Srrrw
  ..SrrRw
  ...SrRw
  ....SRw
  .....SS
  ......S`], [recolor({ q: "r", r: "R", w: "S", S: "s" }, (x, y) => x > 6 && y > 1)])

// Envy (+XP per soul): a green eye coveting a soul.
icon("envy", (c) => {
  c.grid(0, 1, `
    ..eeeee..
    .eWWWEEe.
    eWWWWEkEe
    .eWWWEEe.
    ..eeeee..`)
  soul(c, 10, 10, 3)
})

// Pride (tougher foes): a crowned skull.
sym("pride", [`
  ..y...y
  ..yy.yy
  ..yyyyq
  ..ooooo
  .wwwwww
  wWwwwww
  wwkkwww
  wwkqwww
  .wwwwkk
  ..wwwww
  ..wkwkw
  ..wwwww
  ...wwww`], [recolor({ w: "S" }, (x, y) => x > 8 && y > 4), recolor({ q: "k" }, [9, 7, 1, 1]), px(9, 7, "q")])

sheet("icons", [
  "ic_hellflare", "ic_tail_lash", "ic_brimstone_halo", "ic_allure", "ic_hellgate",
  "ic_whip_of_torment", "ic_scent_of_sin", "ic_reapers_scythe", "ic_profaned_cross",
  "ic_wandering_sigil", "ic_bat_swarm", "ic_black_mass", "ic_gaze_of_desire", "ic_thralls",
  "ic_kiss_of_ruin", "ic_barbed_scourge", "ic_infernal_crown", "ic_sirens_dominion",
  "ic_abyssal_rift", "ic_torment_eternal", "ic_devouring_musk", "ic_midnight_harvest",
  "ic_heretics_brand", "ic_broken_seal", "ic_crimson_host", "ic_witches_sabbath", "ic_evil_eye",
  "ic_infernal_court",
  "ic_wrath", "ic_bloodlust", "ic_legion", "ic_night_wings", "ic_gluttony", "ic_lust",
  "ic_sloth", "ic_dark_sigil", "ic_infernal_hide", "ic_envy", "ic_pride",
], { cols: 39, pad: 0 })
