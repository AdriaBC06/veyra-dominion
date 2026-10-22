// Title scene art (docs/design.md 7.5, V14): logo, logo glow and menu buttons.
// Logo: "VEYRA" in engraved Roman capitals (thick/thin strokes, wedge serifs) cut like a chisel:
// each stroke has a lit facet toward the top-left and a dark one toward the bottom-right, and
// glows with hellfire in the cracks. "DOMINION" sits under it in spaced capitals between two gilded
// rules with a ruby, like the gems on Veyra's corset. The backdrop is in title_backdrop.px.js.
//
// Palette rule: Veyra owns the warm reds, magentas and golds. The logo is hers (infernal, warm);
// the backdrop stays muted and cold so she and the logo read first.

pal({
  k: "#16091a",                                            // ink, Veyra's plum black
  K: "#2a0c16",                                            // drop shadow
  D: "#5c0c1c", d: "#9a2224", l: "#d8642c", L: "#f4a648",  // engraved metal: shadow -> lit facet
  y: "#ffe08a",                                            // ridge / crack highlight
})
ramps("kDdlLy")

// ------------------------------------------------------------------ VEYRA
// Every stroke is a quad (TL, TR, BR, BL) cut along its length like a chisel groove: tall strokes
// split left (lit) / right (shadow), wide ones top (lit) / bottom (shadow). Cap height 36; thick
// stroke 7, thin 3.
const CAP = 36
const mid = (p, r) => [(p[0] + r[0]) / 2, (p[1] + r[1]) / 2]
function q(c, pts) {
  const [a, b, cc, d] = [0, 2, 4, 6].map((i) => [pts[i], pts[i + 1]])
  const tall = Math.abs(d[1] - a[1]) > Math.abs(b[0] - a[0])
  if (tall) {
    const mt = mid(a, b), mb = mid(d, cc)
    c.poly([...a, ...mt, ...mb, ...d], "L"); c.poly([...mt, ...b, ...cc, ...mb], "d")
  } else {
    const ml = mid(a, d), mr = mid(b, cc)
    c.poly([...a, ...b, ...mr, ...ml], "L"); c.poly([...ml, ...mr, ...cc, ...d], "d")
  }
}

