#![windows_subsystem = "windows"]

use macroquad::prelude::*;
use chrono::Local;

fn window_conf() -> Conf {
    let args: Vec<String> = std::env::args().collect();
    let mut fullscreen = false;

    // Check arguments for fullscreen mode (/s)
    for arg in &args {
        let lower = arg.to_lowercase();
        if lower.starts_with("/s") {
            fullscreen = true;
        }
    }

    Conf {
        window_title: "Flip Clock".to_owned(),
        fullscreen,
        high_dpi: true,
        sample_count: 4, // Anti-aliasing
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Handle /c (config) and /p (preview) by exiting immediately
    for arg in &args {
        let lower = arg.to_lowercase();
        if lower.starts_with("/c") || lower.starts_with("/p") {
            return;
        }
    }

    // Capture initial mouse position for threshold check
    let (start_mouse_x, start_mouse_y) = mouse_position();
    let mouse_threshold = 50.0;

    // We use a small grace period (e.g., first few frames or time)
    // to avoid closing immediately due to startup jitter.
    let start_time = get_time();
    let grace_period = 0.5; // seconds

    loop {
        let dt = get_time() - start_time;

        // Input Handling: Exit on key press or significant mouse movement
        // Only check after grace period
        if dt > grace_period {
            if get_last_key_pressed().is_some() {
                break;
            }

            let (mx, my) = mouse_position();
            let dist = ((mx - start_mouse_x).powi(2) + (my - start_mouse_y).powi(2)).sqrt();
            if dist > mouse_threshold {
                break;
            }
        }

        clear_background(Color::new(0.1, 0.1, 0.1, 1.0)); // Dark background

        let now = Local::now();
        let time_str = now.format("%H %M %S").to_string();
        let parts: Vec<&str> = time_str.split_whitespace().collect();

        // Layout calculations
        let screen_w = screen_width();
        let screen_h = screen_height();

        let card_w = screen_w * 0.2;
        let card_h = screen_h * 0.4;
        let spacing = screen_w * 0.05;

        let total_w = 3.0 * card_w + 2.0 * spacing;
        let start_x = (screen_w - total_w) / 2.0;
        let start_y = (screen_h - card_h) / 2.0;

        let card_color = Color::new(0.2, 0.2, 0.2, 1.0); // Dark grey
        let text_color = WHITE;
        let line_color = BLACK;

        for (i, part) in parts.iter().enumerate() {
            let x = start_x + (card_w + spacing) * i as f32;
            let y = start_y;

            // Draw Card Background (Rounded Rectangle)
            draw_rectangle(x, y, card_w, card_h, card_color);

            // Draw Text
            // We need to estimate font size.
            // Macroquad's default font is small. measure_text helps.
            let font_size = card_h * 0.6;
            let text_dims = measure_text(part, None, font_size as u16, 1.0);

            let text_x = x + (card_w - text_dims.width) / 2.0;
            let _text_y = y + (card_h + text_dims.height) / 2.0 - text_dims.offset_y * 0.5; // centering vertically is tricky with fonts
            // Approximate vertical center: y + card_h/2 + text_height/2 is roughly baseline.
            // macroquad draw_text y is the baseline.
            // text_dims.height is usually ascent - descent.
            // Let's try centering simply.

            draw_text(part, text_x, y + card_h * 0.75, font_size, text_color);

            // Draw Split Line
            draw_line(x, y + card_h / 2.0, x + card_w, y + card_h / 2.0, 4.0, line_color);
        }

        next_frame().await
    }
}
