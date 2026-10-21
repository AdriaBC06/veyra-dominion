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
    SinWrath => "Wrath", "Ira";
    SinWrathDesc => "+10% damage to every Art", "+10% de daño a todas las Artes";
    SinBloodlust => "Bloodlust", "Sed de sangre";
    SinBloodlustDesc => "Every Art recharges 8% faster", "Todas las Artes recargan un 8% más rápido";
    SinLegion => "Legion", "Legión";
    SinLegionDesc => "+1 to every Art's amount", "+1 de cantidad a todas las Artes";
    SinNightWings => "Night Wings", "Alas nocturnas";
    SinNightWingsDesc => "+10% move speed", "+10% velocidad";
    SinGluttony => "Gluttony", "Gula";
    SinGluttonyDesc => "+15 max HP, regenerates 0.2 HP/s", "+15 PV máx., regenera 0,2 PV/s";
    SinLust => "Lust", "Lujuria";
    SinLustDesc => "+40% soul pickup radius", "+40% radio de atracción de almas";
    SinSloth => "Sloth", "Pereza";
    SinSlothDesc => "Every Art's effects last 10% longer", "Los efectos de las Artes duran un 10% más";
    SinDarkSigil => "Dark Sigil", "Sello oscuro";
    SinDarkSigilDesc => "+10% area to every Art", "+10% de área a todas las Artes";
    SinInfernalHide => "Infernal Hide", "Piel infernal";
    SinInfernalHideDesc => "+1 armour: blows hurt 1 less", "+1 de armadura: los golpes dañan 1 menos";
    SinEnvy => "Envy", "Envidia";
    SinEnvyDesc => "+10% XP from every soul", "+10% de XP de cada alma";
    SinPride => "Pride", "Soberbia";
    SinPrideDesc => "+10% more, faster, tougher foes", "+10% enemigos, más rápidos y duros";
    LevelUpNew => "New", "Nueva";
    ArtHellflare => "Hellflare", "Llamarada infernal";
    ArtHellflareDesc => "Fireballs at the nearest foe", "Bolas de fuego al enemigo más cercano";
    ArtTailLash => "Tail Lash", "Latigazo de cola";
    ArtTailLashDesc => "Lashes the side she faces", "Azota el lado al que mira";
    ArtBrimstoneHalo => "Brimstone Halo", "Halo de azufre";
    ArtBrimstoneHaloDesc => "A great flame circles around her", "Una gran llama gira a su alrededor";
    ArtAllure => "Allure", "Seducción";
    ArtAllureDesc => "Charms nearby foes to fight for her", "Hechiza a los cercanos para que luchen por ella";
    ArtHellgate => "Hellgate", "Puerta del infierno";
    ArtHellgateDesc => "Pools of hellfire under her foes", "Charcos de fuego infernal bajo sus enemigos";
    ArtChainOfTorment => "Chain of Torment", "Cadena de tormento";
    ArtChainOfTormentDesc => "A chain that leaps from foe to foe", "Una cadena que salta de enemigo en enemigo";
    ArtScentOfSin => "Scent of Sin", "Aroma del pecado";
    ArtScentOfSinDesc => "A haze that burns and weakens foes", "Una bruma que quema y debilita al enemigo";
    ArtMoreCharmed => "+{} charmed", "+{} hechizado";
    ArtMorePools => "+{} pool", "+{} charco";
    ArtMoreJumps => "+{} link", "+{} eslabón";
    ArtMoreFireballs =>"+{} fireball", "+{} bola de fuego";
    ArtMoreLashes => "+{} lash", "+{} latigazo";
    ArtMoreEmbers => "+{} flame", "+{} llama";
    ArtGainDamage => "+{} damage", "+{} de daño";
    ArtGainCooldown => "-{}% cooldown", "-{}% de recarga";
    ArtGainArea => "+{}% area", "+{}% de área";
    ArtGainSpeed => "+{}% speed", "+{}% de velocidad";
    ArtGainPierce => "+{} pierce", "+{} de perforación";
    ArtGainDuration => "+{}s duration", "+{} s de duración";
    EvoKissOfRuin => "Kiss of Ruin", "Beso de la ruina";
    EvoKissOfRuinDesc => "Fireballs become piercing hearts", "Su fuego se vuelve corazones que atraviesan";
    EvoBarbedScourge => "Barbed Scourge", "Azote de púas";
    EvoBarbedScourgeDesc => "Lashes leave foes bleeding", "Los latigazos dejan a los enemigos sangrando";
    EvoInfernalCrown => "Infernal Crown", "Corona infernal";
    EvoInfernalCrownDesc => "The flames never stop circling", "Las llamas no dejan de girar";
    EvoSirensDominion => "Siren's Dominion", "Dominio de la sirena";
    EvoSirensDominionDesc => "Charmed foes burst when the spell ends", "Los hechizados estallan al acabar el hechizo";
    EvoAbyssalRift => "Abyssal Rift", "Grieta abisal";
    EvoAbyssalRiftDesc => "The pools drag foes into the fire", "Los charcos arrastran a los enemigos al fuego";
    EvoTormentEternal => "Torment Eternal", "Tormento eterno";
    EvoTormentEternalDesc => "The chain bites every foe near its first", "La cadena muerde a todos los que rodean al primero";
    EvoDevouringMusk => "Devouring Musk", "Almizcle voraz";
    EvoDevouringMuskDesc => "The haze feeds her on every foe it burns", "La bruma la alimenta con cada enemigo que quema";
    OfferingTitle => "Abyssal Offering", "Ofrenda abisal";
    OfferingEvolved => "{} can evolve", "{} puede evolucionar";
    OfferingGift => "The Abyss offers a gift", "El Abismo ofrece un don";
    OfferingHeal => "The Abyss offers to restore her", "El Abismo ofrece restaurarla";
    OfferingHealDesc => "+{} HP", "+{} PV";
    OfferingAccept => "Accept", "Aceptar";
    OfferingSacrifice => "Sacrifice it", "Sacrificarla";
    OfferingSacrificeDesc => "Destroy it for +{} HP and {} XP", "Destruirla por +{} PV y {} de XP";
    OfferingHelp => "Click, 1/2 or arrows + Enter", "Clic, 1/2 o flechas + Enter";
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