const LETTERS = {
  V: { w: 32, draw(c, x) {
    q(c, [x + 1, 2, x + 9, 2, x + 18, CAP, x + 15, CAP])                 // thick left diagonal
    q(c, [x + 26, 2, x + 29, 2, x + 18, CAP, x + 16, CAP - 4])           // thin right diagonal
    q(c, [x - 1, 0, x + 12, 0, x + 10, 3, x + 1, 3])                    // serifs
    q(c, [x + 23, 0, x + 32, 0, x + 30, 3, x + 24, 3])
  } },
  E: { w: 26, draw(c, x) {
    q(c, [x + 2, 0, x + 9, 0, x + 9, CAP, x + 2, CAP])                  // stem
    q(c, [x, 0, x + 24, 0, x + 24, 3, x, 3])                            // top arm
    q(c, [x + 21, 0, x + 24, 0, x + 25, 9, x + 23, 9, x + 21, 3])       // top serif, down
    q(c, [x + 2, 16, x + 19, 16, x + 19, 19, x + 2, 19])                // middle arm
    q(c, [x + 17, 13, x + 19, 13, x + 19, 22, x + 17, 22])
    q(c, [x, CAP - 3, x + 25, CAP - 3, x + 25, CAP, x, CAP])            // bottom arm
    q(c, [x + 24, CAP - 11, x + 26, CAP - 11, x + 26, CAP, x + 22, CAP - 3])
  } },
  Y: { w: 31, draw(c, x) {
    q(c, [x + 1, 2, x + 9, 2, x + 18, 19, x + 12, 19])                  // thick left arm
    q(c, [x + 26, 2, x + 29, 2, x + 18, 19, x + 15, 19])                // thin right arm
    q(c, [x + 12, 18, x + 18, 18, x + 18, CAP, x + 12, CAP])            // stem
    q(c, [x - 1, 0, x + 12, 0, x + 10, 3, x + 1, 3])
    q(c, [x + 23, 0, x + 32, 0, x + 30, 3, x + 24, 3])
    q(c, [x + 7, CAP - 3, x + 23, CAP - 3, x + 23, CAP, x + 7, CAP])
  } },
  R: { w: 30, draw(c, x) {
    q(c, [x + 2, 0, x + 9, 0, x + 9, CAP, x + 2, CAP])                  // stem
    q(c, [x, 0, x + 17, 0, x + 17, 3, x, 3])                            // top of bowl
    q(c, [x + 2, 17, x + 16, 17, x + 16, 20, x + 2, 20])                // bottom of bowl
    c.clip((px, py) => px >= x + 14 && py < 20, () => {                 // bowl: thick on the right
      c.ellipse(x + 15, 10, 10, 10, "d")
      c.ellipse(x + 14, 10, 7, 8, "L")
      c.ellipse(x + 14, 10, 4, 6, "_")
    })
    q(c, [x + 11, 19, x + 17, 19, x + 28, CAP - 3, x + 22, CAP - 3])    // leg
    q(c, [x, CAP - 3, x + 12, CAP - 3, x + 12, CAP, x, CAP])            // foot serifs
    q(c, [x + 20, CAP - 3, x + 31, CAP - 3, x + 31, CAP, x + 20, CAP])
  } },
  A: { w: 33, draw(c, x) {
    q(c, [x + 14, 0, x + 17, 0, x + 6, CAP, x + 3, CAP])                // thin left diagonal
    q(c, [x + 14, 0, x + 19, 0, x + 30, CAP, x + 23, CAP])              // thick right diagonal
    q(c, [x + 8, 23, x + 25, 23, x + 25, 26, x + 8, 26])                // crossbar
    q(c, [x, CAP - 3, x + 10, CAP - 3, x + 10, CAP, x, CAP])
    q(c, [x + 20, CAP - 3, x + 33, CAP - 3, x + 33, CAP, x + 20, CAP])
  } },
}

function word(c, text, letters, x0, gap) {
  let x = x0
  for (const ch of text) { letters[ch].draw(c, x); x += letters[ch].w + gap }
}

sprite("title_logo", { w: 176, h: 41 },
  layer("shadow", { x: 3, y: 3 }, (c) => { word(c, "VEYRA", LETTERS, 2, 3); c.recolor({ d: "K", L: "K" }) }),
  layer("face", { x: 1, y: 1 }, (c) => { word(c, "VEYRA", LETTERS, 2, 3) }),
  (c) => {
    // ink around the face only (the drop shadow keeps its soft edge)
    const solid = (x, y) => c.is(x, y, "DdlLy")
    const ink = []
    for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++)
      if (!solid(x, y) && !c.is(x, y, "k") && [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([i, j]) => solid(x + i, y + j))) ink.push([x, y])
    ink.forEach(([x, y]) => c.px(x, y, "k"))
  })


