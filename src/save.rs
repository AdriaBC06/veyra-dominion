//! Progress (`save.txt`) and settings (`settings.txt`) as `key=value` text files.
//! Unknown keys and bad values are ignored (the default is kept), so old and newer
//! files both load. Writes go to a temporary file that is renamed over the old one.

use crate::config::{ARTS, UNIONS, WINDOW_H, WINDOW_W};
use crate::i18n::Lang;
use crate::screen::ScaleMode;
use crate::upgrades::Upgrades;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

/// Written to every file and bumped when a key changes meaning; v1 needs no
/// migration, so `from_text` ignores it for now.
pub const SAVE_VERSION: u32 = 1;
pub const SETTINGS_VERSION: u32 = 1;
const DIR_NAME: &str = "veyra-dominion";
const MIN_WINDOW: u32 = 320;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub lang: Lang,
    pub scale_mode: ScaleMode,
    pub fullscreen: bool,
    /// Windowed size, restored at start and when leaving fullscreen.
    pub window: (u32, u32),
    /// Volumes in 0..1.
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
    /// Holding the left mouse button moves Veyra toward the cursor.
    pub mouse_move: bool,
    pub blood: BloodMode,
}

/// How much blood the game shows: UI blood, stained titles, gore and the blood on the ground.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BloodMode {
    #[default]
    Full,
    Reduced,
    Off,
}

impl BloodMode {
    pub const ALL: [BloodMode; 3] = [BloodMode::Full, BloodMode::Reduced, BloodMode::Off];

    /// Share of the blood that is shown, 0..1.
    pub fn factor(self) -> f32 {
        match self {
            BloodMode::Full => 1.0,
            BloodMode::Reduced => crate::config::REDUCED_BLOOD,
            BloodMode::Off => 0.0,
        }
    }

    /// Code used in the settings file.
    fn code(self) -> &'static str {
        match self {
            BloodMode::Full => "full",
            BloodMode::Reduced => "reduced",
            BloodMode::Off => "off",
        }
    }

    fn from_code(code: &str) -> Option<BloodMode> {
        Self::ALL.into_iter().find(|m| m.code() == code)
    }

    /// The next mode, wrapping (`step` -1 or +1).
    pub fn step(self, step: i32) -> BloodMode {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0) as i32;
        Self::ALL[(i + step).rem_euclid(Self::ALL.len() as i32) as usize]
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lang: Lang::En,
            scale_mode: ScaleMode::Fill,
            fullscreen: false,
            window: (WINDOW_W as u32, WINDOW_H as u32),
            master: 1.0,
            music: 0.8,
            sfx: 0.8,
            mouse_move: false,
            blood: BloodMode::Full,
        }
    }
}

/// Lifetime progress. Meta-progression fields (Tribute, Pacts, Familiars) join in V20+.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveData {
    pub runs: u32,
    pub total_kills: u64,
    /// Longest run in seconds.
    pub best_time: f32,
    /// Grimoire entries fused at least once (`evolved=`), in `Entry` order.
    pub evolved: Vec<Entry>,
    /// Fused entries already shown in the Grimoire (`seen=`); the rest are marked new.
    pub seen: Vec<Entry>,
}

/// A Grimoire entry: Art `i`'s evolution or union `u`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Entry {
    Evolution(usize),
    Union(usize),
}

impl Entry {
    /// Every entry, evolutions in `ARTS` order then unions.
    pub fn all() -> impl Iterator<Item = Entry> {
        (0..ARTS.len())
            .filter(|&i| ARTS[i].evolution.is_some())
            .map(Entry::Evolution)
            .chain((0..UNIONS.len()).map(Entry::Union))
    }

    /// Fused in the build `upgrades`.
    pub fn fused(self, upgrades: &Upgrades) -> bool {
        match self {
            Entry::Evolution(i) => upgrades.evolved[i],
            Entry::Union(u) => upgrades.united[u],
        }
    }

    /// Its id in the save file: the Art's kind, or both kinds joined by `+` for a union,
    /// so reordering `ARTS` keeps old saves valid.
    fn id(self) -> String {
        let kind = |i: usize| format!("{:?}", ARTS[i].kind);
        match self {
            Entry::Evolution(i) => kind(i),
            Entry::Union(u) => {
                let [a, b] = UNIONS[u].arts;
                format!("{}+{}", kind(a), kind(b))
            }
        }
    }
}

