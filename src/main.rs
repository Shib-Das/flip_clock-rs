use macroquad::prelude::*;
use chrono::{Local, Timelike};
use std::env;
use std::path::Path;

mod config;
use config::{load_config, save_config};

#[cfg(windows)]
mod windows_utils {
    use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, MONITORINFOF_PRIMARY,
        GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
        SetWindowPos, SetWindowLongW, GetWindowLongW, HWND_TOP, SWP_SHOWWINDOW,
        GWL_STYLE, WS_POPUP, WS_VISIBLE, GetForegroundWindow
    };
    use winapi::shared::windef::{HMONITOR, HDC, LPRECT, HWND, RECT};
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use macroquad::prelude::Rect;

    #[derive(Clone, Debug)]
    pub struct MonitorInfo {
        pub name: String,
        pub x: i32,
        pub y: i32,
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
            let x = info.rcMonitor.left;
            let y = info.rcMonitor.top;

            // Extract name
            let len = info.szDevice.iter().position(|&c| c == 0).unwrap_or(info.szDevice.len());
            let name = OsString::from_wide(&info.szDevice[0..len]).into_string().unwrap_or("Unknown".to_string());

            monitors.push(MonitorInfo {
                name,
                x,
                y,
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

    pub fn get_virtual_screen_rect() -> Rect {
        unsafe {
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN) as f32;
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN) as f32;
            let w = GetSystemMetrics(SM_CXVIRTUALSCREEN) as f32;
            let h = GetSystemMetrics(SM_CYVIRTUALSCREEN) as f32;
            Rect::new(x, y, w, h)
        }
    }

    pub fn make_window_cover_virtual_screen() {
        unsafe {
            // Find the window. Since we are the active process, GetForegroundWindow should work often,
            // but let's be more robust if possible. Macroquad doesn't expose HWND.
            // But usually the screensaver is the foreground window when running /s.
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.is_null() { return; }

            // Remove borders and make popup
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            SetWindowLongW(hwnd, GWL_STYLE, (style & !winapi::um::winuser::WS_OVERLAPPEDWINDOW) | WS_POPUP | WS_VISIBLE as i32);

            let v_rect = get_virtual_screen_rect();
            SetWindowPos(
                hwnd,
                HWND_TOP,
                v_rect.x as i32,
                v_rect.y as i32,
                v_rect.w as i32,
                v_rect.h as i32,
                SWP_SHOWWINDOW
            );
        }
    }
}

#[cfg(not(windows))]
mod windows_utils {
    use macroquad::prelude::Rect;

    #[derive(Clone, Debug)]
    pub struct MonitorInfo {
        pub name: String,
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
        pub is_primary: bool,
    }
    pub fn get_monitors() -> Vec<MonitorInfo> {
        vec![]
    }
    pub fn get_virtual_screen_rect() -> Rect {
        Rect::new(0.0, 0.0, 1920.0, 1080.0)
    }
    pub fn make_window_cover_virtual_screen() {}
}

struct TimeState {
    current_digits: [u32; 4],
    previous_digits: [u32; 4],
    animation_start: Option<f64>,
}

#[derive(PartialEq)]
enum AppMode {
    Clock { preview: bool },
    Setup,
}

#[macroquad::main("Flip Clock")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut mode = AppMode::Setup; // Default to setup if no args

    if args.len() > 1 {
        let arg = args[1].to_lowercase();
        if arg.starts_with("/s") {
            mode = AppMode::Clock { preview: false };
        } else if arg.starts_with("/c") {
             mode = AppMode::Setup;
        } else if arg.starts_with("/p") {
             // Preview mode usually implies drawing in a mini window handle passed as arg.
             // But macroquad doesn't support that easily.
             // We'll interpret it as Clock mode for now, but usually /p gives an HWND.
             // Standard Windows preview sends: /p <HWND>
             // For now, we will ignore the HWND and just exit or run normally.
             // But if the user wants "Try it out", we handle that internally in Setup.
             mode = AppMode::Clock { preview: true };
        }
    }

    loop {
        match mode {
            AppMode::Clock { preview } => {
                if run_clock(preview).await {
                    // if run_clock returns true, it means we want to return to setup (only from preview)
                    mode = AppMode::Setup;
                    // Restore window size? Macroquad window might be stuck at full screen size.
                    // This is tricky. Usually we just exit.
                    // But for "Try it out", we might want to restart the loop.
                    // However, we resized the window. We'd need to restore it.
                    // For simplicity, "Try it out" will just run until exit for now.
                    // Wait, if we return to setup, we need a standard window again.
                    // Let's rely on restarting the app for simplicity if resizing back is hard.
                    // But if `run_clock` returns, it means the user interrupted.
                    break;
                } else {
                    break;
                }
            },
            AppMode::Setup => {
                if let Some(next_mode) = run_setup().await {
                    mode = next_mode;
                } else {
                    break;
                }
            },
        }
    }
}

