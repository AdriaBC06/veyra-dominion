//! Screen-space UI drawn into the logical frame (call with the UI camera set).

mod play;

pub use play::{draw as draw_hud, HudFx};

use crate::assets::{draw_ui_text, measure_ui_text, sprite, SpriteId, TextStyle, ICON_SIZE};
use crate::config::*;
use crate::game::{Game, Reward};

use crate::i18n::{tr, trf, Key};
use crate::kit::{self, Rim};
use crate::motion;
use crate::ui::{self, Focus, Overlay};
use crate::upgrades::{Choice, Upgrades};
use macroquad::prelude::*;

pub const TITLE_COLOR: Color = Color::new(0.88, 0.19, 0.29, 1.0);

/// Horizontal edge of the view a widget is attached to.
#[derive(Clone, Copy)]
pub enum Anchor {
    Center,
    Right,
}

impl Anchor {
    /// Left x of a widget `width` wide, `margin` in from the anchored edge.
    pub fn x(self, view: Vec2, width: f32, margin: f32) -> f32 {
        match self {
            Anchor::Center => ((view.x - width) * 0.5).round(),
            Anchor::Right => (view.x - width - margin).round(),
        }
    }
}

pub fn draw_dim(view: Vec2) {
    draw_rectangle(0.0, 0.0, view.x, view.y, Color::new(0.0, 0.0, 0.0, 0.6));
}

/// `m:ss` for a time in seconds.
pub fn format_time(t: f32) -> String {
    let s = t.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

/// An icon in the `Icons` strip (`assets/icons.px.js` writes it in this order).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Art(usize),
    /// Art `i`'s evolution (only Arts that have one get a frame).
    Evolution(usize),
    Union(usize),
    Sin(usize),
}

/// How many of the first `n` Arts have an evolution.
const fn evolutions_before(n: usize) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < n {
        if ARTS[i].evolution.is_some() {
            count += 1;
        }
        i += 1;
    }
    count
}

const FIRST_EVOLUTION_ICON: usize = ARTS.len();
const FIRST_UNION_ICON: usize = FIRST_EVOLUTION_ICON + evolutions_before(ARTS.len());
const FIRST_SIN_ICON: usize = FIRST_UNION_ICON + UNIONS.len();
pub const ICON_COUNT: usize = FIRST_SIN_ICON + SINS.len();

impl Icon {
    pub const fn frame(self) -> usize {
        match self {
            Icon::Art(i) => i,
            Icon::Evolution(i) => FIRST_EVOLUTION_ICON + evolutions_before(i),
            Icon::Union(u) => FIRST_UNION_ICON + u,
            Icon::Sin(i) => FIRST_SIN_ICON + i,
        }
    }

    /// What owned Art `i` shows: its union, else its evolution, else itself.
    pub fn of_art(upgrades: &Upgrades, i: usize) -> Icon {
        match upgrades.union_of(i) {
            Some(u) => Icon::Union(u),
            None if upgrades.evolved[i] => Icon::Evolution(i),
            None => Icon::Art(i),
        }
    }

    fn of_choice(choice: Choice) -> Icon {
        match choice {
            Choice::Art(i) => Icon::Art(i),
            Choice::Sin(i) => Icon::Sin(i),
        }
    }
}

/// F3 overlay: frame timing and entity counts.
pub fn draw_debug(game: &Game, fps: i32, update_ms: f32) {
    let mut lines = vec![
        format!("FPS {fps}   update {update_ms:.2} ms"),
        format!("enemies {}/{MAX_ENEMIES}", game.enemies.len()),
    ];
    for (k, def) in ENEMY_DEFS.iter().enumerate() {
        let n = game
            .enemies
            .kind
            .iter()
            .filter(|&&kind| kind as usize == k)
            .count();
        lines.push(format!("  {} {n}", def.name));
    }
    lines.push(format!(
        "proj {}  souls {}  fades {}",
        game.arts.projectiles.len(),
        game.souls.len(),
        game.fades.len()
    ));
    lines.push(format!("spawn {:.1}/s", Game::spawn_rate(game.time)));
    let h = lines.len() as f32 * 13.0 + 6.0;
    draw_rectangle(4.0, 64.0, 200.0, h, Color::new(0.0, 0.0, 0.0, 0.6));
    for (i, line) in lines.iter().enumerate() {
        draw_ui_text(line, 8.0, 78.0 + i as f32 * 13.0, TextStyle::Desc, LIME);
    }
}

