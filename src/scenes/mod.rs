//! Scene system: the app is in exactly one scene (title, run, settings or credits)
//! and scenes ask for a change by returning a `Nav`. The run is kept while
//! Settings is open from the pause menu. Every change goes through the ember wipe.
//! The title, credits and settings (outside a run) sit on the painted title backdrop.

mod backdrop;
mod credits;
mod run;
mod settings;
mod title;
mod wipe;

use crate::audio::{Audio, Track};
use crate::hud;
use crate::i18n;
use crate::save::{SaveData, Settings};
use crate::screen::Screen;
use crate::ui::UiInput;
use backdrop::Backdrop;
use credits::Credits;
use macroquad::prelude::*;
use run::Run;
use settings::SettingsScene;
use title::Title;
use wipe::Wipe;

/// A scene change requested by the current scene.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nav {
    Title,
    NewRun,
    /// Back to the paused run (from Settings).
    ResumeRun,
    Settings,
    Credits,
    Quit,
}

enum Scene {
    Title(Title),
    Run,
    Settings(SettingsScene),
    Credits(Credits),
}

pub struct App {
    settings: Settings,
    audio: Audio,
    screen: Screen,
    save: SaveData,
    run: Run,
    scene: Scene,
    backdrop: Backdrop,
    wipe: Wipe,
    debug: bool,
}

impl App {
    pub fn new(settings: Settings, audio: Audio) -> Self {
        let (w, h) = settings.window;
        let screen = Screen::new(
            settings.scale_mode,
            settings.fullscreen,
            vec2(w as f32, h as f32),
        );
        Self {
            settings,
            audio,
            screen,
            save: SaveData::load(),
            run: Run::new(),
            scene: Scene::Title(Title::default()),
            backdrop: Backdrop::default(),
            wipe: Wipe::new(),
            debug: false,
        }
    }

    /// Copies the screen state into the settings and writes them.
    fn store_settings(&mut self) {
        let windowed = self.screen.windowed_size();
        let s = &mut self.settings;
        s.lang = i18n::lang();
        s.scale_mode = self.screen.mode;
        s.fullscreen = self.screen.fullscreen();
        s.window = (windowed.x as u32, windowed.y as u32);
        s.store();
    }

    /// Adds the current run to the lifetime stats (if it started) and replaces it.
    fn end_run(&mut self) {
        let game = &self.run.game;
        if game.time > 0.0 && !self.run.autopiloted {
            self.save.record_run(game.time, game.kills);
            self.save.store();
        }
        self.run = Run::new();
    }

    /// Saves everything before the app closes.
    pub fn shutdown(&mut self) {
        self.end_run();
        self.store_settings();
    }

    /// Global developer and display keys, active in every scene.
    fn hotkeys(&mut self) {
        if is_key_pressed(KeyCode::F3) {
            self.debug = !self.debug;
        }
        let mut changed = false;
        if is_key_pressed(KeyCode::F9) {
            i18n::set_lang(i18n::lang().next());
            changed = true;
        }
        if is_key_pressed(KeyCode::F10) {
            self.screen.toggle_mode();
            changed = true;
        }
        if is_key_pressed(KeyCode::F11) {
            self.screen.toggle_fullscreen();
            changed = true;
        }
        if changed {
            self.store_settings();
        }
    }

    fn navigate(&mut self, nav: Nav) {
        if matches!(self.scene, Scene::Settings(_)) {
            self.store_settings();
        }
        let in_run = matches!(self.scene, Scene::Run);
        if matches!(nav, Nav::Title | Nav::NewRun) {
            // A victory or game over jingle ends with the run.
            self.audio.stop_jingle();
        }
        self.scene = match nav {
            Nav::Title => {
                if in_run {
                    self.end_run();
                }
                Scene::Title(Title::default())
            }
            Nav::NewRun => {
                self.end_run();
                // A new run starts its theme from the top.
                self.audio.stop_music();
                Scene::Run
            }
            Nav::ResumeRun => Scene::Run,
            Nav::Settings => Scene::Settings(SettingsScene::new(in_run)),
            Nav::Credits => Scene::Credits(Credits::new()),
            Nav::Quit => return,
        };
    }

    /// The title backdrop shows behind every scene except a run (and its pause settings).
    fn on_backdrop(&self) -> bool {
        match &self.scene {
            Scene::Title(_) | Scene::Credits(_) => true,
            Scene::Settings(s) => !s.in_run(),
            Scene::Run => false,
        }
    }

    /// Runs one frame. Returns false when the player chose to quit.
    pub fn frame(&mut self) -> bool {
        self.hotkeys();
        self.screen.update();
        let view = self.screen.view();
        self.run.game.set_view(view);
        let input = UiInput::read(self.screen.layout);
        let dt = get_frame_time();
        self.backdrop.update(input.mouse, view, dt);

        // the old scene takes no input while the wipe covers it
        let nav = if self.wipe.covering() {
            None
        } else {
            self.scene_update(&input, view)
        };
        if nav == Some(Nav::Quit) {
            return false;
        }
        if let Some(nav) = nav {
            self.wipe.start(nav);
        }
        if let Some(nav) = self.wipe.update(dt) {
            self.navigate(nav);
        }

        let s = &self.settings;
        self.audio.set_volumes(s.master, s.music, s.sfx);
        // Settings keeps whatever was playing (title, or the paused run's theme).
        match self.scene {
            Scene::Title(_) | Scene::Credits(_) => self.audio.play_music(Track::Title),
            Scene::Run => self.audio.play_music(Track::Ashford),
            Scene::Settings(_) => {}
        }
        self.audio.play_queued(get_time());

        self.draw(view);
        true
    }

    fn scene_update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        match &mut self.scene {
            Scene::Title(t) => t.update(input, view),
            Scene::Run => self.run.update(input, view, self.settings.mouse_move),
            Scene::Settings(s) => s.update(
                input,
                view,
                &mut self.settings,
                &mut self.screen,
                &mut self.save,
            ),
            Scene::Credits(c) => c.update(input, view),
        }
    }

    fn draw(&mut self, view: Vec2) {
        let t = get_time();
        if self.on_backdrop() {
            set_camera(&self.screen.camera(Vec2::ZERO));
            clear_background(BLACK);
            self.backdrop.draw(view, t);
        } else {
            self.run.draw_world(&self.screen);
            set_camera(&self.screen.camera(Vec2::ZERO));
        }
        match &self.scene {
            Scene::Title(title) => title.draw(view, t),
            Scene::Run => self.run.draw_ui(view),
            Scene::Settings(s) => s.draw(view, &self.settings, &self.screen, &self.save),
            Scene::Credits(c) => c.draw(view),
        }
        if self.debug {
            hud::draw_debug(&self.run.game, get_fps(), self.run.update_ms);
        }
        self.wipe.draw(view);
        self.screen.present();
    }
}