async fn run_setup() -> Option<AppMode> {
    let mut config = load_config();
    let monitors = windows_utils::get_monitors();
    let mut install_status = String::new();

    loop {
        clear_background(LIGHTGRAY);

        let mut y = 20.0;
        draw_text("Flip Clock Setup", 20.0, y + 20.0, 40.0, BLACK);
        y += 60.0;

        draw_text("Select Display:", 20.0, y, 30.0, DARKGRAY);
        y += 40.0;

        for m in &monitors {
             let is_selected = if config.selected_monitor.is_empty() {
                 m.is_primary
             } else {
                 m.name == config.selected_monitor
             };

             let color = if is_selected { DARKBLUE } else { BLACK };
             let prefix = if is_selected { "[x] " } else { "[ ] " };
             let primary_text = if m.is_primary { " (Primary)" } else { "" };
             let text = format!("{}{}{}", prefix, m.name, primary_text);

             let text_dims = measure_text(&text, None, 20, 1.0);
             let rect = Rect::new(40.0, y - 15.0, text_dims.width, 25.0);

             if is_mouse_button_pressed(MouseButton::Left) {
                 let mpos = mouse_position();
                 if rect.contains(vec2(mpos.0, mpos.1)) {
                     config.selected_monitor = m.name.clone();
                     save_config(&config);
                 }
             }

             draw_text(&text, 40.0, y, 20.0, color);
             y += 30.0;
        }

        y += 20.0;

        // Try it out Button
        let try_rect = Rect::new(20.0, y, 150.0, 40.0);
        draw_rectangle(try_rect.x, try_rect.y, try_rect.w, try_rect.h, GRAY);
        draw_text("Try it out", try_rect.x + 10.0, try_rect.y + 25.0, 20.0, WHITE);

        if is_mouse_button_pressed(MouseButton::Left) {
            let mpos = mouse_position();
            if try_rect.contains(vec2(mpos.0, mpos.1)) {
                return Some(AppMode::Clock { preview: true });
            }
        }

        y += 50.0;

        // Install Button
        let btn_rect = Rect::new(20.0, y, 200.0, 50.0);
        draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, DARKBLUE);
        draw_text("Install Screensaver", btn_rect.x + 10.0, btn_rect.y + 32.0, 20.0, WHITE);

        if is_mouse_button_pressed(MouseButton::Left) {
            let mpos = mouse_position();
            if btn_rect.contains(vec2(mpos.0, mpos.1)) {
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
            return None;
        }

        next_frame().await;
    }
}

// Returns true if should return to setup (not implemented currently), false if exit
async fn run_clock(_preview: bool) -> bool {
    show_mouse(false);

    // Apply configuration to window
    #[cfg(windows)]
    {
        // Give a small delay to ensure window is created by macroquad?
        // Macroquad creates window before main.
        windows_utils::make_window_cover_virtual_screen();
    }
    #[cfg(not(windows))]
    {
        windows_utils::make_window_cover_virtual_screen();
    }

    // Get Monitor info to decide where to draw
    let config = load_config();
    let monitors = windows_utils::get_monitors();
    let virtual_rect = windows_utils::get_virtual_screen_rect();

    // Determine the target rectangle for the clock (relative to the virtual screen top-left)
    let target_monitor = if config.selected_monitor.is_empty() {
        monitors.iter().find(|m| m.is_primary).or(monitors.first())
    } else {
        monitors.iter().find(|m| m.name == config.selected_monitor).or(monitors.first())
    };

    let (clock_x, clock_y, clock_w, clock_h) = if let Some(m) = target_monitor {
        // Map monitor coordinates to window coordinates
        // Window is at virtual_rect.x, virtual_rect.y
        let rel_x = (m.x as f32) - virtual_rect.x;
        let rel_y = (m.y as f32) - virtual_rect.y;
        (rel_x, rel_y, m.width as f32, m.height as f32)
    } else {
        (0.0, 0.0, screen_width(), screen_height())
    };

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

    // We need to render the clock into a texture? Or just draw it at the correct offset.
    // Drawing at offset is easier.

    let mut flip_target = render_target(10, 10);
    flip_target.texture.set_filter(FilterMode::Linear);

    let mut last_screen_size = (0.0, 0.0);
    let mouse_init_pos = mouse_position();
    let last_mouse_check = get_time();

    loop {
        if get_last_key_pressed().is_some() {
            return false;
        }

        if get_time() - last_mouse_check > 0.5 {
            let current_pos = mouse_position();
            // In preview mode or normal mode, large mouse movement exits
            if (current_pos.0 - mouse_init_pos.0).abs() > 10.0 || (current_pos.1 - mouse_init_pos.1).abs() > 10.0 {
                return false;
            }
        }

        // Draw black everywhere
        clear_background(BLACK);

        let sw = clock_w;
        let sh = clock_h;

        let card_height = sh * 0.4;
        let card_width = sw * 0.15;

        // Resize render target if needed
        if (sw - last_screen_size.0).abs() > 1.0 || (sh - last_screen_size.1).abs() > 1.0 {
            flip_target = render_target(card_width as u32, card_height as u32);
            flip_target.texture.set_filter(FilterMode::Linear);
            last_screen_size = (sw, sh);
        }

        let spacing = sw * 0.02;
        let group_gap = spacing * 3.0;
        let total_width = 4.0 * card_width + 2.0 * spacing + group_gap;

        // Calculate start position relative to the clock monitor's area
        let start_x = clock_x + (sw - total_width) / 2.0;
        let start_y = clock_y + (sh - card_height) / 2.0;

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
