//! `.vtrk` tracker songs: plain-text instruments, patterns and an order list,
//! parsed into timed note events and rendered by the chip synth into one
//! seamless loop. `std` only, like the synth. The format:
//!
//! ```text
//! # comment (a # starting a word, so F#4 is still a note)
//! song Dominion
//! tempo 84 6                  # BPM, rows per beat
//! echo 0.36 0.3 0.25          # delay s, feedback, mix
//! inst lead                   # instrument block, FamiTracker-style macros:
//!   vol 8 11 13 | 12 / 8 4 0  #   | loop point, / release point
//!   arp | 0 3 7               # semitones;  pitch: 1/16 semitone;  duty: index
//!   vib 0.2 5 0.3             # depth st, rate Hz, delay s
//!   sweep -40                 # semitones per second
//!   wave 89ACDEEFFFEEDCA97653211000112356   # 32 hex samples
//! drum k kick C2              # noise shorthand: token, instrument, note
//! pattern hook 12             # name, rows; one line per channel:
//!   p1 lead | D5 . . . Eb5:a . D5 . . . . = |
//!   noi     | k . . h . . k . . h . . |
//! order                       # one frame per line, patterns layered,
//!   hook bass+5 arp@p2-12%pad # +n/-n transposes, @ch moves a 1-line pattern,
//!                             # %inst swaps its notes' instrument (not drums)
//!   2x hook bass              # Nx repeats the frame
//! ```
//!
//! Channels are `p1 p2 tri noi wav`. A step is `.` (nothing), `=` (note off:
//! release), `^` (cut), a note (`C4` = MIDI 60, `F#5`, `Bb3`) or a drum token,
//! with an optional `:v` hex velocity (default `f`). `|` only separates bars.
//! The whole order loops; noise is never transposed.

use super::synth::{Channel, Echo, Instrument, Macro, Synth, Vibrato};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    On { note: f32, inst: usize, vel: f32 },
    Off,
    Cut,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub row: u32,
    pub ch: Channel,
    pub action: Action,
}

#[derive(Debug)]
pub struct Song {
    pub name: String,
    pub bpm: f32,
    pub rows_per_beat: u32,
    pub echo: Echo,
    pub instruments: Vec<Instrument>,
    /// Sorted by row.
    pub events: Vec<Event>,
    /// Length of one loop.
    pub rows: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    Note { note: i32, drum: bool, inst: Option<usize>, vel: f32 },
    Off,
    Cut,
}

struct Pattern {
    name: String,
    rows: u32,
    lines: Vec<(Channel, Vec<Option<Step>>)>,
}

enum Block {
    None,
    Inst,
    Pattern,
    Order,
}

const KEYWORDS: [&str; 7] = ["song", "tempo", "echo", "inst", "drum", "pattern", "order"];

fn channel(name: &str) -> Option<Channel> {
    Some(match name {
        "p1" => Channel::Pulse1,
        "p2" => Channel::Pulse2,
        "tri" => Channel::Triangle,
        "noi" => Channel::Noise,
        "wav" => Channel::Wave,
        _ => return None,
    })
}

