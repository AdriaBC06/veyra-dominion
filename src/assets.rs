//! Pixel art lives in `assets/sprites/*.png` (one horizontal strip of equal frames per
//! sprite), embedded with `include_bytes!` and packed into a single nearest-filtered
//! atlas at startup. `sprite()` is the only accessor.
//!
//! Fonts (OFL, licences in `assets/fonts/`) are embedded too. Both are pixel fonts that
//! macroquad only rasterises crisp at specific sizes, so text goes through `TextStyle`,
//! which pins each style to one of those sizes.

use macroquad::prelude::*;
use std::cell::OnceCell;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteId {
    VeyraIdle,
    VeyraMove,
    VeyraCast,
    VeyraHurt,
    VeyraDeath,
    VeyraWings,
    VeyraTail,
    VeyraHair,
    VeyraHairMove,
    Rat,
    Villager,
    Knight,
    Brute,
    TailLash,
    HeartBolt,
    Ember,
    CharmHeart,
    GateFlame,
    ChainSpark,
    Offering,
    Bat,
    BatHost,
    BloodBomb,
    BlackMass,
    Sabbath,
    EvilEye,
    Scythe0,
    Scythe1,
    Scythe2,
    Scythe3,
    Scythe4,
    Scythe5,
    Cross0,
    Cross1,
    Cross2,
    Cross3,
    Cross4,
    Cross5,
    CrossBurning0,
    CrossBurning1,
    CrossBurning2,
    CrossBurning3,
    CrossBurning4,
    CrossBurning5,
    Sigil0,
    Sigil1,
    Sigil2,
    Sigil3,
    Sigil4,
    Sigil5,
    Fireball,
    Smoke,
    Soul,
    Ground,
}

const SPRITE_COUNT: usize = 54;

/// Sizes the area-grown Arts are baked at (`assets/arts.px.js`): tier k is 1 + k/4
/// times the base sprite.
pub const SIZE_TIERS: usize = 6;
pub const SCYTHE_TIERS: [SpriteId; SIZE_TIERS] = [
    SpriteId::Scythe0,
    SpriteId::Scythe1,
    SpriteId::Scythe2,
    SpriteId::Scythe3,
    SpriteId::Scythe4,
    SpriteId::Scythe5,
];
pub const CROSS_TIERS: [SpriteId; SIZE_TIERS] = [
    SpriteId::Cross0,
    SpriteId::Cross1,
    SpriteId::Cross2,
    SpriteId::Cross3,
    SpriteId::Cross4,
    SpriteId::Cross5,
];
pub const CROSS_BURNING_TIERS: [SpriteId; SIZE_TIERS] = [
    SpriteId::CrossBurning0,
    SpriteId::CrossBurning1,
    SpriteId::CrossBurning2,
    SpriteId::CrossBurning3,
    SpriteId::CrossBurning4,
    SpriteId::CrossBurning5,
];
pub const SIGIL_TIERS: [SpriteId; SIZE_TIERS] = [
    SpriteId::Sigil0,
    SpriteId::Sigil1,
    SpriteId::Sigil2,
    SpriteId::Sigil3,
    SpriteId::Sigil4,
    SpriteId::Sigil5,
];

/// The tier closest to `area` (clamped to the baked ones).
pub fn size_tier(area: f32) -> usize {
    (((area - 1.0) * 4.0).round().max(0.0) as usize).min(SIZE_TIERS - 1)
}

/// Enemy sprites have 2 animation frames followed by their white hit-flash copies.
pub const FLASH_FRAME_OFFSET: usize = 2;

pub const GROUND_TILE: u16 = 32;
pub const GROUND_VARIANTS: usize = 6;

const ATLAS_W: u16 = 1024;
const ATLAS_H: u16 = 1024;

/// Veyra's layers (`assets/veyra.px.js`) are 66x50 frames sharing one origin; the
/// pivot sits under her hips and she faces left.
pub const VEYRA_FRAME: Vec2 = vec2(66.0, 50.0);
pub const VEYRA_PIVOT: Vec2 = vec2(31.0, 36.0);

/// An embedded sprite strip: `frames` equal-width frames side by side.
struct Sheet {
    id: SpriteId,
    png: &'static [u8],
    frames: usize,
    /// Whether white hit-flash copies of every frame are generated.
    flash: bool,
}

