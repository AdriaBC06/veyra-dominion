//! Title menu: the VEYRA logo with its flickering hellfire glow, DOMINION under it and Play,
//! Settings, Credits, Quit, centred in the space left of Veyra's portrait. The backdrop and the
//! portrait are drawn by `Backdrop`.

use super::Nav;
use crate::assets::{self, TitleArt};
use crate::i18n::{tr, Key};
use crate::ui::{self, Action, Focus, Overlay, UiInput};
use macroquad::prelude::*;

const ITEMS: [(Key, Nav); 4] = [
    (Key::MenuPlay, Nav::NewRun),
    (Key::MenuSettings, Nav::Settings),
    (Key::MenuCredits, Nav::Credits),
    (Key::MenuQuit, Nav::Quit),
];

const LOGO_Y: f32 = 26.0;
/// Gap between VEYRA and DOMINION.
const DOMINION_GAP: f32 = 5.0;
const BUTTONS_Y: f32 = 120.0;
/// The kit's stone buttons, narrower than the panel ones to fit left of the portrait.
const BUTTON: Vec2 = vec2(168.0, ui::BUTTON_H);
/// Margin around the logo in its glow sprite.
const GLOW_MARGIN: f32 = 8.0;

#[derive(Default)]
pub struct Title {
    focus: Focus,
    overlay: Overlay,
}

/// The part of the view left of the portrait (never narrower than the logo), as a view.
fn menu_view(view: Vec2) -> Vec2 {
    let w = assets::title_portrait().size().x;
    let logo = assets::title_art_size(TitleArt::Logo).x;
    vec2((view.x - w).max(logo + 32.0), view.y)
}

/// Left edge that centres something `w` wide in the menu space.
fn centred(view: Vec2, w: f32) -> f32 {
    ((menu_view(view).x - w) * 0.5).round()
}

fn buttons(view: Vec2) -> Vec<Rect> {
    let x = centred(view, BUTTON.x);
    (0..ITEMS.len())
        .map(|i| {
            let y = BUTTONS_Y + i as f32 * (BUTTON.y + ui::BUTTON_GAP);
            Rect::new(x, y, BUTTON.x, BUTTON.y)
        })
        .collect()
}

/// Glow strength (frame of the glow strip) at time `t`: a slow breath with quick flickers.
fn glow_frame(t: f64) -> u32 {
    let v = 0.5 + 0.3 * (t * 1.7).sin() + 0.15 * (t * 6.3 + 1.0).sin() + 0.1 * (t * 13.1).sin();
    (v * 4.0).clamp(0.0, 3.0) as u32
}

impl Title {
    pub fn update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        let items = buttons(view);
        let action = self.focus.update(input, &items);
        self.overlay
            .update(input, &self.focus, &items, get_frame_time());
        match action {
            Action::Confirm(i) => Some(ITEMS[i].1),
            _ => None,
        }
    }

    pub fn draw(&self, view: Vec2, t: f64) {
        let logo = assets::title_art_size(TitleArt::Logo);
        let logo_pos = vec2(centred(view, logo.x), LOGO_Y);
        assets::draw_title_art(TitleArt::LogoGlow, glow_frame(t), logo_pos - GLOW_MARGIN);
        assets::draw_title_art(TitleArt::Logo, 0, logo_pos);
        let dominion = assets::title_art_size(TitleArt::Dominion);
        let y = LOGO_Y + logo.y + DOMINION_GAP;
        assets::draw_title_art(TitleArt::Dominion, 0, vec2(centred(view, dominion.x), y));

        let labels = ITEMS.map(|(k, _)| tr(k));
        ui::draw_buttons(&buttons(view), &labels, &self.focus, &self.overlay);
    }
}
