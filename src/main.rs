use macroquad::prelude::*;
use chrono::{Local, Timelike};
use std::env;
use std::path::Path;
use egui_macroquad::egui;
use macroquad::miniquad;

mod config;
use config::{load_config, save_config, AppConfig};

#[cfg(windows)]
mod windows_utils {
    use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, MONITORINFOF_PRIMARY,
        GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
        SetWindowPos, SetWindowLongW, GetWindowLongW, HWND_TOP, SWP_SHOWWINDOW,
        GWL_STYLE, WS_POPUP, WS_VISIBLE, GetForegroundWindow
    };
    use winapi::shared::windef::{HMONITOR, HDC, LPRECT, HWND};
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
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.is_null() { return; }

            let style = GetWindowLongW(hwnd, GWL_STYLE);
            SetWindowLongW(hwnd, GWL_STYLE, ((style as u32 & !winapi::um::winuser::WS_OVERLAPPEDWINDOW) | WS_POPUP | WS_VISIBLE) as i32);

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

#[derive(Clone, Copy, PartialEq)]
struct TimeState {
    current_digits: [u32; 4],
    current_seconds: [u32; 2],
    previous_digits: [u32; 4],
    previous_seconds: [u32; 2],
    animation_start: Option<f64>,
}

impl TimeState {
    fn new() -> Self {
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let second = now.second();
        let digits = [hour / 10, hour % 10, minute / 10, minute % 10];
        let seconds = [second / 10, second % 10];
        Self {
            current_digits: digits,
            current_seconds: seconds,
            previous_digits: digits,
            previous_seconds: seconds,
            animation_start: None,
        }
    }

    fn update(&mut self, use_12h: bool) {
        let now = Local::now();
        let mut hour = now.hour();
        if use_12h {
            hour = hour % 12;
            if hour == 0 { hour = 12; }
        }
        let minute = now.minute();
        let second = now.second();

        let new_digits = [hour / 10, hour % 10, minute / 10, minute % 10];
        let new_seconds = [second / 10, second % 10];

        if new_digits != self.current_digits || new_seconds != self.current_seconds {
             if self.animation_start.is_none() {
                 self.previous_digits = self.current_digits;
                 self.previous_seconds = self.current_seconds;
                 self.current_digits = new_digits;
                 self.current_seconds = new_seconds;
                 self.animation_start = Some(get_time());
             }
        }
    }
}

#[derive(PartialEq)]
enum AppMode {
    Clock { preview: bool },
    Setup,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flip Clock".to_owned(),
        high_dpi: true,
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut mode = AppMode::Setup;

    if args.len() > 1 {
        let arg = args[1].to_lowercase();
        if arg.starts_with("/s") {
            mode = AppMode::Clock { preview: false };
        } else if arg.starts_with("/c") {
             mode = AppMode::Setup;
        } else if arg.starts_with("/p") {
             mode = AppMode::Clock { preview: true };
        }
    }

    // Load font once
    let font_path = "assets/fonts/Roboto-Bold.ttf";
    let font = load_ttf_font(font_path).await.ok();
    if font.is_none() {
        eprintln!("Warning: Failed to load font");
    }

    loop {
        match mode {
            AppMode::Clock { preview } => {
                if run_clock(preview, font.as_ref()).await {
                    break;
                } else {
                    break;
                }
            },
            AppMode::Setup => {
                if let Some(next_mode) = run_setup(font.as_ref()).await {
                    mode = next_mode;
                } else {
                    break;
                }
            },
        }
    }
}

#[derive(PartialEq)]
enum SetupTab {
    General,
    Layout,
    Theme,
}