/// `C4` = 60; sharps `#`, flats `b`, octaves -1..=9.
fn parse_note(s: &str) -> Option<i32> {
    let mut chars = s.chars();
    let base = match chars.next()? {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest = chars.as_str();
    let (acc, oct) = match rest.as_bytes().first()? {
        b'#' => (1, &rest[1..]),
        b'b' => (-1, &rest[1..]),
        _ => (0, rest),
    };
    let oct: i32 = oct.parse().ok().filter(|o| (-1..=9).contains(o))?;
    Some(12 * (oct + 1) + base + acc)
}

fn parse_macro(tokens: &[&str]) -> Result<Macro, String> {
    let mut m = Macro::default();
    for &t in tokens {
        match t {
            "|" => m.loop_from = Some(m.steps.len()),
            "/" => m.release = Some(m.steps.len()),
            _ => m
                .steps
                .push(t.parse().map_err(|_| format!("bad macro value '{t}'"))?),
        }
    }
    if m.steps.is_empty() {
        return Err("empty macro".into());
    }
    Ok(m)
}

fn parse_f32(t: Option<&&str>) -> Result<f32, String> {
    let t = t.ok_or("missing number")?;
    t.parse()
        .ok()
        .filter(|v: &f32| v.is_finite())
        .ok_or_else(|| format!("bad number '{t}'"))
}

fn parse_wave(hex: &str) -> Result<[u8; 32], String> {
    let digits: Vec<u8> = hex
        .chars()
        .map(|c| c.to_digit(16).map(|d| d as u8))
        .collect::<Option<_>>()
        .ok_or("wave must be hex digits")?;
    digits
        .try_into()
        .map_err(|_| "wave needs exactly 32 hex digits".into())
}

struct Parser {
    song: Song,
    inst_names: Vec<String>,
    drums: Vec<(String, usize, i32)>,
    patterns: Vec<Pattern>,
    block: Block,
}

impl Parser {
    fn inst(&self, name: &str) -> Result<usize, String> {
        self.inst_names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| format!("unknown instrument '{name}'"))
    }

    fn line(&mut self, t: &[&str]) -> Result<(), String> {
        let rest = &t[1..];
        if KEYWORDS.contains(&t[0]) {
            self.block = Block::None;
        }
        match t[0] {
            "song" => self.song.name = rest.join(" "),
            "tempo" => {
                self.song.bpm = parse_f32(rest.first())?;
                self.song.rows_per_beat = rest
                    .get(1)
                    .and_then(|r| r.parse().ok())
                    .filter(|&r| r > 0)
                    .ok_or("tempo needs BPM and rows per beat")?;
                if self.song.bpm <= 0.0 {
                    return Err("BPM must be positive".into());
                }
            }
            "echo" => {
                self.song.echo = Echo {
                    delay: parse_f32(rest.first())?,
                    feedback: parse_f32(rest.get(1))?,
                    mix: parse_f32(rest.get(2))?,
                }
            }
            "inst" => {
                let name = self.new_name(rest)?;
                if self.inst_names.contains(&name) {
                    return Err(format!("instrument '{name}' defined twice"));
                }
                self.inst_names.push(name);
                self.song.instruments.push(Instrument::default());
                self.block = Block::Inst;
            }
            "drum" => {
                let [token, inst, note] = rest else {
                    return Err("drum needs a token, an instrument and a note".into());
                };
                if !token.chars().all(|c| c.is_ascii_lowercase()) {
                    return Err("drum tokens are lowercase letters".into());
                }
                let note = parse_note(note).ok_or_else(|| format!("bad note '{note}'"))?;
                self.drums.push((token.to_string(), self.inst(inst)?, note));
            }
            "pattern" => {
                let name = self.new_name(&rest[..rest.len().min(1)])?;
                if self.patterns.iter().any(|p| p.name == name) {
                    return Err(format!("pattern '{name}' defined twice"));
                }
                let rows = rest
                    .get(1)
                    .and_then(|r| r.parse().ok())
                    .filter(|&r| r > 0)
                    .ok_or("pattern needs a name and a row count")?;
                self.patterns.push(Pattern {
                    name,
                    rows,
                    lines: Vec::new(),
                });
                self.block = Block::Pattern;
            }
            "order" => self.block = Block::Order,
            _ => match self.block {
                Block::Inst => self.inst_line(t)?,
                Block::Pattern => self.pattern_line(t)?,
                Block::Order => self.frame(t)?,
                Block::None => return Err(format!("unexpected '{}'", t[0])),
            },
        }
        Ok(())
    }

    fn new_name(&self, rest: &[&str]) -> Result<String, String> {
        let [name] = rest else {
            return Err("expected one name".into());
        };
        let ok = name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !ok || KEYWORDS.contains(name) || channel(name).is_some() {
            return Err(format!("bad name '{name}'"));
        }
        Ok(name.to_string())
    }

    fn inst_line(&mut self, t: &[&str]) -> Result<(), String> {
        let inst = self.song.instruments.last_mut().expect("inside inst");
        let rest = &t[1..];
        match t[0] {
            "vol" => inst.volume = parse_macro(rest)?,
            "arp" => inst.arp = parse_macro(rest)?,
            "pitch" => inst.pitch = parse_macro(rest)?,
            "duty" => inst.duty = parse_macro(rest)?,
            "vib" => {
                inst.vibrato = Vibrato {
                    depth: parse_f32(rest.first())?,
                    rate_hz: parse_f32(rest.get(1))?,
                    delay: parse_f32(rest.get(2))?,
                }
            }
            "sweep" => inst.sweep = parse_f32(rest.first())?,
            "wave" => inst.wave = parse_wave(rest.first().ok_or("missing wave")?)?,
            other => return Err(format!("unknown instrument field '{other}'")),
        }
        Ok(())
    }

    fn step(&self, token: &str, inst: Option<usize>) -> Result<Option<Step>, String> {
        let (body, vel) = match token.split_once(':') {
            Some((b, v)) => {
                let v = u8::from_str_radix(v, 16)
                    .ok()
                    .filter(|&v| v <= 15)
                    .ok_or_else(|| format!("bad velocity in '{token}'"))?;
                (b, f32::from(v) / 15.0)
            }
            None => (token, 1.0),
        };
        Ok(Some(match body {
            "." => return Ok(None),
            "=" => Step::Off,
            "^" => Step::Cut,
            _ => {
                if let Some(note) = parse_note(body) {
                    if inst.is_none() {
                        return Err(format!("note '{body}' on a line without instrument"));
                    }
                    Step::Note {
                        note,
                        drum: false,
                        inst,
                        vel,
                    }
                } else if let Some(&(_, i, note)) = self.drums.iter().find(|d| d.0 == body) {
                    Step::Note {
                        note,
                        drum: true,
                        inst: Some(i),
                        vel,
                    }
                } else {
                    return Err(format!("bad step '{token}'"));
                }
            }
        }))
    }

    fn pattern_line(&mut self, t: &[&str]) -> Result<(), String> {
        let bar = t.iter().position(|&s| s == "|").ok_or("pattern line needs '|'")?;
        let ch = channel(t[0]).ok_or_else(|| format!("unknown channel '{}'", t[0]))?;
        let inst = match &t[1..bar] {
            [] => None,
            [name] => Some(self.inst(name)?),
            _ => return Err("expected 'channel [instrument] | steps'".into()),
        };
        let steps = t[bar..]
            .iter()
            .filter(|&&s| s != "|")
            .map(|s| self.step(s, inst))
            .collect::<Result<Vec<_>, _>>()?;
        let p = self.patterns.last_mut().expect("inside pattern");
        if steps.len() != p.rows as usize {
            return Err(format!("{} steps, pattern has {}", steps.len(), p.rows));
        }
        if p.lines.iter().any(|l| l.0 == ch) {
            return Err(format!("channel '{}' twice in a pattern", t[0]));
        }
        p.lines.push((ch, steps));
        Ok(())
    }

    fn frame(&mut self, t: &[&str]) -> Result<(), String> {
        let (repeat, items) = match t[0].strip_suffix('x').and_then(|n| n.parse().ok()) {
            Some(n) if n > 0 => (n, &t[1..]),
            _ => (1, t),
        };
        // Each item: pattern index, transpose, channel and instrument override.
        let mut layers = Vec::new();
        for item in items {
            let end = item
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(item.len());
            let name = &item[..end];
            let p = self
                .patterns
                .iter()
                .position(|p| p.name == name)
                .ok_or_else(|| format!("unknown pattern '{name}'"))?;
            let (mut transpose, mut to, mut swap) = (0, None, None);
            let mut mods = &item[end..];
            while let Some(c) = mods.chars().next() {
                let len = mods[1..]
                    .find(['+', '-', '@', '%'])
                    .map_or(mods.len(), |i| i + 1);
                let arg = &mods[1..len];
                match c {
                    '+' | '-' => {
                        let n: i32 = arg
                            .parse()
                            .map_err(|_| format!("bad transpose in '{item}'"))?;
                        transpose += if c == '-' { -n } else { n };
                    }
                    '@' => {
                        to = Some(channel(arg).ok_or_else(|| format!("unknown channel '{arg}'"))?);
                        if self.patterns[p].lines.len() != 1 {
                            return Err(format!("'{item}': @ needs a one-line pattern"));
                        }
                    }
                    '%' => swap = Some(self.inst(arg)?),
                    _ => return Err(format!("bad modifier in '{item}'")),
                }
                mods = &mods[len..];
            }
            layers.push((p, transpose, to, swap));
        }
        if layers.is_empty() {
            return Err("empty frame".into());
        }
        let rows = layers.iter().map(|l| self.patterns[l.0].rows).max().unwrap_or(0);
        let mut used = Vec::new();
        for _ in 0..repeat {
            used.clear();
            for &(p, transpose, to, swap) in &layers {
                for (ch, steps) in &self.patterns[p].lines {
                    let ch = to.unwrap_or(*ch);
                    if used.contains(&ch) {
                        return Err("two patterns use the same channel in a frame".into());
                    }
                    used.push(ch);
                    for (row, step) in steps.iter().enumerate() {
                        let Some(step) = step else { continue };
                        let action = match *step {
                            Step::Note {
                                note,
                                drum,
                                inst,
                                vel,
                            } => {
                                let shift = if drum || ch == Channel::Noise {
                                    0
                                } else {
                                    transpose
                                };
                                let inst = inst.expect("checked when parsed");
                                Action::On {
                                    note: (note + shift) as f32,
                                    inst: if drum { inst } else { swap.unwrap_or(inst) },
                                    vel,
                                }
                            }
                            Step::Off => Action::Off,
                            Step::Cut => Action::Cut,
                        };
                        self.song.events.push(Event {
                            row: self.song.rows + row as u32,
                            ch,
                            action,
                        });
                    }
                }
            }
            self.song.rows += rows;
        }
        Ok(())
    }
}