/// Comma-separated entry ids, in `Entry` order.
fn entry_list(entries: &[Entry]) -> String {
    entries.iter().map(|e| e.id()).collect::<Vec<_>>().join(",")
}

/// Known ids from a comma-separated list, sorted and without repeats.
fn parse_entries(value: &str) -> Vec<Entry> {
    let mut out: Vec<Entry> = value
        .split(',')
        .filter_map(|id| Entry::all().find(|e| e.id() == id.trim()))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Adds `entry` to the sorted `list`; false if it was there already.
fn insert(list: &mut Vec<Entry>, entry: Entry) -> bool {
    match list.binary_search(&entry) {
        Ok(_) => false,
        Err(at) => {
            list.insert(at, entry);
            true
        }
    }
}

impl SaveData {
    pub fn record_run(&mut self, time: f32, kills: u32) {
        self.runs += 1;
        self.total_kills += kills as u64;
        self.best_time = self.best_time.max(time);
    }

    pub fn has_evolved(&self, entry: Entry) -> bool {
        self.evolved.binary_search(&entry).is_ok()
    }

    /// Fused but not yet shown in the Grimoire.
    pub fn is_new(&self, entry: Entry) -> bool {
        self.has_evolved(entry) && self.seen.binary_search(&entry).is_err()
    }

    /// Unlocks every entry fused in `upgrades`; true if any is new (the save needs writing).
    pub fn record_fusions(&mut self, upgrades: &Upgrades) -> bool {
        let mut changed = false;
        for entry in Entry::all().filter(|e| e.fused(upgrades)) {
            changed |= insert(&mut self.evolved, entry);
        }
        changed
    }

    /// Marks every unlocked entry as seen; true if any was new.
    pub fn mark_seen(&mut self) -> bool {
        let before = self.seen.len();
        self.seen = self.evolved.clone();
        self.seen.len() != before
    }
}

/// `key=value` pairs, skipping blank lines, `#` comments and lines without `=`.
fn pairs(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
}

fn set<T: FromStr>(field: &mut T, value: &str) {
    if let Ok(v) = value.parse() {
        *field = v;
    }
}

fn set_volume(field: &mut f32, value: &str) {
    if let Ok(v) = value.parse::<f32>() {
        if v.is_finite() {
            *field = v.clamp(0.0, 1.0);
        }
    }
}

fn scale_mode_code(mode: ScaleMode) -> &'static str {
    match mode {
        ScaleMode::PixelPerfect => "pixel",
        ScaleMode::Fill => "fill",
    }
}

impl Settings {
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "version={SETTINGS_VERSION}");
        let _ = writeln!(s, "lang={}", self.lang.code());
        let _ = writeln!(s, "scale_mode={}", scale_mode_code(self.scale_mode));
        let _ = writeln!(s, "fullscreen={}", self.fullscreen);
        let _ = writeln!(s, "window_w={}", self.window.0);
        let _ = writeln!(s, "window_h={}", self.window.1);
        let _ = writeln!(s, "master_volume={}", self.master);
        let _ = writeln!(s, "music_volume={}", self.music);
        let _ = writeln!(s, "sfx_volume={}", self.sfx);
        let _ = writeln!(s, "mouse_move={}", self.mouse_move);
        let _ = writeln!(s, "blood={}", self.blood.code());
        s
    }

    pub fn from_text(text: &str) -> Self {
        let mut s = Self::default();
        for (key, value) in pairs(text) {
            match key {
                "lang" => s.lang = Lang::from_code(value).unwrap_or(s.lang),
                "scale_mode" => {
                    s.scale_mode = match value {
                        "pixel" => ScaleMode::PixelPerfect,
                        "fill" => ScaleMode::Fill,
                        _ => s.scale_mode,
                    }
                }
                "fullscreen" => set(&mut s.fullscreen, value),
                "window_w" => set(&mut s.window.0, value),
                "window_h" => set(&mut s.window.1, value),
                "master_volume" => set_volume(&mut s.master, value),
                "music_volume" => set_volume(&mut s.music, value),
                "sfx_volume" => set_volume(&mut s.sfx, value),
                "mouse_move" => set(&mut s.mouse_move, value),
                "blood" => s.blood = BloodMode::from_code(value).unwrap_or(s.blood),
                _ => {}
            }
        }
        s.window = (s.window.0.max(MIN_WINDOW), s.window.1.max(MIN_WINDOW));
        s
    }

    pub fn load() -> Self {
        read(SETTINGS_FILE).map_or_else(Self::default, |t| Self::from_text(&t))
    }

    /// False on the very first launch, before any settings were written (the first-time
    /// setup shows then).
    pub fn exist() -> bool {
        data_dir().is_some_and(|d| d.join(SETTINGS_FILE).exists())
    }

    pub fn store(&self) {
        write(SETTINGS_FILE, &self.to_text());
    }
}