/// F12: which bot is playing, top right under the HUD (developer-only, not translated).
pub fn draw_autopilot(view: Vec2, label: &str) {
    let w = measure_ui_text(label, TextStyle::Desc).width;
    draw_ui_text(label, view.x - w - 8.0, 76.0, TextStyle::Desc, LIME);
}

/// Panel menus (pause, game over, victory): side padding, title baseline from the panel top,
/// the title rule's height, text line height and bottom padding.
const MENU_PAD_X: f32 = 30.0;
const MENU_TITLE_Y: f32 = 50.0;
const MENU_RULE_H: f32 = 17.0;
const MENU_LINE: f32 = 17.0;
const MENU_BOTTOM: f32 = 30.0;

/// Where a panel menu with `lines` lines of text and `n` buttons goes, centred in the view.
pub struct MenuLayout {
    pub panel: Rect,
    pub title_y: f32,
    pub lines_y: f32,
    pub buttons: Vec<Rect>,
}

pub fn menu_layout(view: Vec2, lines: usize, n: usize) -> MenuLayout {
    let w = ui::BUTTON_W + 2.0 * MENU_PAD_X;
    let text_top = MENU_TITLE_Y + MENU_RULE_H + 10.0;
    let text_h = lines as f32 * MENU_LINE + if lines > 0 { 8.0 } else { 0.0 };
    let buttons_h = n as f32 * (ui::BUTTON_H + ui::BUTTON_GAP) - ui::BUTTON_GAP;
    let h = text_top + text_h + buttons_h + MENU_BOTTOM;
    let panel = Rect::new(
        Anchor::Center.x(view, w, 0.0),
        ((view.y - h) * 0.5).round(),
        w,
        h,
    );
    let buttons = ui::button_column(view, panel.y + text_top + text_h, n);
    MenuLayout {
        panel,
        title_y: panel.y + MENU_TITLE_Y,
        lines_y: panel.y + text_top + 12.0,
        buttons,
    }
}

/// A panel menu over the dimmed run: title, lines of text and buttons, opening with `overlay`.
pub fn draw_menu(
    view: Vec2,
    title: &str,
    lines: &[&str],
    labels: &[&str],
    focus: &Focus,
    overlay: &Overlay,
) {
    let m = menu_layout(view, lines.len(), labels.len());
    let (alpha, dy) = motion::open(overlay.t);
    kit::dim(view, alpha);
    let p = m.panel;
    kit::panel(Rect::new(p.x, p.y + dy, p.w, p.h), false, alpha, kit::seed_of(title));
    let cx = p.center().x;
    kit::title(title, cx, m.title_y + dy, alpha);
    for (k, line) in lines.iter().enumerate() {
        let y = m.lines_y + dy + k as f32 * MENU_LINE;
        kit::text_centered(line, cx, y, TextStyle::Desc, kit::fade(kit::pal().desc, alpha));
    }
    ui::draw_buttons(&m.buttons, labels, focus, overlay);
}

/// Cards take the view's width up to this, less a margin on each side (room for the cursors).
const CARD_MAX_W: f32 = 500.0;
const CARD_MARGIN: f32 = 26.0;
/// Tall enough for the name and two lines of description inside the frame.
const CARD_H: f32 = 80.0;
const CARD_GAP: f32 = 6.0;
const CARDS_Y: f32 = 94.0;
const CARDS_TITLE_Y: f32 = 64.0;
/// The level-up flash: peak alpha and how long it takes to fade.
const FLASH_ALPHA: f32 = 0.35;
const FLASH_TIME: f32 = 0.14;

/// Hit rects of the level-up cards, one per choice slot.
pub fn level_up_cards(view: Vec2) -> [Rect; 3] {
    let w = (view.x - 2.0 * CARD_MARGIN).min(CARD_MAX_W);
    let x = Anchor::Center.x(view, w, 0.0);
    std::array::from_fn(|slot| {
        let y = CARDS_Y + slot as f32 * (CARD_H + CARD_GAP);
        Rect::new(x, y, w, CARD_H)
    })
}

