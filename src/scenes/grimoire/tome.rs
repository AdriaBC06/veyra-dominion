//! The Grimoire as Veyra's own book: an old tome bound in dark leather, iron at the corners,
//! worn and chipped vellum leaves.
//! - Left page: a summoning circle. Every entry is an inked roundel on its ring, over a
//!   pentagram drawn from the centre to the border; the whole circle turns slowly while runes
//!   turn the other way, and the tally of revealed entries sits at the star's heart.
//! - Right page: the focused entry's rite. The two ingredients run down in blood into the
//!   result's circle; changing entry sets the old leaf alight and it burns off the new one,
//!   whose name then writes itself in.
//! - Never fused: black silhouettes in dashed roundels, and on the rite a wax seal over the
//!   result. Fused but unseen: the silhouette waits in a beating red ring and blooms into colour
//!   the first time its rite is shown.
//! - All red ink is drawn like blood (slightly uneven, beads and short drips). Blood stains of
//!   many kinds mark the pages, never over text; more of them as the run's blood climbs. Each
//!   right leaf has its own, so they change (and burn) with it.
//! - Back is a red ribbon bookmark peeking out of the top of the book; it slides out and glows
//!   when highlighted.

use super::{ingredients, ready, result, texts, GrimoireScene};
use crate::assets::{draw_ui_text, measure_ui_text, TextStyle, ICON_SIZE};
use crate::burn::{self, value_noise};
use crate::hud::{self, Icon};
use crate::i18n::{tr, trf, Key};
use crate::kit;
use crate::motion;
use crate::save::{Entry, SaveData};
use crate::upgrades::Upgrades;
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};

/// Inner corners of a pentagram sit at this fraction of its tips' radius.
const PENTAGON: f32 = 0.382;
const BOOK_W: f32 = 600.0;
const BOOK_H: f32 = 306.0;
/// Room above the book for the ribbon.
const BOOK_Y: f32 = 34.0;
/// Leather showing around the pages, and the crease where the leaves meet.
const COVER: f32 = 9.0;
const SPINE: f32 = 2.0;
/// Radius of an entry's roundel (icons ×2 inside).
const RING: f32 = 19.0;
/// Clear vellum between the circle's border and the page edge.
const MARGIN: f32 = 12.0;
/// How far a leaf's outer bottom corner is worn round.
const WORN_CORNER: f32 = 5.0;
/// The ribbon: how far it peeks out (and out more when highlighted), its forked end, the
/// padding around its label.
const RIBBON_PEEK: f32 = 19.0;
const RIBBON_OUT: f32 = 10.0;
const RIBBON_FORK: f32 = 5.0;
const RIBBON_PAD: f32 = 9.0;
/// Seconds the old leaf takes to burn off the new one; the name writes once it is mostly gone.
const BURN: f32 = 0.6;
const WRITE: f32 = 0.4;

const LEATHER: Color = Color::new(0.19, 0.055, 0.075, 1.0);
const LEATHER_DARK: Color = Color::new(0.075, 0.02, 0.03, 1.0);
const LEATHER_LIT: Color = Color::new(0.3, 0.1, 0.11, 1.0);
const STITCH: Color = Color::new(0.47, 0.19, 0.14, 1.0);
const IRON: Color = Color::new(0.19, 0.18, 0.2, 1.0);
const IRON_DARK: Color = Color::new(0.08, 0.07, 0.09, 1.0);
const IRON_LIT: Color = Color::new(0.5, 0.47, 0.49, 1.0);
const VELLUM: Color = Color::new(0.56, 0.45, 0.33, 1.0);
const VELLUM_DARK: Color = Color::new(0.33, 0.24, 0.16, 1.0);
const VELLUM_EDGE: Color = Color::new(0.24, 0.15, 0.09, 1.0);
const INK: Color = Color::new(0.13, 0.055, 0.045, 1.0);
const RED_INK: Color = Color::new(0.54, 0.07, 0.07, 1.0);
/// Old blood, for the quieter lines of the circle.
const DRIED: Color = Color::new(0.38, 0.07, 0.055, 1.0);
/// What is still hidden: black silhouettes, plain ink for dashes and question marks.
const SHADOW: Color = Color::new(0.035, 0.015, 0.02, 1.0);
const SKETCH: Color = Color::new(0.25, 0.16, 0.1, 1.0);
/// The inked ground of a revealed roundel, so the icon's colours glow on the page.
const ROUNDEL: Color = Color::new(0.1, 0.035, 0.05, 1.0);
const WAX: Color = Color::new(0.5, 0.06, 0.08, 1.0);
const WAX_DARK: Color = Color::new(0.3, 0.025, 0.045, 1.0);
const WAX_LIT: Color = Color::new(0.72, 0.16, 0.16, 1.0);
const RIBBON: Color = Color::new(0.55, 0.06, 0.09, 1.0);
const RIBBON_DARK: Color = Color::new(0.33, 0.03, 0.06, 1.0);
const RIBBON_LIT: Color = Color::new(0.8, 0.14, 0.15, 1.0);
const HELLFIRE: Color = Color::new(1.0, 0.55, 0.15, 1.0);

struct Book {
    rect: Rect,
    left: Rect,
    right: Rect,
    centre: Vec2,
    radius: f32,
}

impl Book {
    fn new(view: Vec2) -> Self {
        let w = BOOK_W.min(view.x - 30.0).round();
        let rect = Rect::new(((view.x - w) * 0.5).round(), BOOK_Y, w, BOOK_H);
        let page_w = ((w - SPINE) * 0.5 - COVER).floor();
        let page_h = BOOK_H - 2.0 * COVER;
        let left = Rect::new(rect.x + COVER, rect.y + COVER, page_w, page_h);
        let right = Rect::new(
            rect.right() - COVER - page_w,
            rect.y + COVER,
            page_w,
            page_h,
        );
        let radius = (page_w.min(page_h) * 0.5 - MARGIN - RING - 9.0).floor();
        Book {
            rect,
            left,
            right,
            centre: left.center().round(),
            radius,
        }
    }

    /// Radius of the circle's border.
    fn outer(&self) -> f32 {
        self.radius + RING + 6.0
    }

    /// Centre of entry `i` of `n`, from the top, clockwise, turned by `spin`.
    fn on_ring(&self, i: usize, n: usize, spin: f32) -> Vec2 {
        let a = -FRAC_PI_2 + spin + i as f32 * TAU / n as f32;
        (self.centre + vec2(a.cos(), a.sin()) * self.radius).round()
    }

    /// The ribbon's label, the width that fits it, and its rect pulled out by `out` (0..1).
    fn ribbon(&self, out: f32) -> (&'static str, Rect) {
        let label = tr(Key::MenuBack);
        let w = (measure_ui_text(label, TextStyle::Desc).width + 2.0 * RIBBON_PAD).round();
        let peek = (RIBBON_PEEK + RIBBON_OUT * out).round();
        let x = self.right.right() - 40.0 - w;
        (label, Rect::new(x, self.rect.y - peek, w, peek))
    }
}