impl SaveData {
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "version={SAVE_VERSION}");
        let _ = writeln!(s, "runs={}", self.runs);
        let _ = writeln!(s, "total_kills={}", self.total_kills);
        let _ = writeln!(s, "best_time={}", self.best_time);
        let _ = writeln!(s, "evolved={}", entry_list(&self.evolved));
        let _ = writeln!(s, "seen={}", entry_list(&self.seen));
        s
    }

    pub fn from_text(text: &str) -> Self {
        let mut s = Self::default();
        for (key, value) in pairs(text) {
            match key {
                "runs" => set(&mut s.runs, value),
                "total_kills" => set(&mut s.total_kills, value),
                "best_time" => set(&mut s.best_time, value),
                "evolved" => s.evolved = parse_entries(value),
                "seen" => s.seen = parse_entries(value),
                _ => {}
            }
        }
        if !s.best_time.is_finite() || s.best_time < 0.0 {
            s.best_time = 0.0;
        }
        // only fused entries can have been seen
        s.seen.retain(|e| s.evolved.binary_search(e).is_ok());
        s
    }

    pub fn load() -> Self {
        read(SAVE_FILE).map_or_else(Self::default, |t| Self::from_text(&t))
    }

    pub fn store(&self) {
        write(SAVE_FILE, &self.to_text());
    }
}

const SAVE_FILE: &str = "save.txt";
const SETTINGS_FILE: &str = "settings.txt";

/// Per-user data folder: `%APPDATA%` on Windows, `$XDG_DATA_HOME` (or
/// `~/.local/share`) elsewhere, `~/Library/Application Support` on macOS.
fn data_dir() -> Option<PathBuf> {
    let env = |k| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_DATA_HOME").or_else(|| env("HOME").map(|h| h.join(".local/share")))
    };
    base.map(|b| b.join(DIR_NAME))
}

fn read(file: &str) -> Option<String> {
    fs::read_to_string(data_dir()?.join(file)).ok()
}

/// Failing to save must never stop the game, so errors are only reported.
fn write(file: &str, text: &str) {
    let Some(dir) = data_dir() else { return };
    if let Err(e) = write_atomic(&dir.join(file), text) {
        eprintln!("could not write {file}: {e}");
    }
}

