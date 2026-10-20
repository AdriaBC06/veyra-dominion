//! The soundtrack: `.vtrk` songs embedded from `assets/music/`. Tracks render
//! to one seamless loop each, jingles to one pass with its tail. `std` only, so
//! `render_audio` can export them.

use super::synth::SAMPLE_RATE;
use super::tracker::Song;

/// Every track is levelled to this long-term RMS (about -26 dBFS): a bed under
/// the combat effects (-19 to -25 dB peaks), never over the events.
const TARGET_RMS: f32 = 0.05;
/// Jingles are levelled like the event effects: their loudest 50 ms RMS at
/// about -16 dB, next to the level-up (-15.7).
const JINGLE_LEVEL: f32 = 0.16;
/// The longest a jingle's release and echo may ring after its last row.
const JINGLE_TAIL: f32 = 4.0;
/// Levelling never pushes a peak past this.
const MAX_PEAK: f32 = 0.95;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Title,
    Ashford,
    Abbey,
    Bastion,
    Celestial,
    BossCaptain,
    BossInquisitor,
    BossPaladin,
    BossArchangel,
}

impl Track {
    pub const ALL: [Track; 9] = [
        Track::Title,
        Track::Ashford,
        Track::Abbey,
        Track::Bastion,
        Track::Celestial,
        Track::BossCaptain,
        Track::BossInquisitor,
        Track::BossPaladin,
        Track::BossArchangel,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Track::Title => "title",
            Track::Ashford => "ashford",
            Track::Abbey => "abbey",
            Track::Bastion => "bastion",
            Track::Celestial => "celestial",
            Track::BossCaptain => "boss_captain",
            Track::BossInquisitor => "boss_inquisitor",
            Track::BossPaladin => "boss_paladin",
            Track::BossArchangel => "boss_archangel",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Track::Title => include_str!("../../assets/music/title.vtrk"),
            Track::Ashford => include_str!("../../assets/music/ashford.vtrk"),
            Track::Abbey => include_str!("../../assets/music/abbey.vtrk"),
            Track::Bastion => include_str!("../../assets/music/bastion.vtrk"),
            Track::Celestial => include_str!("../../assets/music/celestial.vtrk"),
            Track::BossCaptain => include_str!("../../assets/music/boss_captain.vtrk"),
            Track::BossInquisitor => include_str!("../../assets/music/boss_inquisitor.vtrk"),
            Track::BossPaladin => include_str!("../../assets/music/boss_paladin.vtrk"),
            Track::BossArchangel => include_str!("../../assets/music/boss_archangel.vtrk"),
        }
    }

    pub fn song(self) -> Result<Song, String> {
        Song::parse(self.source()).map_err(|e| format!("{}.vtrk: {e}", self.name()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Jingle {
    Victory,
    GameOver,
    SummonLesser,
    SummonGreater,
    SummonAbyssal,
}

impl Jingle {
    pub const ALL: [Jingle; 5] = [
        Jingle::Victory,
        Jingle::GameOver,
        Jingle::SummonLesser,
        Jingle::SummonGreater,
        Jingle::SummonAbyssal,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Jingle::Victory => "jingle_victory",
            Jingle::GameOver => "jingle_game_over",
            Jingle::SummonLesser => "jingle_summon_lesser",
            Jingle::SummonGreater => "jingle_summon_greater",
            Jingle::SummonAbyssal => "jingle_summon_abyssal",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Jingle::Victory => include_str!("../../assets/music/jingle_victory.vtrk"),
            Jingle::GameOver => include_str!("../../assets/music/jingle_game_over.vtrk"),
            Jingle::SummonLesser => include_str!("../../assets/music/jingle_summon_lesser.vtrk"),
            Jingle::SummonGreater => include_str!("../../assets/music/jingle_summon_greater.vtrk"),
            Jingle::SummonAbyssal => include_str!("../../assets/music/jingle_summon_abyssal.vtrk"),
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
    level(&mut out, rms, TARGET_RMS);
    Ok(out)
}

/// `jingle` once with its tail, its loudest 50 ms levelled to `JINGLE_LEVEL`.
pub fn render_jingle(jingle: Jingle) -> Result<Vec<f32>, String> {
    let mut out = jingle.song()?.render_once(SAMPLE_RATE, JINGLE_TAIL);
    let loudest = loudest_rms(&out, SAMPLE_RATE as usize / 20);
    level(&mut out, loudest, JINGLE_LEVEL);
    Ok(out)
}

/// Scales `out` so `measured` becomes `target`, without a peak over `MAX_PEAK`.
fn level(out: &mut [f32], measured: f32, target: f32) {
    let peak = out.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    if measured > 0.0 {
        let gain = (target / measured).min(MAX_PEAK / peak);
        out.iter_mut().for_each(|x| *x *= gain);
    }
}

/// The highest RMS over consecutive `window`-sample blocks.
fn loudest_rms(samples: &[f32], window: usize) -> f32 {
    samples
        .chunks(window)
        .map(|c| (c.iter().map(|x| x * x).sum::<f32>() / c.len() as f32).sqrt())
        .fold(0.0, f32::max)
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

    #[test]
    fn every_jingle_is_short_levelled_and_ends_silent() {
        for jingle in Jingle::ALL {
            let out = render_jingle(jingle).unwrap();
            let secs = out.len() as f32 / SAMPLE_RATE as f32;
            assert!((0.5..12.0).contains(&secs), "{jingle:?}: {secs} s");
            assert!(out.iter().all(|x| x.abs() <= MAX_PEAK + 1e-4));
            let loudest = loudest_rms(&out, SAMPLE_RATE as usize / 20);
            assert!(
                (loudest - JINGLE_LEVEL).abs() < 0.01,
                "{jingle:?}: {loudest}"
            );
            assert!(out[out.len() - 100..].iter().all(|x| x.abs() < 0.01));
        }
    }
}