/// Focusable rects: every roundel, then the ribbon (fully pulled out, so it doesn't slip
/// from under the mouse as it slides).
pub(super) fn items(view: Vec2, n: usize, spin: f32) -> Vec<Rect> {
    let b = Book::new(view);
    let mut items: Vec<Rect> = (0..n)
        .map(|i| {
            let p = b.on_ring(i, n, spin);
            Rect::new(p.x - RING, p.y - RING, 2.0 * RING, 2.0 * RING)
        })
        .collect();
    items.push(b.ribbon(1.0).1);
    items
}

pub(super) fn draw(g: &GrimoireScene, view: Vec2, save: &SaveData, build: Option<&Upgrades>) {
    kit::dim(view, 1.0);
    let (alpha, dy) = motion::open(g.overlay.t);
    let mut b = Book::new(view);
    for r in [&mut b.rect, &mut b.left, &mut b.right] {
        r.y += dy;
    }
    b.centre.y += dy;
    let t = g.overlay.t;
    let n = g.entries.len();
    let back = n;
    let lit = |i: usize| g.focus.is_highlighted(i);
    let level = kit::blood_level();
    // a few stains always, none with the Blood setting off
    let base = (kit::gore() > 0.0) as u32;

    ribbon(&b, g.ribbon, lit(back), t, alpha);
    cover(b.rect, alpha);
    leaf(b.left, true, 11, alpha);
    leaf(b.right, false, leaf_seed(g.page), alpha);

    circle(&b, g.spin, alpha);
    stains(&b, Side::Left, base * 4 + (level * 8.0) as u32, alpha);
    let revealed = g.entries.iter().filter(|&&e| save.has_evolved(e)).count();
    let c = b.centre;
    // the tally sits in the pentagon at the heart of the star
    let tally = format!("{revealed} / {n}");
    ink_centred(
        &tally,
        c.x,
        c.y + 3.0,
        TextStyle::Name,
        kit::fade(RED_INK, alpha),
    );
    ink_centred(
        tr(Key::GrimoireRevealed),
        c.x,
        c.y + 17.0,
        TextStyle::Desc,
        kit::fade(INK, alpha),
    );

    for (i, &entry) in g.entries.iter().enumerate() {
        let (a, _) = motion::deal(g.overlay.dealt, i);
        let p = b.on_ring(i, n, g.spin);
        if build.is_some_and(|u| ready(entry, u)) {
            halo(p, RING, t, alpha * a);
        }
        let state = state_of(g, i, entry, save);
        roundel(result(entry), p, RING, state, i as u32, t, alpha * a);
        if lit(i) {
            blood_ring(
                p,
                RING + 3.0,
                900 + i as u32,
                kit::fade(RED_INK, alpha * a),
                true,
            );
        }
    }
    if let Some(i) = g.focus.highlight().filter(|&i| i < n) {
        let p = b.on_ring(i, n, g.spin);
        let r = Rect::new(p.x - RING, p.y - RING, 2.0 * RING, 2.0 * RING);
        kit::sparks(r, t, alpha);
    }

    // the right leaf: the new rite lies under the old one, which burns away off it
    let entry = g.entries[g.page];
    let state = state_of(g, g.page, entry, save);
    let ready = build.is_some_and(|u| ready(entry, u));
    let burning = g.prev.filter(|_| g.page_t < BURN);
    let since = if g.prev.is_some() {
        g.page_t - BURN * 0.6
    } else {
        g.page_t
    };
    let leaf_stains = base * 3 + (level * 8.0) as u32;
    stains(&b, Side::Right(g.page), leaf_stains, alpha);
    rite(&b, g.page, entry, state, ready, since.max(0.0), t, alpha);
    if let Some(old) = burning {
        let entry = g.entries[old];
        let state = state_of(g, old, entry, save);
        let old_ready = build.is_some_and(|u| super::ready(entry, u));
        let draw_old = || {
            leaf(b.right, false, leaf_seed(old), 1.0);
            stains(&b, Side::Right(old), leaf_stains, 1.0);
            rite(&b, old, entry, state, old_ready, WRITE * 4.0, t, 1.0);
        };
        burn::burn(b.right, g.page_t / BURN, old as u32, 0, draw_old, t, alpha);
    }
}

/// Each right leaf is its own sheet: its own browning, chips and tears.
fn leaf_seed(entry: usize) -> u32 {
    29 + entry as u32 * 7
}

/// How an entry shows: hidden, waiting to be revealed (fused, page not yet shown), revealing
/// (0..1) or plain revealed.
#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Locked,
    Waiting,
    Revealing(f32),
    Open,
}

fn state_of(g: &GrimoireScene, i: usize, entry: Entry, save: &SaveData) -> State {
    if !save.has_evolved(entry) {
        return State::Locked;
    }
    match (g.fresh[i], g.reveal[i]) {
        (false, _) => State::Open,
        (true, r) if r <= 0.0 => State::Waiting,
        (true, r) if r < 1.0 => State::Revealing(r),
        _ => State::Open,
    }
}

/// The circle on the left page: its double border, a pentagram from the centre to the
/// border (turning with the roundels), a circle through the star's inner corners and runes
/// turning the other way.
fn circle(b: &Book, spin: f32, alpha: f32) {
    let c = b.centre;
    let red = kit::fade(RED_INK, alpha);
    let dried = kit::fade(DRIED, alpha);
    let outer = b.outer();
    blood_ring(c, outer, 1, red, true);
    blood_ring(c, outer + 3.0, 2, dried, false);
    let tips: [Vec2; 5] = std::array::from_fn(|k| {
        c + Vec2::from_angle(-FRAC_PI_2 + spin + k as f32 * TAU / 5.0) * outer
    });
    for k in 0..5 {
        blood_line(tips[k], tips[(k + 2) % 5], 40 + k as u32, red);
    }
    // the star's inner corners lie on this circle
    blood_ring(c, (outer * PENTAGON).round(), 3, dried, false);
    runes(c, b.radius - RING - 8.0, -spin * 1.6, alpha);
}