impl Song {
    pub fn parse(src: &str) -> Result<Song, String> {
        let mut p = Parser {
            song: Song {
                name: String::new(),
                bpm: 120.0,
                rows_per_beat: 4,
                echo: Echo::OFF,
                instruments: Vec::new(),
                events: Vec::new(),
                rows: 0,
            },
            inst_names: Vec::new(),
            drums: Vec::new(),
            patterns: Vec::new(),
            block: Block::None,
        };
        for (i, line) in src.lines().enumerate() {
            let tokens: Vec<&str> = line
                .split_whitespace()
                .take_while(|t| !t.starts_with('#'))
                .collect();
            if !tokens.is_empty() {
                p.line(&tokens).map_err(|e| format!("line {}: {e}", i + 1))?;
            }
        }
        if p.song.rows == 0 {
            return Err("the order is empty".into());
        }
        p.song.events.sort_by_key(|e| e.row);
        Ok(p.song)
    }

    pub fn samples_per_row(&self, sample_rate: u32) -> f64 {
        f64::from(sample_rate) * 60.0 / (f64::from(self.bpm) * f64::from(self.rows_per_beat))
    }

    pub fn loop_samples(&self, sample_rate: u32) -> usize {
        (f64::from(self.rows) * self.samples_per_row(sample_rate)).round() as usize
    }