macro_rules! sheet {
    ($id:ident, $file:literal, $frames:expr, $flash:expr) => {
        Sheet {
            id: SpriteId::$id,
            png: include_bytes!(concat!("../assets/sprites/", $file, ".png")),
            frames: $frames,
            flash: $flash,
        }
    };
}

/// One of Veyra's strips in `assets/sprites/veyra/anims/`.
macro_rules! veyra {
    ($id:ident, $file:literal) => {
        Sheet {
            id: SpriteId::$id,
            png: include_bytes!(concat!("../assets/sprites/veyra/anims/", $file, ".png")),
            frames: frame_ms(SpriteId::$id).len(),
            flash: false,
        }
    };
}

/// One of the enemy strips in `assets/sprites/enemies/anims/` (`assets/enemies.px.js`).
macro_rules! enemy {
    ($id:ident, $file:literal) => {
        Sheet {
            id: SpriteId::$id,
            png: include_bytes!(concat!("../assets/sprites/enemies/anims/", $file, ".png")),
            frames: FLASH_FRAME_OFFSET,
            flash: true,
        }
    };
}

/// One of the Arts' effect strips in `assets/sprites/arts/anims/` (`assets/arts.px.js`).
macro_rules! art {
    ($id:ident, $file:literal) => {
        Sheet {
            id: SpriteId::$id,
            png: include_bytes!(concat!("../assets/sprites/arts/anims/", $file, ".png")),
            frames: frame_ms(SpriteId::$id).len(),
            flash: false,
        }
    };
}

/// Per-frame durations (ms) of the animated strips, as in the `.json` beside each one.
pub const fn frame_ms(id: SpriteId) -> &'static [u16] {
    match id {
        SpriteId::VeyraIdle | SpriteId::VeyraWings | SpriteId::VeyraTail | SpriteId::VeyraHair => {
            &[200; 4]
        }
        SpriteId::VeyraMove | SpriteId::VeyraHairMove => &[125; 4],
        SpriteId::VeyraCast => &[80, 80, 160],
        SpriteId::VeyraHurt => &[60, 120],
        SpriteId::VeyraDeath => &[143; 6],
        SpriteId::TailLash => &[63; 3],
        SpriteId::HeartBolt => &[125; 2],
        SpriteId::Ember => &[100; 4],
        SpriteId::CharmHeart => &[167; 2],
        SpriteId::GateFlame => &[100; 4],

        SpriteId::ChainSpark => &[63; 2],
        SpriteId::Offering => &[167; 4],
        SpriteId::Bat | SpriteId::BatHost => &[83; 4],
        SpriteId::BloodBomb => &[125; 2],
        SpriteId::BlackMass | SpriteId::Sabbath => &[125; 4],
        SpriteId::EvilEye => &[250; 2],
        SpriteId::Scythe0
        | SpriteId::Scythe1
        | SpriteId::Scythe2
        | SpriteId::Scythe3
        | SpriteId::Scythe4
        | SpriteId::Scythe5
        | SpriteId::Sigil0
        | SpriteId::Sigil1
        | SpriteId::Sigil2
        | SpriteId::Sigil3
        | SpriteId::Sigil4
        | SpriteId::Sigil5 => &[50; 8],
        SpriteId::Cross0
        | SpriteId::Cross1
        | SpriteId::Cross2
        | SpriteId::Cross3
        | SpriteId::Cross4
        | SpriteId::Cross5
        | SpriteId::CrossBurning0
        | SpriteId::CrossBurning1
        | SpriteId::CrossBurning2
        | SpriteId::CrossBurning3
        | SpriteId::CrossBurning4
        | SpriteId::CrossBurning5 => &[40; 8],
        _ => &[],
    }
}

/// Length of an animated strip in seconds.
pub fn anim_secs(id: SpriteId) -> f32 {
    frame_ms(id).iter().map(|&ms| ms as u32).sum::<u32>() as f32 / 1000.0
}

/// Frame of an animated strip `secs` after it started; past the end it loops or
/// holds the last frame.
pub fn anim_frame(id: SpriteId, secs: f32, looped: bool) -> usize {
    let ms = frame_ms(id);
    let total: u32 = ms.iter().map(|&d| d as u32).sum();
    let mut t = (secs.max(0.0) * 1000.0) as u32;
    if looped {
        t %= total;
    }
    let mut end = 0;
    for (i, &d) in ms.iter().enumerate() {
        end += d as u32;
        if t < end {
            return i;
        }
    }
    ms.len() - 1
}