/// An entry's roundel centred on `p`, `r` its radius and the icon at `r / RING` × 2.
fn roundel(icon: Icon, p: Vec2, r: f32, state: State, seed: u32, t: f32, alpha: f32) {
    let scale = (2.0 * r / RING).round();
    let size = ICON_SIZE * scale;
    let at = (p - Vec2::splat(size * 0.5)).round();
    let c = |col: Color| kit::fade(col, alpha);
    let ring_seed = 300 + seed * 7;
    match state {
        State::Locked => {
            dashed_circle(p, r, 18, c(SKETCH));
            hud::draw_icon_tinted(icon, at.x, at.y, scale, c(SHADOW));
        }
        State::Waiting => {
            // something stirs on the page: a beating ring around the black shape
            let beat = motion::pulse(t * 1.6);
            blood_ring(p, r, ring_seed, c(RED_INK), true);
            let ring = Color::new(RED_INK.r, RED_INK.g, RED_INK.b, alpha * beat);
            draw_circle_lines(p.x, p.y, r + 2.0 + 2.0 * beat, 1.0, ring);
            hud::draw_icon_tinted(icon, at.x, at.y, scale, c(SHADOW));
        }
        State::Revealing(k) => {
            let e = motion::ease_out_cubic(k);
            draw_circle(p.x, p.y, r * e, kit::fade(ROUNDEL, alpha * e));
            blood_ring(p, r, ring_seed, c(RED_INK), true);
            let tint = Color::new(
                SHADOW.r + (1.0 - SHADOW.r) * e,
                SHADOW.g + (1.0 - SHADOW.g) * e,
                SHADOW.b + (1.0 - SHADOW.b) * e,
                alpha,
            );
            hud::draw_icon_tinted(icon, at.x, at.y, scale, tint);
            // a ring of hellfire spreads out as the colour floods in
            let wave = r + 4.0 + 18.0 * e;
            draw_circle_lines(p.x, p.y, wave, 2.0, kit::fade(HELLFIRE, alpha * (1.0 - e)));
            let rect = Rect::new(p.x - r, p.y - r, 2.0 * r, 2.0 * r);
            kit::sparks(rect, t, alpha * (1.0 - e));
        }
        State::Open => {
            draw_circle(p.x, p.y, r, c(ROUNDEL));
            blood_ring(p, r, ring_seed, c(RED_INK), true);
            hud::draw_icon_tinted(icon, at.x, at.y, scale, c(WHITE));
        }
    }
}

/// Where the rite's parts sit on the right page.
struct RiteLayout {
    cx: f32,
    label_y: f32,
    name_y: f32,
    rule_y: f32,
    a: Vec2,
    b: Vec2,
    result: Vec2,
    desc_y: f32,
    note_y: f32,
    tail_y: f32,
}

impl RiteLayout {
    fn new(page: Rect) -> Self {
        let (cx, y) = (page.center().x.round(), page.y);
        RiteLayout {
            cx,
            label_y: y + 26.0,
            name_y: y + 55.0,
            rule_y: y + 66.0,
            a: vec2(cx - 66.0, y + 104.0),
            b: vec2(cx + 66.0, y + 104.0),
            result: vec2(cx, y + 172.0),
            desc_y: y + 228.0,
            note_y: y + 263.0,
            tail_y: y + 276.0,
        }
    }

    /// Where stains must not land: every line of text and the recipe.
    fn keep_clear(&self, page: Rect) -> [Rect; 5] {
        let cx = self.cx;
        [
            Rect::new(cx - 45.0, self.label_y - 14.0, 90.0, 18.0),
            Rect::new(page.x + 12.0, self.name_y - 20.0, page.w - 24.0, 32.0),
            Rect::new(
                cx - 92.0,
                self.a.y - RING - 6.0,
                184.0,
                self.result.y - self.a.y + 2.0 * RING + 22.0,
            ),
            Rect::new(page.x + 12.0, self.desc_y - 14.0, page.w - 24.0, 34.0),
            Rect::new(cx - 60.0, self.note_y - 14.0, 120.0, 28.0),
        ]
    }
}

/// The right page: the rite's number, the name writing itself in, the ingredients running
/// into the result (a wax seal while hidden), the description and a closing note.
#[allow(clippy::too_many_arguments)]
fn rite(
    b: &Book,
    idx: usize,
    entry: Entry,
    state: State,
    ready: bool,
    since: f32,
    t: f32,
    alpha: f32,
) {
    let page = b.right;
    let l = RiteLayout::new(page);
    let c = |col: Color| kit::fade(col, alpha);
    let open = state != State::Locked;
    let cx = l.cx;

    let label = trf(Key::GrimoireRite, &[&roman(idx + 1)]);
    ink_centred(&label, cx, l.label_y, TextStyle::Desc, c(RED_INK));
    let (name, desc) = match open {
        true => {
            let (name, desc) = texts(entry);
            (tr(name), tr(desc))
        }
        false => (tr(Key::GrimoireUnknown), ""),
    };
    // the name writes itself in, letter by letter, once the leaf has formed
    let k = (since / WRITE).clamp(0.0, 1.0);
    let written = (k * name.chars().count() as f32).ceil() as usize;
    let end = name
        .char_indices()
        .nth(written)
        .map_or(name.len(), |(i, _)| i);
    let full = measure_ui_text(name, TextStyle::Name).width;
    let name_color = if open { INK } else { SKETCH };
    draw_ui_text(
        &name[..end],
        (cx - full * 0.5).round(),
        l.name_y,
        TextStyle::Name,
        c(name_color),
    );
    flourish(vec2(cx, l.rule_y), 70.0, 50 + idx as u32, c(RED_INK));

    // the rite
    let [first, second] = ingredients(entry);
    let big = RING * 1.5;
    for (k, p) in [l.a, l.b].into_iter().enumerate() {
        let dir = (l.result - p).normalize();
        let from = p + dir * (RING + 3.0);
        let to = l.result - dir * (big + 7.0);
        if open {
            blood_line(from, to, 60 + k as u32 + idx as u32 * 3, c(RED_INK));
            draw_poly(to.x, to.y, 3, 3.5, dir.to_angle().to_degrees(), c(RED_INK));
        } else {
            dotted_line(from, to, c(SKETCH));
        }
    }
    let plus = if open { RED_INK } else { SKETCH };
    ink_centred("+", cx, l.a.y + 5.0, TextStyle::Name, c(plus));
    if open {
        let shown = if state == State::Waiting {
            State::Waiting
        } else {
            State::Open
        };
        roundel(first, l.a, RING, shown, 70 + idx as u32, t, alpha);
        roundel(second, l.b, RING, shown, 71 + idx as u32, t, alpha);
    } else {
        for p in [l.a, l.b] {
            dashed_circle(p, RING, 18, c(SKETCH));
            ink_centred("?", p.x, p.y + 7.0, TextStyle::Name, c(SKETCH));
        }
    }
    if ready {
        halo(l.result, big, t, alpha);
    }
    blood_ring(l.result, big + 4.0, 80 + idx as u32, c(DRIED), false);
    if open {
        roundel(
            result(entry),
            l.result,
            big,
            state,
            81 + idx as u32,
            t,
            alpha,
        );
    } else {
        seal(result(entry), l.result, big, alpha);
    }

    let (line1, line2) = hud::wrap_two(desc, page.w - 40.0);
    ink_centred(line1, cx, l.desc_y, TextStyle::Desc, c(INK));
    ink_centred(line2, cx, l.desc_y + 14.0, TextStyle::Desc, c(INK));
    let note = match () {
        _ if ready => Some(Key::GrimoireReady),
        _ if matches!(state, State::Waiting | State::Revealing(_)) => Some(Key::LevelUpNew),
        _ => None,
    };
    if let Some(key) = note {
        let s = tr(key);
        let w = measure_ui_text(s, TextStyle::Desc).width;
        ink_centred(s, cx, l.note_y, TextStyle::Desc, c(RED_INK));
        for side in [-1.0, 1.0] {
            draw_poly(
                cx + side * (w * 0.5 + 8.0),
                l.note_y - 4.0,
                4,
                2.5,
                0.0,
                c(RED_INK),
            );
        }
    }
    flourish(vec2(cx, l.tail_y), 40.0, 90 + idx as u32, c(RED_INK));
}

