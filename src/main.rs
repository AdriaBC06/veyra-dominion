mod arts;
mod assets;
mod audio;
#[cfg(test)]
mod balance;
mod blood;
mod bot;
mod config;
mod enemies;
mod game;
mod hud;
mod i18n;
mod save;
mod scenes;
mod screen;
mod spatial;
mod ui;
mod upgrades;

use audio::Audio;
use config::*;
use macroquad::prelude::*;
use save::Settings;
use scenes::App;

fn window_conf() -> Conf {
    let settings = Settings::load();
    Conf {
        window_title: GAME_TITLE.to_owned(),
        window_width: settings.window.0 as i32,
        window_height: settings.window.1 as i32,
        fullscreen: settings.fullscreen,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    assets::init();
    let settings = Settings::load();
    i18n::set_lang(settings.lang);
    let mut app = App::new(settings, Audio::load().await);
    prevent_quit();

    loop {
        if is_quit_requested() || !app.frame() {
            app.shutdown();
            break;
        }
        next_frame().await;
    }
}