/// Packing order; enemies face left and the renderer flips them toward Veyra.
/// Ground holds the grass (0..4) and dirt (4..6) tile variants.
const SHEETS: [Sheet; SPRITE_COUNT] = [
    veyra!(VeyraIdle, "veyra_idle"),
    veyra!(VeyraMove, "veyra_move"),
    veyra!(VeyraCast, "veyra_cast"),
    veyra!(VeyraHurt, "veyra_hurt"),
    veyra!(VeyraDeath, "veyra_death"),
    veyra!(VeyraWings, "veyra_wings"),
    veyra!(VeyraTail, "veyra_tail"),
    veyra!(VeyraHair, "veyra_hair"),
    veyra!(VeyraHairMove, "veyra_hair_move"),
    enemy!(Rat, "rat"),
    enemy!(Villager, "villager"),
    enemy!(Knight, "knight"),
    enemy!(Brute, "brute"),
    art!(TailLash, "tail_lash"),
    art!(HeartBolt, "heart_bolt"),
    art!(Ember, "ember"),
    art!(CharmHeart, "charm_heart"),
    art!(GateFlame, "gate_flame"),
    art!(ChainSpark, "chain_spark"),
    art!(Offering, "offering"),
    art!(Bat, "bat"),
    art!(BatHost, "bat_host"),
    art!(BloodBomb, "blood_bomb"),
    art!(BlackMass, "black_mass"),
    art!(Sabbath, "sabbath"),
    art!(EvilEye, "evil_eye"),
    art!(Scythe0, "scythe_0"),
    art!(Scythe1, "scythe_1"),
    art!(Scythe2, "scythe_2"),
    art!(Scythe3, "scythe_3"),
    art!(Scythe4, "scythe_4"),
    art!(Scythe5, "scythe_5"),
    art!(Cross0, "cross_0"),
    art!(Cross1, "cross_1"),
    art!(Cross2, "cross_2"),
    art!(Cross3, "cross_3"),
    art!(Cross4, "cross_4"),
    art!(Cross5, "cross_5"),
    art!(CrossBurning0, "cross_burning_0"),
    art!(CrossBurning1, "cross_burning_1"),
    art!(CrossBurning2, "cross_burning_2"),
    art!(CrossBurning3, "cross_burning_3"),
    art!(CrossBurning4, "cross_burning_4"),
    art!(CrossBurning5, "cross_burning_5"),
    art!(Sigil0, "sigil_0"),
    art!(Sigil1, "sigil_1"),
    art!(Sigil2, "sigil_2"),
    art!(Sigil3, "sigil_3"),
    art!(Sigil4, "sigil_4"),
    art!(Sigil5, "sigil_5"),
    sheet!(Fireball, "fireball", 1, false),
    sheet!(Smoke, "smoke", 1, false),
    sheet!(Soul, "soul", 1, false),
    sheet!(Ground, "ground", GROUND_VARIANTS, false),
];

struct Atlas {
    tex: Texture2D,
    rects: Vec<Rect>,
    /// (first rect index, frame count) per sprite id.
    ranges: [(usize, usize); SPRITE_COUNT],
}

/// Gothic display face (Jacquard 24): crisp at 43 px.
const GOTHIC_TTF: &[u8] = include_bytes!("../assets/fonts/Jacquard24-Regular.ttf");
/// Readable body face (Jersey 15): crisp at 27 px.
const BODY_TTF: &[u8] = include_bytes!("../assets/fonts/Jersey15-Regular.ttf");
/// Compact face for dense text such as the debug overlay (Press Start 2P): crisp at 8 px.
const SMALL_TTF: &[u8] = include_bytes!("../assets/fonts/PressStart2P-Regular.ttf");

/// Every text style the UI uses; each maps to a font and a crisp size.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextStyle {
    /// Game and screen titles (gothic 43).
    Title,
    /// Everything else (body 27).
    Body,
    /// Dense secondary text, e.g. the debug overlay (small 8).
    Small,
}

impl TextStyle {
    const ALL: [TextStyle; 3] = [TextStyle::Title, TextStyle::Body, TextStyle::Small];

    fn spec(self) -> (Face, u16) {
        match self {
            TextStyle::Title => (Face::Gothic, 43),
            TextStyle::Body => (Face::Body, 27),
            TextStyle::Small => (Face::Small, 8),
        }
    }
}

#[derive(Clone, Copy)]
enum Face {
    Gothic,
    Body,
    Small,
}

struct Fonts {
    gothic: Font,
    body: Font,
    small: Font,
}