async fn run_setup(font: Option<&Font>) -> Option<AppMode> {
    let mut config = load_config();
    let monitors = windows_utils::get_monitors();
    let mut active_tab = SetupTab::Layout;

    let mut install_status = String::new();
    let mut time_state = TimeState::new();

    // Preview Render Target
    let preview_width = 400;
    let preview_height = 225; // 16:9 aspect roughly
    let preview_target = render_target(preview_width as u32, preview_height as u32);
    preview_target.texture.set_filter(FilterMode::Linear);

    loop {
        // Update Time
        time_state.update(config.use_12h_format);

        // --- Render Preview Clock to Texture ---
        {
            let mut camera = Camera2D {
                render_target: Some(preview_target.clone()),
                ..Default::default()
            };

            // Map logical pixels to render target
            camera.zoom = vec2(2.0 / preview_width as f32, 2.0 / preview_height as f32);
            camera.target = vec2(preview_width as f32 / 2.0, preview_height as f32 / 2.0);

            set_camera(&camera);

            // Draw Background
            let bg = mq_color_from_config(config.bg_color);
            clear_background(bg);

            // Draw Clock
            let rect = Rect::new(0.0, 0.0, preview_width as f32, preview_height as f32);
            draw_clock_face(&config, &mut time_state, rect, font, true);

            // Pixelated Overlay (simulated)
            if config.pixelated {
                draw_rectangle(0.0, 0.0, preview_width as f32, preview_height as f32, Color::new(0.0, 0.0, 0.0, 0.2));
            }

            set_default_camera();
        }

        clear_background(BLACK);

        let mut next_mode: Option<AppMode> = None;
        let mut exit_setup = false;

        egui_macroquad::ui(|ctx| {
             // Dark Theme Setup
             let mut visuals = egui::Visuals::dark();
             visuals.panel_fill = egui::Color32::from_rgb(23, 23, 23); // #171717
             ctx.set_visuals(visuals);

             // Styles
             let mut style = (*ctx.style()).clone();
             style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional));
             style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional));
             ctx.set_style(style);

             // SIDEBAR
             egui::SidePanel::left("sidebar")
                 .default_width(250.0)
                 .resizable(false)
                 .show(ctx, |ui| {
                     ui.add_space(20.0);
                     ui.heading("Flip Clock");
                     ui.label(egui::RichText::new("Configuration Utility v1.0").size(10.0).color(egui::Color32::from_gray(120)));
                     ui.add_space(20.0);

                     // Navigation
                     let nav_btn = |ui: &mut egui::Ui, text: &str, tab: SetupTab, current: &SetupTab| {
                         let selected = *current == tab;
                         let btn = ui.add_sized([ui.available_width(), 40.0], egui::SelectableLabel::new(selected, text));
                         if btn.clicked() {
                             return Some(tab);
                         }
                         None
                     };

                     if let Some(t) = nav_btn(ui, "General", SetupTab::General, &active_tab) { active_tab = t; }
                     if let Some(t) = nav_btn(ui, "Layout & Size", SetupTab::Layout, &active_tab) { active_tab = t; }
                     if let Some(t) = nav_btn(ui, "Theme & Color", SetupTab::Theme, &active_tab) { active_tab = t; }

                     ui.add_space(40.0);

                     // PREVIEW
                     ui.label("PREVIEW");

                     // Retrieve raw OpenGL Texture ID from Miniquad
                     let gl = unsafe { get_internal_gl() };
                     let mq_tex = preview_target.texture.raw_miniquad_id();
                     let raw_id = match unsafe { gl.quad_context.texture_raw_id(mq_tex) } {
                         miniquad::RawId::OpenGl(id) => id as u64,
                         _ => 0, // Should not happen on OpenGL platforms
                     };

                     if raw_id != 0 {
                         let texture_id = egui::TextureId::User(raw_id);
                         ui.image(egui::load::SizedTexture::new(texture_id, [240.0, 135.0]));
                     } else {
                         ui.label("Preview unavailable (Render Backend not supported)");
                     }

                     ui.label(egui::RichText::new("Updates live").size(10.0).color(egui::Color32::from_gray(100)));

                     ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                         ui.add_space(10.0);
                         ui.label("Press ESC to exit");
                     });
                 });

             // BOTTOM BAR (Action Bar)
             egui::TopBottomPanel::bottom("bottom_bar")
                 .min_height(60.0)
                 .show(ctx, |ui| {
                     ui.horizontal(|ui| {
                         ui.add_space(20.0);
                         if !install_status.is_empty() {
                             ui.label(egui::RichText::new(&install_status).color(egui::Color32::GREEN));
                         } else {
                             ui.label("Ready to install");
                         }

                         ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                             ui.add_space(20.0);

                             if ui.button("Install Screensaver").clicked() {
                                 if let Ok(exe_path) = env::current_exe() {
                                     let target = Path::new("C:\\Windows\\System32\\rust_flip_clock.scr");
                                     match std::fs::copy(&exe_path, target) {
                                         Ok(_) => install_status = "Successfully installed to System32!".to_string(),
                                         Err(e) => install_status = format!("Error: {}", e),
                                     }
                                 } else {
                                     install_status = "Could not locate current executable.".to_string();
                                 }
                             }

                             ui.add_space(10.0);

                             if ui.button("Try it out").clicked() {
                                 next_mode = Some(AppMode::Clock { preview: true });
                             }
                         });
                     });
                 });

             // MAIN CONTENT
             egui::CentralPanel::default().show(ctx, |ui| {
                 egui::ScrollArea::vertical().show(ui, |ui| {
                     ui.add_space(20.0);
                     match active_tab {
                         SetupTab::General => {
                             ui.heading("Display Selection");
                             ui.label("Choose which monitor plays the screensaver");
                             ui.add_space(10.0);

                             for m in &monitors {
                                 let is_selected = if config.selected_monitor.is_empty() { m.is_primary } else { m.name == config.selected_monitor };
                                 let primary_txt = if m.is_primary { " • Primary" } else { "" };
                                 let label = format!("{} ({}x{}){}", m.name, m.width, m.height, primary_txt);

                                 if ui.selectable_label(is_selected, label).clicked() {
                                     config.selected_monitor = m.name.clone();
                                     save_config(&config);
                                 }
                             }

                             ui.add_space(20.0);
                             ui.separator();
                             ui.add_space(20.0);

                             ui.heading("Clock Behavior");
                             if ui.checkbox(&mut config.use_12h_format, "12-Hour Format").changed() { save_config(&config); }
                             if ui.checkbox(&mut config.show_seconds, "Show Seconds").changed() { save_config(&config); }
                         },
                         SetupTab::Layout => {
                             ui.heading("Dimensions");
                             ui.add_space(10.0);

                             ui.label("Overall Scale (%)");
                             // Scale 20% to 100%
                             let mut scale_pct = config.scale * 100.0;
                             if ui.add(egui::Slider::new(&mut scale_pct, 20.0..=100.0)).changed() {
                                 config.scale = scale_pct / 100.0;
                                 save_config(&config);
                             }

                             ui.label("Card Spacing (%)");
                             let mut spacing_pct = config.spacing * 100.0;
                             if ui.add(egui::Slider::new(&mut spacing_pct, 0.0..=10.0)).changed() {
                                 config.spacing = spacing_pct / 100.0;
                                 save_config(&config);
                             }

                             ui.label("Corner Radius (px)");
                             if ui.add(egui::Slider::new(&mut config.corner_radius, 0.0..=20.0)).changed() {
                                 save_config(&config);
                             }

                             ui.add_space(20.0);
                             ui.heading("Rendering Style");
                             if ui.checkbox(&mut config.pixelated, "Retro Pixelated Mode").changed() {
                                 save_config(&config);
                             }
                         },
                         SetupTab::Theme => {
                             ui.heading("Colors");
                             ui.add_space(10.0);

                             fn color_edit(ui: &mut egui::Ui, label: &str, color: &mut [f32; 3]) -> bool {
                                 let mut rgb = [color[0], color[1], color[2]];
                                 let changed = ui.color_edit_button_rgb(&mut rgb).changed();
                                 if changed {
                                     *color = rgb;
                                 }
                                 ui.label(label);
                                 changed
                             }

                             ui.horizontal(|ui| {
                                 if color_edit(ui, "Background", &mut config.bg_color) { save_config(&config); }
                             });
                             ui.horizontal(|ui| {
                                 if color_edit(ui, "Card Background", &mut config.card_color) { save_config(&config); }
                             });
                             ui.horizontal(|ui| {
                                 if color_edit(ui, "Text / Digits", &mut config.text_color) { save_config(&config); }
                             });

                             ui.add_space(20.0);
                             ui.heading("Animation");
                             ui.label("Flip Duration (ms)");
                             if ui.add(egui::Slider::new(&mut config.animation_speed, 100..=2000)).changed() {
                                 save_config(&config);
                             }
                         }
                     }
                 });
             });
        });

        egui_macroquad::draw();

        if is_key_pressed(KeyCode::Escape) {
            exit_setup = true;
        }

        if exit_setup {
            return None;
        }
        if let Some(nm) = next_mode {
            return Some(nm);
        }

        next_frame().await;
    }
}