// ------------------------------------------------------------------ DOMINION
// Same chisel cut as VEYRA at 13 px: thick strokes 4 px, hairlines 2 px, lit facet left/top.
const SC = 13
const SMALL = {
  D: { w: 14, draw(c, x) {
    q(c, [x + 1, 0, x + 5, 0, x + 5, SC, x + 1, SC])
    q(c, [x - 1, 0, x + 8, 0, x + 8, 2, x - 1, 2]); q(c, [x - 1, SC - 2, x + 8, SC - 2, x + 8, SC, x - 1, SC])
    c.clip((px) => px >= x + 7, () => { c.ellipse(x + 7, 6, 6, 6, "d"); c.clip((px) => px < x + 10, () => c.ellipse(x + 7, 6, 6, 6, "L")); c.ellipse(x + 6, 6, 3, 4, "_") })
  } },
  O: { w: 13, draw(c, x) {
    c.ellipse(x + 6, 6, 6, 6, "d"); c.clip((px) => px < x + 4, () => c.ellipse(x + 6, 6, 6, 6, "L"))
    c.clip((px, py) => py < 2 && px < x + 7, () => c.ellipse(x + 6, 6, 6, 6, "L"))
    c.ellipse(x + 6, 6, 3, 4, "_")
  } },
  M: { w: 16, draw(c, x) {
    q(c, [x + 1, 0, x + 3, 0, x + 3, SC, x + 1, SC])
    q(c, [x + 1, 0, x + 5, 0, x + 9, SC, x + 7, SC])
    q(c, [x + 11, 0, x + 13, 0, x + 9, SC, x + 7, SC - 3])
    q(c, [x + 11, 0, x + 15, 0, x + 15, SC, x + 11, SC])
    q(c, [x - 1, SC - 2, x + 5, SC - 2, x + 5, SC, x - 1, SC]); q(c, [x + 9, SC - 2, x + 17, SC - 2, x + 17, SC, x + 9, SC])
    q(c, [x - 1, 0, x + 3, 0, x + 3, 2, x - 1, 2]); q(c, [x + 13, 0, x + 17, 0, x + 17, 2, x + 13, 2])
  } },
  I: { w: 6, draw(c, x) {
    q(c, [x + 1, 0, x + 5, 0, x + 5, SC, x + 1, SC])
    q(c, [x - 1, 0, x + 7, 0, x + 7, 2, x - 1, 2]); q(c, [x - 1, SC - 2, x + 7, SC - 2, x + 7, SC, x - 1, SC])
  } },
  N: { w: 14, draw(c, x) {
    q(c, [x + 1, 0, x + 3, 0, x + 3, SC, x + 1, SC])
    q(c, [x + 1, 0, x + 5, 0, x + 13, SC, x + 9, SC])
    q(c, [x + 11, 0, x + 13, 0, x + 13, SC, x + 11, SC])
    q(c, [x - 1, 0, x + 5, 0, x + 5, 2, x - 1, 2]); q(c, [x + 9, 0, x + 15, 0, x + 15, 2, x + 9, 2])
    q(c, [x - 1, SC - 2, x + 5, SC - 2, x + 5, SC, x - 1, SC])
  } },
}

pal({ g: "#7a5226", G: "#d8a040", Y: "#ffe08a", R: "#5c0c1c", r: "#b01c34", p: "#ff7088" })
// a gilded rule ending in a small ruby; `dir` -1 runs left from x, 1 right
function rule(c, x, y, len, dir) {
  for (let i = 0; i < len; i++) c.px(x + dir * i, y, i < len * 0.4 ? "G" : i % 2 ? "g" : i < len * 0.7 ? "G" : ".")
  const gx = x + dir * (len + 3)
  c.poly([gx - 3, y + 0.5, gx + 0.5, y - 3, gx + 4, y + 0.5, gx + 0.5, y + 4], "r")
  c.px(gx, y - 1, "p"); c.px(gx + (dir > 0 ? 1 : -1), y + 1, "R"); c.px(gx, y + 2, "R")
}
const DOM_W = 132
sprite("title_dominion", { w: 182, h: 18 },
  layer("shadow", { x: 26, y: 3 }, (c) => { word(c, "DOMINION", SMALL, 1, 5); c.recolor({ d: "K", L: "K" }) }),
  layer("face", { x: 25, y: 2 }, (c) => { word(c, "DOMINION", SMALL, 1, 5) }),
  (c) => {
    const solid = (x, y) => c.is(x, y, "DdlLy")
    const ink = []
    for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++)
      if (!solid(x, y) && !c.is(x, y, "kK") && [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([i, j]) => solid(x + i, y + j))) ink.push([x, y])
    ink.forEach(([x, y]) => c.px(x, y, "k"))
    rule(c, 21, 8, 13, -1); rule(c, 27 + DOM_W, 8, 13, 1)
  })

