//! Title menu over the idle world: Play, Settings, Credits, Quit, with the Veyra portrait
//! (V14 trial) bottom right and the menu centred in the space left of it.

use super::Nav;
use crate::assets;
use crate::config::{GAME_TITLE, VICTORY_TIME};
use crate::hud;
use crate::i18n::{tr, trf, Key};
use crate::ui::{self, Action, Focus, UiInput};
use macroquad::prelude::*;

const ITEMS: [(Key, Nav); 4] = [
    (Key::MenuPlay, Nav::NewRun),
    (Key::MenuSettings, Nav::Settings),
    (Key::MenuCredits, Nav::Credits),
    (Key::MenuQuit, Nav::Quit),
];

#[derive(Default)]
pub struct Title {
    focus: Focus,
    /// Portrait candidate shown (V14 trial, F2 cycles `assets::TITLE_PORTRAITS`).
    portrait: usize,
}

/// The part of the view left of the portrait (never narrower than the buttons), as a view.
fn menu_view(view: Vec2, portrait: usize) -> Vec2 {
    let w = assets::title_portrait(portrait).size().x;
    vec2((view.x - w).max(ui::BUTTON_W + 32.0), view.y)
}

fn buttons(view: Vec2, portrait: usize) -> Vec<Rect> {
    ui::button_column(
        menu_view(view, portrait),
        hud::center_screen_buttons(1),
        ITEMS.len(),
    )
}

impl Title {
    pub fn update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        if is_key_pressed(KeyCode::F2) {
            self.portrait = (self.portrait + 1) % assets::TITLE_PORTRAITS.len();
        }
        match self.focus.update(input, &buttons(view, self.portrait)) {
            Action::Confirm(i) => Some(ITEMS[i].1),
            _ => None,
        }
    }

    pub fn draw(&self, view: Vec2) {
        let tagline = trf(Key::TitleTagline, &[&hud::format_time(VICTORY_TIME)]);
        hud::draw_dim(view);
        let portrait = assets::title_portrait(self.portrait);
        portrait.draw(view - portrait.size(), get_time());
        let label = format!("F2: {}", assets::TITLE_PORTRAITS[self.portrait].name);
        draw_text(&label, view.x - 40.0, 12.0, 16.0, GRAY);
        hud::draw_center_text(menu_view(view, self.portrait), GAME_TITLE, &[&tagline]);
        let labels = ITEMS.map(|(k, _)| tr(k));
        ui::draw_buttons(&buttons(view, self.portrait), &labels, &self.focus);
    }
}
