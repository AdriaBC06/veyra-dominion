//! Audio: own chip synthesizer, sound effect presets, the tracker music and the
//! mixer. Every effect variant and every track is rendered at load and handed to
//! macroquad as WAV bytes; tracks play as seamless loops. Anything on
//! the main thread can queue an effect with `cue`; the app plays the queue once
//! per frame. The simulation never cues directly: `Game` collects `Cue`s and the
//! run scene translates them.

pub mod mixer;
pub mod music;
pub mod sfx;
// `render_seconds` and `Channel::ALL` are for the example and tests.
#[allow(dead_code)]
pub mod synth;
pub mod tracker;
pub mod wav;

use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use mixer::Mixer;
pub use music::Track;
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

/// Queues `new` in place of a queued `old`, e.g. a specific confirm sound over
/// the generic one the focus already cued.
pub fn recue(old: Sfx, new: Sfx) {
    QUEUE.with_borrow_mut(|q| q.retain(|&s| s != old));
    cue(new);
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
    /// One loop per `Track`; `None` if it failed to render or load.
    music: Vec<Option<Sound>>,
    playing: Option<Track>,
    music_volume: f32,
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
        let mut music = Vec::with_capacity(Track::ALL.len());
        for track in Track::ALL {
            let sound = match music::render(track) {
                Ok(samples) => load_sound_from_bytes(&wav::encode(&samples, synth::SAMPLE_RATE))
                    .await
                    .map_err(|e| e.to_string()),
                Err(e) => Err(e),
            };
            music.push(sound.map_err(|e| eprintln!("audio: {e}")).ok());
        }
        Self {
            sounds,
            mixer: Mixer::default(),
            sfx_volume: 1.0,
            music,
            playing: None,
            music_volume: 1.0,
        }
    }

    pub fn set_volumes(&mut self, master: f32, music: f32, sfx: f32) {
        self.sfx_volume = master * sfx;
        let music = master * music;
        if music != self.music_volume {
            self.music_volume = music;
            if let Some(sound) = self.playing.and_then(|t| self.music[t as usize].as_ref()) {
                set_sound_volume(sound, music);
            }
        }
    }

    /// Loops `track` from its start, unless it is already playing.
    pub fn play_music(&mut self, track: Track) {
        if self.playing == Some(track) {
            return;
        }
        self.stop_music();
        if let Some(sound) = &self.music[track as usize] {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: self.music_volume,
                },
            );
        }
        self.playing = Some(track);
    }

    pub fn stop_music(&mut self) {
        if let Some(sound) = self.playing.take().and_then(|t| self.music[t as usize].as_ref()) {
            stop_sound(sound);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_dedupes_and_recue_swaps() {
        take_cues();
        cue(Sfx::UiConfirm);
        cue(Sfx::UiConfirm);
        cue(Sfx::Soul);
        recue(Sfx::UiConfirm, Sfx::UpgradePick);
        assert_eq!(take_cues(), [Sfx::Soul, Sfx::UpgradePick]);
    }
}