// ------------------------------------------------------------------ logo glow
// Hellfire halo behind VEYRA in hard bands; 4 strengths the game flickers between.
pal({ h: "#24060e", H: "#3c0a12", j: "#5a1016" })
anim("title_logo_glow", { w: 192, h: 57, frames: 4, fps: 1 }, (c) => {
  c.stamp("title_logo", 8, 8)
  const s = [0.75, 0.9, 1.0, 1.15][c.frame]
  const mask = []
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) if (c.is(x, y, "*")) mask.push([x, y])
  c.rect(0, 0, c.w, c.h, "_")
  for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) {
    let d = 99
    for (const [mx, my] of mask) { const e = Math.hypot(mx - x, my - y); if (e < d) d = e; if (d < 1) break }
    if (d < 1) continue
    if (d <= 2.2 * s) c.px(x, y, "j")
    else if (d <= 4.5 * s) c.px(x, y, "H")
    else if (d <= 7.5 * s && bayer(x, y) < 0.5) c.px(x, y, "h")
  }
})
function bayer(x, y) { return ([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]][y & 3][x & 3] + 0.5) / 16 }

// ------------------------------------------------------------------ menu buttons
// 168x26 with a 4 px margin for the hover glow. Frame 0 idle, 1 highlighted: dark plum lacquer in a
// chamfered gilded frame with a ruby at each end; highlighted, the gold brightens, the panel warms
// and a red ember glow surrounds it. The label is drawn by the game.
pal({
  P: "#0f0712", Q: "#170b1a", S: "#2a0c1a", T: "#3c1222",                     // panel idle / lit
  b: "#4a3320", B: "#7a5226", G: "#d8a040", Y: "#ffe08a",                     // gilt: dim -> bright
  e: "#3a0a14", E: "#6a1420",                                                 // ember glow
  R: "#5c0c1c", r: "#b01c34", p: "#ff7088", z: "#06030a",
})
const BTW = 168, BTH = 26, M = 4
anim("title_button", { w: BTW + 2 * M, h: BTH + 2 * M, frames: 2, fps: 1 }, (c) => {
  const on = c.frame === 1, x0 = M, y0 = M, x1 = M + BTW - 1, y1 = M + BTH - 1, ch = 3
  const inside = (x, y, i) => x >= x0 + i && x <= x1 - i && y >= y0 + i && y <= y1 - i &&
    Math.min(x - x0, x1 - x) + Math.min(y - y0, y1 - y) >= ch + i
  if (on) for (let y = 0; y < c.h; y++) for (let x = 0; x < c.w; x++) {
    let d = 9; for (let i = 1; i <= 4 && d === 9; i++) if (!inside(x, y, -i + 1) && inside(x, y, -i)) d = i
    if (d === 1 || d === 2) c.px(x, y, d === 1 ? "E" : "e")
  }
  for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) {
    if (!inside(x, y, 0)) continue
    if (!inside(x, y, 1)) c.px(x, y, on ? (y < y0 + 3 || x < x0 + 2 ? "Y" : "G") : (y < y0 + 3 ? "G" : "B"))   // gilt frame
    else if (!inside(x, y, 2)) c.px(x, y, "z")                                                         // inner groove
    else c.px(x, y, on ? (y < y0 + 11 ? "T" : "S") : (y < y0 + 9 ? "Q" : "P"))                          // lacquer
  }
  // rubies set into the frame at both ends
  const my = y0 + Math.floor(BTH / 2)
  for (const gx of [x0 + 1, x1 - 1]) {
    c.poly([gx - 3, my, gx + 0.5, my - 3.5, gx + 4, my, gx + 0.5, my + 4], on ? "r" : "R")
    c.px(gx, my - 1, on ? "p" : "r"); c.px(gx, my + 2, "R")
  }
})