async fn run_clock(_preview: bool, font: Option<&Font>) -> bool {
    show_mouse(false);

    #[cfg(windows)]
    { windows_utils::make_window_cover_virtual_screen(); }
    #[cfg(not(windows))]
    { windows_utils::make_window_cover_virtual_screen(); }

    let config = load_config();
    let monitors = windows_utils::get_monitors();
    let virtual_rect = windows_utils::get_virtual_screen_rect();

    let target_monitor = if config.selected_monitor.is_empty() {
        monitors.iter().find(|m| m.is_primary).or(monitors.first())
    } else {
        monitors.iter().find(|m| m.name == config.selected_monitor).or(monitors.first())
    };

    let clock_rect = if let Some(m) = target_monitor {
        let rel_x = (m.x as f32) - virtual_rect.x;
        let rel_y = (m.y as f32) - virtual_rect.y;
        Rect::new(rel_x, rel_y, m.width as f32, m.height as f32)
    } else {
        Rect::new(0.0, 0.0, screen_width(), screen_height())
    };

    let mut time_state = TimeState::new();
    let mouse_init_pos = mouse_position();
    let last_mouse_check = get_time();

    loop {
        if get_last_key_pressed().is_some() {
            return false;
        }
        if get_time() - last_mouse_check > 0.5 {
            let current_pos = mouse_position();
            if (current_pos.0 - mouse_init_pos.0).abs() > 10.0 || (current_pos.1 - mouse_init_pos.1).abs() > 10.0 {
                return false;
            }
        }

        time_state.update(config.use_12h_format);

        // Draw background
        let bg_color = mq_color_from_config(config.bg_color);
        clear_background(bg_color);

        draw_clock_face(&config, &mut time_state, clock_rect, font, false);

        next_frame().await;
    }
}