    /// One loop, ready to repeat without a seam: the song is rendered twice and
    /// the second pass is kept, so its start already holds the echo and the
    /// ringing notes from the end.
    pub fn render_loop(&self, sample_rate: u32) -> Vec<f32> {
        let len = self.loop_samples(sample_rate);
        let spr = self.samples_per_row(sample_rate);
        let mut synth = Synth::new(sample_rate, self.instruments.clone());
        synth.set_echo(self.echo);
        let mut out = vec![0.0; len * 2];
        let mut pos = 0;
        for pass in 0..2 {
            for e in &self.events {
                let at = (pass * len + (f64::from(e.row) * spr).round() as usize).max(pos);
                synth.render(&mut out[pos..at]);
                pos = at;
                match e.action {
                    Action::On { note, inst, vel } => synth.note_on(e.ch, note, inst, vel),
                    Action::Off => synth.note_off(e.ch),
                    Action::Cut => synth.note_cut(e.ch),
                }
            }
        }
        synth.render(&mut out[pos..]);
        out.split_off(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "tempo 120 4\ninst a\n  vol 15 / 0\ndrum k a C2\n";

    fn song(body: &str) -> Result<Song, String> {
        Song::parse(&format!("{HEAD}{body}"))
    }

    fn notes(s: &Song) -> Vec<(u32, Channel, f32)> {
        s.events
            .iter()
            .filter_map(|e| match e.action {
                Action::On { note, .. } => Some((e.row, e.ch, note)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn note_names() {
        assert_eq!(parse_note("C4"), Some(60));
        assert_eq!(parse_note("A4"), Some(69));
        assert_eq!(parse_note("F#5"), Some(78));
        assert_eq!(parse_note("Bb3"), Some(58));
        assert_eq!(parse_note("C-1"), Some(0));
        for bad in ["H4", "C", "c4", "C#", "E10", "Cx4"] {
            assert_eq!(parse_note(bad), None, "{bad}");
        }
    }

    #[test]
    fn macro_markers() {
        let m = parse_macro(&["8", "11", "|", "13", "/", "6", "0"]).unwrap();
        assert_eq!(m.steps, [8, 11, 13, 6, 0]);
        assert_eq!((m.loop_from, m.release), (Some(2), Some(3)));
        assert!(parse_macro(&["|"]).is_err());
        assert!(parse_macro(&["300"]).is_err());
    }

    #[test]
    fn instrument_fields() {
        let s = song(
            "inst b\n  duty 1\n  vib 0.2 5 0.3\n  sweep -12\n  wave 0123456789ABCDEFFEDCBA9876543210\n\
             pattern p 1\n  p1 b | C4 |\norder\n  p\n",
        )
        .unwrap();
        let b = &s.instruments[1];
        assert_eq!(b.duty.steps, [1]);
        assert_eq!(b.vibrato.rate_hz, 5.0);
        assert_eq!(b.sweep, -12.0);
        assert_eq!(b.wave[10], 10);
        assert_eq!(s.instruments[0].volume.release, Some(1));
    }

    #[test]
    fn steps_velocity_and_drums() {
        let s = song(
            "pattern p 4 # four rows\n  p1 a | C4:8 = . ^ |\n  noi | k . k:0 . |\norder\n  p\n",
        )
        .unwrap();
        assert_eq!(s.rows, 4);
        assert_eq!(
            s.events[0].action,
            Action::On {
                note: 60.0,
                inst: 0,
                vel: 8.0 / 15.0
            }
        );
        let kinds: Vec<_> = s.events.iter().map(|e| (e.row, e.ch)).collect();
        assert!(kinds.contains(&(1, Channel::Pulse1)) && kinds.contains(&(3, Channel::Pulse1)));
        assert_eq!(s.events.iter().filter(|e| e.ch == Channel::Noise).count(), 2);
    }

    #[test]
    fn order_layers_transposes_moves_and_repeats() {
        let s = song(
            "pattern m 2\n  p1 a | C4 . |\npattern d 4\n  noi | k . . . |\n\
             order\n  m+2 d\n  2x m@tri-12\n",
        )
        .unwrap();
        // The first frame lasts as long as its longest layer.
        assert_eq!(s.rows, 8);
        assert_eq!(
            notes(&s),
            [
                (0, Channel::Pulse1, 62.0),
                (0, Channel::Noise, 36.0),
                (4, Channel::Triangle, 48.0),
                (6, Channel::Triangle, 48.0),
            ]
        );
        // Noise is never transposed.
        let s = song("pattern d 1\n  noi a | C#4 |\norder\n  d+5\n").unwrap();
        assert_eq!(notes(&s)[0].2, 61.0);
        // % swaps the instrument of notes but not of drums.
        let s = song(
            "inst b\n  vol 9\npattern m 1\n  p1 a | C4 |\n  noi | k |\norder\n  m%b\n",
        )
        .unwrap();
        let insts: Vec<_> = s
            .events
            .iter()
            .map(|e| match e.action {
                Action::On { inst, .. } => (e.ch, inst),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(insts, [(Channel::Pulse1, 1), (Channel::Noise, 0)]);
    }

    #[test]
    fn errors_name_the_line() {
        let bad = [
            ("pattern p 3\n  p1 a | C4 . |\n", "line 6: 2 steps"),
            ("pattern p 1\n  p1 x | C4 |\n", "unknown instrument 'x'"),
            ("pattern p 1\n  p1 | C4 |\n", "without instrument"),
            ("pattern p 1\n  p1 a | Q4 |\n", "bad step"),
            ("pattern p 1\n  zz a | C4 |\n", "unknown channel"),
            ("order\n  nope\n", "unknown pattern"),
            (
                "pattern p 1\n  p1 a | C4 |\norder\n  p p\n",
                "same channel",
            ),
            ("pattern p 1\n  p1 a | C4 |\n", "order is empty"),
            ("bogus 1\n", "unexpected 'bogus'"),
            ("inst song\n", "bad name"),
        ];
        for (body, msg) in bad {
            let e = song(body).unwrap_err();
            assert!(e.contains(msg), "{body:?}: {e}");
        }
    }

    #[test]
    fn loop_length_follows_tempo() {
        let s = song("pattern p 8\n  p1 a | C4 . . . . . . . |\norder\n  p\n").unwrap();
        // 8 sixteenths at 120 BPM = 1 s.
        assert_eq!(s.loop_samples(44_100), 44_100);
        let out = s.render_loop(44_100);
        assert_eq!(out.len(), 44_100);
        assert_eq!(out, s.render_loop(44_100));
    }

    #[test]
    fn loop_start_carries_the_tail_of_the_end() {
        // A note only on the last row: the loop's start must already hold that
        // note and its echo, as it will when the loop repeats.
        let s = song(
            "echo 0.1 0.5 0.5\npattern p 8\n  p1 a | . . . . . . . C4 |\norder\n  p\n",
        )
        .unwrap();
        let out = s.render_loop(44_100);
        let start = &out[..2_000];
        assert!(start.iter().any(|x| x.abs() > 0.01));
    }
}
