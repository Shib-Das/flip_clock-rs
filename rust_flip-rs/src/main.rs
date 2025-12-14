use std::env;

fn run_screensaver() {
    println!("Running...");
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
