//! English and Spanish UI strings. Every player-facing string goes through `tr`;
//! `{}` marks a slot filled in order by `trf`. The debug overlay stays English.

use std::fmt::{Display, Write};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Es,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Es];

    /// Code used in the settings file.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    pub fn next(self) -> Lang {
        match self {
            Lang::En => Lang::Es,
            Lang::Es => Lang::En,
        }
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn lang() -> Lang {
    Lang::ALL[LANG.load(Ordering::Relaxed) as usize]
}

pub fn set_lang(lang: Lang) {
    LANG.store(lang as u8, Ordering::Relaxed);
}

macro_rules! keys {
    ($($key:ident => $en:literal, $es:literal;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Key { $($key),* }

        #[cfg(test)]
        impl Key {
            pub const ALL: &'static [Key] = &[$(Key::$key),*];
        }

        const TABLE: &[[&str; 2]] = &[$([$en, $es]),*];
    };
}

keys! {
    TitleTagline => "Survive {}", "Sobrevive {}";
    TitlePrompt => "Enter to rise", "Enter para alzarte";
    Paused => "Paused", "Pausa";
    PausedHelp => "Esc resume   T title", "Esc continuar   T título";
    VictoryTitle => "The village burns", "La aldea arde";
    VictoryStats => "Kills {}   Level {}", "Bajas {}   Nivel {}";
    VictoryHelp => "E endless   R restart   T title", "E sin fin   R reiniciar   T título";
    GameOverTitle => "Banished", "Desterrada";
    GameOverStats => "Survived {}   Kills {}", "Resististe {}   Bajas {}";
    GameOverHelp => "R restart   T title", "R reiniciar   T título";
    HudRight => "LV {}   KILLS {}", "NV {}   BAJAS {}";
    LevelUpTitle => "Dominion grows", "El dominio crece";
    LevelUpLevel => "Lv {} / {}", "Nv {} / {}";
    LevelUpHelp => "Click, 1/2/3 or arrows + Enter", "Clic, 1/2/3 o flechas + Enter";
    On => "On", "Sí";
    Off => "Off", "No";
    UpFury => "Fury", "Furia";
    UpFuryDesc => "+25% fireball damage", "+25% daño de bola de fuego";
    UpFrenzy => "Frenzy", "Frenesí";
    UpFrenzyDesc => "+15% fire rate", "+15% cadencia de fuego";
    UpEmberLegion => "Ember Legion", "Legión de ascuas";
    UpEmberLegionDesc => "+1 fireball per volley", "+1 bola de fuego por ráfaga";
    UpLightWings => "Light Wings", "Alas ligeras";
    UpLightWingsDesc => "+10% move speed", "+10% velocidad";
    UpBloodPact => "Blood Pact", "Pacto de sangre";
    UpBloodPactDesc => "+20 max HP, heal 30", "+20 PV máx., cura 30";
    UpSoulPull => "Soul Pull", "Atracción de almas";
    UpSoulPullDesc => "+40% soul pickup radius", "+40% radio de atracción";
}

/// The string for `key` in the current language.
pub fn tr(key: Key) -> &'static str {
    TABLE[key as usize][lang() as usize]
}

/// `tr(key)` with each `{}` replaced by the next argument.
pub fn trf(key: Key, args: &[&dyn Display]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    for (i, part) in tr(key).split("{}").enumerate() {
        if i > 0 {
            if let Some(arg) = args.next() {
                let _ = write!(out, "{arg}");
            }
        }
        out.push_str(part);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::EXTRA_GLYPHS;

    #[test]
    fn every_string_is_present_and_slots_match() {
        assert_eq!(Key::ALL.len(), TABLE.len());
        for (key, row) in Key::ALL.iter().zip(TABLE) {
            for s in row {
                assert!(!s.trim().is_empty(), "{key:?} empty");
            }
            assert_eq!(
                row[0].matches("{}").count(),
                row[1].matches("{}").count(),
                "{key:?} slot count"
            );
        }
    }

    #[test]
    fn every_char_is_in_the_font_cache() {
        for row in TABLE {
            for c in row.iter().flat_map(|s| s.chars()) {
                assert!(
                    (' '..='~').contains(&c) || EXTRA_GLYPHS.contains(c),
                    "{c:?} is not pre-rasterised"
                );
            }
        }
    }

    #[test]
    fn switching_language_and_formatting() {
        set_lang(Lang::Es);
        assert_eq!(tr(Key::Paused), "Pausa");
        assert_eq!(trf(Key::LevelUpLevel, &[&2, &5]), "Nv 2 / 5");
        set_lang(Lang::En);
        assert_eq!(trf(Key::TitleTagline, &[&"5:00"]), "Survive 5:00");
        for lang in Lang::ALL {
            assert_eq!(Lang::from_code(lang.code()), Some(lang));
        }
    }
}
