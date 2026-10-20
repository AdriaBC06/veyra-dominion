//! Audio: own chip synthesizer, sound effect presets, the tracker music and the
//! mixer. Every effect variant, jingle and preloaded track is rendered at load
//! and handed to macroquad as WAV bytes; tracks play as seamless loops and duck
//! under jingles. Anything on the main thread can queue an effect with `cue` or a
//! jingle with `jingle`; the app plays the queue once per frame. The simulation
//! never cues directly: `Game` collects `Cue`s and the run scene translates them.

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
pub use music::{Jingle, Track};
pub use sfx::Sfx;
use std::cell::{Cell, RefCell};

/// Tracks rendered at load: the ones the game plays so far. macroquad keeps a
/// loaded sound as stereo f32 (about 350 KB a second), so all nine loops would
/// take about 160 MB; the map and boss themes load with their map instead
/// (V19/V23).
const PRELOADED: [Track; 2] = [Track::Title, Track::Ashford];

thread_local! {
    /// Effects requested this frame, at most one of each.
    static QUEUE: RefCell<Vec<Sfx>> = RefCell::new(Vec::with_capacity(Sfx::COUNT));
    /// The jingle requested this frame; a later request replaces it.
    static JINGLE: Cell<Option<Jingle>> = const { Cell::new(None) };
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

/// Plays `jingle` at the end of this frame, over any jingle still playing, with
/// the music ducked under it.
pub fn jingle(jingle: Jingle) {
    JINGLE.set(Some(jingle));
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
    /// One loop per `Track`; `None` if not preloaded or it failed to load.
    music: Vec<Option<Sound>>,
    playing: Option<Track>,
    /// master × music, before ducking.
    music_volume: f32,
    /// Per `Jingle`, the sound and its length in seconds.
    jingles: Vec<Option<(Sound, f64)>>,
    jingle: Option<Jingle>,
    /// When the current jingle ends, on the app clock.
    jingle_end: f64,
    /// The music gain from ducking, last applied.
    duck: f32,
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
        let mut music: Vec<Option<Sound>> = Track::ALL.iter().map(|_| None).collect();
        for track in PRELOADED {
            music[track as usize] = load_samples(music::render(track)).await.map(|s| s.0);
        }
        let mut jingles = Vec::with_capacity(Jingle::ALL.len());
        for jingle in Jingle::ALL {
            jingles.push(load_samples(music::render_jingle(jingle)).await);
        }
        Self {
            sounds,
            mixer: Mixer::default(),
            sfx_volume: 1.0,
            music,
            playing: None,
            music_volume: 1.0,
            jingles,
            jingle: None,
            jingle_end: f64::NEG_INFINITY,
            duck: 1.0,
        }
    }

    fn playing_track(&self) -> Option<&Sound> {
        self.playing.and_then(|t| self.music[t as usize].as_ref())
    }

    pub fn set_volumes(&mut self, master: f32, music: f32, sfx: f32) {
        self.sfx_volume = master * sfx;
        let music = master * music;
        if music != self.music_volume {
            self.music_volume = music;
            if let Some(sound) = self.playing_track() {
                set_sound_volume(sound, music * self.duck);
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
                    volume: self.music_volume * self.duck,
                },
            );
        }
        self.playing = Some(track);
    }

    pub fn stop_music(&mut self) {
        if let Some(sound) = self.playing_track() {
            stop_sound(sound);
        }
        self.playing = None;
    }

    /// Cuts the current jingle short and lifts the ducking at once.
    pub fn stop_jingle(&mut self) {
        if let Some((sound, _)) = self
            .jingle
            .take()
            .and_then(|j| self.jingles[j as usize].as_ref())
        {
            stop_sound(sound);
        }
        self.jingle_end = f64::NEG_INFINITY;
    }

    /// Plays this frame's jingle and queued effects and updates the ducking;
    /// `now` is the app clock in seconds.
    pub fn play_queued(&mut self, now: f64) {
        if let Some(jingle) = JINGLE.take() {
            self.stop_jingle();
            if let Some((sound, secs)) = &self.jingles[jingle as usize] {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: self.music_volume,
                    },
                );
                self.jingle = Some(jingle);
                self.jingle_end = now + secs;
            }
        }
        let duck = mixer::duck(now, self.jingle_end);
        if duck != self.duck {
            self.duck = duck;
            if let Some(sound) = self.playing_track() {
                set_sound_volume(sound, self.music_volume * duck);
            }
        }
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

/// Loads rendered samples as a sound with its length in seconds; failures are
/// only reported.
async fn load_samples(samples: Result<Vec<f32>, String>) -> Option<(Sound, f64)> {
    let samples = samples.map_err(|e| eprintln!("audio: {e}")).ok()?;
    let secs = samples.len() as f64 / f64::from(synth::SAMPLE_RATE);
    match load_sound_from_bytes(&wav::encode(&samples, synth::SAMPLE_RATE)).await {
        Ok(sound) => Some((sound, secs)),
        Err(e) => {
            eprintln!("audio: {e}");
            None
        }
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
