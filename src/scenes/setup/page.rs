//! Drawing of the setup pages: one stone panel with the page's title and the step gems, the
//! option cards (waving flags, the blood modes, little screens for the display modes) or the
//! volume slider, and Skip / Confirm / Begin at the bottom. Unpicked options burn away after a pick.

use super::{SetupScene, Step, BURN, FADE, FULLSCREEN, HOLD, SCALES, STEPS, VOLUME_STEP};
use crate::assets::TextStyle;
use crate::burn;
use crate::i18n::{tr, Key, Lang};
use crate::kit;
use crate::motion;
use crate::save::{BloodMode, Settings};
use crate::screen::{ScaleMode, Screen};
use crate::ui::{self, Slider};
use macroquad::prelude::*;
use std::cell::RefCell;

const PANEL_W: f32 = 600.0;
const PANEL_Y: f32 = 14.0;
const PANEL_BOTTOM: f32 = 8.0;
const BUTTON_PAD: f32 = 30.0;
const TITLE_Y: f32 = PANEL_Y + 50.0;
const GEMS_Y: f32 = TITLE_Y + 22.0;
/// The options are centred between here and `CONTENT_GAP` above the bottom button.
const CONTENT_TOP: f32 = GEMS_Y + 14.0;
const CONTENT_GAP: f32 = 8.0;
const GAP: f32 = 12.0;
const LANG_CARD: Vec2 = vec2(160.0, 98.0);
const BLOOD_CARD: Vec2 = vec2(168.0, 64.0);
const DISPLAY_CARD: Vec2 = vec2(262.0, 72.0);
const SLIDER_W: f32 = 440.0;

/// Flags are painted at this size (a 1 px dark edge around 36×22 of cloth), drawn ×2.
const FLAG_W: u32 = 38;
const FLAG_H: u32 = 24;
const FLAG_SCALE: f32 = 2.0;

thread_local! {
    static FLAGS: RefCell<Vec<Texture2D>> = const { RefCell::new(Vec::new()) };
}

fn panel_rect(view: Vec2) -> Rect {
    let w = PANEL_W.min(view.x - 16.0);
    Rect::new(
        ((view.x - w) * 0.5).round(),
        PANEL_Y,
        w,
        view.y - PANEL_Y - PANEL_BOTTOM,
    )
}

pub(super) fn bottom_button(view: Vec2) -> Rect {
    let y = view.y - PANEL_BOTTOM - BUTTON_PAD - ui::BUTTON_H;
    ui::button_column(view, y, 1)[0]
}

/// `cols` × `rows` cards of `size`, centred in the content area.
fn grid(view: Vec2, size: Vec2, cols: usize, rows: usize) -> Vec<Rect> {
    let w = cols as f32 * size.x + (cols - 1) as f32 * GAP;
    let h = rows as f32 * size.y + (rows - 1) as f32 * GAP;
    let bottom = bottom_button(view).y - CONTENT_GAP;
    let x0 = ((view.x - w) * 0.5).round();
    let y0 = (CONTENT_TOP + (bottom - CONTENT_TOP - h) * 0.5).round();
    (0..cols * rows)
        .map(|i| {
            let (c, r) = ((i % cols) as f32, (i / cols) as f32);
            Rect::new(
                x0 + c * (size.x + GAP),
                y0 + r * (size.y + GAP),
                size.x,
                size.y,
            )
        })
        .collect()
}

pub(super) fn option_rects(step: Step, view: Vec2) -> Vec<Rect> {
    match step {
        Step::Language => grid(view, LANG_CARD, Lang::ALL.len(), 1),
        Step::Blood => {
            // three cards fit the narrowest view
            let room = panel_rect(view).w - 2.0 * BUTTON_PAD - 2.0 * GAP;
            let size = vec2(BLOOD_CARD.x.min((room / 3.0).floor()), BLOOD_CARD.y);
            grid(view, size, BloodMode::ALL.len(), 1)
        }
        Step::Display => grid(view, DISPLAY_CARD, 2, 2),
        Step::Volume => grid(view, vec2(SLIDER_W.min(view.x - 80.0), ui::BUTTON_H), 1, 1),
    }
}

fn title_key(step: Step) -> Key {
    match step {
        Step::Language => Key::SetLanguage,
        Step::Blood => Key::SetBlood,
        Step::Display => Key::SetupDisplay,
        Step::Volume => Key::SetupVolume,
    }
}