/// The leather binding: shadow, grained and stitched cover, the crease and iron corners.
fn cover(r: Rect, alpha: f32) {
    let c = |col: Color| kit::fade(col, alpha);
    draw_rectangle(
        r.x + 3.0,
        r.y + 5.0,
        r.w,
        r.h,
        kit::fade(BLACK, alpha * 0.6),
    );
    draw_rectangle(r.x, r.y, r.w, r.h, c(LEATHER));
    // grain and scuffs
    for k in 0..360u32 {
        let x = (r.x + hash(k * 3 + 5) * r.w).round();
        let y = (r.y + hash(k * 3 + 6) * r.h).round();
        let col = if hash(k * 3 + 7) > 0.6 {
            LEATHER_LIT
        } else {
            LEATHER_DARK
        };
        draw_rectangle(x, y, 1.0 + (hash(k + 999) > 0.85) as u8 as f32, 1.0, c(col));
    }
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, c(LEATHER_DARK));
    draw_rectangle(r.x + 1.0, r.y + 1.0, r.w - 2.0, 1.0, c(LEATHER_LIT));
    // stitches along the edge
    let (x0, y0, x1, y1) = (r.x + 3.0, r.y + 3.0, r.right() - 4.0, r.bottom() - 4.0);
    let mut x = x0 + 4.0;
    while x < x1 - 4.0 {
        draw_rectangle(x, y0, 3.0, 1.0, c(STITCH));
        draw_rectangle(x, y1, 3.0, 1.0, c(STITCH));
        x += 6.0;
    }
    let mut y = y0 + 4.0;
    while y < y1 - 4.0 {
        draw_rectangle(x0, y, 1.0, 3.0, c(STITCH));
        draw_rectangle(x1, y, 1.0, 3.0, c(STITCH));
        y += 6.0;
    }
    // where the leaves meet
    let mid = r.center().x.round();
    draw_rectangle(mid - SPINE * 0.5, r.y, SPINE, r.h, c(LEATHER_DARK));
    for (cx, cy, sx, sy) in [
        (r.x, r.y, 1.0, 1.0),
        (r.right(), r.y, -1.0, 1.0),
        (r.x, r.bottom(), 1.0, -1.0),
        (r.right(), r.bottom(), -1.0, -1.0),
    ] {
        corner(vec2(cx, cy), sx, sy, alpha);
    }
}

/// An iron corner: two arms with a claw curl at each end, a triangular plate and a rivet.
fn corner(at: Vec2, sx: f32, sy: f32, alpha: f32) {
    let c = |col: Color| kit::fade(col, alpha);
    let px = |dx: f32, dy: f32, w: f32, h: f32, col: Color| {
        let x = if sx > 0.0 { at.x + dx } else { at.x - dx - w };
        let y = if sy > 0.0 { at.y + dy } else { at.y - dy - h };
        draw_rectangle(x, y, w, h, c(col));
    };
    const ARM: f32 = 22.0;
    px(0.0, 0.0, ARM, 5.0, IRON_DARK);
    px(0.0, 0.0, 5.0, ARM, IRON_DARK);
    px(0.0, 0.0, ARM - 1.0, 4.0, IRON);
    px(0.0, 0.0, 4.0, ARM - 1.0, IRON);
    // the plate fills the corner in steps
    for k in 0..9 {
        let k = k as f32;
        px(4.0, 4.0 + k, 9.0 - k, 1.0, IRON);
    }
    px(0.0, 0.0, ARM - 1.0, 1.0, IRON_LIT);
    px(0.0, 0.0, 1.0, ARM - 1.0, IRON_LIT);
    // claw curls at the arm ends
    px(ARM - 3.0, 5.0, 3.0, 2.0, IRON);
    px(ARM - 2.0, 7.0, 2.0, 1.0, IRON_DARK);
    px(5.0, ARM - 3.0, 2.0, 3.0, IRON);
    px(7.0, ARM - 2.0, 1.0, 2.0, IRON_DARK);
    // rivets
    px(5.0, 5.0, 2.0, 2.0, IRON_LIT);
    px(14.0, 1.0, 2.0, 2.0, IRON_DARK);
    px(1.0, 14.0, 2.0, 2.0, IRON_DARK);
}