impl Fonts {
    fn get(&self, face: Face) -> &Font {
        match face {
            Face::Gothic => &self.gothic,
            Face::Body => &self.body,
            Face::Small => &self.small,
        }
    }
}

/// Title-screen Veyra: too big for the atlas, so it is its own texture. The generated portrait
/// (`assets/refs/veyra_gpt_fake_pixel.png`) rebuilt as real 1:1 pixel art by `assets/depixelize.py`
/// and animated in layers by `assets/animate_portrait.py`: an 8x6 sheet of 48 frames at 12 fps.
const TITLE_PORTRAIT_PNG: &[u8] =
    include_bytes!("../assets/sprites/title/veyra_portrait_px_sheet.png");
const TITLE_PORTRAIT_COLS: u32 = 8;
const TITLE_PORTRAIT_FRAMES: u32 = 48;
const TITLE_PORTRAIT_FPS: f64 = 12.0;

/// The loaded title portrait: a sheet of equal frames, drawn 1:1.
#[derive(Clone)]
pub struct TitlePortrait {
    tex: Texture2D,
}

impl TitlePortrait {
    /// On-screen size (one frame).
    pub fn size(&self) -> Vec2 {
        let rows = TITLE_PORTRAIT_FRAMES.div_ceil(TITLE_PORTRAIT_COLS);
        vec2(
            self.tex.width() / TITLE_PORTRAIT_COLS as f32,
            self.tex.height() / rows as f32,
        )
    }

    /// Draws the frame for time `t` (seconds) with its top-left corner at `pos`.
    pub fn draw(&self, pos: Vec2, t: f64) {
        let frame = (t * TITLE_PORTRAIT_FPS) as u32 % TITLE_PORTRAIT_FRAMES;
        let fs = self.size();
        let src = Rect::new(
            (frame % TITLE_PORTRAIT_COLS) as f32 * fs.x,
            (frame / TITLE_PORTRAIT_COLS) as f32 * fs.y,
            fs.x,
            fs.y,
        );
        let params = DrawTextureParams {
            source: Some(src),
            dest_size: Some(self.size()),
            ..Default::default()
        };
        draw_texture_ex(&self.tex, pos.x, pos.y, WHITE, params);
    }
}

/// The rest of the title scene art (`assets/title.px.js`, `assets/title_backdrop.px.js`): big
/// backdrop layers and a few title-only strips, one texture each. Strips are horizontal, equal frames.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TitleArt {
    Sky,
    Hills,
    /// 12 frames at 12 fps: fires and embers flicker.
    Village,
    /// 3 x 24 puff tiers: shade, rim and body passes.
    Smoke,
    Crest,
    Logo,
    /// 4 glow strengths, flickered by the title scene.
    LogoGlow,
    Dominion,
    /// Idle and highlighted.
    Button,
}

impl TitleArt {
    pub const ALL: [TitleArt; 9] = [
        TitleArt::Sky,
        TitleArt::Hills,
        TitleArt::Village,
        TitleArt::Smoke,
        TitleArt::Crest,
        TitleArt::Logo,
        TitleArt::LogoGlow,
        TitleArt::Dominion,
        TitleArt::Button,
    ];

    pub const fn frames(self) -> u32 {
        match self {
            TitleArt::Village => 12,
            TitleArt::Smoke => 72,
            TitleArt::LogoGlow => 4,
            TitleArt::Button => 2,
            _ => 1,
        }
    }

    const fn png(self) -> &'static [u8] {
        macro_rules! png {
            ($p:literal) => {
                include_bytes!(concat!("../assets/sprites/title/", $p, ".png"))
            };
        }
        match self {
            TitleArt::Sky => png!("sprites/title_sky"),
            TitleArt::Hills => png!("sprites/title_hills"),
            TitleArt::Village => png!("anims/title_village"),
            TitleArt::Smoke => png!("anims/title_smoke"),
            TitleArt::Crest => png!("sprites/title_crest"),
            TitleArt::Logo => png!("sprites/title_logo"),
            TitleArt::LogoGlow => png!("anims/title_logo_glow"),
            TitleArt::Dominion => png!("sprites/title_dominion"),
            TitleArt::Button => png!("anims/title_button"),
        }
    }
}

thread_local! {
    static TITLE_ART: OnceCell<Vec<Texture2D>> = const { OnceCell::new() };
    static TITLE_PORTRAIT: OnceCell<TitlePortrait> = const { OnceCell::new() };
    static ATLAS: OnceCell<Atlas> = const { OnceCell::new() };
    static FONTS: OnceCell<Fonts> = const { OnceCell::new() };
}