// -- Helpers --

fn mq_color_from_config(c: [f32; 3]) -> Color {
    Color::new(c[0], c[1], c[2], 1.0)
}

fn draw_clock_face(
    config: &AppConfig,
    time_state: &mut TimeState,
    rect: Rect, // Draw area
    font: Option<&Font>,
    is_preview: bool
) {
    let sw = rect.w;
    let sh = rect.h;

    let base_card_height = sh * 0.4;
    let card_height = base_card_height * config.scale;
    let card_width = card_height * 0.6; // Aspect ratio

    let spacing = card_width * config.spacing; // spacing is relative to card width
    let group_gap = spacing * 3.0;

    // Digits: [H H] : [M M] (: [S S])
    let mut total_cards = 4;
    let mut total_groups_gaps = 1;
    let mut total_spacing = 2; // gaps between HH and MM

    if config.show_seconds {
        total_cards += 2;
        total_groups_gaps += 1;
        total_spacing += 1;
    }

    let total_width = (total_cards as f32 * card_width) + (total_spacing as f32 * spacing) + (total_groups_gaps as f32 * group_gap);

    let start_x = rect.x + (sw - total_width) / 2.0;
    let start_y = rect.y + (sh - card_height) / 2.0;

    let font_size = (card_height * 0.8) as u16;
    let corner_radius = config.corner_radius * (if is_preview { 0.5 } else { 1.0 }); // Scale down radius for preview slightly

    // Animation progress
    let mut progress = 0.0;
    if let Some(start) = time_state.animation_start {
        let elapsed = (get_time() - start) * 1000.0;
        let duration = config.animation_speed as f64;
        progress = (elapsed / duration) as f32;
        if progress >= 1.0 {
            progress = 1.0;
            // Finish animation
            time_state.animation_start = None;
            time_state.previous_digits = time_state.current_digits;
            time_state.previous_seconds = time_state.current_seconds;
        }
    }

    let mut x = start_x;

    let card_color = mq_color_from_config(config.card_color);
    let text_color = mq_color_from_config(config.text_color);

    // Draw Digits
    for (i, &digit) in time_state.current_digits.iter().enumerate() {
        let prev_digit = time_state.previous_digits[i];
        let p = if digit == prev_digit { 1.0 } else { progress };

        draw_single_flip_card(x, start_y, card_width, card_height, digit, prev_digit, p, font, font_size, card_color, text_color, corner_radius);

        x += card_width + spacing;
        if i == 1 {
            // Draw Separator
            draw_separator(x + (group_gap - spacing) / 2.0, start_y, card_height, text_color);
            x += group_gap;
        }
    }

    if config.show_seconds {
        draw_separator(x - group_gap + (group_gap - spacing) / 2.0, start_y, card_height, text_color);

        for (i, &digit) in time_state.current_seconds.iter().enumerate() {
            let prev_digit = time_state.previous_seconds[i];
            let p = if digit == prev_digit { 1.0 } else { progress };

            draw_single_flip_card(x, start_y, card_width, card_height, digit, prev_digit, p, font, font_size, card_color, text_color, corner_radius);

            x += card_width + spacing;
        }
    }
}

