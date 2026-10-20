// Ashford enemies (PLAN.md 7.6, V13): rat, villager, knight and brute farmer.
// Side views facing left (the game flips them toward Veyra), feet on the bottom row. Sizes follow
// 7.6: rat 12 px long, villager 20 tall, knight 26, brute 40, outline included.
//
// Palette rule: Veyra owns the warm reds, magentas and golds, so enemies stay cold and muted
// (slate, steel, denim, dusty browns) with a blue-black outline instead of her plum.

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
})
ramps("kABCD", "kMm", "kSsz", "kHh", "kTtu", "kPp", "kWwe", "kLlIij", "kNno", "kQqr")

// ------------------------------------------------------------------ rat (12 long)
// Arched back, big ear, a pale glint for the eye so it reads at 1x; the tail whips between frames.
anim("rat", { w: 15, h: 7, outline: "k", fps: 6 }, `
  .....mM........
  ...CCCCCC......
  ..CDCCCBBB.....
  .CjCCBBBBBB....
  mBBBBBBBBBBA...
  .BDDDBBBAAAA...`,
  frame(grid(0, 4, "...........Amm.\n.BDDDBBBAAAA..m\n..M.M...M.M...")),
  frame(grid(0, 4, "...........Am..\n.BDDDBBBAAAAm..\n...M.M...M.Mmm")))

// ------------------------------------------------------------------ villager (20 tall)
anim("villager", { w: 12, h: 18, outline: "k", fps: 6 }, `
  j.j.j.......
  j.j.j.......
  iiiii.hHHh..
  ..W..hHHHHh.
  ..W.hzzzsHh.
  ..W.zkzzsHH.
  ..W..zzss...
  ..W..uTTT...
  ..W.uttTTT..
  .zzttttTTTT.
  ..WttttTTT..
  ..W.ttttTT..
  ..W.TTTTT...
  ..W.ppPPP...`,
  frame(grid(2, 14, "W.pP.PP..\nW.pP.PP..\nW.pP..PP.\nWpp...PPP")),
  frame(grid(2, 14, "W.pP.PP..\nW..pPP...\nW..pPP...\nW.ppPPP..")))

// ------------------------------------------------------------------ knight (26 tall)
anim("knight", { w: 16, h: 24, outline: "k", fps: 6 }, `
  ........o.......
  .......onn......
  ........n.......
  ......jjjj......
  .....jiiiiI.....
  .....iiiiiII....
  .....kkkiiIL....
  .....iiiiiIL....
  ..j..IiiiIL.....
  ..j..lIIIIL.....
  ..j.jiioonnI....
  ..j.iiooXonnIL..
  ..i..ioXXXnnIL..
  ..i..ooXonnnLL..
  .YYY.ioooonnL...
  ..ii.ooooonnn...
  ..Y..ooooonnN...
  .....nooonnNN...
  .....llllllL....`,
  frame(grid(5, 19, "Li..Li..\nLi..Li..\nLi...Li.\nLi...Li.\nLLi...LLi")),
  frame(grid(5, 19, "Li..Li..\n.Li.Li..\n.Li.Li..\n.LiLi...\n.LLLLi..")))

// ------------------------------------------------------------------ brute farmer (40 tall)
// Hunched giant in denim overalls, the small head jutting forward at shoulder height, a wooden
// maul over his shoulder. Drawn back to front; the stride swaps the legs.
function brute(c, step) {
  // maul: handle from the fist over the shoulder, iron-banded head behind him
  c.poly([18, 3, 24, -1, 30, 6, 24, 11], "w")
  c.poly([24, 11, 30, 6, 30, 8, 25, 12], "W")
  c.line(19, 3, 24, -1, "e")
  c.line(20, 5, 25, 0, "I"); c.line(23, 9, 29, 4, "I")
  // legs and boots
  const legs = step ? [[8, 30, 0], [18, 30, 1]] : [[11, 30, 1], [16, 30, 0]]
  for (const [x, y, back] of legs) {
    c.rect(x, y, 6, 5, back ? "N" : "n")
    c.rect(x - 1, y + 5, 8, 3, "P"); c.hline(x - 1, y + 5, 7, "p")
  }
  // hunched back and shoulders in linen, belly and bib in denim
  c.ellipse(18, 18, 11, 10, "q")
  c.ellipse(21, 14, 6, 4, "r")
  c.ellipse(13, 25, 11, 7, "n")
  c.rect(7, 16, 10, 10, "n")
  c.line(16, 16, 20, 9, "n", 2)
  c.px(9, 18, "I"); c.px(15, 18, "I")
  c.ellipse(13, 29, 10, 2, "N")
  c.line(23, 22, 27, 18, "Q", 2)
  // the handle rests on the shoulder, over the shirt
  c.line(9, 20, 21, 8, "W", 2)
  c.line(10, 18, 20, 8, "w")
  // head forward and low: bald dome, heavy brow, stubbled jaw
  c.circ(7, 10, 5, "s")
  c.circ(6, 8, 2, "z")
  c.rect(2, 11, 8, 4, "s")
  c.hline(2, 10, 5, "S")
  c.px(3, 11, "k")
  c.px(1, 12, "s")
  c.rect(2, 13, 7, 2, "S"); c.px(4, 13, "s"); c.px(7, 14, "s")
  c.px(10, 10, "S"); c.px(10, 11, "S")
  // near arm: sleeve at the shoulder, thick forearm up to the fist on the handle
  c.circ(15, 14, 3, "r")
  c.line(16, 17, 14, 19, "Q", 2)
  c.line(13, 20, 9, 20, "s", 3)
  c.circ(8, 20, 2, "z")
  c.px(7, 21, "s"); c.px(8, 22, "S")
}
anim("brute", { w: 30, h: 38, outline: "k", fps: 6, frames: 2 }, (c) => brute(c, c.frame))
