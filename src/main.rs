//! claude-status-bar (Linux/KDE) — a single binary that is both halves of the app:
//!
//!   claude-status-bar                 run the tray (default)
//!   claude-status-bar hook <event>    called by Claude Code; updates state.json
//!   claude-status-bar install         register hooks + autostart
//!   claude-status-bar uninstall       remove them
//!
//! Layer A (hook) and Layer B (tray) communicate only through files under
//! ~/.claude/statusbar/, exactly like the macOS original.

mod hook;
mod icon;
mod install;
mod paths;
mod state;
mod tray;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        None => tray::run(),
        Some("hook") => match args.get(2) {
            Some(event) => hook::run(event),
            None => {
                eprintln!("usage: claude-status-bar hook <event>");
                std::process::exit(2);
            }
        },
        Some("sessions") => hook::print_sessions(),
        Some("install") => install::install(),
        Some("uninstall") => install::uninstall(),
        Some("--help") | Some("-h") => print_help(),
        Some(other) => {
            eprintln!("unknown command '{other}'");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!(
        "claude-status-bar\n\n\
         USAGE:\n\
         \tclaude-status-bar              run the tray icon (default)\n\
         \tclaude-status-bar install      register Claude Code hooks + autostart\n\
         \tclaude-status-bar uninstall    remove hooks + autostart\n\
         \tclaude-status-bar hook <evt>   internal: called by Claude Code hooks\n"
    );
}