/// A vellum leaf: the stack of leaves under it, soft browning towards the edges, foxing,
/// chipped edges and the shade of the fold.
fn leaf(r: Rect, left: bool, seed: u32, alpha: f32) {
    let c = |col: Color, a: f32| kit::fade(col, alpha * a);
    // leaves under this one: uneven edges peeking out at the side and the bottom, rounded
    // at the outer bottom corner like the leaf itself
    let bend = WORN_CORNER;
    for k in (1..=4).rev() {
        let kf = k as f32;
        let shade = if k % 2 == 0 { VELLUM_DARK } else { VELLUM_EDGE };
        let side_x = if left { r.x - kf } else { r.right() + kf - 1.0 };
        let side_end = r.bottom() - bend + kf;
        let mut y = r.y + kf;
        while y < side_end {
            let len = 3.0 + (hash(seed + k * 97 + y as u32) * 6.0).round();
            if hash(seed * 5 + k + y as u32 * 3) > 0.12 {
                draw_rectangle(side_x, y, 1.0, len.min(side_end - y), c(shade, 1.0));
            }
            y += len;
        }
        // the bottom edge runs from the fold to where the corner bends
        let (x0, x1) = if left {
            (r.x + bend - kf, r.right())
        } else {
            (r.x, r.right() - bend + kf)
        };
        let mut x = x0;
        while x < x1 {
            let len = 4.0 + (hash(seed + k * 71 + x as u32) * 8.0).round();
            if hash(seed * 7 + k + x as u32 * 5) > 0.12 {
                draw_rectangle(
                    x,
                    r.bottom() + kf - 1.0,
                    len.min(x1 - x),
                    1.0,
                    c(shade, 1.0),
                );
            }
            x += len;
        }
        for j in 0..bend as i32 {
            let j = j as f32;
            let x = if left { side_x + j } else { side_x - j };
            draw_rectangle(x, side_end + j, 1.0, 1.0, c(shade, 1.0));
        }
    }
    draw_rectangle(r.x, r.y, r.w, r.h, c(VELLUM, 1.0));
    // aged towards the edges in soft, uneven browning; the middle stays clean
    let cell = 4.0;
    let (cols, rows) = ((r.w / cell).ceil() as i32, (r.h / cell).ceil() as i32);
    for row in 0..rows {
        for col in 0..cols {
            let (x, y) = (r.x + col as f32 * cell, r.y + row as f32 * cell);
            let edge = (x - r.x)
                .min(r.right() - x)
                .min(y - r.y)
                .min(r.bottom() - y);
            let near = (1.0 - edge / 46.0).max(0.0);
            let v = value_noise(col as f32 / 11.0, row as f32 / 11.0, seed);
            let a = near * near * (0.35 + 0.65 * v) * 0.3;
            if a < 0.02 {
                continue;
            }
            let w = cell.min(r.right() - x);
            let h = cell.min(r.bottom() - y);
            draw_rectangle(x, y, w, h, c(VELLUM_EDGE, a));
        }
    }
    for k in 0..12u32 {
        let s = seed * 613 + k * 17;
        let x = r.x + 6.0 + hash(s) * (r.w - 12.0);
        let y = r.y + 6.0 + hash(s + 1) * (r.h - 12.0);
        let rad = 0.8 + hash(s + 2) * hash(s + 3) * 3.5;
        draw_circle(
            x.round(),
            y.round(),
            rad,
            c(VELLUM_EDGE, 0.25 + 0.25 * hash(s + 4)),
        );
    }
    // the fold's shade
    for i in 0..16 {
        let x = if left {
            r.right() - 1.0 - i as f32
        } else {
            r.x + i as f32
        };
        let a = 0.55 * (1.0 - i as f32 / 16.0).powi(2);
        draw_rectangle(x, r.y, 1.0, r.h, c(Color::new(0.15, 0.08, 0.045, 1.0), a));
    }
    worn_edges(r, left, seed, alpha);
}

/// Chips and nicks along a leaf's free edges (the top, the bottom and its outer side), a
/// few torn notches and a rubbed bottom corner.
fn worn_edges(r: Rect, left: bool, seed: u32, alpha: f32) {
    let bed = kit::fade(LEATHER, alpha);
    let edge = kit::fade(VELLUM_EDGE, alpha);
    let nick = |i: u32| -> f32 {
        let h = hash(seed * 131 + i);
        if h > 0.82 {
            1.0 + (hash(seed * 7 + i) * 2.5).floor()
        } else if h > 0.55 {
            1.0
        } else {
            0.0
        }
    };
    // top and bottom
    let mut x = r.x;
    let mut i = 0;
    while x < r.right() {
        let (dt, db) = (nick(i), nick(i + 5000));
        draw_rectangle(x, r.y, 2.0, dt, bed);
        draw_rectangle(x, r.y + dt, 2.0, 1.0, edge);
        draw_rectangle(x, r.bottom() - db, 2.0, db, bed);
        draw_rectangle(x, r.bottom() - db - 1.0, 2.0, 1.0, edge);
        x += 2.0;
        i += 1;
    }
    // the outer side
    let mut y = r.y;
    while y < r.bottom() {
        let d = nick(i + 9000);
        let (bx, ex) = if left {
            (r.x, r.x + d)
        } else {
            (r.right() - d, r.right() - d - 1.0)
        };
        draw_rectangle(bx, y, d, 2.0, bed);
        draw_rectangle(ex, y, 1.0, 2.0, edge);
        y += 2.0;
        i += 1;
    }
    // torn notches on the free edges
    for k in 0..3u32 {
        let s = seed * 37 + k * 13;
        let depth = 4.0 + hash(s) * 5.0;
        let half = 2.0 + hash(s + 1) * 3.0;
        match k {
            0 => {
                let x = r.x + 30.0 + hash(s + 2) * (r.w - 60.0);
                draw_triangle(
                    vec2(x - half, r.y),
                    vec2(x + half, r.y),
                    vec2(x + 1.0, r.y + depth),
                    bed,
                );
            }
            1 => {
                let x = r.x + 30.0 + hash(s + 2) * (r.w - 60.0);
                let b = r.bottom();
                draw_triangle(
                    vec2(x - half, b),
                    vec2(x + half, b),
                    vec2(x - 1.0, b - depth),
                    bed,
                );
            }
            _ => {
                let y = r.y + 40.0 + hash(s + 2) * (r.h - 80.0);
                let side = if left { r.x } else { r.right() };
                let tip = side + if left { depth } else { -depth };
                draw_triangle(
                    vec2(side, y - half),
                    vec2(side, y + half),
                    vec2(tip, y + 1.0),
                    bed,
                );
            }
        }
    }
    // a rubbed, rounded bottom corner on the outer side
    for k in 0..WORN_CORNER as i32 {
        let k = k as f32;
        // the cut widens towards the bottom edge
        let w = k + 1.0;
        let x = if left { r.x } else { r.right() - w };
        draw_rectangle(x, r.bottom() - WORN_CORNER + k, w, 1.0, bed);
    }
}

/// Where a set of stains lands.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    /// The right leaf showing this entry: each leaf has its own stains.
    Right(usize),
}

/// `count` stains of every kind on `side`, placed in a fixed order for each seed (so more
/// blood only adds to what is there) and never over text, the circle's roundels or a recipe.
fn stains(b: &Book, side: Side, count: u32, alpha: f32) {
    let seed = match side {
        Side::Left => 0x51ab,
        Side::Right(i) => 0x7e11 + i as u32 * 131,
    };
    let rite = RiteLayout::new(b.right).keep_clear(b.right);
    let c = b.centre;
    let clear_left = |p: Vec2, r: f32| {
        let d = p.distance(c);
        let title = Rect::new(c.x - 40.0, c.y - 18.0, 80.0, 42.0);
        (d + r < b.radius - RING - 3.0 || d - r > b.radius + RING + 3.0) && !overlaps(title, p, r)
    };
    let clear_right = |p: Vec2, r: f32| rite.iter().all(|&z| !overlaps(z, p, r));
    for i in 0..count {
        let s = seed.wrapping_mul(2_654_435_761) ^ (i * 7919);
        let kind = Stain::pick(hash(s));
        let size = kind.size(hash(s + 1));
        for attempt in 0..12u32 {
            let h = |k: u32| hash(s + 100 + attempt * 17 + k);
            let p = match side {
                Side::Left => vec2(
                    b.left.x + size + 6.0 + h(0) * (b.left.w - 2.0 * size - 12.0),
                    b.left.y + size + 6.0 + h(1) * (b.left.h - 2.0 * size - 12.0),
                ),
                Side::Right(_) => vec2(
                    b.right.x + size + 6.0 + h(0) * (b.right.w - 2.0 * size - 12.0),
                    b.right.y + size + 6.0 + h(1) * (b.right.h - 2.0 * size - 12.0),
                ),
            };
            let ok = match side {
                Side::Left => clear_left(p, size),
                Side::Right(_) => clear_right(p, size),
            };
            if ok {
                kind.draw(p.round(), size, s, alpha);
                break;
            }
        }
    }
}