/// Opens a card screen: the gold flash of a level-up (`flash`), the dim and the title.
fn draw_card_screen(view: Vec2, title: &str, overlay: &Overlay, flash: bool) {
    let (alpha, dy) = motion::open(overlay.t);
    kit::dim(view, alpha);
    if flash {
        let a = FLASH_ALPHA * (1.0 - overlay.t / FLASH_TIME).max(0.0);
        draw_rectangle(0.0, 0.0, view.x, view.y, Color::new(1.0, 0.86, 0.6, a));
    }
    kit::title(title, view.x * 0.5, CARDS_TITLE_Y + dy, alpha);
}

/// Card `slot` of a dealt row, moved and faded by the deal; the focus cursors follow it.
fn dealt(rect: Rect, slot: usize, overlay: &Overlay) -> (Rect, f32) {
    let (alpha, dy) = motion::deal(overlay.dealt, slot);
    (Rect::new(rect.x, rect.y + dy, rect.w, rect.h), alpha)
}

/// Cursors and sparks around the highlighted card once it has landed.
fn draw_card_focus(cards: &[Rect], focus: &Focus, overlay: &Overlay) {
    let Some(i) = focus.highlight().filter(|&i| i < cards.len()) else {
        return;
    };
    let (_, alpha) = dealt(cards[i], i, overlay);
    kit::cursors(overlay.cursor_rect(cards[i]), overlay.t, alpha);
    kit::sparks(cards[i], overlay.t, alpha);
}

/// Level-up screen: up to three upgrade cards dealt in one after another.
pub fn draw_level_up(game: &Game, view: Vec2, focus: &Focus, overlay: &Overlay) {
    draw_card_screen(view, tr(Key::LevelUpTitle), overlay, true);
    let cards = level_up_cards(view);
    let mut count = 0;
    for (slot, choice) in game.choices.iter().enumerate() {
        let Some(choice) = *choice else { continue };
        count += 1;
        let (name, tag, desc) = card_text(game, choice);
        let rim = if matches!(tag, Tag::New) {
            Rim::New
        } else {
            Rim::Plain
        };
        let card = Card {
            num: slot + 1,
            icon: Some((Icon::of_choice(choice), rim)),
            name,
            tag,
            desc: &desc,
        };
        let (rect, alpha) = dealt(cards[slot], slot, overlay);
        card.draw(rect, focus.is_highlighted(slot), overlay.t, alpha);
    }
    draw_card_focus(&cards[..count], focus, overlay);
}

/// What a card shows on the right of its name.
enum Tag {
    None,
    /// Never owned.
    New,
    /// Pips: the level owned and the next one this pick gives.
    Level(u32, u32),
    Text(String),
}

/// One upgrade card.
struct Card<'a> {
    /// Hotkey (1-3), shown on the icon slot.
    num: usize,
    icon: Option<(Icon, Rim)>,
    name: &'a str,
    tag: Tag,
    desc: &'a str,
}

/// Card icons are drawn at twice their pixel size in a slot, centred in the card's height.
const CARD_ICON_SCALE: f32 = 2.0;
const CARD_SLOT: f32 = ICON_SIZE * CARD_ICON_SCALE + 6.0;
const CARD_SLOT_X: f32 = 16.0;
const CARD_TEXT_X: f32 = CARD_SLOT_X + CARD_SLOT + 10.0;
const CARD_PAD_R: f32 = 16.0;

