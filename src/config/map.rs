//! Maps: the blocking grid, enemy navigation and the authored layouts (design.md 7.14, V19d).

/// Side of a map cell in world pixels: the blocking grid, the flow field and the dual-grid
/// terrain tiles all use it.
pub const MAP_CELL: f32 = 16.0;
/// Ashford's authored layout (`docs/tools/ashford_layout.py` writes it).
pub const ASHFORD_MAP: &str = include_str!("../../assets/maps/ashford.map");
/// Ticks between flow-field rebuilds (only when Veyra has changed cell since the last one).
pub const FLOW_REFRESH_TICKS: u32 = 6;
/// Path cost the flow field explores out to (orthogonal step 2, diagonal 3): about 90 cells,
/// past the spawn ring and the relocation radius at every view size.
pub const FLOW_RANGE: u16 = 180;
/// A walker heads straight for Veyra while its path is at most this much longer than the
/// straight line (the grid metric alone is up to 1.12x)...
pub const NAV_DIRECT_RATIO: f32 = 1.13;
/// ...plus this many cells of grid slack; past that it follows the flow field.
pub const NAV_DIRECT_SLACK: f32 = 0.75;
/// Tries at finding an open, reachable spot on the spawn ring before a spawn is skipped.
pub const SPAWN_TRIES: usize = 12;
/// Opacity of a prop while it covers Veyra from in front.
pub const ROOF_FADE: f32 = 0.4;
/// Her body (no wings) above her hover height, in pixels: a prop covering it fades.
pub const VEYRA_BODY: macroquad::math::Vec2 = macroquad::math::vec2(26.0, 44.0);
