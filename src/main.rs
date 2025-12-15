use macroquad::prelude::*;
use chrono::{Local, Timelike};
use std::env;
use std::path::Path;

#[cfg(windows)]
mod windows_utils {
    use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, MONITORINFOF_PRIMARY};
    use winapi::shared::windef::{HMONITOR, HDC, LPRECT};
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    #[derive(Clone)]
    pub struct MonitorInfo {
        pub name: String,
        pub width: i32,
        pub height: i32,
        pub is_primary: bool,
    }

    unsafe extern "system" fn monitor_enum_proc(hmonitor: HMONITOR, _: HDC, _: LPRECT, lparam: LPARAM) -> BOOL {
        let monitors = &mut *(lparam as *mut Vec<MonitorInfo>);

        let mut info: MONITORINFOEXW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _) != 0 {
            let width = (info.rcMonitor.right - info.rcMonitor.left).abs();
            let height = (info.rcMonitor.bottom - info.rcMonitor.top).abs();

            // Extract name
            let len = info.szDevice.iter().position(|&c| c == 0).unwrap_or(info.szDevice.len());
            let name = OsString::from_wide(&info.szDevice[0..len]).into_string().unwrap_or("Unknown".to_string());

            monitors.push(MonitorInfo {
                name,
                width,
                height,
                is_primary: (info.dwFlags & MONITORINFOF_PRIMARY) != 0,
            });
        }
        TRUE
    }

    pub fn get_monitors() -> Vec<MonitorInfo> {
        let mut monitors = Vec::new();
        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                Some(monitor_enum_proc),
                &mut monitors as *mut _ as LPARAM,
            );
        }
        monitors
    }
}

#[cfg(not(windows))]
mod windows_utils {
    #[derive(Clone)]
    pub struct MonitorInfo {
        pub name: String,
        pub width: i32,
        pub height: i32,
        pub is_primary: bool,
    }
    pub fn get_monitors() -> Vec<MonitorInfo> {
        vec![]
    }
}

use windows_utils::MonitorInfo;

struct TimeState {
    current_digits: [u32; 4],
    previous_digits: [u32; 4],
    animation_start: Option<f64>,
}

enum AppMode {
    Clock,
    Setup,
}

#[macroquad::main("Flip Clock")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut mode = AppMode::Setup; // Default to setup if no args

    if args.len() > 1 {
        let arg = args[1].to_lowercase();
        if arg.starts_with("/s") {
            mode = AppMode::Clock;
        } else if arg.starts_with("/c") {
             // Config mode
             mode = AppMode::Setup;
        } else if arg.starts_with("/p") {
             // Preview - run clock for now
             mode = AppMode::Clock;
        }
    }

    match mode {
        AppMode::Clock => run_clock().await,
        AppMode::Setup => run_setup().await,
    }
}

async fn run_setup() {
    // Basic UI for setup
    let monitors = windows_utils::get_monitors();
    let mut install_status = String::new();

    // We'll just loop and draw
    loop {
        clear_background(LIGHTGRAY);

        let mut y = 20.0;
        draw_text("Flip Clock Setup", 20.0, y + 20.0, 40.0, BLACK);
        y += 60.0;

        draw_text("Connected Displays:", 20.0, y, 30.0, DARKGRAY);
        y += 40.0;

        for m in &monitors {
             let primary_text = if m.is_primary { " (Primary)" } else { "" };
             let text = format!("- {} [{}x{}]{}", m.name, m.width, m.height, primary_text);
             draw_text(&text, 40.0, y, 20.0, BLACK);
             y += 25.0;
        }

        y += 20.0;

        // Install Button
        let btn_rect = Rect::new(20.0, y, 200.0, 50.0);
        draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, DARKBLUE);
        draw_text("Install Screensaver", btn_rect.x + 10.0, btn_rect.y + 32.0, 20.0, WHITE);

        if is_mouse_button_pressed(MouseButton::Left) {
            let mpos = mouse_position();
            if btn_rect.contains(vec2(mpos.0, mpos.1)) {
                // Try install
                if let Ok(exe_path) = env::current_exe() {
                    let target = Path::new("C:\\Windows\\System32\\rust_flip_clock.scr");
                    match std::fs::copy(&exe_path, target) {
                        Ok(_) => install_status = "Successfully installed to System32!".to_string(),
                        Err(e) => install_status = format!("Error: {}. Try running as Admin.", e),
                    }
                } else {
                    install_status = "Could not locate current executable.".to_string();
                }
            }
        }

        if !install_status.is_empty() {
             draw_text(&install_status, 230.0, y + 32.0, 20.0, RED);
        }

        y += 70.0;
        draw_text("Press ESC to exit", 20.0, y, 20.0, DARKGRAY);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}