/// Simple shelf packer writing frames into the atlas image.
struct Packer {
    image: Image,
    x: u16,
    y: u16,
    shelf_h: u16,
}

impl Packer {
    fn place(&mut self, w: u16, h: u16) -> (u16, u16) {
        if self.x + w > ATLAS_W {
            self.x = 0;
            self.y += self.shelf_h + 1;
            self.shelf_h = 0;
        }
        assert!(self.y + h <= ATLAS_H, "sprite atlas full");
        let pos = (self.x, self.y);
        self.x += w + 1;
        self.shelf_h = self.shelf_h.max(h);
        pos
    }

    /// Copies a `w`×`h` region of `src` at (`sx`, 0), optionally as a white silhouette.
    fn add(&mut self, src: &Image, sx: u16, w: u16, h: u16, white: bool) -> Rect {
        let (ox, oy) = self.place(w, h);
        for y in 0..h as u32 {
            for x in 0..w as u32 {
                let c = src.get_pixel(sx as u32 + x, y);
                if c.a > 0.0 {
                    let c = if white { WHITE } else { c };
                    self.image.set_pixel(ox as u32 + x, oy as u32 + y, c);
                }
            }
        }
        Rect::new(ox as f32, oy as f32, w as f32, h as f32)
    }
}

fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h =
        x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

/// Ground tile variant for a world tile: mostly grass, occasional dirt patches.
pub fn ground_variant(ix: i32, iy: i32) -> usize {
    let h = hash(ix as u32, iy as u32, 0x51ed);
    if h % 100 < 12 {
        4 + (h >> 8) as usize % 2
    } else {
        (h >> 8) as usize % 4
    }
}

/// Decodes every sheet and packs it; CPU only, so tests can run it headless.
fn build() -> (Image, Vec<Rect>, [(usize, usize); SPRITE_COUNT]) {
    let mut packer = Packer {
        image: Image::gen_image_color(ATLAS_W, ATLAS_H, BLANK),
        x: 0,
        y: 0,
        shelf_h: 0,
    };
    let mut rects = Vec::new();
    let mut ranges = [(0, 0); SPRITE_COUNT];

    for sheet in &SHEETS {
        let src = Image::from_file_with_format(sheet.png, Some(ImageFormat::Png))
            .unwrap_or_else(|e| panic!("{:?}: {e}", sheet.id));
        assert_eq!(
            src.width as usize % sheet.frames,
            0,
            "{:?} strip width",
            sheet.id
        );
        let fw = src.width / sheet.frames as u16;
        let copies = if sheet.flash { 2 } else { 1 };
        ranges[sheet.id as usize] = (rects.len(), sheet.frames * copies);
        for white in [false, true].into_iter().take(copies) {
            for f in 0..sheet.frames as u16 {
                rects.push(packer.add(&src, f * fw, fw, src.height, white));
            }
        }
    }
    (packer.image, rects, ranges)
}

/// Non-ASCII letters rasterised at init; `i18n` tests check every string against it.
pub const EXTRA_GLYPHS: &str = "áàéíóúüñÁÀÉÍÓÚÜÑ¿¡";

/// Builds the atlas texture. Must run once after the window exists.
pub fn init() {
    let (image, rects, ranges) = build();
    let tex = Texture2D::from_image(&image);
    tex.set_filter(FilterMode::Nearest);
    ATLAS.with(|a| {
        let _ = a.set(Atlas { tex, rects, ranges });
    });
    let tex = Texture2D::from_file_with_format(TITLE_PORTRAIT_PNG, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Nearest);
    TITLE_PORTRAIT.with(|p| {
        let _ = p.set(TitlePortrait { tex });
    });
    let title_art = TitleArt::ALL
        .map(|art| {
            let tex = Texture2D::from_file_with_format(art.png(), Some(ImageFormat::Png));
            tex.set_filter(FilterMode::Nearest);
            tex
        })
        .to_vec();
    TITLE_ART.with(|t| {
        let _ = t.set(title_art);
    });

    let load = |bytes| {
        let mut font = load_ttf_font_from_bytes(bytes).expect("embedded font");
        font.set_filter(FilterMode::Nearest);
        font
    };
    let fonts = Fonts {
        gothic: load(GOTHIC_TTF),
        body: load(BODY_TTF),
        small: load(SMALL_TTF),
    };
    // Rasterise printable ASCII plus Spanish letters up front to avoid first-use hitches.
    let chars: Vec<char> = (' '..='~').chain(EXTRA_GLYPHS.chars()).collect();
    for style in TextStyle::ALL {
        let (face, size) = style.spec();
        fonts.get(face).populate_font_cache(&chars, size);
    }
    FONTS.with(|f| {
        let _ = f.set(fonts);
    });
}

