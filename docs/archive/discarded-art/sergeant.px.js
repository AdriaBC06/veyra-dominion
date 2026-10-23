// Discarded (S16, 2026-10-23): first 32x40 draft of the Captain of the Watch, replaced by the giant
// one in assets/enemies.px.js. Same palette as that file; rebuild with
// `node .claude/skills/pxl-pixel-art/bin/pxl.js build docs/archive/discarded-art/sergeant.px.js --out <dir>`.

pal({
  k: "#10131c",                                            // outline, blue-black
  A: "#34343e", B: "#55535f", C: "#7d7b88", D: "#a6a4ae",  // rat fur
  M: "#54444e", m: "#7c6670",                              // rat ears, tail, feet: dusty mauve
  S: "#6e5246", s: "#96705e", z: "#b8927c",                // skin, desaturated
  H: "#2e2620", h: "#4a3e32",                              // hair
  T: "#2a3c44", t: "#3f5a64", u: "#5f7f88",                // villager tunic: slate teal
  P: "#3a342e", p: "#544c42",                              // trousers, boots
  W: "#4a3a2a", w: "#6e5a44", e: "#8a7458",                // wood
  L: "#2a313c", l: "#46505c", I: "#5e6a74", i: "#9aa6b0", j: "#c8d2da",  // steel
  N: "#1a2644", n: "#2a3e68", o: "#4660a0",                // royal blue cloth
  X: "#b8c4d4",                                            // tabard emblem
  Y: "#5e5236",                                            // dull bronze hilt
  Q: "#6c6a62", q: "#908e84", r: "#b2b0a4",                // linen shirt
  E: "#4a1c3a", F: "#7e3058", G: "#b0567e",                // thrall tunic: Veyra's plum rose
  O: "#ff6f9a",                                            // thrall eyes and heart: her pink
  // V18 Ashford roster
  a: "#2c3142", b: "#465068", c: "#6b7892",                // slate: crow feathers, curate cassock
  d: "#c4bca4",                                            // bone: beak, claws
  f: "#4c443c", g: "#75695a", v: "#9e8f78",                // dusty grey-brown: hound, coats
  K: "#7a5e34", y: "#b8924a", Z: "#e8dca4",                // dull torch fire: ochre to pale yellow
  R: "#a29b86", V: "#d0c9b4",                              // sheepskin, sackcloth highlights
  U: "#56685c", J: "#8aa092",                              // oil flask glass
})
ramps("kABCD", "kMm", "kSsz", "kHh", "kTtu", "kPp", "kWwe", "kLlIij", "kNno", "kQqr", "kEFG",
  "kabc", "kfgv", "kKyZ", "kRV", "kUJ")

