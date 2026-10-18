mod config;

use config::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: GAME_TITLE.to_owned(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        window_resizable: true,
        ..Default::default()
    }
}

/// Camera that renders the logical 640x360 frame into the render target.
/// Render targets are stored flipped, hence the inverted y zoom.
fn target_camera(target: &RenderTarget, origin: Vec2) -> Camera2D {
    let mut cam = Camera2D::from_display_rect(Rect::new(origin.x, origin.y, LOGICAL_W, LOGICAL_H));
    cam.zoom.y = -cam.zoom.y;
    cam.render_target = Some(target.clone());
    cam
}

/// Draws the logical frame centred in the window, integer-scaled when it fits.
fn present(target: &RenderTarget) {
    set_default_camera();
    clear_background(BLACK);
    let (sw, sh) = (screen_width(), screen_height());
    let mut scale = (sw / LOGICAL_W).min(sh / LOGICAL_H);
    if scale >= 1.0 {
        scale = scale.floor();
    }
    let (w, h) = (LOGICAL_W * scale, LOGICAL_H * scale);
    draw_texture_ex(
        &target.texture,
        ((sw - w) * 0.5).floor(),
        ((sh - h) * 0.5).floor(),
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        },
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let target = render_target(LOGICAL_W as u32, LOGICAL_H as u32);
    target.texture.set_filter(FilterMode::Nearest);

    let mut accumulator = 0.0f32;
    let mut sim_time = 0.0f32;

    loop {
        accumulator += get_frame_time().min(MAX_FRAME_DT);
        let mut ticks = 0;
        while accumulator >= TICK_DT && ticks < MAX_TICKS_PER_FRAME {
            sim_time += TICK_DT;
            accumulator -= TICK_DT;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            accumulator = 0.0;
        }

        set_camera(&target_camera(&target, Vec2::ZERO));
        clear_background(Color::from_rgba(24, 20, 28, 255));
        draw_text(GAME_TITLE, 16.0, 32.0, 32.0, WHITE);
        draw_rectangle(
            16.0 + (sim_time * 60.0) % (LOGICAL_W - 48.0),
            300.0,
            16.0,
            16.0,
            RED,
        );

        present(&target);
        next_frame().await;
    }
}