pub(super) fn draw(s: &SetupScene, view: Vec2, settings: &Settings, _screen: &Screen) {
    kit::set_blood(s.blood);
    let t = s.overlay.t;
    let since = s.picked.map(|(_, since)| since);
    // the page fades out once the burn is over, and the next one opens
    let out = since.map_or(1.0, |k| 1.0 - ((k - BURN - HOLD) / FADE).clamp(0.0, 1.0));
    let (open, dy) = motion::open(if since.is_some() { 1.0 } else { t });
    let alpha = open * out;

    kit::dim(view, 0.7);
    kit::panel(panel_rect(view), false, 1.0, kit::seed_of("setup"));
    let step = s.step();
    kit::title(tr(title_key(step)), view.x * 0.5, TITLE_Y, alpha);
    let total = STEPS.len() as u32;
    let gems = vec2(((view.x - kit::pips_width(total)) * 0.5).round(), GEMS_Y);
    kit::pips(gems, s.step as u32, true, total, t, alpha);

    let items = s.items(view);
    let options = s.options();
    let raised = |r: Rect| Rect::new(r.x, r.y + dy, r.w, r.h);
    for (i, &rect) in items[..options].iter().enumerate() {
        let rect = raised(rect);
        // Display cards stay lit while picked in their row; elsewhere the focus lights them
        let look = match step {
            Step::Display => Look {
                lit: s.kept(i),
                focused: s.picked.is_none() && s.focus.is_highlighted(i),
            },
            _ => {
                let on = s.kept(i) || s.picked.is_none() && s.focus.is_highlighted(i);
                Look {
                    lit: on,
                    focused: on,
                }
            }
        };
        match since {
            Some(k) if !s.kept(i) => {
                if k < BURN {
                    let around = Rect::new(rect.x - 3.0, rect.y - 3.0, rect.w + 6.0, rect.h + 6.0);
                    let seed = (s.step * 16 + i) as u32 * 7 + 3;
                    let idle = Look {
                        lit: false,
                        focused: false,
                    };
                    let draw_old = || option(step, i, rect, idle, 1.0, t, settings);
                    burn::burn(around, k / BURN, seed, i, draw_old, t, alpha);
                }
            }
            _ => option(step, i, rect, look, alpha, t, settings),
        }
    }

    let button = items[options];
    let label = tr(match step {
        Step::Volume => Key::SetupBegin,
        Step::Display => Key::SetupConfirm,
        _ => Key::SetupSkip,
    });
    let button_lit = since.is_none() && s.focus.is_highlighted(options);
    kit::button(
        button,
        if button_lit {
            kit::ButtonLook::Lit
        } else {
            kit::ButtonLook::Idle
        },
        label,
        alpha,
    );
    if since.is_some() {
        // the kept options glow while the rest burn
        for i in (0..options).filter(|&i| s.kept(i)) {
            kit::sparks(raised(items[i]), t, alpha);
        }
    } else if let Some(i) = s.focus.highlight().filter(|&i| i < items.len()) {
        let r = raised(items[i]);
        kit::cursors(s.overlay.cursor_rect(r), t, alpha);
        kit::sparks(r, t, alpha);
    }
}

/// How an option card looks: `lit` (picked or focused) warms its frame, `focused` its name.
#[derive(Clone, Copy)]
struct Look {
    lit: bool,
    focused: bool,
}

