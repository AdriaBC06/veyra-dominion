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

// ------------------------------------------------------------------ thrall (20 tall, Thralls Art)
// A villager she seduced: same build, her plum rose instead of slate, pink eyes, a heart on the
// chest. Hers, so it breaks the cold-enemy palette rule on purpose.
anim("thrall", { w: 12, h: 18, outline: "k", fps: 6 }, `
  j.j.j.......
  j.j.j.......
  iiiii.hHHh..
  ..W..hHHHHh.
  ..W.hzzzsHh.
  ..W.zOzzsHH.
  ..W..zzss...
  ..W..GFFF...
  ..W.GFFOFF..
  .zzFFOOOFFE.
  ..WFFFOFFE..
  ..W.FFFFEE..
  ..W.EEEEE...
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

// ================================================================== V18 Ashford roster
// Same rules: facing left, feet on the bottom row (the crow flies; its shadow is separate),
// 2 frames, blue-black outline, light from the top left.

// ------------------------------------------------------------------ crow (15x11)
// Slate-blue sheen so it reads on grass; bone beak and a glint for the eye. Wings up / down.
anim("crow", { w: 13, h: 9, outline: "k", fps: 8 },
  frame(grid(0, 0, `
    ...........c.
    ..........cb.
    ........ccbb.
    .......cbbba.
    ..cc..cbba...
    ddjbbcbba....
    ..abbbbbbbbcc
    ...aaaaaaabcb
    ..........a..`)),
  frame(grid(0, 0, `
    .............
    .............
    ..cc.........
    ddjbbbbbbb...
    ..abbcccbbbcc
    ...abbbbbbbcb
    ....bbbbba...
    ....bbbba....
    ....b.b.b....`)))

// ------------------------------------------------------------------ hound (21x13)
// Lean hunting dog: long muzzle, ears back, deep chest, tucked waist, leather collar.
anim("hound", { w: 19, h: 11, outline: "k", fps: 8 }, `
  ..ff...............
  .fgvf..............
  gvkgvf.............
  gggggWW...........f
  .f.ggWWvvvvvvvvvvf.
  ....gggggggggggggf.
  ....fgggggggggfff..
  .....fgggggf.fgf...`,
  frame(grid(0, 8, `
    .....fg.f....f..g..
    ....fg..f....f...g.
    ...ff..ff...ff..gg.`)),
  frame(grid(0, 8, `
    ......fgf....fgf...
    ......fgf....fg....
    .....ffgg...ffgg...`)))

// ------------------------------------------------------------------ torchbearer (14x22)
// Villager build in a linen shirt, torch raised in the front hand: the flame is the only warm spot.
anim("torchbearer", { w: 12, h: 20, outline: "k", fps: 6 }, `
  ..Z.........
  .ZZy........
  .yZy........
  .KyK........
  ..W...hHHh..
  ..W..hHHHHh.
  ..W.hzzzsHh.
  ..W.zkzzsHH.
  .zz..zzss...
  ..zs.rqQQ...
  ...srqqQQQ..
  ....qqqqQQQ.
  ....qqqqQQ..
  ....qqqqQQ..
  ....QQQQQ...
  ....ppPPP...`,
  frame(grid(3, 16, ".pP.PP..\n.pP.PP..\n.pP..PP.\npp...PPP")),
  frame(grid(3, 16, ".pP.PP..\n..pPP...\n..pPP...\n.ppPPP..")))

// ------------------------------------------------------------------ shepherd (14x21)
// Cloth hood, sheepskin vest, crook in the front hand, sling pouch (with a stone) at the hip.
anim("shepherd", { w: 12, h: 19, outline: "k", fps: 6 }, `
  .ee.........
  e..e........
  e..e........
  ...e..wwww..
  ...e.weeeew.
  ...e.wzzzwW.
  ...ezkzzsWW.
  ...e.zzssWW.
  ...e.VVRRR..
  ..zzVVVRRRR.
  ...eVVgVRRR.
  ...eVVgVRRRR
  ...e.VVVRRWw
  ...eWWWWWWwe
  ...ewDggggf.`,
  frame(grid(4, 15, ".pP.PP..\n.pP.PP..\n.pP..PP.\npp...PPP")),
  frame(grid(4, 15, ".pP.PP..\n..pPP...\n..pPP...\n.ppPPP..")))

// ------------------------------------------------------------------ barricader (20x22)
// A villager crouched behind a nailed barn door held in front (the side facing Veyra): planks,
// a Z brace, two iron bands with nails. Only his head, hands and back show.
anim("barricader", { w: 18, h: 20, outline: "k", fps: 6 }, `
  ........hHHh......
  .......hHHHHh.....
  .......zkzsHh.....
  .......zzssHH.....
  ...........uTT....
  ...........ttTT...
  ...........tttTT..
  ...........ttttT..
  ...........tttTT..
  ...........ttTTT..
  ...........tTTT...
  ...........TTTT...
  ...........TTT....
  ...........ppP....
  ...........ppPP...`,
  (c) => {
    c.rect(0, 4, 11, 15, "w")
    for (const x of [3, 7]) c.vline(x, 4, 15, "W")
    for (const x of [0, 4, 8]) c.vline(x, 4, 15, "e")
    c.vline(10, 4, 15, "W")
    c.line(1, 15, 9, 7, "e"); c.line(2, 15, 9, 8, "W")
    for (const y of [6, 16]) {
      c.hline(0, y, 11, "l"); c.hline(0, y - 1, 11, "W")
      for (const x of [1, 5, 9]) c.px(x, y, "j")
    }
    c.hline(0, 4, 11, "e")
    c.px(3, 4, "z"); c.px(3, 3, "z"); c.px(8, 4, "z"); c.px(8, 3, "z")
  },
  frame(grid(10, 15, "pP.PP..\npP.PP..\npP..PP.\nPP...PPP\n........")),
  frame(grid(10, 15, "pP.PP..\n.pPP...\n.pPP...\n.pPPP...\n........")))

// ------------------------------------------------------------------ curate (14x21)
// Parish priest: slate cassock to the ground, white collar, a censer swinging on its chain with a
// curl of pale smoke. The censer swings forward and back between the frames.
anim("curate", { w: 12, h: 19, outline: "k", fps: 4 }, `
  ......HHh...
  .....hHHHh..
  .....zzzsH..
  ....zkzzsH..
  .....zzss...
  .....jjjb...
  ....cbbbbba.
  ...cbbbbbbaa
  ..zcbbbbbbaa
  ....cbbbbbaa
  ....cbbbbbaa
  ....cbbbbbba
  ....cbbbbbba
  ....cbbbbbba
  ...cbbbbbbba
  ...cbbbbbbba
  ...cbbbbbbba
  ...cbbbbbbba
  ...ppP..ppP.`,
  frame(grid(0, 9, "..i.\n..i.\n.iji\n.IiI\n..I."), grid(0, 4, "D...\n.D..\nDD..\n.D..")),
  frame(grid(0, 9, ".i..\ni...\niji.\nIiI.\n.I.."), grid(0, 3, "..D.\n.D..\n.DD.\nD...")))

// ------------------------------------------------------------------ ratcatcher (20x24)
// Hunched man in a grey-brown coat and leather cap, a wooden cage of live rats strapped to his
// back, a pole over his front with dead rats hanging by the tails.
anim("ratcatcher", { w: 18, h: 22, outline: "k", fps: 6 }, `
  ...e..............
  wwwwm.....WWWWWWWW
  m..e.m....waaaaaae
  m..eCC....wjDCaaae
  CC.eDC....wCCCBmae
  DC.eCB....waaaaaae
  CB.eBA....waajDCae
  BA.e.WwwW.wamCCCBe
  ...e.zkzshwaaaaaae
  ...ezzzssgWWWWWWWW
  ...e.zssgvgf......
  ..zz.vvggggggf....
  ...es.vggggggf....
  ...e.svgggggggf...
  ...e.vggggggff....
  ...e.vgggggff.....
  ...e.WWWwWWf......
  ...e..pPPPP.......`,
  (c) => { for (const x of [13, 16]) for (let y = 2; y <= 8; y++) if (y !== 5 || x === 16) c.px(x, y, "w") },
  frame(grid(5, 18, ".pP.PP..\n.pP.PP..\n.pP..PP.\npp...PPP")),
  frame(grid(5, 18, ".pP.PP..\n..pPP...\n..pPP...\n.ppPPP..")))

// ------------------------------------------------------------------ zealot (14x21)
// Hooded flagellant in sackcloth with a rope belt, bare scarred arms, an oil flask with a lit
// rag held out in front. He runs.
anim("zealot", { w: 12, h: 19, outline: "k", fps: 8 }, `
  ......QQQ...
  .....QrrrQ..
  ....QrrqqQQ.
  ....QzkqqQQ.
  ....zzSqqQQ.
  .....zsqQQQ.
  .Z..rrqqqQQ.
  .Zy.srrqqqQQ
  .r.sSrqqqqQQ
  .Uzs.rqqqqQ.
  JjJU.qqqqqQ.
  JJJU.eeeeeQ.
  UJUU.qqeqqQ.
  .UU..qqeqQQQ
  .....qqqqqQQ`,
  frame(grid(4, 15, "..sz..S.\n.sz....s\nsz.....sz\nPP.....PP")),
  frame(grid(4, 15, "...sS...\n...sz...\n...zs...\n..PPP...")))

// ================================================================== V19 boss
// ------------------------------------------------------------------ Captain of the Watch (52x66)
// The boss: a giant knight-captain, over twice a knight's height. Plumed barbute, full plate over
// the royal blue tabard, a blue cloak behind, a tower-high kite shield in front (the side facing
// Veyra) with the Watch's pale tower, a greatsword raised over the back shoulder. `charge` lowers
// him behind the shield, sword swept back, crest streaming.
function captain(c, step, charge) {
  const lean = charge ? 4 : 0, dy = charge ? 4 : 0
  // cloak behind, flaring at the hem, a fold down the middle
  c.poly([27, 20, 37, 19, 47, 56, 44, 61, 35, 58, 29, 60], "N")
  c.poly([30, 20, 37, 19, 45, 52, 39, 54], "n")
  c.line(36, 26, 41, 55, "N")
  // legs: cuisses, knee cops, greaves and sabatons; the stride swaps them
  const legs = step ? [[14, 0, 0], [27, 1, 0]] : [[19, 1, 1], [23, 0, 0]]
  for (const [x, back, up] of legs) {
    const y = 46 - up, [d, m, l] = back ? ["L", "l", "I"] : ["l", "I", "i"]
    c.rect(x, y, 7, 11, m)
    c.vline(x + 6, y, 11, d); c.vline(x, y, 11, l)
    c.rect(x, y + 4, 7, 3, l); c.hline(x, y + 6, 7, d); c.px(x + 2, y + 5, "j")
    c.rect(x - 2, y + 11, 10, 6 + up, m)
    c.hline(x - 2, y + 11, 9, l); c.hline(x - 2, y + 16 + up, 10, d)
    c.px(x - 2, y + 12, back ? "l" : "j")
  }
  // fauld and tabard skirt in royal blue, a darker back half
  c.rect(15, 37, 18, 12, "o"); c.rect(26, 37, 7, 12, "n")
  c.hline(15, 48, 18, "N"); c.vline(24, 41, 7, "n"); c.vline(19, 42, 6, "n")
  // breastplate: rounded, lit from the top left, a ridge down the middle
  const by = dy / 2
  c.ellipse(24 - lean, 29 + by, 10, 9, "L")
  c.ellipse(23 - lean, 28 + by, 9, 8, "l")
  c.ellipse(22 - lean, 27 + by, 7, 6, "I")
  c.ellipse(20 - lean, 25 + by, 4, 3, "i")
  c.px(18 - lean, 23 + by, "j"); c.px(19 - lean, 23 + by, "j"); c.px(18 - lean, 24 + by, "j")
  c.vline(23 - lean, 21 + by, 13, "l")
  c.hline(14 - lean + 1, 36, 19, "Y"); c.hline(14 - lean + 1, 37, 19, "K"); c.rect(21 - lean, 36, 3, 2, "e")
  // back pauldron and sword arm
  if (charge) {
    c.circ(31, 25 + by, 4, "l"); c.circ(30, 24 + by, 2, "I")
    c.line(30, 29 + by, 35, 33 + by, "I", 3)
    c.rect(35, 32 + by, 3, 3, "l")
    c.line(36, 30 + by, 36, 37 + by, "Y")
    c.line(38, 33 + by, 49, 35 + by, "j", 2); c.line(38, 35 + by, 48, 36 + by, "i")
    c.px(49, 36 + by, "i")
  } else {
    c.circ(32, 23, 4, "l"); c.circ(31, 22, 2, "I")
    c.line(33, 26, 37, 19, "I", 3)
    c.rect(36, 15, 4, 4, "l")
    c.line(33, 16, 43, 19, "Y"); c.px(43, 18, "Y"); c.px(33, 17, "Y")
    c.line(39, 15, 47, -1, "j", 2); c.line(40, 16, 48, 0, "i")
    c.line(37, 14, 38, 12, "e")
  }
  // gorget and helm: a barbute with a T slit, rivets, a blue horsehair crest sweeping back
  const hx = 16 - lean, hy = 4 + dy
  c.rect(hx + 2, hy + 15, 10, 3, "l"); c.hline(hx + 2, hy + 15, 10, "I")
  c.circ(hx + 7, hy + 7, 7, "l")
  c.circ(hx + 6, hy + 6, 6, "I")
  c.rect(hx, hy + 7, 13, 8, "I")
  c.rect(hx + 10, hy + 7, 3, 8, "l"); c.vline(hx + 13, hy + 6, 9, "L")
  c.circ(hx + 4, hy + 4, 3, "i"); c.px(hx + 3, hy + 2, "j"); c.px(hx + 4, hy + 2, "j"); c.px(hx + 2, hy + 3, "j")
  c.hline(hx + 1, hy + 8, 6, "k"); c.vline(hx + 4, hy + 8, 6, "k")
  c.hline(hx + 1, hy + 7, 6, "i")
  for (const [x, y] of [[9, 9], [9, 12], [11, 4]]) c.px(hx + x, hy + y, "j")
  c.hline(hx + 1, hy + 14, 11, "l")
  c.line(hx + 4, hy - 1, hx + 11, hy - 3, "o", 3)
  c.line(hx + 11, hy - 3, hx + 19 + lean, hy + 4 - lean * 1.5, "n", 3)
  c.line(hx + 17 + lean, hy + 2 - lean, hx + 22 + lean, hy + 9 - lean * 1.5, "N", 2)
  c.line(hx + 4, hy - 2, hx + 10, hy - 4, "X")
  // front pauldron, arm hidden behind the shield
  const sx = 2 - lean * 0.5, sy = 21 + dy / 2
  // kite shield: steel rim, blue field darker on the turned half, the pale tower of the Watch
  c.poly([sx, sy, sx + 19, sy, sx + 19, sy + 21, sx + 10, sy + 35, sx + 9, sy + 35, sx, sy + 21], "i")
  c.poly([sx + 2, sy + 2, sx + 17, sy + 2, sx + 17, sy + 21, sx + 9.5, sy + 32, sx + 2, sy + 21], "n")
  c.poly([sx + 10, sy + 2, sx + 17, sy + 2, sx + 17, sy + 21, sx + 10, sy + 31.5], "N")
  c.hline(sx + 2, sy + 2, 15, "o"); c.vline(sx + 2, sy + 2, 19, "o")
  c.vline(sx + 19, sy, 21, "I"); c.line(sx + 19, sy + 21, sx + 10, sy + 35, "I")
  // tower: battlements, body, a dark door and window
  c.rect(sx + 6, sy + 10, 7, 13, "X")
  c.rect(sx + 5, sy + 7, 9, 4, "X")
  for (const x of [5, 7, 9, 11, 13]) c.px(sx + x, sy + 6, "X")
  c.rect(sx + 9, sy + 18, 2, 5, "N"); c.px(sx + 9, sy + 12, "N"); c.px(sx + 9, sy + 13, "N")
  c.vline(sx + 12, sy + 10, 13, "j")
  c.circ(sx + 18, sy + 1, 4, "I"); c.circ(sx + 17, sy, 2, "i"); c.px(sx + 16, sy - 1, "j")
}
anim("captain", { w: 50, h: 64, outline: "k", fps: 5, frames: 2 }, (c) => captain(c, c.frame, false))
anim("captain_charge", { w: 50, h: 64, outline: "k", fps: 10, frames: 2 }, (c) => captain(c, c.frame, true))
