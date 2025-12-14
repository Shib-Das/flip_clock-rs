use std::env;
use sdl2::event::Event;
use sdl2::pixels::Color;

fn run_screensaver() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust_flip-rs", 800, 600)
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    sdl_context.mouse().show_cursor(false);

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Get initial mouse position to handle jitter
    let mouse_state = event_pump.mouse_state();
    let initial_x = mouse_state.x();
    let initial_y = mouse_state.y();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown { .. } => {
                     break 'running
                },
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
        canvas.present();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // If no arguments are provided (only program name), default to /s (Show)
    if args.len() <= 1 {
        run_screensaver();
        return;
    }

    // Windows screensaver arguments are usually case-insensitive.
    // However, the requirement specifically mentions /s, /c, /p.
    // We will check if it starts with them.
    let arg = args[1].to_lowercase();

    if arg.starts_with("/s") {
        run_screensaver();
    } else if arg.starts_with("/c") {
        // Config mode - just exit cleanly for now
    } else if arg.starts_with("/p") {
        // Preview mode - just exit cleanly for now
    }
    // Any other argument: do nothing/exit
}
