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

    /// The language's own name, shown in the settings whatever the current language.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Es => "Español",
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
    Paused => "Paused", "Pausa";
    VictoryTitle => "The village burns", "La aldea arde";
    VictoryStats => "Kills {}   Level {}", "Bajas {}   Nivel {}";
    GameOverTitle => "Banished", "Desterrada";
    GameOverStats => "Survived {}   Kills {}", "Resististe {}   Bajas {}";
    HudRight => "LV {}   KILLS {}", "NV {}   BAJAS {}";
    LevelUpTitle => "Dominion grows", "El dominio crece";
    LevelUpLevel => "Lv {} / {}", "Nv {} / {}";
    LevelUpHelp => "Click, 1/2/3 or arrows + Enter", "Clic, 1/2/3 o flechas + Enter";
    MenuPlay => "Play", "Jugar";
    MenuSettings => "Settings", "Ajustes";
    MenuCredits => "Credits", "Créditos";
    MenuQuit => "Quit", "Salir";
    MenuResume => "Resume", "Continuar";
    MenuTitle => "Title screen", "Menú principal";
    MenuRestart => "Restart", "Reiniciar";
    MenuEndless => "Endless", "Sin fin";
    MenuBack => "Back", "Volver";
    TabAudio => "Audio", "Audio";
    TabVideo => "Video", "Vídeo";
    TabGame => "Game", "Juego";
    TabControls => "Keys", "Teclas";
    TabData => "Data", "Datos";
    SetMaster => "Master volume", "Volumen general";
    SetMusic => "Music", "Música";
    SetSfx => "Effects", "Efectos";
    SetFullscreen => "Fullscreen", "Pantalla completa";
    SetScale => "Scaling", "Escalado";
    ScalePixel => "Pixel-perfect", "Píxel exacto";
    ScaleFill => "Fill", "Llenar";
    SetLanguage => "Language", "Idioma";
    SetMouseMove => "Hold click to move", "Mantén clic para moverte";
    CtlMove => "Move", "Moverse";
    CtlMoveKeys => "WASD / arrows", "WASD / flechas";
    CtlMouse => "Move (mouse)", "Moverse (ratón)";
    CtlMouseKeys => "Hold left click", "Mantén clic izquierdo";
    CtlPause => "Pause", "Pausa";
    CtlPick => "Pick upgrade", "Elegir mejora";
    CtlPickKeys => "1/2/3, click, Enter", "1/2/3, clic, Enter";
    CtlConfirm => "Confirm / back", "Aceptar / volver";
    DataRuns => "Runs: {}", "Partidas: {}";
    DataKills => "Total kills: {}", "Bajas totales: {}";
    DataBest => "Best time: {}", "Mejor tiempo: {}";
    ResetProgress => "Reset progress", "Borrar progreso";
    ResetConfirm => "Sure? Press again", "¿Seguro? Pulsa otra vez";
    ResetDone => "Progress erased", "Progreso borrado";
    CreditsDesign => "Design & code", "Diseño y código";
    CreditsArt => "Art", "Arte";
    CreditsMusic => "Music & sound", "Música y sonido";
    CreditsSynth => "Made with our own chip synth", "Hechos con un sintetizador chip propio";
    CreditsFonts => "Fonts", "Tipografías";
    CreditsEngine => "Engine", "Motor";
    CreditsThanks => "Special thanks", "Agradecimientos";
    CreditsThanksYou => "You, for playing", "A ti, por jugar";
    CreditsLicences => "Licences", "Licencias";
    CreditsLicenceFiles => "Full texts in the licences folder", "Textos completos en la carpeta de licencias";
    CreditsHelp => "Esc or click to return", "Esc o clic para volver";
    On => "On", "Sí";
    Off => "Off", "No";
    UpFury => "Fury", "Furia";
    UpFuryDesc => "+25% damage to every Art", "+25% de daño a todas las Artes";
    UpFrenzy => "Frenzy", "Frenesí";
    UpFrenzyDesc => "Every Art attacks 15% faster", "Todas las Artes atacan un 15% más rápido";
    UpEmberLegion => "Ember Legion", "Legión de ascuas";
    UpEmberLegionDesc => "+1 to every Art's amount", "+1 de cantidad a todas las Artes";
    UpLightWings => "Light Wings", "Alas ligeras";
    UpLightWingsDesc => "+10% move speed", "+10% velocidad";
    UpBloodPact => "Blood Pact", "Pacto de sangre";
    UpBloodPactDesc => "+20 max HP, heal 30", "+20 PV máx., cura 30";
    UpSoulPull => "Soul Pull", "Atracción de almas";
    UpSoulPullDesc => "+40% soul pickup radius", "+40% radio de atracción";
    LevelUpNew => "New", "Nueva";
    ArtHellflare => "Hellflare", "Llamarada infernal";
    ArtHellflareDesc => "Fireballs at the nearest foe", "Bolas de fuego al enemigo más cercano";
    ArtTailLash => "Tail Lash", "Latigazo de cola";
    ArtTailLashDesc => "Lashes the side she faces", "Azota el lado al que mira";
    ArtKissOfRuin => "Kiss of Ruin", "Beso de la ruina";
    ArtKissOfRuinDesc => "Piercing hearts where she flies", "Corazones que atraviesan, hacia donde vuela";
    ArtBrimstoneHalo => "Brimstone Halo", "Halo de azufre";
    ArtBrimstoneHaloDesc => "Embers circle around her", "Ascuas que giran a su alrededor";
    ArtMoreFireballs => "+{} fireball", "+{} bola de fuego";
    ArtMoreLashes => "+{} lash", "+{} latigazo";
    ArtMoreHearts => "+{} heart", "+{} corazón";
    ArtMoreEmbers => "+{} ember", "+{} ascua";
    ArtGainDamage => "+{} damage", "+{} de daño";
    ArtGainCooldown => "-{}% cooldown", "-{}% de recarga";
    ArtGainArea => "+{}% area", "+{}% de área";
    ArtGainSpeed => "+{}% speed", "+{}% de velocidad";
    ArtGainPierce => "+{} pierce", "+{} de perforación";
    ArtGainDuration => "+{}s duration", "+{} s de duración";
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
