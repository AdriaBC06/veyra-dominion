//! Grimoire of Evolutions: every evolution and union with its recipe (Art + Sin, or Art + Art,
//! then the result), name and description, drawn as Veyra's own tome (`tome.rs`): the entries
//! on a summoning circle on the left page, the focused one's rite on the right.
//! Entries never fused are black silhouettes with "???" and a hidden recipe; fused ones not yet
//! seen here wait to be revealed. Opened from a run, it also glows on the recipes the build can
//! complete. Leaving marks everything seen (the caller writes the save).

use super::Nav;
use crate::config::{ARTS, UNIONS};
use crate::hud::Icon;
use crate::i18n::Key;
use crate::save::{Entry, SaveData};
use crate::ui::{Action, Focus, Overlay, UiInput};
use crate::upgrades::Upgrades;
use macroquad::prelude::*;
use std::f32::consts::TAU;

mod tome;

/// Seconds a fresh entry takes to bloom from silhouette into colour.
const REVEAL: f32 = 1.1;
/// Radians per second the circle turns.
const SPIN: f32 = 0.035;
/// How quickly the ribbon slides out of the book when it is highlighted (1/s).
const RIBBON_RATE: f32 = 14.0;

pub struct GrimoireScene {
    entries: Vec<Entry>,
    focus: Focus,
    overlay: Overlay,
    from_run: bool,
    /// The entry on show (the last one focused or hovered), the one it replaced (it burns away
    /// first) and seconds since it changed.
    page: usize,
    prev: Option<usize>,
    page_t: f32,
    /// Entries fused but unseen when the Grimoire opened, and how far each one's reveal has
    /// played (0..1, it plays the first time its page is shown).
    fresh: Vec<bool>,
    reveal: Vec<f32>,
    /// Turn of the circle, and how far the ribbon is pulled out (0..1).
    spin: f32,
    ribbon: f32,
}

impl GrimoireScene {
    pub fn new(from_run: bool, save: &SaveData) -> Self {
        let entries: Vec<Entry> = Entry::all().collect();
        let fresh = entries.iter().map(|&e| save.is_new(e)).collect();
        Self {
            reveal: vec![0.0; entries.len()],
            fresh,
            entries,
            focus: Focus::default(),
            overlay: Overlay::default(),
            from_run,
            page: 0,
            prev: None,
            page_t: 0.0,
            spin: 0.0,
            ribbon: 0.0,
        }
    }

    /// Opened from the pause menu (the run stays behind it).
    pub fn in_run(&self) -> bool {
        self.from_run
    }

    pub fn update(&mut self, input: &UiInput, view: Vec2) -> Option<Nav> {
        let dt = get_frame_time();
        self.spin = (self.spin + SPIN * dt) % TAU;
        let n = self.entries.len();
        let items = tome::items(view, n, self.spin);
        // around the circle, then the ribbon: every arrow steps along that one ring
        let mut flat = *input;
        flat.up |= flat.left;
        flat.down |= flat.right;
        flat.left = false;
        flat.right = false;
        let action = self.focus.update(&flat, &items);
        self.overlay.update(input, &self.focus, &items, dt);
        let out = if self.focus.is_highlighted(n) {
            1.0
        } else {
            0.0
        };
        self.ribbon += (out - self.ribbon) * (1.0 - (-RIBBON_RATE * dt).exp());

        let shown = self.focus.highlight().unwrap_or(self.focus.index);
        if shown < n && shown != self.page {
            self.prev = Some(self.page);
            self.page = shown;
            self.page_t = 0.0;
        }
        self.page_t += dt;
        if self.fresh[self.page] {
            let r = &mut self.reveal[self.page];
            *r = (*r + dt / REVEAL).min(1.0);
        }
        if action == Action::Back || action == Action::Confirm(n) {
            return Some(if self.from_run {
                Nav::ResumeRun
            } else {
                Nav::Title
            });
        }
        None
    }

    pub fn draw(&self, view: Vec2, save: &SaveData, build: Option<&Upgrades>) {
        tome::draw(self, view, save, build);
    }
}

fn result(entry: Entry) -> Icon {
    match entry {
        Entry::Evolution(i) => Icon::Evolution(i),
        Entry::Union(u) => Icon::Union(u),
    }
}

/// Art + Sin for an evolution, both Arts for a union.
fn ingredients(entry: Entry) -> [Icon; 2] {
    match entry {
        Entry::Evolution(i) => {
            let sin = ARTS[i].evolution.as_ref().map_or(0, |e| e.sin);
            [Icon::Art(i), Icon::Sin(sin)]
        }
        Entry::Union(u) => UNIONS[u].arts.map(Icon::Art),
    }
}

fn texts(entry: Entry) -> (Key, Key) {
    match entry {
        Entry::Evolution(i) => {
            let evo = ARTS[i]
                .evolution
                .as_ref()
                .expect("entries only list evolving Arts");
            (evo.name, evo.desc)
        }
        Entry::Union(u) => (UNIONS[u].name, UNIONS[u].desc),
    }
}

/// The current build owns every ingredient and hasn't fused the entry yet.
fn ready(entry: Entry, build: &Upgrades) -> bool {
    !entry.fused(build)
        && match entry {
            Entry::Evolution(i) => {
                let sin = ARTS[i].evolution.as_ref().map_or(0, |e| e.sin);
                build.arts[i] > 0 && build.sins[sin] > 0
            }
            Entry::Union(u) => UNIONS[u].arts.iter().all(|&a| build.arts[a] > 0),
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ALLURE, HELLFLARE, INFERNAL_COURT, LEGION, THRALLS};

    #[test]
    fn every_evolution_and_union_has_an_entry() {
        let g = GrimoireScene::new(false, &SaveData::default());
        let evolutions = ARTS.iter().filter(|a| a.evolution.is_some()).count();
        assert_eq!(g.entries.len(), evolutions + UNIONS.len());
        assert!(g.entries.contains(&Entry::Union(INFERNAL_COURT)));
    }

    #[test]
    fn ready_needs_every_ingredient_and_no_fusion_yet() {
        let mut b = Upgrades::default();
        let kiss = Entry::Evolution(HELLFLARE);
        assert!(!ready(kiss, &b), "Hellflare alone");
        b.sins[LEGION] = 1;
        assert!(ready(kiss, &b));
        b.evolve(HELLFLARE);
        assert!(!ready(kiss, &b), "already fused");
        let court = Entry::Union(INFERNAL_COURT);
        b.arts[ALLURE] = 1;
        assert!(!ready(court, &b));
        b.arts[THRALLS] = 1;
        assert!(ready(court, &b));
    }
}
