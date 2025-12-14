#![windows_subsystem = "windows"]

use macroquad::prelude::*;
use chrono::{Local, Timelike};
use std::env;

fn config() -> Conf {
    Conf {
        window_title: "Rust Flip Clock".to_string(),
        fullscreen: true,
        high_dpi: true,
        sample_count: 4, // Anti-aliasing
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Windows Screensaver arguments:
    // /s : Show (Fullscreen)
    // /c : Config (Settings)
    // /p : Preview (Miniature view in settings)

    if args.len() > 1 {
        let arg = args[1].to_lowercase();
        if arg.starts_with("/c") {
            return;
        } else if arg.starts_with("/p") {
            return;
        }
    }

    // Input Handling State
    // We wait a few frames to let the mouse position settle and avoid startup jitter
    let mut frames_rendered = 0;
    let mut last_mouse_x = 0.0;
    let mut last_mouse_y = 0.0;
    let threshold = 50.0;

    // Try to load font
    let font = load_ttf_font("font.ttf").await.ok();

    loop {
        // --- Input Handling ---
        // Exit on key press
        if get_last_key_pressed().is_some() {
            break;
        }

        let (mouse_x, mouse_y) = mouse_position();

        // On the first few frames, we just record the position.
        // Screensavers often get a tiny mouse move event on startup or windows snaps it.
        // We'll give it a grace period of ~10 frames or so.
        if frames_rendered < 10 {
            last_mouse_x = mouse_x;
            last_mouse_y = mouse_y;
        } else {
            // Check distance from the initial position we locked in
            let dist = ((mouse_x - last_mouse_x).powi(2) + (mouse_y - last_mouse_y).powi(2)).sqrt();
            if dist > threshold {
                break;
            }
        }

        frames_rendered += 1;

        // --- Logic ---
        let now = Local::now();
        let time_str = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
        let parts: Vec<&str> = time_str.split(':').collect();

        // --- Rendering ---
        clear_background(BLACK);

        let screen_w = screen_width();
        let screen_h = screen_height();

        // Dynamic sizing
        let card_count = 3;
        let spacing = screen_w * 0.02;
        let total_spacing = spacing * (card_count as f32 - 1.0);

        let available_width = screen_w * 0.8;
        let card_width = (available_width - total_spacing) / card_count as f32;
        let card_height = card_width * 1.4; // 1:1.4 aspect ratio

        let start_x = (screen_w - available_width) / 2.0;
        let start_y = (screen_h - card_height) / 2.0;

        let card_color = Color::new(0.16, 0.16, 0.16, 1.0); // Dark Grey (40,40,40)
        let text_color = WHITE;
        let split_line_color = BLACK;

        for (i, part) in parts.iter().enumerate() {
            let x = start_x + (card_width + spacing) * i as f32;
            let y = start_y;
            let radius = card_width * 0.1;

            // Draw Card Background (Rounded Rectangle)
            // 1. Draw corners
            draw_circle(x + radius, y + radius, radius, card_color);
            draw_circle(x + card_width - radius, y + radius, radius, card_color);
            draw_circle(x + radius, y + card_height - radius, radius, card_color);
            draw_circle(x + card_width - radius, y + card_height - radius, radius, card_color);

            // 2. Draw filling rects (vertical inner, horizontal inner)
            // Vertical rect (between top and bottom circles)
            draw_rectangle(x + radius, y, card_width - 2.0 * radius, card_height, card_color);
            // Horizontal rect (between left and right circles)
            draw_rectangle(x, y + radius, card_width, card_height - 2.0 * radius, card_color);


            // Draw Text
            let font_size = (card_height * 0.7) as u16;
            let text_dims = measure_text(part, font.as_ref(), font_size, 1.0);
            let text_x = x + (card_width - text_dims.width) / 2.0;
            // Center vertically, accounting for font baseline/height quirks usually needing slight nudge
            let text_y = y + (card_height + text_dims.height) / 2.0 - (text_dims.height * 0.1);

            draw_text_ex(part, text_x, text_y, TextParams {
                font: font.as_ref(),
                font_size,
                color: text_color,
                ..Default::default()
            });

            // Draw Split Line
            let line_thickness = card_height * 0.02;
            let line_y = y + card_height / 2.0;
            draw_line(x, line_y, x + card_width, line_y, line_thickness, split_line_color);
        }

        next_frame().await
    }
}