fn with_font<R>(style: TextStyle, f: impl FnOnce(&Font, u16) -> R) -> R {
    FONTS.with(|fonts| {
        let fonts = fonts.get().expect("assets::init not called");
        let (face, size) = style.spec();
        f(fonts.get(face), size)
    })
}

/// Draws `text` with its baseline at `y` (snapped to whole pixels to stay crisp).
pub fn draw_ui_text(text: &str, x: f32, y: f32, style: TextStyle, color: Color) {
    with_font(style, |font, size| {
        let params = TextParams {
            font: Some(font),
            font_size: size,
            color,
            ..Default::default()
        };
        draw_text_ex(text, x.round(), y.round(), params);
    });
}

pub fn measure_ui_text(text: &str, style: TextStyle) -> TextDimensions {
    with_font(style, |font, size| {
        measure_text(text, Some(font), size, 1.0)
    })
}

/// The animated title portrait.
pub fn title_portrait() -> TitlePortrait {
    TITLE_PORTRAIT.with(|p| p.get().expect("assets::init not called").clone())
}

/// Size of one frame of a title art strip.
pub fn title_art_size(art: TitleArt) -> Vec2 {
    TITLE_ART.with(|t| {
        let tex = &t.get().expect("assets::init not called")[art as usize];
        vec2(tex.width() / art.frames() as f32, tex.height())
    })
}

/// Draws a frame of a title art strip 1:1 with its top-left corner at `pos` (snapped to pixels).
pub fn draw_title_art(art: TitleArt, frame: u32, pos: Vec2) {
    TITLE_ART.with(|t| {
        let tex = &t.get().expect("assets::init not called")[art as usize];
        let w = tex.width() / art.frames() as f32;
        let params = DrawTextureParams {
            source: Some(Rect::new(
                (frame % art.frames()) as f32 * w,
                0.0,
                w,
                tex.height(),
            )),
            ..Default::default()
        };
        draw_texture_ex(tex, pos.x.round(), pos.y.round(), WHITE, params);
    });
}