fn overlaps(z: Rect, p: Vec2, r: f32) -> bool {
    p.x + r > z.x && p.x - r < z.right() && p.y + r > z.y && p.y - r < z.bottom()
}

/// The kinds of stain: each its own shape and range of sizes.
#[derive(Clone, Copy)]
enum Stain {
    /// A round drop with a crown of droplets.
    Drop,
    /// A fine spray flung in one direction.
    Spatter,
    /// A wide pool, darker at its rim where it dried first.
    Pool,
    /// A streak dragged across the page.
    Smear,
    /// A drop that ran down the page and ended in a bead.
    Run,
    /// A few loose specks.
    Specks,
}

impl Stain {
    fn pick(h: f32) -> Stain {
        match h {
            h if h < 0.24 => Stain::Drop,
            h if h < 0.42 => Stain::Spatter,
            h if h < 0.52 => Stain::Pool,
            h if h < 0.64 => Stain::Smear,
            h if h < 0.78 => Stain::Run,
            _ => Stain::Specks,
        }
    }

    /// The radius it covers, from a 0..1 roll: some tiny, some large.
    fn size(self, h: f32) -> f32 {
        let (lo, hi) = match self {
            Stain::Drop => (1.5, 7.0),
            Stain::Spatter => (8.0, 22.0),
            Stain::Pool => (9.0, 20.0),
            Stain::Smear => (10.0, 26.0),
            Stain::Run => (5.0, 18.0),
            Stain::Specks => (5.0, 14.0),
        };
        // most stains small, the odd one large
        (lo + (hi - lo) * h * h).round()
    }

    fn draw(self, p: Vec2, size: f32, s: u32, alpha: f32) {
        let h = |k: u32| hash(s + 300 + k);
        let body = blood_colour(s, 1.0);
        let col = |c: Color| kit::fade(c, alpha);
        let dark = darker(body, 0.7);
        match self {
            Stain::Drop => {
                // an oval when it landed at a slant
                let dir = Vec2::from_angle(h(0) * TAU);
                let stretch = h(1) * size * 0.5;
                draw_circle(p.x, p.y, size, col(dark));
                draw_circle(p.x - 0.5, p.y - 0.5, (size - 0.8).max(0.6), col(body));
                for k in 1..=3 {
                    let q = p + dir * stretch * k as f32 / 3.0;
                    draw_circle(
                        q.x,
                        q.y,
                        (size * (1.0 - k as f32 * 0.25)).max(0.6),
                        col(body),
                    );
                }
                let crown = (h(2) * size * 1.5) as u32;
                for k in 0..crown {
                    let a = h(10 + k) * TAU;
                    let d = size + 1.5 + h(30 + k) * size * 0.9;
                    let q = p + Vec2::from_angle(a) * d;
                    draw_circle(q.x.round(), q.y.round(), 0.5 + h(50 + k) * 1.0, col(body));
                }
                if size > 3.0 {
                    let lit = lighter(body, 1.35);
                    draw_rectangle(
                        (p.x - size * 0.4).round(),
                        (p.y - size * 0.45).round(),
                        2.0,
                        1.0,
                        col(lit),
                    );
                }
            }
            Stain::Spatter => {
                let dir = Vec2::from_angle(h(0) * TAU);
                let side = vec2(-dir.y, dir.x);
                let n = 10 + (h(1) * 26.0) as u32;
                for k in 0..n {
                    let along = h(10 + k) * size;
                    let across = (h(40 + k) - 0.5) * size * 0.8 * (along / size + 0.2);
                    let q = p - dir * size * 0.5 + dir * along + side * across;
                    let r = (0.5 + h(70 + k) * h(80 + k) * 2.4) * (1.2 - along / size * 0.6);
                    draw_circle(q.x.round(), q.y.round(), r.max(0.5), col(body));
                    if r > 1.3 {
                        // the bigger drops leave a tail
                        let tail = q + dir * (r + 1.0);
                        draw_line(q.x, q.y, tail.x, tail.y, 1.0, col(body));
                    }
                }
            }
            Stain::Pool => {
                let n = 6 + (h(0) * 6.0) as usize;
                let mut blobs = [(Vec2::ZERO, 0.0); 12];
                for (k, blob) in blobs.iter_mut().enumerate().take(n) {
                    let k = k as u32;
                    let a = h(10 + k) * TAU;
                    let d = h(20 + k) * size * 0.5;
                    *blob = (p + Vec2::from_angle(a) * d, size * (0.35 + 0.3 * h(30 + k)));
                }
                let blobs = &blobs[..n];
                for &(q, r) in blobs {
                    draw_circle(q.x, q.y, r + 1.0, col(dark));
                }
                for &(q, r) in blobs {
                    draw_circle(q.x, q.y, r, col(body));
                }
                let wet = lighter(body, 1.25);
                for &(q, r) in blobs.iter().take(2) {
                    draw_circle(q.x - r * 0.3, q.y - r * 0.3, r * 0.35, col(wet));
                }
                for k in 0..(h(5) * 8.0) as u32 {
                    let q = p + Vec2::from_angle(h(60 + k) * TAU) * (size + 2.0 + h(70 + k) * 5.0);
                    draw_circle(q.x.round(), q.y.round(), 0.6 + h(80 + k), col(body));
                }
            }
            Stain::Smear => {
                // a dragged finger: wide where it started, thinning out, streaked inside
                let dir = Vec2::from_angle(h(0) * TAU);
                let bend = vec2(-dir.y, dir.x) * (h(1) - 0.5) * size * 0.4;
                let thin = mix(body, VELLUM, 0.35);
                let steps = (size * 1.4) as u32;
                for k in 0..steps {
                    let f = k as f32 / steps as f32;
                    let q = p - dir * size + dir * 2.0 * size * f + bend * (f * (1.0 - f) * 4.0);
                    let r = (size * 0.32 * (1.0 - f * 0.8)).max(0.7);
                    draw_circle(q.x, q.y, r, col(if f < 0.25 { body } else { thin }));
                }
                for k in 0..3 {
                    let off = vec2(-dir.y, dir.x) * (k as f32 - 1.0) * size * 0.14;
                    let a = p - dir * size * 0.9 + off;
                    let b = p + dir * size * (0.4 + 0.4 * h(90 + k)) + off + bend;
                    draw_line(a.x, a.y, b.x, b.y, 1.0, col(dark));
                }
            }
            Stain::Run => {
                let r = 1.8 + h(0) * 2.5;
                let top = p - vec2(0.0, size * 0.6);
                draw_circle(top.x, top.y, r + 0.8, col(dark));
                draw_circle(top.x, top.y, r, col(body));
                let len = size * 1.4;
                let mut y = 0.0;
                while y < len {
                    let w = (r * (1.0 - y / len) * 1.2).max(1.0).round();
                    let wobble = ((y * 0.4 + h(1) * 9.0).sin() * 0.6).round();
                    draw_rectangle(
                        (top.x - w * 0.5 + wobble).round(),
                        top.y + y,
                        w,
                        1.0,
                        col(body),
                    );
                    y += 1.0;
                }
                draw_circle(top.x, top.y + len + 0.5, 1.6 + h(2), col(dark));
            }
            Stain::Specks => {
                let n = 3 + (h(0) * 9.0) as u32;
                for k in 0..n {
                    let q = p + vec2(h(10 + k) - 0.5, h(20 + k) - 0.5) * size * 2.0;
                    let r = 0.5 + h(30 + k) * h(40 + k) * 1.8;
                    draw_circle(
                        q.x.round(),
                        q.y.round(),
                        r,
                        col(darker(body, 0.85 + 0.3 * h(50 + k))),
                    );
                }
            }
        }
    }
}

