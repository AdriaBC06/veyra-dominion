//! Credits: a slowly scrolling list of names. The full licence texts (OFL for the
//! fonts, MIT and friends for macroquad and its dependencies) ship in the `licenses/`
//! folder next to the game, with the EULA, not on screen.

use super::Nav;
use crate::assets::{draw_ui_text, measure_ui_text, TextStyle};
use crate::config::GAME_TITLE;
use crate::hud::{self, Anchor};
use crate::i18n::{self, tr, Key, Lang};
use crate::ui::{self, UiInput};
use macroquad::prelude::*;

const AUTHOR: &str = "Adrià Bonnin Catalán";
/// The game is proprietary (see `LICENSE`); `©` is not pre-rasterised, hence `(c)`.
const COPYRIGHT_YEAR: u32 = 2026;
const TOP_Y: f32 = 70.0;
/// Seconds before the list starts scrolling on its own.
const SCROLL_DELAY: f32 = 2.0;
const SCROLL_SPEED: f32 = 24.0;
/// Scroll per wheel notch or arrow press.
const SCROLL_STEP: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Title,
    Heading,
    Name,
    Gap,
}

impl Kind {
    fn height(self) -> f32 {
        match self {
            Kind::Title => 40.0,
            Kind::Heading | Kind::Name => 20.0,
            Kind::Gap => 14.0,
        }
    }
}

pub struct Credits {
    lines: Vec<(Kind, String)>,
    lang: Lang,
    scroll: f32,
    time: f32,
}

impl Credits {
    pub fn new() -> Self {
        Self {
            lines: build_lines(),
            lang: i18n::lang(),
            scroll: 0.0,
            time: 0.0,
        }
    }

    fn content_height(&self) -> f32 {
        self.lines.iter().map(|(k, _)| k.height()).sum()
    }

    pub fn update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        if input.back || input.confirm || input.clicked {
            return Some(Nav::Title);
        }
        if self.lang != i18n::lang() {
            self.lines = build_lines();
            self.lang = i18n::lang();
        }
        self.time += get_frame_time();
        let manual = input.wheel != 0.0 || input.up || input.down;
        if manual {
            // Taking control stops the automatic scroll for a while.
            self.time = 0.0;
            let up = input.wheel > 0.0 || input.up;
            self.scroll += if up { -SCROLL_STEP } else { SCROLL_STEP };
        } else if self.time > SCROLL_DELAY {
            self.scroll += SCROLL_SPEED * get_frame_time();
        }
        let max = (TOP_Y + self.content_height() - view.y * 0.5).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max);
        None
    }

    pub fn draw(&self, view: Vec2) {
        hud::draw_dim(view);
        draw_rectangle(0.0, 0.0, view.x, view.y, Color::new(0.0, 0.0, 0.0, 0.3));
        let mut y = TOP_Y - self.scroll.round();
        for (kind, text) in &self.lines {
            y += kind.height();
            if y < -10.0 || y > view.y + 40.0 {
                continue;
            }
            let (style, color) = match kind {
                Kind::Title => (TextStyle::Title, hud::TITLE_COLOR),
                Kind::Heading => (TextStyle::Body, ui::ACCENT),
                Kind::Name => (TextStyle::Body, WHITE),
                Kind::Gap => continue,
            };
            hud::draw_text_centered(view, text, y, style, color);
        }
        let help = tr(Key::CreditsHelp);
        let w = measure_ui_text(help, TextStyle::Small).width;
        let bar = view.y - 16.0;
        draw_rectangle(0.0, bar, view.x, 16.0, BLACK);
        draw_ui_text(
            help,
            Anchor::Right.x(view, w, 6.0),
            bar + 12.0,
            TextStyle::Small,
            GRAY,
        );
    }
}

fn build_lines() -> Vec<(Kind, String)> {
    let mut lines = vec![
        (Kind::Title, GAME_TITLE.to_owned()),
        (Kind::Name, format!("(c) {COPYRIGHT_YEAR} {AUTHOR}")),
        (Kind::Name, tr(Key::CreditsRights).to_owned()),
        (Kind::Gap, String::new()),
    ];
    let mut section = |heading: Key, names: &[&str]| {
        lines.push((Kind::Heading, tr(heading).to_owned()));
        lines.extend(names.iter().map(|n| (Kind::Name, (*n).to_owned())));
        lines.push((Kind::Gap, String::new()));
    };
    section(Key::CreditsDesign, &[AUTHOR]);
    section(Key::CreditsArt, &[AUTHOR]);
    section(Key::CreditsMusic, &[AUTHOR, tr(Key::CreditsSynth)]);
    section(
        Key::CreditsFonts,
        &[
            "Jacquard 24, Jersey 15 - Sarah Cadigan-Fried",
            "Press Start 2P - Cody \"CodeMan38\" Boisclair",
            "SIL Open Font License 1.1",
        ],
    );
    section(
        Key::CreditsEngine,
        &["macroquad - Fedor Logachev", "MIT / Apache-2.0"],
    );
    section(Key::CreditsThanks, &[tr(Key::CreditsThanksYou)]);
    section(
        Key::CreditsLicences,
        &[tr(Key::CreditsLicenceFiles), tr(Key::CreditsEula)],
    );
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::EXTRA_GLYPHS;

    #[test]
    fn author_is_in_the_font_cache() {
        for c in AUTHOR.chars() {
            assert!(
                (' '..='~').contains(&c) || EXTRA_GLYPHS.contains(c),
                "{c:?}"
            );
        }
    }
}