/// Texture and source rect for a sprite frame (frame wraps around).
pub fn sprite(id: SpriteId, frame: usize) -> (Texture2D, Rect) {
    ATLAS.with(|a| {
        let atlas = a.get().expect("assets::init not called");
        let (start, count) = atlas.ranges[id as usize];
        (atlas.tex.clone(), atlas.rects[start + frame % count])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_art_strips_split_into_their_frames() {
        for art in TitleArt::ALL {
            let img = Image::from_file_with_format(art.png(), Some(ImageFormat::Png)).unwrap();
            assert_eq!(img.width as u32 % art.frames(), 0, "{art:?}");
            if art.frames() > 1 {
                let name = match art {
                    TitleArt::Village => "title_village",
                    TitleArt::Smoke => "title_smoke",
                    TitleArt::LogoGlow => "title_logo_glow",
                    _ => "title_button",
                };
                let path = format!(
                    "{}/assets/sprites/title/anims/{name}.json",
                    env!("CARGO_MANIFEST_DIR")
                );
                let json = std::fs::read_to_string(path).unwrap();
                assert_eq!(
                    json.matches("\"duration\"").count() as u32,
                    art.frames(),
                    "{art:?}"
                );
            }
        }
    }

    #[test]
    fn sheet_ids_cover_every_slot_once() {
        let mut seen = [false; SPRITE_COUNT];
        for sheet in &SHEETS {
            assert!(!seen[sheet.id as usize]);
            seen[sheet.id as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn every_sprite_rect_exists_and_is_inside_the_atlas() {
        let (_, rects, ranges) = build();
        for sheet in &SHEETS {
            let (start, count) = ranges[sheet.id as usize];
            let expected = sheet.frames * if sheet.flash { 2 } else { 1 };
            assert_eq!(count, expected, "{:?} frame count", sheet.id);
            for r in &rects[start..start + count] {
                assert!(r.w > 0.0 && r.h > 0.0, "{:?} empty rect", sheet.id);
                assert!(r.right() <= ATLAS_W as f32 && r.bottom() <= ATLAS_H as f32);
            }
        }
    }

    #[test]
    fn key_sprite_sizes() {
        let (_, rects, ranges) = build();
        let size = |id: SpriteId| {
            let r = rects[ranges[id as usize].0];
            (r.w as u16, r.h as u16)
        };
        for id in VEYRA {
            assert_eq!(size(id), (66, 50), "{id:?}");
        }
        assert_eq!(size(SpriteId::Ground), (GROUND_TILE, GROUND_TILE));
        // Enemy sizes from docs/design.md 7.6, outline included (the rat's 12 is its body length);
        // `EnemyDef::size` must match its strip, since radius and burst derive from it.
        assert_eq!(size(SpriteId::Rat), (17, 9));
        assert_eq!(size(SpriteId::Villager), (14, 20));
        assert_eq!(size(SpriteId::Knight), (18, 26));
        assert_eq!(size(SpriteId::Brute), (32, 40));
        for def in &crate::config::ENEMY_DEFS {
            let (w, h) = def.size;
            assert_eq!(size(def.sprite), (w as u16, h as u16), "{}", def.name);
        }
    }

    const VEYRA: [SpriteId; 9] = [
        SpriteId::VeyraIdle,
        SpriteId::VeyraMove,
        SpriteId::VeyraCast,
        SpriteId::VeyraHurt,
        SpriteId::VeyraDeath,
        SpriteId::VeyraWings,
        SpriteId::VeyraTail,
        SpriteId::VeyraHair,
        SpriteId::VeyraHairMove,
    ];

    #[test]
    fn strip_durations_match_the_exported_json() {
        let json = [
            include_str!("../assets/sprites/veyra/anims/veyra_idle.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_move.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_cast.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_hurt.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_death.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_wings.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_tail.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_hair.json"),
            include_str!("../assets/sprites/veyra/anims/veyra_hair_move.json"),
            include_str!("../assets/sprites/arts/anims/tail_lash.json"),
            include_str!("../assets/sprites/arts/anims/heart_bolt.json"),
            include_str!("../assets/sprites/arts/anims/ember.json"),
            include_str!("../assets/sprites/arts/anims/charm_heart.json"),
            include_str!("../assets/sprites/arts/anims/gate_flame.json"),
            include_str!("../assets/sprites/arts/anims/chain_spark.json"),
            include_str!("../assets/sprites/arts/anims/offering.json"),
            include_str!("../assets/sprites/arts/anims/bat.json"),
            include_str!("../assets/sprites/arts/anims/bat_host.json"),
            include_str!("../assets/sprites/arts/anims/blood_bomb.json"),
            include_str!("../assets/sprites/arts/anims/black_mass.json"),
            include_str!("../assets/sprites/arts/anims/sabbath.json"),
            include_str!("../assets/sprites/arts/anims/evil_eye.json"),
            include_str!("../assets/sprites/arts/anims/scythe_0.json"),
            include_str!("../assets/sprites/arts/anims/scythe_1.json"),
            include_str!("../assets/sprites/arts/anims/scythe_2.json"),
            include_str!("../assets/sprites/arts/anims/scythe_3.json"),
            include_str!("../assets/sprites/arts/anims/scythe_4.json"),
            include_str!("../assets/sprites/arts/anims/scythe_5.json"),
            include_str!("../assets/sprites/arts/anims/cross_0.json"),
            include_str!("../assets/sprites/arts/anims/cross_1.json"),
            include_str!("../assets/sprites/arts/anims/cross_2.json"),
            include_str!("../assets/sprites/arts/anims/cross_3.json"),
            include_str!("../assets/sprites/arts/anims/cross_4.json"),
            include_str!("../assets/sprites/arts/anims/cross_5.json"),
            include_str!("../assets/sprites/arts/anims/cross_burning_0.json"),
            include_str!("../assets/sprites/arts/anims/cross_burning_1.json"),
            include_str!("../assets/sprites/arts/anims/cross_burning_2.json"),
            include_str!("../assets/sprites/arts/anims/cross_burning_3.json"),
            include_str!("../assets/sprites/arts/anims/cross_burning_4.json"),
            include_str!("../assets/sprites/arts/anims/cross_burning_5.json"),
            include_str!("../assets/sprites/arts/anims/sigil_0.json"),
            include_str!("../assets/sprites/arts/anims/sigil_1.json"),
            include_str!("../assets/sprites/arts/anims/sigil_2.json"),
            include_str!("../assets/sprites/arts/anims/sigil_3.json"),
            include_str!("../assets/sprites/arts/anims/sigil_4.json"),
            include_str!("../assets/sprites/arts/anims/sigil_5.json"),
        ];
        let arts = [
            SpriteId::TailLash,
            SpriteId::HeartBolt,
            SpriteId::Ember,
            SpriteId::CharmHeart,
            SpriteId::GateFlame,
            SpriteId::ChainSpark,
            SpriteId::Offering,
            SpriteId::Bat,
            SpriteId::BatHost,
            SpriteId::BloodBomb,
            SpriteId::BlackMass,
            SpriteId::Sabbath,
            SpriteId::EvilEye,
            SpriteId::Scythe0,
            SpriteId::Scythe1,
            SpriteId::Scythe2,
            SpriteId::Scythe3,
            SpriteId::Scythe4,
            SpriteId::Scythe5,
            SpriteId::Cross0,
            SpriteId::Cross1,
            SpriteId::Cross2,
            SpriteId::Cross3,
            SpriteId::Cross4,
            SpriteId::Cross5,
            SpriteId::CrossBurning0,
            SpriteId::CrossBurning1,
            SpriteId::CrossBurning2,
            SpriteId::CrossBurning3,
            SpriteId::CrossBurning4,
            SpriteId::CrossBurning5,
            SpriteId::Sigil0,
            SpriteId::Sigil1,
            SpriteId::Sigil2,
            SpriteId::Sigil3,
            SpriteId::Sigil4,
            SpriteId::Sigil5,
        ];
        for (id, json) in VEYRA.into_iter().chain(arts).zip(json) {
            let ms: Vec<u16> = json
                .split("\"duration\":")
                .skip(1)
                .map(|s| {
                    let digits: String = s
                        .trim_start()
                        .chars()
                        .take_while(char::is_ascii_digit)
                        .collect();
                    digits.parse().unwrap()
                })
                .collect();
            assert_eq!(frame_ms(id), ms.as_slice(), "{id:?}");
        }
    }

    #[test]
    fn size_tiers_follow_area() {
        assert_eq!(size_tier(0.9), 0);
        assert_eq!(size_tier(1.0), 0);
        assert_eq!(size_tier(1.1), 0);
        assert_eq!(size_tier(1.45), 2);
        assert_eq!(size_tier(2.2), 5);
        assert_eq!(size_tier(9.0), SIZE_TIERS - 1);
        // Each tier is bigger than the one before.
        let (_, rects, ranges) = build();
        let size = |id: SpriteId| (rects[ranges[id as usize].0].w as u16, 0);
        for tiers in [SCYTHE_TIERS, CROSS_TIERS, CROSS_BURNING_TIERS, SIGIL_TIERS] {
            for w in tiers.windows(2) {
                assert!(size(w[1]).0 > size(w[0]).0, "{:?}", w[1]);
            }
        }
    }

    #[test]
    fn anim_frame_follows_durations() {
        let cast = SpriteId::VeyraCast;
        assert_eq!(anim_frame(cast, 0.0, false), 0);
        assert_eq!(anim_frame(cast, 0.079, false), 0);
        assert_eq!(anim_frame(cast, 0.081, false), 1);
        assert_eq!(anim_frame(cast, 0.2, false), 2);
        assert_eq!(anim_frame(cast, 5.0, false), 2);
        assert!((anim_secs(cast) - 0.32).abs() < 1e-6);
        assert_eq!(anim_frame(SpriteId::VeyraIdle, 0.85, true), 0);
        assert_eq!(anim_frame(SpriteId::VeyraIdle, 0.65, true), 3);
    }

    #[test]
    fn flash_frames_are_white_silhouettes() {
        let (image, rects, ranges) = build();
        let (start, _) = ranges[SpriteId::Rat as usize];
        let (base, flash) = (rects[start], rects[start + FLASH_FRAME_OFFSET]);
        for y in 0..base.h as u32 {
            for x in 0..base.w as u32 {
                let a = image.get_pixel(base.x as u32 + x, base.y as u32 + y);
                let b = image.get_pixel(flash.x as u32 + x, flash.y as u32 + y);
                assert_eq!(a.a > 0.0, b.a > 0.0);
                if b.a > 0.0 {
                    assert_eq!(b, WHITE);
                }
            }
        }
    }
}