impl Card<'_> {
    /// The card in `rect`: a panel (lit and warm when highlighted), the icon in its slot with
    /// the hotkey, the name with its tag on the right and the description below.
    fn draw(&self, rect: Rect, lit: bool, t: f32, alpha: f32) {
        let Rect { x, y, w, h } = rect;
        kit::panel(rect, lit, alpha, kit::seed_of(self.name));
        let slot = Rect::new(
            x + CARD_SLOT_X,
            (y + (h - CARD_SLOT) * 0.5).round(),
            CARD_SLOT,
            CARD_SLOT,
        );
        if let Some((icon, rim)) = self.icon {
            let rim = if lit && rim == Rim::Plain { Rim::Lit } else { rim };
            kit::slot(slot, rim, alpha);
            draw_icon_alpha(icon, slot.x + 3.0, slot.y + 3.0, CARD_ICON_SCALE, alpha);
            let num = self.num.to_string();
            let nx = slot.right() - 6.0;
            let ny = slot.bottom() - 6.0;
            draw_rectangle(nx - 2.0, ny - 2.0, 10.0, 11.0, kit::fade(BLACK, alpha * 0.85));
            let color = if lit { kit::pal().gold_lit } else { kit::pal().dim };
            draw_ui_text(&num, nx, ny + 8.0, TextStyle::Desc, kit::fade(color, alpha));
        }
        let name_color = if lit { kit::pal().gold_lit } else { kit::pal().gold };
        // Name and description are centred between the slot and the tag column.
        let cx = (x + CARD_TEXT_X + x + w - CARD_PAD_R) * 0.5;
        let (first, second) = wrap_two(self.desc, w - CARD_TEXT_X - CARD_PAD_R);
        // a one-line description drops the text block half a line, so it stays centred
        let ty = y + if second.is_empty() { 7.0 } else { 0.0 };
        kit::text_centered(self.name, cx, ty + 31.0, TextStyle::Name, kit::fade(name_color, alpha));
        let right = x + w - CARD_PAD_R;
        match &self.tag {
            Tag::None => {}
            Tag::New => {
                let s = tr(Key::LevelUpNew);
                let tw = measure_ui_text(s, TextStyle::Desc).width;
                kit::text(s, right - tw, y + 26.0, TextStyle::Desc, kit::fade(kit::pal().ruby, alpha));
            }
            Tag::Level(lv, max) => {
                let pw = kit::pips_width(*max);
                kit::pips(vec2(right - pw, y + 21.0), *lv, true, *max, t, alpha);
            }
            Tag::Text(s) => {
                let tw = measure_ui_text(s, TextStyle::Desc).width;
                kit::text(s, right - tw, y + 26.0, TextStyle::Desc, kit::fade(kit::pal().desc, alpha));
            }
        }
        let color = kit::fade(if lit { kit::pal().label_lit } else { kit::pal().desc }, alpha);
        kit::text_centered(first, cx, ty + 50.0, TextStyle::Desc, color);
        kit::text_centered(second, cx, ty + 64.0, TextStyle::Desc, color);
    }
}

/// `draw_icon` faded by `alpha`.
fn draw_icon_alpha(icon: Icon, x: f32, y: f32, scale: f32, alpha: f32) {
    let (tex, src) = sprite(SpriteId::Icons, icon.frame());
    let params = DrawTextureParams {
        source: Some(src),
        dest_size: Some(Vec2::splat(ICON_SIZE * scale)),
        ..Default::default()
    };
    draw_texture_ex(&tex, x.round(), y.round(), Color::new(1.0, 1.0, 1.0, alpha), params);
}

/// Splits `text` at a space into two lines when it is wider than `width`: the first
/// takes as many words as fit.
fn wrap_two(text: &str, width: f32) -> (&str, &str) {
    let fits = |t: &str| measure_ui_text(t, TextStyle::Desc).width <= width;
    if fits(text) {
        return (text, "");
    }
    let cut = text
        .match_indices(' ')
        .map(|(i, _)| i)
        .take_while(|&i| fits(&text[..i]))
        .last();
    match cut {
        Some(i) => (&text[..i], &text[i + 1..]),
        None => (text, ""),
    }
}

/// Name, level tag and description of a level-up card. A new Art or Sin shows what
/// it does; an owned Art shows what its next level adds.
fn card_text(game: &Game, choice: Choice) -> (&'static str, Tag, String) {
    match choice {
        Choice::Sin(i) => {
            let def = &SINS[i];
            let lv = game.upgrades.sins[i];
            let tag = if lv == 0 {
                Tag::New
            } else {
                Tag::Level(lv.into(), def.max_level.into())
            };
            (tr(def.name), tag, tr(def.desc).to_string())
        }
        Choice::Art(i) => {
            let def = &ARTS[i];
            let lv = game.upgrades.arts[i];
            if lv == 0 {
                return (tr(def.name), Tag::New, tr(def.desc).to_string());
            }
            (
                tr(def.name),
                Tag::Level(lv.into(), ART_MAX_LEVEL.into()),
                gain_text(def, &def.gains[lv as usize - 1]),
            )
        }
    }
}

