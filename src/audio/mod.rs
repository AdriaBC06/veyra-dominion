//! Audio: own chip synthesizer, sound effect presets and the mixer. Every effect
//! variant is rendered at load and handed to macroquad as WAV bytes. Anything on
//! the main thread can queue an effect with `cue`; the app plays the queue once
//! per frame. The simulation never cues directly: `Game` collects `Cue`s and the
//! run scene translates them.

pub mod mixer;
pub mod sfx;
// Releases, loops and `render_seconds` are for the tracker (V10) and the example.
#[allow(dead_code)]
pub mod synth;
pub mod wav;

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use mixer::Mixer;
pub use sfx::Sfx;
use std::cell::RefCell;

thread_local! {
    /// Effects requested this frame, at most one of each.
    static QUEUE: RefCell<Vec<Sfx>> = RefCell::new(Vec::with_capacity(Sfx::COUNT));
}

/// Queues `sfx` to be played at the end of this frame.
pub fn cue(sfx: Sfx) {
    QUEUE.with_borrow_mut(|q| {
        if !q.contains(&sfx) {
            q.push(sfx);
        }
    });
}

/// Empties the queue and returns it, for tests.
#[cfg(test)]
pub fn take_cues() -> Vec<Sfx> {
    QUEUE.with_borrow_mut(std::mem::take)
}

pub struct Audio {
    /// Per effect, one sound per pitch variant (`Sfx::variants`).
    sounds: Vec<Vec<Sound>>,
    mixer: Mixer,
    sfx_volume: f32,
}

impl Audio {
    /// Renders and loads every effect variant. A variant that fails to load is
    /// only reported; that effect then stays silent.
    pub async fn load() -> Self {
        let mut sounds = Vec::with_capacity(Sfx::COUNT);
        for sfx in Sfx::ALL {
            let mut variants = Vec::with_capacity(sfx.variants().len());
            for &pitch in sfx.variants() {
                let bytes = wav::encode(&sfx::render(sfx, pitch), synth::SAMPLE_RATE);
                match load_sound_from_bytes(&bytes).await {
                    Ok(sound) => variants.push(sound),
                    Err(e) => eprintln!("audio: cannot load {}: {e}", sfx.name()),
                }
            }
            sounds.push(variants);
        }
        Self {
            sounds,
            mixer: Mixer::default(),
            sfx_volume: 1.0,
        }
    }

    pub fn set_volumes(&mut self, master: f32, sfx: f32) {
        self.sfx_volume = master * sfx;
    }

    /// Plays this frame's queued effects; `now` is the app clock in seconds.
    pub fn play_queued(&mut self, now: f64) {
        QUEUE.with_borrow_mut(|q| {
            for sfx in q.drain(..) {
                let Some(variant) = self.mixer.trigger(sfx, now) else {
                    continue;
                };
                let volume = self.sfx_volume * sfx.gain();
                if let Some(sound) = self.sounds[sfx as usize].get(variant) {
                    if volume > 0.0 {
                        play_sound(
                            sound,
                            PlaySoundParams {
                                looped: false,
                                volume,
                            },
                        );
                    }
                }
            }
        });
    }
}
