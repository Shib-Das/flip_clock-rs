use macroquad::prelude::*;
use chrono::{Local, Timelike};
use std::env;

struct TimeState {
    current_digits: [u32; 4],
    previous_digits: [u32; 4],
    animation_start: Option<f64>, // macroquad::time::get_time() returns f64 seconds
}

#[macroquad::main("Flip Clock")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    // Basic screensaver arg parsing (very simple)
    if args.len() > 1 {
        let arg = args[1].to_lowercase();
        if arg.starts_with("/p") || arg.starts_with("/c") {
             // Preview or config mode - usually generic exit or show nothing for now
             // For preview, we might actually want to render, but in a specific window handle...
             // Macroquad doesn't support embedding in a window handle easily. 
             // We'll just exit for config to avoid confusion, or run normally for preview if testing.
             if arg.starts_with("/c") {
                 return;
             }
        }
    }

    show_mouse(false);

    // Try to load font
    // Macroquad assets are asynchronous
    // We look in assets/fonts/Roboto-Bold.ttf
    let font_path = "assets/fonts/Roboto-Bold.ttf";
    let font = load_ttf_font(font_path).await;
    
    // If font fails, we can't do much, but we'll try to continue with default if possible or panic
    let font = match font {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("Warning: Failed to load font '{}': {}. text will be default.", font_path, e);
            None
        }
    };

    // Initialize TimeState
    let now = Local::now();
    let hour = now.hour();
    let minute = now.minute();
    let initial_digits = [hour / 10, hour % 10, minute / 10, minute % 10];

    let mut time_state = TimeState {
        current_digits: initial_digits,
        previous_digits: initial_digits,
        animation_start: None,
    };

    // We need a render target for the "new" digit to clip it.
    // We'll initialize it with a dummy size and resize if needed.
    let mut flip_target = render_target(10, 10);
    flip_target.texture.set_filter(FilterMode::Linear);

    let mut last_screen_size = (0.0, 0.0);

    let mouse_init_pos = mouse_position();
    let last_mouse_check = get_time();

    loop {
        // Exit on key press
        if get_last_key_pressed().is_some() {
            break;
        }

        // Exit on mouse move (debounced slightly or threshold)
        if get_time() - last_mouse_check > 0.5 {
            let current_pos = mouse_position();
            if (current_pos.0 - mouse_init_pos.0).abs() > 10.0 || (current_pos.1 - mouse_init_pos.1).abs() > 10.0 {
                break;
            }
        }

        clear_background(Color::new(0.08, 0.08, 0.08, 1.0)); // Dark grey background

        let sw = screen_width();
        let sh = screen_height();

        // Check resize for render target
        // We use the card size as target size
        let card_height = sh * 0.4;
        let card_width = sw * 0.15;
        
        // Resize target if screen changed substantially (e.g. > 1px)
        if (sw - last_screen_size.0).abs() > 1.0 || (sh - last_screen_size.1).abs() > 1.0 {
            flip_target = render_target(card_width as u32, card_height as u32);
            flip_target.texture.set_filter(FilterMode::Linear);
            last_screen_size = (sw, sh);
        }

        // Dimensions
        let spacing = sw * 0.02;
        let group_gap = spacing * 3.0;
        let total_width = 4.0 * card_width + 2.0 * spacing + group_gap;
        let start_x = (sw - total_width) / 2.0;
        let start_y = (sh - card_height) / 2.0;
        let font_size = (card_height * 0.8) as u16;

        // Update time
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let new_digits = [hour / 10, hour % 10, minute / 10, minute % 10];

        if new_digits != time_state.current_digits {
            time_state.previous_digits = time_state.current_digits;
            time_state.current_digits = new_digits;
            time_state.animation_start = Some(get_time());
        }

        // Animation logic
        let mut progress = 0.0;
        if let Some(start) = time_state.animation_start {
            let elapsed = get_time() - start;
            let duration = 0.6; // 600ms
            progress = (elapsed / duration) as f32;
            if progress >= 1.0 {
                progress = 1.0;
                time_state.animation_start = None;
                time_state.previous_digits = time_state.current_digits;
            }
        }

        let mut x_offset = start_x;

        for (i, &digit) in time_state.current_digits.iter().enumerate() {
            let prev_digit = time_state.previous_digits[i];
            let digit_progress = if digit == prev_digit { 1.0 } else { progress };

            // We pass the render target to the draw function
            draw_flip_card(
                x_offset,
                start_y,
                card_width,
                card_height,
                digit,
                prev_digit,
                digit_progress,
                font.as_ref(), // Pass Option<&Font>
                font_size,
                &flip_target,
            );

            x_offset += card_width + spacing;
            if i == 1 {
                x_offset += group_gap - spacing;
            }
        }

        next_frame().await
    }
}

fn draw_flip_card(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    digit: u32,
    prev_digit: u32,
    progress: f32,
    font: Option<&Font>,
    font_size: u16,
    flip_target: &RenderTarget,
) {
    let bg_color = Color::new(0.16, 0.16, 0.16, 1.0); // lighter grey

    if progress >= 1.0 {
        // Static
        draw_card_bg(x, y, w, h, bg_color);
        draw_digit_centered(x, y, w, h, digit, font, font_size);
    } else {
        // Animation
        // 1. Draw Previous Digit on Screen
        draw_card_bg(x, y, w, h, bg_color);
        draw_digit_centered(x, y, w, h, prev_digit, font, font_size);

        // 2. Draw New Digit to Render Target
        // Set camera to the target
        let mut camera = Camera2D {
            render_target: Some(flip_target.clone()),
            ..Default::default()
        };
        // To get pixel-perfect 0,0 top-left on a `w x h` texture:
        camera.zoom = vec2(2.0 / w, -2.0 / h); // Flip Y to make top-left (0,0) work like screen?
        camera.target = vec2(w / 2.0, h / 2.0); // Center the camera at center of texture
        
        set_camera(&camera);
        
        // Clear the texture
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0)); // Transparent
        // Draw the NEW digit onto the texture at (0,0)
        draw_card_bg(0.0, 0.0, w, h, bg_color);
        draw_digit_centered(0.0, 0.0, w, h, digit, font, font_size);

        // Reset to default camera
        set_default_camera();

        // 3. Draw the Render Target Texture (clipped)
        // Wipe height
        let wipe_height = h * progress;
        
        draw_texture_ex(
            &flip_target.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, wipe_height)),
                source: Some(Rect::new(0.0, 0.0, w, wipe_height)),
                ..Default::default()
            },
        );
    }

    // Split line
    let mid_y = y + h / 2.0;
    draw_line(x, mid_y, x + w, mid_y, 4.0, BLACK);
}

fn draw_card_bg(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x, y, w, h, color);
    // Optional: Border
    // draw_rectangle_lines(x, y, w, h, 2.0, BLACK);
}

fn draw_digit_centered(x: f32, y: f32, w: f32, h: f32, digit: u32, font: Option<&Font>, font_size: u16) {
    let text = digit.to_string();
    let dims = measure_text(&text, font, font_size, 1.0);
    
    let tx = x + (w - dims.width) / 2.0;
    // measure_text_dimensions is centered on baseline?
    // offset_y is ascender
    let ty = y + (h - dims.height) / 2.0 + dims.offset_y;

    draw_text_ex(&text, tx, ty, TextParams {
        font,
        font_size,
        color: WHITE,
        ..Default::default()
    });
}