/// The two cards on the Abyssal Offering screen: what it holds, and its sacrifice.
pub fn offering_cards(view: Vec2) -> [Rect; 2] {
    let cards = level_up_cards(view);
    [cards[1], cards[2]]
}

/// Abyssal Offering screen: what it holds on the first card, the sacrifice (a minor
/// reward) on the second.
pub fn draw_offering(game: &Game, view: Vec2, ready: bool, focus: &Focus, overlay: &Overlay) {
    draw_card_screen(view, tr(Key::OfferingTitle), overlay, false);
    let Some(reward) = game.reward else { return };
    let (line, icon, name, tag, desc) = match reward {
        Reward::Evolution(i) => {
            let line = trf(Key::OfferingEvolved, &[&tr(ARTS[i].name)]);
            let evo = ARTS[i]
                .evolution
                .as_ref()
                .expect("only Arts with one evolve");
            let icon = Some((Icon::Evolution(i), Rim::Evolved));
            (line, icon, tr(evo.name), Tag::None, tr(evo.desc).to_string())
        }
        Reward::Union(u) => {
            let def = &UNIONS[u];
            let [a, b] = def.arts.map(|i| tr(ARTS[i].name));
            (
                trf(Key::OfferingUnited, &[&a, &b]),
                Some((Icon::Union(u), Rim::Union)),
                tr(def.name),
                Tag::None,
                tr(def.desc).to_string(),
            )
        }
        // Not taken yet: show the level it would reach and what that gives.
        Reward::Gift(choice) => {
            let (name, tag, desc) = card_text(game, choice);
            let rim = if matches!(tag, Tag::New) {
                Rim::New
            } else {
                Rim::Plain
            };
            let icon = Some((Icon::of_choice(choice), rim));
            (tr(Key::OfferingGift).to_string(), icon, name, tag, desc)
        }
        Reward::Heal => (
            tr(Key::OfferingHeal).to_string(),
            None,
            tr(Key::OfferingAccept),
            Tag::None,
            trf(Key::OfferingHealDesc, &[&OFFERING_HEAL]),
        ),
    };
    let (alpha, dy) = motion::open(overlay.t);
    let line_color = kit::fade(kit::pal().label, alpha);
    let cards = offering_cards(view);
    // the line sits centred (by its 10 px capitals) in the free slot above the cards
    let line_y = ((CARDS_Y + cards[0].y) * 0.5 + 5.0).round() + dy;
    kit::text_centered(&line, view.x * 0.5, line_y, TextStyle::Desc, line_color);
    let tag = match tag {
        Tag::None => Tag::Text(tr(Key::OfferingAccept).to_string()),
        tag => tag,
    };
    let accept = Card {
        num: 1,
        icon,
        name,
        tag,
        desc: &desc,
    };
    let (rect, a) = dealt(cards[0], 0, overlay);
    accept.draw(rect, focus.is_highlighted(0), overlay.t, a);
    let gain = trf(
        Key::OfferingSacrificeDesc,
        &[&SACRIFICE_HEAL, &SACRIFICE_XP],
    );
    let sacrifice = Card {
        num: 2,
        icon: None,
        name: tr(Key::OfferingSacrifice),
        tag: Tag::None,
        desc: &gain,
    };
    let (rect, a) = dealt(cards[1], 1, overlay);
    sacrifice.draw(rect, focus.is_highlighted(1), overlay.t, a);
    // The cursors wait until the Offering takes input.
    if ready {
        draw_card_focus(&cards, focus, overlay);
    }
}