/// Option `i` of `step` in `rect`.
fn option(step: Step, i: usize, rect: Rect, look: Look, alpha: f32, t: f32, settings: &Settings) {
    let p = kit::pal();
    let lit = look.lit;
    let name_color = kit::fade(
        match (look.lit, look.focused) {
            (_, true) => p.label_lit,
            (true, false) => p.gold,
            _ => p.label,
        },
        alpha,
    );
    let desc_color = kit::fade(p.desc, alpha);
    match step {
        Step::Language => {
            let lang = Lang::ALL[i];
            kit::panel(rect, lit, alpha, kit::seed_of(lang.code()));
            let fw = FLAG_W as f32 * FLAG_SCALE;
            let pos = vec2((rect.center().x - fw * 0.5).round(), rect.y + 11.0);
            flag(i, pos, t, alpha);
            let band = Rect::new(rect.x, rect.y + 64.0, rect.w, 25.0);
            name(lang.name(), rect.center().x, band, name_color);
        }
        Step::Blood => {
            let k = match BloodMode::ALL[i] {
                BloodMode::Full => Key::BloodFull,
                BloodMode::Reduced => Key::BloodReduced,
                BloodMode::Off => Key::BloodOff,
            };
            kit::panel(rect, lit, alpha, kit::seed_of(tr(k)) ^ 0xb100d);
            name(tr(k), rect.center().x, rect, name_color);
        }
        Step::Display => {
            // the first row picks the scale (drawn fullscreen), the second the window (filled)
            let (mode, full, k, sub) = match i {
                0 | 1 => {
                    let mode = SCALES[i];
                    let (k, sub) = match mode {
                        ScaleMode::Fill => (Key::ScaleFill, Key::ScaleFillDesc),
                        ScaleMode::PixelPerfect => (Key::ScalePixel, Key::ScalePixelDesc),
                    };
                    (mode, true, k, sub)
                }
                _ => {
                    let full = FULLSCREEN[i - 2];
                    let (k, sub) = match full {
                        false => (Key::SetupWindow, Key::SetupWindowDesc),
                        true => (Key::SetupFullscreen, Key::SetupFullscreenDesc),
                    };
                    (ScaleMode::Fill, full, k, sub)
                }
            };
            kit::panel(rect, lit, alpha, kit::seed_of(tr(k)) ^ i as u32);
            let glyph = Rect::new(rect.x + 18.0, (rect.center().y - 15.0).round(), 40.0, 30.0);
            screen_glyph(glyph, mode, full, lit, alpha);
            let x = glyph.right() + 14.0;
            let band = Rect::new(x, rect.y + 11.0, rect.right() - x, 26.0);
            let y = kit::ink_baseline(tr(k), TextStyle::Name, band);
            kit::text(tr(k), x, y, TextStyle::Name, name_color);
            kit::text(tr(sub), x, rect.y + 52.0, TextStyle::Desc, desc_color);
        }
        Step::Volume => Slider {
            rect,
            label: tr(Key::SetMaster),
            value: settings.master,
            step: VOLUME_STEP,
        }
        .draw(lit),
    }
}

/// A Blackletter name centred on its ink in `band`.
fn name(s: &str, cx: f32, band: Rect, color: Color) {
    let y = kit::ink_baseline(s, TextStyle::Name, band);
    kit::text_centered(
        s,
        cx,
        y + 1.0,
        TextStyle::Name,
        kit::fade(kit::pal().shadow, color.a),
    );
    kit::text_centered(s, cx, y, TextStyle::Name, color);
}

/// A little screen: in a window or filling the monitor, the picture filling it or kept at a
/// whole scale between bars.
fn screen_glyph(r: Rect, mode: ScaleMode, full: bool, lit: bool, alpha: f32) {
    let p = kit::pal();
    let c = |col: Color| kit::fade(col, alpha);
    let edge = if lit { p.gold } else { p.dim };
    // the monitor and its stand
    draw_rectangle(r.x, r.y, r.w, r.h - 4.0, c(edge));
    draw_rectangle(
        r.x + 1.0,
        r.y + 1.0,
        r.w - 2.0,
        r.h - 6.0,
        c(Color::new(0.03, 0.02, 0.03, 1.0)),
    );
    draw_rectangle(r.center().x - 4.0, r.bottom() - 4.0, 8.0, 2.0, c(edge));
    draw_rectangle(r.center().x - 8.0, r.bottom() - 2.0, 16.0, 2.0, c(edge));
    let screen = Rect::new(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 8.0);
    let area = if full {
        screen
    } else {
        let w = Rect::new(
            screen.x + 5.0,
            screen.y + 3.0,
            screen.w - 10.0,
            screen.h - 6.0,
        );
        draw_rectangle(w.x - 1.0, w.y - 3.0, w.w + 2.0, w.h + 4.0, c(p.label));
        draw_rectangle(w.right() - 3.0, w.y - 2.0, 2.0, 1.0, c(p.ruby));
        w
    };
    let picture = match mode {
        ScaleMode::Fill => area,
        ScaleMode::PixelPerfect => {
            let (w, h) = ((area.w * 0.66).round(), (area.h * 0.66).round());
            draw_rectangle(area.x, area.y, area.w, area.h, c(BLACK));
            Rect::new(
                (area.center().x - w * 0.5).round(),
                (area.center().y - h * 0.5).round(),
                w,
                h,
            )
        }
    };
    // a sunset over the village: hellfire sky, dark ground
    let sky = picture.h - (picture.h * 0.35).round();
    draw_rectangle(picture.x, picture.y, picture.w, sky, c(p.gold));
    draw_rectangle(picture.x, picture.y + sky - 2.0, picture.w, 2.0, c(p.ruby));
    draw_rectangle(
        picture.x,
        picture.y + sky,
        picture.w,
        picture.h - sky,
        c(p.shadow),
    );
}