/// A blood colour from fresh crimson to old brown, `k` scaling its brightness.
fn blood_colour(s: u32, k: f32) -> Color {
    let fresh = Color::new(0.56, 0.035, 0.05, 1.0);
    let old = Color::new(0.27, 0.06, 0.035, 1.0);
    let c = mix(fresh, old, hash(s ^ 0xb100d));
    darker(c, k)
}

fn mix(a: Color, b: Color, k: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * k,
        a.g + (b.g - a.g) * k,
        a.b + (b.b - a.b) * k,
        a.a,
    )
}

fn darker(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

fn lighter(c: Color, k: f32) -> Color {
    Color::new(
        (c.r * k).min(1.0),
        (c.g * k).min(1.0),
        (c.b * k).min(1.0),
        c.a,
    )
}

/// A stroke of blood from `a` to `b`: it wavers a little, thickens here and there, and
/// leaves the odd bead.
fn blood_line(a: Vec2, b: Vec2, seed: u32, color: Color) {
    let len = a.distance(b);
    if len < 0.5 {
        return;
    }
    let dir = (b - a) / len;
    let nrm = vec2(-dir.y, dir.x);
    let steps = (len / 5.0).ceil().max(1.0) as u32;
    let mut prev = a;
    for k in 1..=steps {
        let s = seed.wrapping_mul(97) + k * 13;
        let f = k as f32 / steps as f32;
        let jitter = if k == steps {
            0.0
        } else {
            (hash(s) - 0.5) * 0.9
        };
        let p = a + (b - a) * f + nrm * jitter;
        let thick = if hash(s + 1) > 0.78 { 2.0 } else { 1.0 };
        draw_line(prev.x, prev.y, p.x, p.y, thick, shade(color, s + 2));
        if hash(s + 3) > 0.94 {
            draw_circle(p.x, p.y, 1.2, color);
        }
        prev = p;
    }
}

/// A ring of blood: uneven like `blood_line`; with `drips`, short drips hang from its lower
/// half.
fn blood_ring(c: Vec2, r: f32, seed: u32, color: Color, drips: bool) {
    let segments = ((TAU * r) / 4.0).ceil().max(16.0) as u32;
    let point = |k: u32| {
        let a = k as f32 * TAU / segments as f32;
        let s = seed.wrapping_mul(131) + (k % segments) * 7;
        c + Vec2::from_angle(a) * (r + (hash(s) - 0.5) * 0.9)
    };
    let mut prev = point(0);
    for k in 1..=segments {
        let s = seed.wrapping_mul(131) + k * 7;
        let p = point(k);
        let thick = if hash(s + 1) > 0.8 { 2.0 } else { 1.0 };
        draw_line(prev.x, prev.y, p.x, p.y, thick, shade(color, s + 2));
        if hash(s + 3) > 0.96 {
            draw_circle(p.x, p.y, 1.2, color);
        }
        if drips && p.y > c.y + r * 0.5 && hash(s + 4) > 0.9 {
            let len = 1.0 + (hash(s + 5) * 3.0).round();
            draw_rectangle(p.x.round(), p.y.round(), 1.0, len, color);
        }
        prev = p;
    }
}

/// `color` a touch lighter or darker, so a stroke isn't one flat red.
fn shade(color: Color, s: u32) -> Color {
    let k = 0.82 + 0.3 * hash(s);
    Color::new((color.r * k).min(1.0), color.g * k, color.b * k, color.a)
}

/// The red ribbon bookmark peeking out of the top of the book: forked end up, "Back" across
/// it. Pulled out by `out` (0..1); highlighted it brightens in a soft hellfire glow.
fn ribbon(b: &Book, out: f32, lit: bool, t: f32, alpha: f32) {
    let (label, r) = b.ribbon(out);
    let body = mix(RIBBON, RIBBON_LIT, out);
    if lit {
        halo_rect(r, t, alpha);
    }
    let col = kit::fade(body, alpha);
    let dark = kit::fade(RIBBON_DARK, alpha);
    let (l, m, rr, top) = (r.x, r.x + r.w * 0.5, r.right(), r.y);
    let fork = top + RIBBON_FORK;
    // the body runs on under the book
    draw_rectangle(l, fork, r.w, r.h - RIBBON_FORK + 12.0, col);
    draw_triangle(vec2(l, top), vec2(l, fork), vec2(m, fork), col);
    draw_triangle(vec2(rr, top), vec2(m, fork), vec2(rr, fork), col);
    draw_rectangle(l, top, 1.0, r.h + 12.0, dark);
    draw_rectangle(rr - 1.0, top, 1.0, r.h + 12.0, dark);
    // woven ridges
    let mut y = fork + 2.0;
    while y < r.bottom() {
        draw_rectangle(
            l + 2.0,
            y,
            r.w - 4.0,
            1.0,
            kit::fade(darker(body, 0.82), alpha),
        );
        y += 3.0;
    }
    let bone = if lit {
        kit::pal().label_lit
    } else {
        kit::pal().label
    };
    let y = top + RIBBON_FORK + 11.0;
    kit::text_centered(
        label,
        r.center().x,
        y,
        TextStyle::Desc,
        kit::fade(bone, alpha),
    );
}

/// A breathing hellfire glow around a rect.
fn halo_rect(r: Rect, t: f32, alpha: f32) {
    let g = motion::pulse(t);
    for (k, base) in [(4.0, 0.12), (2.0, 0.3)] {
        let a = alpha * base * (0.6 + 0.4 * g);
        draw_rectangle(
            r.x - k,
            r.y - k,
            r.w + 2.0 * k,
            r.h + k,
            kit::fade(HELLFIRE, a),
        );
    }
}

/// A red wax seal pressed over a hidden result: a blob with a rim and the black silhouette
/// pressed into it.
fn seal(icon: Icon, p: Vec2, r: f32, alpha: f32) {
    let c = |col: Color| kit::fade(col, alpha);
    for k in 0..7u32 {
        let a = k as f32 * TAU / 7.0 + 0.4;
        let d = r - 6.0 + 3.0 * hash(k + 91);
        let q = p + Vec2::from_angle(a) * d;
        draw_circle(q.x, q.y, 6.0 + 3.0 * hash(k + 17), c(WAX));
    }
    draw_circle(p.x + 1.0, p.y + 2.0, r - 2.0, c(WAX_DARK));
    draw_circle(p.x, p.y, r - 2.0, c(WAX));
    draw_circle_lines(p.x, p.y, r - 7.0, 2.0, c(WAX_DARK));
    draw_circle_lines(
        p.x - 1.0,
        p.y - 1.0,
        r - 7.0,
        1.0,
        c(mix(WAX, WAX_LIT, 0.5)),
    );
    let size = ICON_SIZE * 2.0;
    let at = (p - Vec2::splat(size * 0.5)).round();
    hud::draw_icon_tinted(icon, at.x + 1.0, at.y + 1.0, 2.0, c(mix(WAX, WAX_LIT, 0.4)));
    hud::draw_icon_tinted(icon, at.x, at.y, 2.0, c(SHADOW));
    // a glint on the wax
    draw_rectangle(
        (p.x - r * 0.5).round(),
        (p.y - r * 0.55).round(),
        3.0,
        1.0,
        c(WAX_LIT),
    );
}

/// Runes on a ring at turn `turn`: short strokes, crosses and dots, each keeping its shape.
fn runes(c: Vec2, r: f32, turn: f32, alpha: f32) {
    let n = 30;
    let col = kit::fade(DRIED, alpha);
    for k in 0..n {
        let a = k as f32 * TAU / n as f32 + turn;
        let (dir, side) = (Vec2::from_angle(a), Vec2::from_angle(a + FRAC_PI_2));
        let p = c + dir * r;
        match (hash(k as u32 + 400) * 3.0) as u32 {
            0 => {
                let (q, s) = (p - dir * 2.5, p + dir * 2.5);
                draw_line(q.x, q.y, s.x, s.y, 1.0, col);
            }
            1 => {
                let (q, s) = (p - side * 2.0 - dir * 2.0, p + side * 2.0 + dir * 2.0);
                draw_line(q.x, q.y, s.x, s.y, 1.0, col);
                let (q, s) = (p - side * 2.0 + dir * 2.0, p + side * 2.0 - dir * 2.0);
                draw_line(q.x, q.y, s.x, s.y, 1.0, col);
            }
            _ => draw_rectangle(p.x.round(), p.y.round(), 2.0, 2.0, col),
        }
    }
}

/// A breathing hellfire halo behind a circle whose recipe the build can complete.
fn halo(p: Vec2, r: f32, t: f32, alpha: f32) {
    let g = motion::pulse(t);
    for (k, base) in [(6.0, 0.12), (4.0, 0.25), (2.0, 0.5)] {
        let a = alpha * base * (0.6 + 0.4 * g);
        draw_circle(p.x, p.y, r + k, kit::fade(HELLFIRE, a));
    }
}

/// A rule of blood with a diamond in the middle and a dot at each end.
fn flourish(c: Vec2, half: f32, seed: u32, color: Color) {
    let (x, y) = (c.x.round(), c.y.round());
    blood_line(vec2(x - half, y), vec2(x - 5.0, y), seed, color);
    blood_line(vec2(x + 5.0, y), vec2(x + half, y), seed + 1, color);
    draw_poly(x, y + 0.5, 4, 3.5, 0.0, color);
    for side in [-1.0, 1.0] {
        draw_circle(x + side * (half + 3.0), y + 0.5, 1.2, color);
    }
}

fn dashed_circle(p: Vec2, r: f32, dashes: u32, color: Color) {
    for k in 0..dashes {
        let a0 = k as f32 * TAU / dashes as f32;
        let a1 = a0 + TAU / dashes as f32 * 0.5;
        let (q, s) = (p + Vec2::from_angle(a0) * r, p + Vec2::from_angle(a1) * r);
        draw_line(q.x, q.y, s.x, s.y, 1.0, color);
    }
}

fn dotted_line(from: Vec2, to: Vec2, color: Color) {
    let len = from.distance(to);
    let dir = (to - from) / len;
    let mut d = 0.0;
    while d < len {
        let p = (from + dir * d).round();
        draw_rectangle(p.x, p.y, 1.0, 1.0, color);
        d += 4.0;
    }
}

/// Text in ink: no drop shadow (it would smudge on vellum).
fn ink_centred(s: &str, cx: f32, y: f32, style: TextStyle, color: Color) {
    let w = measure_ui_text(s, style).width;
    draw_ui_text(s, (cx - w * 0.5).round(), y, style, color);
}

fn roman(mut n: usize) -> String {
    let mut s = String::new();
    for (v, r) in [(10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")] {
        while n >= v {
            s.push_str(r);
            n -= v;
        }
    }
    s
}

fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x5bd1_e995;
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    (x & 0xFFFF) as f32 / 65535.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rites_are_numbered_in_roman() {
        let r: Vec<String> = [1, 4, 9, 14].map(roman).to_vec();
        assert_eq!(r, ["I", "IV", "IX", "XIV"]);
    }

    #[test]
    fn the_circle_stays_inside_its_page() {
        for view in [vec2(640.0, 360.0), vec2(480.0, 360.0)] {
            let b = Book::new(view);
            let border = b.outer() + 3.0 + 1.0;
            let c = b.centre;
            assert!(
                c.x - border >= b.left.x && c.x + border <= b.left.right(),
                "{view}"
            );
            assert!(
                c.y - border >= b.left.y && c.y + border <= b.left.bottom(),
                "{view}"
            );
        }
        let b = Book::new(vec2(640.0, 360.0));
        assert!(
            b.outer() * PENTAGON > 45.0,
            "room for the tally inside the star"
        );
    }

    #[test]
    fn the_book_leaves_room_for_the_ribbon() {
        let b = Book::new(vec2(640.0, 360.0));
        assert!(b.rect.y >= RIBBON_PEEK + RIBBON_OUT);
        assert!(b.rect.bottom() + 5.0 <= 360.0, "and its shadow");
    }

    #[test]
    fn noise_stays_in_range() {
        for k in 0..200 {
            let v = value_noise(k as f32 * 0.37, k as f32 * 0.91, 5);
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