/// "+1 heart, +4 damage": every non-zero part of a level gain.
fn gain_text(def: &ArtDef, g: &ArtGain) -> String {
    let pct = |f: f32| (f * 100.0).round() as u32;
    let parts = [
        (g.amount > 0, def.amount_key, g.amount.to_string()),
        (g.damage > 0.0, Key::ArtGainDamage, g.damage.to_string()),
        (
            g.cooldown > 0.0,
            Key::ArtGainCooldown,
            pct(g.cooldown).to_string(),
        ),
        (g.area > 0.0, Key::ArtGainArea, pct(g.area).to_string()),
        (g.speed > 0.0, Key::ArtGainSpeed, pct(g.speed).to_string()),
        (g.pierce > 0, Key::ArtGainPierce, g.pierce.to_string()),
        (
            g.duration > 0.0,
            Key::ArtGainDuration,
            g.duration.to_string(),
        ),
    ];
    let texts: Vec<String> = parts
        .iter()
        .filter(|p| p.0)
        .map(|(_, key, n)| trf(*key, &[n]))
        .collect();
    texts.join(", ")
}

// --- Debug panel (F2) ------------------------------------------------------
// Developer-only, like the F3 overlay: labels stay in English, names go through `tr`.

const DBG_TOP: f32 = 44.0;
const DBG_ROW: f32 = 18.0;
const DBG_ROW_H: f32 = 16.0;
const DBG_NAME_W: f32 = 230.0;
const DBG_EVO_W: f32 = 134.0;
const DBG_SIN_W: f32 = 230.0;
const DBG_OWNED: Color = Color::new(0.42, 0.06, 0.16, 0.95);
const DBG_FREE: Color = Color::new(0.08, 0.06, 0.1, 0.92);
const DBG_GOLD: Color = Color::new(0.91, 0.69, 0.29, 1.0);

/// Hit rects of the debug panel: one row per Art, then one Evolve button per Art,
/// then one row per Sin.
pub fn debug_rects(view: Vec2) -> Vec<Rect> {
    let x0 = Anchor::Center.x(view, DBG_NAME_W + DBG_EVO_W + 10.0 + DBG_SIN_W, 0.0);
    let row = |k: usize| DBG_TOP + 14.0 + k as f32 * DBG_ROW;
    let arts = (0..ARTS.len()).map(|k| Rect::new(x0, row(k), DBG_NAME_W, DBG_ROW_H));
    let evos = (0..ARTS.len())
        .map(|k| Rect::new(x0 + DBG_NAME_W + 4.0, row(k), DBG_EVO_W - 4.0, DBG_ROW_H));
    let sin_x = x0 + DBG_NAME_W + DBG_EVO_W + 10.0;
    let sins = (0..SINS.len()).map(|k| Rect::new(sin_x, row(k), DBG_SIN_W, DBG_ROW_H));
    arts.chain(evos).chain(sins).collect()
}

/// The union whose Unite button sits on Art `art`'s row (an Art without an evolution
/// of its own).
pub fn debug_union(art: usize) -> usize {
    (0..UNIONS.len())
        .find(|&k| UNIONS[k].arts.contains(&art))
        .unwrap_or(0)
}

/// A panel row: filled when owned, dimmed when it can't be taken, outlined when focused.
fn debug_row(r: Rect, left: &str, right: &str, owned: bool, dim: bool, lit: bool, color: Color) {
    draw_rectangle(r.x, r.y, r.w, r.h, if owned { DBG_OWNED } else { DBG_FREE });
    let border = if lit {
        WHITE
    } else if owned {
        color
    } else {
        DARKGRAY
    };
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, border);
    let text = if dim {
        DARKGRAY
    } else if owned {
        color
    } else {
        GRAY
    };
    let y = r.y + 13.0;
    draw_ui_text(left, r.x + 5.0, y, TextStyle::Desc, text);
    let w = measure_ui_text(right, TextStyle::Desc).width;
    draw_ui_text(right, r.x + r.w - w - 5.0, y, TextStyle::Desc, text);
}