/// Lang `i`'s flag with its top-left at `pos`, waving from its hoist.
fn flag(i: usize, pos: Vec2, t: f32, alpha: f32) {
    let tex = FLAGS.with(|f| {
        let mut f = f.borrow_mut();
        if f.is_empty() {
            *f = Lang::ALL.iter().map(|&l| flag_texture(l)).collect();
        }
        f[i].clone()
    });
    let (w, h) = (FLAG_W as f32, FLAG_H as f32);
    // its shadow, then the cloth column by column
    draw_rectangle(
        pos.x + 3.0,
        pos.y + 4.0,
        w * FLAG_SCALE,
        h * FLAG_SCALE,
        kit::fade(BLACK, 0.45 * alpha),
    );
    for col in 0..FLAG_W {
        let k = col as f32 / w;
        let phase = t * 3.2 - col as f32 * 0.3;
        let dy = (phase.sin() * 1.6 * k).round();
        let light = 0.9 + 0.12 * phase.cos() * k;
        let params = DrawTextureParams {
            source: Some(Rect::new(col as f32, 0.0, 1.0, h)),
            dest_size: Some(vec2(FLAG_SCALE, h * FLAG_SCALE)),
            ..Default::default()
        };
        let tint = Color::new(light, light, light, alpha);
        draw_texture_ex(
            &tex,
            pos.x + col as f32 * FLAG_SCALE,
            pos.y + dy,
            tint,
            params,
        );
    }
}

fn flag_texture(lang: Lang) -> Texture2D {
    let mut img = Image::gen_image_color(FLAG_W as u16, FLAG_H as u16, Color::from_hex(0x1a0c10));
    for y in 1..FLAG_H - 1 {
        for x in 1..FLAG_W - 1 {
            let c = match lang {
                Lang::En => union_flag(x - 1, y - 1),
                Lang::Es => spain(x - 1, y - 1),
            };
            img.set_pixel(x, y, Color::from_hex(c));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

/// The Union Flag on 36×22: white-edged red cross over a counterchanged saltire, broad white
/// uppermost at the hoist.
fn union_flag(x: u32, y: u32) -> u32 {
    const BLUE: u32 = 0x1c2b6b;
    const RED: u32 = 0xc4203a;
    const WHITE: u32 = 0xf2ece4;
    let (w, h) = (36.0_f32, 22.0_f32);
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    let (dx, dy) = (px - w * 0.5, py - h * 0.5);
    if dx.abs() < 2.5 || dy.abs() < 2.5 {
        return RED;
    }
    if dx.abs() < 4.5 || dy.abs() < 4.5 {
        return WHITE;
    }
    let len = (w * w + h * h).sqrt();
    // signed distances below each diagonal (top-left to bottom-right, bottom-left to top-right)
    let d1 = (py * w - px * h) / len;
    let d2 = (px * h + py * w - h * w) / len;
    let hoist = px < w * 0.5;
    for d in [d1, d2] {
        if d.abs() < 2.4 {
            let red = if hoist {
                (0.0..1.7).contains(&d)
            } else {
                (-1.7..0.0).contains(&d)
            };
            return if red { RED } else { WHITE };
        }
    }
    BLUE
}

/// Spain on 36×22: red, yellow twice as tall, red, with a small arms on the hoist side.
fn spain(x: u32, y: u32) -> u32 {
    const RED: u32 = 0xb0182a;
    const YELLOW: u32 = 0xf0b820;
    const GOLD: u32 = 0xa06418;
    const SILVER: u32 = 0xd8d0c8;
    if !(5..17).contains(&y) {
        return RED;
    }
    let (sx, sy) = (10, 8);
    match (x, y) {
        // the crown
        (10..=15, 6) if x.is_multiple_of(2) => GOLD,
        (10..=15, 7) => GOLD,
        // the pillars of Hercules
        (8 | 17, 8..=15) => SILVER,
        (7..=9 | 16..=18, 8 | 15) => GOLD,
        // the shield, quartered, with a rounded foot
        (10..=15, 8..=14) => {
            if (x == 10 || x == 15) && y == 14 {
                YELLOW
            } else if (x < sx + 3) == (y < sy + 3) {
                RED
            } else {
                SILVER
            }
        }
        (12..=13, 15) => RED,
        _ => YELLOW,
    }
}