async fn run_clock() {
    show_mouse(false);

    let font_path = "assets/fonts/Roboto-Bold.ttf";
    let font = load_ttf_font(font_path).await;
    let font = match font {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("Warning: Failed to load font: {}", e);
            None
        }
    };

    let now = Local::now();
    let hour = now.hour();
    let minute = now.minute();
    let initial_digits = [hour / 10, hour % 10, minute / 10, minute % 10];

    let mut time_state = TimeState {
        current_digits: initial_digits,
        previous_digits: initial_digits,
        animation_start: None,
    };

    let mut flip_target = render_target(10, 10);
    flip_target.texture.set_filter(FilterMode::Linear);

    let mut last_screen_size = (0.0, 0.0);
    let mouse_init_pos = mouse_position();
    let last_mouse_check = get_time();

    loop {
        if get_last_key_pressed().is_some() {
            break;
        }

        if get_time() - last_mouse_check > 0.5 {
            let current_pos = mouse_position();
            if (current_pos.0 - mouse_init_pos.0).abs() > 10.0 || (current_pos.1 - mouse_init_pos.1).abs() > 10.0 {
                break;
            }
        }

        clear_background(Color::new(0.08, 0.08, 0.08, 1.0));

        let sw = screen_width();
        let sh = screen_height();

        let card_height = sh * 0.4;
        let card_width = sw * 0.15;

        if (sw - last_screen_size.0).abs() > 1.0 || (sh - last_screen_size.1).abs() > 1.0 {
            flip_target = render_target(card_width as u32, card_height as u32);
            flip_target.texture.set_filter(FilterMode::Linear);
            last_screen_size = (sw, sh);
        }

        let spacing = sw * 0.02;
        let group_gap = spacing * 3.0;
        let total_width = 4.0 * card_width + 2.0 * spacing + group_gap;
        let start_x = (sw - total_width) / 2.0;
        let start_y = (sh - card_height) / 2.0;
        let font_size = (card_height * 0.8) as u16;

        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let new_digits = [hour / 10, hour % 10, minute / 10, minute % 10];

        if new_digits != time_state.current_digits {
            time_state.previous_digits = time_state.current_digits;
            time_state.current_digits = new_digits;
            time_state.animation_start = Some(get_time());
        }

        let mut progress = 0.0;
        if let Some(start) = time_state.animation_start {
            let elapsed = get_time() - start;
            let duration = 0.6;
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

            draw_flip_card(
                x_offset,
                start_y,
                card_width,
                card_height,
                digit,
                prev_digit,
                digit_progress,
                font.as_ref(),
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

// Reuse the draw helper functions
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
    let bg_color = Color::new(0.16, 0.16, 0.16, 1.0);

    if progress >= 1.0 {
        draw_card_bg(x, y, w, h, bg_color);
        draw_digit_centered(x, y, w, h, digit, font, font_size);
    } else {
        draw_card_bg(x, y, w, h, bg_color);
        draw_digit_centered(x, y, w, h, prev_digit, font, font_size);

        let mut camera = Camera2D {
            render_target: Some(flip_target.clone()),
            ..Default::default()
        };
        camera.zoom = vec2(2.0 / w, -2.0 / h);
        camera.target = vec2(w / 2.0, h / 2.0);

        set_camera(&camera);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        draw_card_bg(0.0, 0.0, w, h, bg_color);
        draw_digit_centered(0.0, 0.0, w, h, digit, font, font_size);
        set_default_camera();

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

    let mid_y = y + h / 2.0;
    draw_line(x, mid_y, x + w, mid_y, 4.0, BLACK);
}

fn draw_card_bg(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x, y, w, h, color);
}

fn draw_digit_centered(x: f32, y: f32, w: f32, h: f32, digit: u32, font: Option<&Font>, font_size: u16) {
    let text = digit.to_string();
    let dims = measure_text(&text, font, font_size, 1.0);
    let tx = x + (w - dims.width) / 2.0;
    let ty = y + (h - dims.height) / 2.0 + dims.offset_y;

    draw_text_ex(&text, tx, ty, TextParams {
        font,
        font_size,
        color: WHITE,
        ..Default::default()
    });
}
