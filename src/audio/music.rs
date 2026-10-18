//! The soundtrack: `.vtrk` songs embedded from `assets/music/`, rendered to one
//! seamless loop each at load. `std` only, so `render_audio` can export them.

use super::synth::SAMPLE_RATE;
use super::tracker::Song;

/// Every track is levelled to this long-term RMS (about -26 dBFS): a bed under
/// the combat effects (-19 to -25 dB peaks), never over the events.
const TARGET_RMS: f32 = 0.05;
/// Levelling never pushes a peak past this.
const MAX_PEAK: f32 = 0.95;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Title,
    Ashford,
}

impl Track {
    pub const ALL: [Track; 2] = [Track::Title, Track::Ashford];

    pub fn name(self) -> &'static str {
        match self {
            Track::Title => "title",
            Track::Ashford => "ashford",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Track::Title => include_str!("../../assets/music/title.vtrk"),
            Track::Ashford => include_str!("../../assets/music/ashford.vtrk"),
        }
    }

    pub fn song(self) -> Result<Song, String> {
        Song::parse(self.source()).map_err(|e| format!("{}.vtrk: {e}", self.name()))
    }
}

/// One loop of `track`, levelled to `TARGET_RMS`.
pub fn render(track: Track) -> Result<Vec<f32>, String> {
    let mut out = track.song()?.render_loop(SAMPLE_RATE);
    let rms = (out.iter().map(|x| x * x).sum::<f32>() / out.len().max(1) as f32).sqrt();
    let peak = out.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    if rms > 0.0 {
        let gain = (TARGET_RMS / rms).min(MAX_PEAK / peak);
        out.iter_mut().for_each(|x| *x *= gain);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_parses_renders_and_is_levelled() {
        for track in Track::ALL {
            let out = render(track).unwrap();
            let secs = out.len() as f32 / SAMPLE_RATE as f32;
            assert!((20.0..120.0).contains(&secs), "{track:?}: {secs} s");
            assert!(out.iter().all(|x| x.abs() <= MAX_PEAK + 1e-4));
            let rms = (out.iter().map(|x| x * x).sum::<f32>() / out.len() as f32).sqrt();
            assert!((rms - TARGET_RMS).abs() < 0.005, "{track:?}: rms {rms}");
        }
    }
}