fn draw_separator(cx: f32, y: f32, h: f32, color: Color) {
    let dot_size = h * 0.05;
    let gap = h * 0.15;
    let cy = y + h / 2.0;
    draw_circle(cx, cy - gap, dot_size, color);
    draw_circle(cx, cy + gap, dot_size, color);
}

fn draw_single_flip_card(
    x: f32, y: f32, w: f32, h: f32,
    digit: u32, prev_digit: u32,
    progress: f32,
    font: Option<&Font>,
    font_size: u16,
    bg_color: Color,
    text_color: Color,
    radius: f32,
) {
    // Draw Background
    if radius > 0.0 {
        draw_rounded_rectangle(x, y, w, h, radius, bg_color);
    } else {
        draw_rectangle(x, y, w, h, bg_color);
    }

    let display_digit = if progress > 0.5 { digit } else { prev_digit };
    draw_digit_centered(x, y, w, h, display_digit, font, font_size, text_color);

    // Split line
    let mid_y = y + h / 2.0;
    draw_line(x, mid_y, x + w, mid_y, 2.0, Color::new(0.0, 0.0, 0.0, 0.5));

    if progress < 1.0 {
        let flip_y = y + (h * progress);
        draw_line(x, flip_y, x + w, flip_y, 2.0, Color::new(0.0, 0.0, 0.0, 0.3));
    }
}

fn draw_rounded_rectangle(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

fn draw_digit_centered(x: f32, y: f32, w: f32, h: f32, digit: u32, font: Option<&Font>, font_size: u16, color: Color) {
    let text = digit.to_string();
    let dims = measure_text(&text, font, font_size, 1.0);
    let tx = x + (w - dims.width) / 2.0;
    let ty = y + (h - dims.height) / 2.0 + dims.offset_y;

    draw_text_ex(&text, tx, ty, TextParams {
        font,
        font_size,
        color,
        ..Default::default()
    });
}
