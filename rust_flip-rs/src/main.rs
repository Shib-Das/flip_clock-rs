use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::gfx::primitives::DrawRenderer;
use chrono::{Local, Timelike};

struct TimeState {
    current_digits: [u32; 4],
    previous_digits: [u32; 4],
    animation_start: Option<Instant>,
}

struct FlipClockRenderer<'a> {
    digit_textures: Vec<Texture<'a>>,
    card_width: i16,
    card_height: i16,
}

impl<'a> FlipClockRenderer<'a> {
    fn draw_digit_content(
        &self,
        canvas: &mut Canvas<Window>,
        x: i16,
        y: i16,
        number: u32,
    ) -> Result<(), String> {
        let texture = &self.digit_textures[number as usize];
        let query = texture.query();
        let w = query.width;
        let h = query.height;
        let width = self.card_width;
        let height = self.card_height;
        let center_x = x as i32 + width as i32 / 2;
        let center_y = y as i32 + height as i32 / 2;
        let target = Rect::new(center_x - w as i32 / 2, center_y - h as i32 / 2, w, h);
        canvas.copy(texture, None, target)?;
        Ok(())
    }

    fn draw_card(
        &self,
        canvas: &mut Canvas<Window>,
        x: i16,
        y: i16,
        number: u32,
        prev_number: u32,
        progress: f32,
    ) -> Result<(), String> {
        let width = self.card_width;
        let height = self.card_height;

        // If static or progress complete
        if progress >= 1.0 || number == prev_number {
            // Draw background card (Dark Grey)
            canvas.rounded_box(x, y, x + width, y + height, 10, Color::RGB(40, 40, 40))?;
            // Draw digit
            self.draw_digit_content(canvas, x, y, number)?;
        } else {
            // Animation: "Slide down" / Wipe effect
            // 1. Draw Previous Digit Fully (Background)
            canvas.rounded_box(x, y, x + width, y + height, 10, Color::RGB(40, 40, 40))?;
            self.draw_digit_content(canvas, x, y, prev_number)?;

            // 2. Draw New Digit (Foreground) with clipping
            // Wipe from Top to Bottom
            let wipe_height = (height as f32 * progress) as u32;
            let clip_rect = Rect::new(x as i32, y as i32, width as u32, wipe_height);

            canvas.set_clip_rect(clip_rect);

            // Redraw background for the new part (to cover old digit parts)
            canvas.rounded_box(x, y, x + width, y + height, 10, Color::RGB(40, 40, 40))?;
            self.draw_digit_content(canvas, x, y, number)?;

            canvas.set_clip_rect(None);
        }

        // Draw horizontal split line (thick black line)
        let mid_y = y + height / 2;
        // box_ coordinates are inclusive
        canvas.box_(x, mid_y - 2, x + width, mid_y + 2, Color::BLACK)?;

        Ok(())
    }
}

fn run_screensaver() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window("rust_flip-rs", 800, 600)
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();

    sdl_context.mouse().show_cursor(false);

    // Font loading strategy
    let mut font_path = PathBuf::from("assets/fonts/Roboto-Bold.ttf");
    if !font_path.exists() {
         if let Ok(exe_path) = env::current_exe() {
             let p = exe_path.parent().unwrap().join("assets/fonts/Roboto-Bold.ttf");
             let p2 = exe_path.parent().unwrap().parent().unwrap().parent().unwrap().join("assets/fonts/Roboto-Bold.ttf");

             if p.exists() {
                 font_path = p;
             } else if p2.exists() {
                 font_path = p2;
             }
         }
    }

    // Dynamically calculate font size based on screen height
    let (w_u32, h_u32) = canvas.output_size().unwrap();

    // Card height is 40% of screen height
    let card_height = (h_u32 as f32 * 0.4) as i16;
    let card_width = (w_u32 as f32 * 0.15) as i16;

    // Font size should be slightly smaller than card height, say 80% of card height
    let font_size = (card_height as f32 * 0.8) as u16;

    let font = ttf_context.load_font(&font_path, font_size).expect("Failed to load font. Make sure assets/fonts/Roboto-Bold.ttf exists.");

    // Pre-render numbers 0-9
    let mut digit_textures: Vec<Texture> = Vec::with_capacity(10);
    for i in 0..10 {
        let text = i.to_string();
        let surface = font.render(&text)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string()).unwrap();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();
        digit_textures.push(texture);
    }

    let renderer = FlipClockRenderer {
        digit_textures,
        card_width,
        card_height,
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mouse_state = event_pump.mouse_state();
    let initial_x = mouse_state.x();
    let initial_y = mouse_state.y();

    // Layout calculations
    let w = w_u32 as i16;
    let h = h_u32 as i16;
    let spacing = (w_u32 as f32 * 0.02) as i16;
    let group_gap = spacing * 3;
    let total_width = 4 * card_width + 2 * spacing + group_gap;
    let start_x = (w - total_width) / 2;
    let start_y = (h - card_height) / 2;

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

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                Event::KeyDown { .. } => break 'running,
                Event::MouseMotion { x, y, .. } => {
                    if (x - initial_x).abs() > 10 || (y - initial_y).abs() > 10 {
                        break 'running;
                    }
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.clear();

        // Get time
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();

        let h1 = hour / 10;
        let h2 = hour % 10;
        let m1 = minute / 10;
        let m2 = minute % 10;

        let new_digits = [h1, h2, m1, m2];

        if new_digits != time_state.current_digits {
            time_state.previous_digits = time_state.current_digits;
            time_state.current_digits = new_digits;
            time_state.animation_start = Some(Instant::now());
        }

        let mut progress = 0.0;
        if let Some(start) = time_state.animation_start {
            let elapsed = start.elapsed().as_millis() as f32;
            let duration = 600.0; // Animation duration in ms
            progress = elapsed / duration;
            if progress >= 1.0 {
                progress = 1.0;
                time_state.animation_start = None;
                // Once animation is done, previous becomes current to stop triggering animation logic
                time_state.previous_digits = time_state.current_digits;
            }
        }

        let mut x_offset = start_x;

        for (i, &digit) in time_state.current_digits.iter().enumerate() {
            let prev_digit = time_state.previous_digits[i];

            // If this specific digit didn't change, we treat it as static
            let digit_progress = if digit == prev_digit { 1.0 } else { progress };

            renderer.draw_card(&mut canvas, x_offset, start_y, digit, prev_digit, digit_progress).unwrap();

            x_offset += card_width + spacing;
            if i == 1 {
                x_offset += group_gap - spacing;
            }
        }

        canvas.present();

        // Cap framerate slightly
        std::thread::sleep(Duration::from_millis(16));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        run_screensaver();
        return;
    }

    let arg = args[1].to_lowercase();

    if arg.starts_with("/s") {
        run_screensaver();
    } else if arg.starts_with("/c") {
    } else if arg.starts_with("/p") {
    }
}