/// Writes `path.tmp`, syncs it and renames it over `path`, so a crash mid-write
/// leaves either the old file or the new one, never a truncated mix.
pub fn write_atomic(path: &Path, text: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    {
        use std::io::Write;
        let mut f = fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ALLURE, HELLFLARE, INFERNAL_COURT};

    #[test]
    fn settings_round_trip() {
        let s = Settings {
            lang: Lang::Es,
            scale_mode: ScaleMode::Fill,
            fullscreen: true,
            window: (1920, 1080),
            master: 0.5,
            music: 0.25,
            sfx: 0.0,
            mouse_move: true,
            blood: BloodMode::Reduced,
        };
        assert_eq!(Settings::from_text(&s.to_text()), s);
        let d = Settings::default();
        assert_eq!(Settings::from_text(&d.to_text()), d);
    }

    #[test]
    fn first_launch_defaults_and_blood_modes() {
        let d = Settings::default();
        assert_eq!(
            (d.scale_mode, d.fullscreen, d.blood),
            (ScaleMode::Fill, false, BloodMode::Full)
        );
        for m in BloodMode::ALL {
            let s = Settings {
                blood: m,
                ..Settings::default()
            };
            assert_eq!(Settings::from_text(&s.to_text()).blood, m);
        }
        // files from before the setting keep full blood
        assert_eq!(Settings::from_text("lang=es\n").blood, BloodMode::Full);
        assert_eq!(BloodMode::Full.factor(), 1.0);
        assert_eq!(BloodMode::Off.factor(), 0.0);
        let r = BloodMode::Reduced.factor();
        assert!(r > 0.0 && r < 1.0);
        assert_eq!(BloodMode::Full.step(-1), BloodMode::Off);
        assert_eq!(BloodMode::Off.step(1), BloodMode::Full);
    }

    #[test]
    fn save_round_trip_and_record() {
        let mut s = SaveData::default();
        s.record_run(123.5, 400);
        s.record_run(60.0, 100);
        assert_eq!((s.runs, s.total_kills, s.best_time), (2, 500, 123.5));
        assert_eq!(SaveData::from_text(&s.to_text()), s);
    }

    #[test]
    fn bad_lines_and_values_keep_defaults() {
        let text = "# comment\n\ngarbage\nlang=fr\nscale_mode=zoom\nfullscreen=maybe\n\
                    window_w=10\nblood=gory\nmaster_volume=7\nmusic_volume=NaN\nsfx_volume=-1\nunknown=1\n";
        let s = Settings::from_text(text);
        let d = Settings::default();
        assert_eq!(s.lang, d.lang);
        assert_eq!(s.scale_mode, d.scale_mode);
        assert_eq!(s.fullscreen, d.fullscreen);
        assert_eq!(s.blood, d.blood);
        assert_eq!(s.window, (MIN_WINDOW, d.window.1));
        assert_eq!((s.master, s.music, s.sfx), (1.0, d.music, 0.0));
        let save = SaveData::from_text("runs=-3\nbest_time=inf\ntotal_kills= 42 \n");
        assert_eq!((save.runs, save.total_kills, save.best_time), (0, 42, 0.0));
    }

    #[test]
    fn missing_or_newer_version_still_loads_known_keys() {
        assert_eq!(SaveData::from_text("runs=4\n").runs, 4);
        let newer = format!("version={}\nruns=9\nfuture_key=x\n", SAVE_VERSION + 1);
        assert_eq!(SaveData::from_text(&newer).runs, 9);
    }

    #[test]
    fn grimoire_round_trip_records_fusions_and_marks_seen() {
        let mut u = Upgrades::default();
        let mut s = SaveData::default();
        assert!(!s.record_fusions(&u), "nothing fused yet");
        u.evolve(HELLFLARE);
        u.united[INFERNAL_COURT] = true;
        assert!(s.record_fusions(&u));
        assert!(!s.record_fusions(&u), "already unlocked");
        let court = Entry::Union(INFERNAL_COURT);
        assert!(s.has_evolved(Entry::Evolution(HELLFLARE)) && s.is_new(court));
        assert!(!s.has_evolved(Entry::Evolution(ALLURE)));
        assert!(s.to_text().contains("evolved=Hellflare,Allure+Thralls\n"));
        assert_eq!(SaveData::from_text(&s.to_text()), s);
        assert!(s.mark_seen() && !s.mark_seen());
        assert!(!s.is_new(court) && s.has_evolved(court));
        assert_eq!(SaveData::from_text(&s.to_text()), s);
    }

    #[test]
    fn grimoire_lists_ignore_unknown_ids_and_unfused_seen() {
        let s = SaveData::from_text(
            "evolved= Allure+Thralls,Nope,Hellflare,Hellflare\nseen=Hellflare,Allure\n",
        );
        assert_eq!(
            s.evolved,
            [Entry::Evolution(HELLFLARE), Entry::Union(INFERNAL_COURT)]
        );
        assert_eq!(s.seen, [Entry::Evolution(HELLFLARE)]);
        // older saves have neither key
        let old = SaveData::from_text("runs=3\n");
        assert!(old.evolved.is_empty() && old.seen.is_empty());
        assert_eq!(SaveData::from_text("evolved=\n").evolved, []);
    }

    #[test]
    fn grimoire_ids_are_unique() {
        let ids: Vec<String> = Entry::all().map(Entry::id).collect();
        for (i, id) in ids.iter().enumerate() {
            assert!(!ids[..i].contains(id), "{id} repeats");
        }
    }

    #[test]
    fn atomic_write_replaces_and_leaves_no_tmp() {
        let dir = std::env::temp_dir().join(format!("veyra-save-test-{}", std::process::id()));
        let path = dir.join("nested/save.txt");
        write_atomic(&path, "runs=1\n").unwrap();
        write_atomic(&path, "runs=2\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "runs=2\n");
        assert!(!dir.join("nested/save.txt.tmp").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