// ------------------------------------------------------------------ sergeant of the Watch (32x40)
// Drawn as the first Captain draft; the user kept it for a bigger enemy (not in the game yet).
// Plumed barbute, steel breastplate over the royal blue tabard, a blue cloak behind, a kite shield
// in front with the Watch's pale tower, and a sword over the back shoulder. `charge` drops the
// sword and leans into the shield.
function sergeant(c, step, charge) {
  const lean = charge ? 3 : 0, dy = charge ? 2 : 0
  // cloak behind, flaring at the hem
  c.poly([17, 13, 23, 13, 28, 34, 24, 36, 18, 34], "N")
  c.poly([19, 13, 23, 13, 27, 32, 24, 33], "n")
  // legs: steel greaves and sabatons, the stride swaps them
  const legs = step ? [[9, 0, 0], [17, 1, 0]] : [[12, 1, 1], [15, 0, 0]]
  for (const [x, back, up] of legs) {
    const y = 29 - up
    c.rect(x, y, 4, 5, back ? "l" : "I")
    c.vline(x, y, 5, back ? "I" : "i")
    c.px(x + 1, y + 1, back ? "I" : "j")
    c.rect(x - 1, y + 5, 6, 3 - (up ? 0 : 0), back ? "L" : "l")
    c.hline(x - 1, y + 5, 5, back ? "l" : "I")
  }
  // tabard with the Watch's belt, breastplate and fauld
  c.rect(11, 21, 10, 9, "o"); c.rect(17, 21, 4, 9, "n"); c.hline(11, 29, 10, "N")
  c.vline(16, 23, 6, "n")
  c.ellipse(16 - lean, 17 + dy / 2, 6, 5, "l")
  c.ellipse(15 - lean, 16 + dy / 2, 5, 4, "I")
  c.ellipse(14 - lean, 15 + dy / 2, 3, 2, "i")
  c.px(13 - lean, 14 + dy / 2, "j"); c.px(14 - lean, 14 + dy / 2, "j")
  c.hline(11 - lean, 21, 10, "Y"); c.px(15 - lean, 21, "e")
  // helm: barbute with a T slit, a blue horsehair crest sweeping back
  const hx = 12 - lean
    c.circ(hx + 4, 8 + dy, 4, "I")
  c.rect(hx, 8 + dy, 9, 5, "I")
  c.circ(hx + 3, 7 + dy, 2, "i"); c.px(hx + 2, 6 + dy, "j"); c.px(hx + 3, 5 + dy, "j")
  c.hline(hx + 1, 9 + dy, 3, "k"); c.vline(hx + 2, 9 + dy, 3, "k")
  c.vline(hx + 8, 8 + dy, 5, "l")
  c.rect(hx + 1, 12 + dy, 7, 1, "l")
  c.line(hx + 3, 3 + dy, hx + 8, 2 + dy, "o", 2)
  c.line(hx + 9, 2 + dy, hx + 13 + lean, 6 + dy - lean, "n", 2)
  c.px(hx + 3, 2 + dy, "X"); c.px(hx + 5, 2 + dy, "X")
  // sword arm: raised over the back shoulder, or lowered behind while charging
  if (charge) {
    c.line(19, 19, 22, 21, "I", 2)
    c.line(24, 21, 30, 23, "j"); c.line(24, 22, 29, 24, "i")
    c.line(23, 19, 22, 24, "Y")
  } else {
    c.circ(21, 16, 2, "l")
    c.line(22, 16, 24, 12, "I", 2)
    c.line(25, 10, 29, 1, "j"); c.line(26, 10, 29, 3, "i")
    c.line(22, 10, 27, 12, "Y")
    c.px(24, 12, "e")
  }
  // kite shield on the front arm: steel rim, blue field, the pale tower of the Watch
  const sx = 2 - lean, sy = 13 + (charge ? 1 : 0)
  c.poly([sx, sy, sx + 11, sy, sx + 11, sy + 12, sx + 6, sy + 20, sx + 5, sy + 20, sx, sy + 12], "i")
  c.poly([sx + 1, sy + 1, sx + 10, sy + 1, sx + 10, sy + 12, sx + 5.5, sy + 19, sx + 1, sy + 12], "n")
  c.poly([sx + 6, sy + 1, sx + 10, sy + 1, sx + 10, sy + 12, sx + 6, sy + 18.5], "N")
  c.hline(sx + 1, sy + 1, 9, "o")
  c.rect(sx + 4, sy + 6, 3, 7, "X"); c.rect(sx + 3, sy + 5, 5, 2, "X")
  c.px(sx + 3, sy + 4, "X"); c.px(sx + 5, sy + 4, "X"); c.px(sx + 7, sy + 4, "X")
  c.px(sx + 5, sy + 9, "N"); c.px(sx + 5, sy + 10, "N")
  c.vline(sx + 11, sy, 12, "I")
  // pauldron over the shield's top edge
  c.circ(sx + 11, sy, 2, "I"); c.px(sx + 10, sy - 1, "i")
}
anim("sergeant", { w: 30, h: 38, outline: "k", fps: 6, frames: 2 }, (c) => sergeant(c, c.frame, false))
anim("sergeant_charge", { w: 30, h: 38, outline: "k", fps: 10, frames: 2 }, (c) => sergeant(c, c.frame, true))
