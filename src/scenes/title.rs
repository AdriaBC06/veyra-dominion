//! Title menu over the idle world: Play, Settings, Credits, Quit, with the animated Veyra
//! portrait bottom right and the menu centred in the space left of it.

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
}

/// The part of the view left of the portrait (never narrower than the buttons), as a view.
fn menu_view(view: Vec2) -> Vec2 {
    let w = assets::title_portrait().size().x;
    vec2((view.x - w).max(ui::BUTTON_W + 32.0), view.y)
}

fn buttons(view: Vec2) -> Vec<Rect> {
    ui::button_column(menu_view(view), hud::center_screen_buttons(1), ITEMS.len())
}

impl Title {
    pub fn update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        match self.focus.update(input, &buttons(view)) {
            Action::Confirm(i) => Some(ITEMS[i].1),
            _ => None,
        }
    }

    pub fn draw(&self, view: Vec2) {
        let tagline = trf(Key::TitleTagline, &[&hud::format_time(VICTORY_TIME)]);
        hud::draw_dim(view);
        let portrait = assets::title_portrait();
        portrait.draw(view - portrait.size(), get_time());
        hud::draw_center_text(menu_view(view), GAME_TITLE, &[&tagline]);
        let labels = ITEMS.map(|(k, _)| tr(k));
        ui::draw_buttons(&buttons(view), &labels, &self.focus);
    }
}