/// Debug panel: every Art and Sin, what is owned and at what level, the slot limits,
/// and an Evolve button per Art (lit when it can evolve: top level and its Sin owned).
pub fn draw_debug_panel(game: &Game, view: Vec2, focus: &Focus) {
    draw_dim(view);
    let u = &game.upgrades;
    let rects = debug_rects(view);
    let (n, arts_full, sins_full) = (
        ARTS.len(),
        u.owned_arts() >= MAX_ARTS,
        u.owned_sins() >= MAX_SINS,
    );
    let header = |r: Rect, text: String, full: bool| {
        let color = if full { TITLE_COLOR } else { WHITE };
        draw_ui_text(&text, r.x, DBG_TOP + 8.0, TextStyle::Desc, color);
    };
    let full = |f: bool| if f { "  FULL" } else { "" };
    header(
        rects[0],
        format!("ARTS {}/{MAX_ARTS}{}", u.owned_arts(), full(arts_full)),
        arts_full,
    );
    header(
        rects[2 * n],
        format!("SINS {}/{MAX_SINS}{}", u.owned_sins(), full(sins_full)),
        sins_full,
    );
    for (i, def) in ARTS.iter().enumerate() {
        let lv = u.arts[i];
        let union = u.union_of(i);
        let color = if u.evolved[i] || union.is_some() {
            DBG_GOLD
        } else {
            WHITE
        };
        // The union's first Art carries its name; the second is out of the slots.
        let name = match union {
            Some(k) if UNIONS[k].arts[0] == i => tr(UNIONS[k].name),
            _ => tr(def.name),
        };
        let level = if lv > 0 {
            format!("{lv}/{ART_MAX_LEVEL}")
        } else {
            "-".to_string()
        };
        let dim = lv == 0 && arts_full;
        debug_row(
            rects[i],
            name,
            &level,
            lv > 0,
            dim,
            focus.is_highlighted(i),
            color,
        );
        // An Art with no evolution of its own has a Unite button instead.
        let (label, owned, dim, color) = match &def.evolution {
            Some(_) if u.evolved[i] => ("EVOLVED".to_string(), true, false, DBG_GOLD),
            Some(_) if u.can_evolve(i) => ("EVOLVE".to_string(), true, false, WHITE),
            Some(evo) => (format!("+{}", tr(SINS[evo.sin].name)), false, true, GRAY),
            None => {
                let k = debug_union(i);
                let other = UNIONS[k].arts.into_iter().find(|&a| a != i).unwrap_or(i);
                if u.united[k] {
                    ("UNITED".to_string(), true, false, DBG_GOLD)
                } else if u.can_unite(k) {
                    ("UNITE".to_string(), true, false, WHITE)
                } else {
                    (format!("+{}", tr(ARTS[other].name)), false, true, GRAY)
                }
            }
        };
        let lit = focus.is_highlighted(n + i);
        debug_row(rects[n + i], &label, "", owned, dim, lit, color);
    }
    for (i, def) in SINS.iter().enumerate() {
        let lv = u.sins[i];
        let level = if lv > 0 {
            format!("{lv}/{}", def.max_level)
        } else {
            "-".to_string()
        };
        let dim = lv == 0 && sins_full;
        let lit = focus.is_highlighted(2 * n + i);
        debug_row(
            rects[2 * n + i],
            tr(def.name),
            &level,
            lv > 0,
            dim,
            lit,
            WHITE,
        );
    }
    let help_y = DBG_TOP + 14.0 + SINS.len() as f32 * DBG_ROW + 14.0;
    for (k, line) in [
        "Click / Enter / Right: level up    Right-click / Left: level down",
        "Evolve needs Lv 8 and the Sin shown    Unite needs both Arts at Lv 8",
        "F2 / Esc: close",
    ]
    .iter()
    .enumerate()
    {
        let w = measure_ui_text(line, TextStyle::Desc).width;
        draw_ui_text(
            line,
            ((view.x - w) * 0.5).round(),
            help_y + k as f32 * 13.0,
            TextStyle::Desc,
            LIGHTGRAY,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_frames_cover_the_strip_once() {
        let mut seen = [false; ICON_COUNT];
        let evolving = (0..ARTS.len()).filter(|&i| ARTS[i].evolution.is_some());
        let icons = (0..ARTS.len())
            .map(Icon::Art)
            .chain(evolving.map(Icon::Evolution))
            .chain((0..UNIONS.len()).map(Icon::Union))
            .chain((0..SINS.len()).map(Icon::Sin));
        for icon in icons {
            let f = icon.frame();
            assert!(f < ICON_COUNT && !seen[f], "{icon:?}");
            seen[f] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(Icon::Evolution(HELLFLARE).frame(), ARTS.len());
        assert_eq!(Icon::Sin(0).frame(), ICON_COUNT - SINS.len());
    }
}
