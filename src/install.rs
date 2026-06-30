//! `install` / `uninstall`: wire our hooks into `~/.claude/settings.json` and set
//! up a KDE autostart entry. The merge is additive and idempotent — it backs the
//! file up first and never disturbs hooks it didn't create.

use serde_json::{Map, Value, json};

/// (Claude Code event name, our `hook` subcommand argument).
const EVENTS: &[(&str, &str)] = &[
    ("SessionStart", "session-start"),
    ("SessionEnd", "session-end"),
    ("UserPromptSubmit", "prompt"),
    ("PreToolUse", "pre"),
    ("PostToolUse", "post"),
    ("Notification", "notify"),
    ("Stop", "stop"),
];

fn exe_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.canonicalize().ok())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "claude-status-bar".into())
}

/// True if a Claude Code hook entry's command refers to our binary.
fn is_ours(entry: &Value) -> bool {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .map(|hooks| {
            hooks.iter().any(|h| {
                h.get("command")
                    .and_then(Value::as_str)
                    .map(|c| c.contains("claude-status-bar"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn read_settings() -> Map<String, Value> {
    match std::fs::read_to_string(crate::paths::settings_file()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Map::new(),
    }
}

fn write_settings(settings: &Map<String, Value>) -> std::io::Result<()> {
    let _ = std::fs::create_dir_all(crate::paths::claude_dir());
    let json = serde_json::to_string_pretty(&Value::Object(settings.clone())).unwrap();
    std::fs::write(crate::paths::settings_file(), json)
}

pub fn install() {
    let exe = exe_path();
    let mut settings = read_settings();

    // Back up the original once.
    if let Ok(text) = std::fs::read_to_string(crate::paths::settings_file()) {
        let _ = std::fs::write(crate::paths::settings_backup(), text);
    }

    // Ensure settings["hooks"] is an object.
    let hooks = settings
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks = hooks.as_object_mut().expect("hooks must be an object");

    for (event, arg) in EVENTS {
        let entry = json!({
            "matcher": "",
            "hooks": [ { "type": "command", "command": format!("{exe} hook {arg}") } ]
        });

        let arr = hooks
            .entry(*event)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .expect("event value must be an array");

        // Drop any prior entry of ours (idempotent re-install), keep the rest.
        arr.retain(|e| !is_ours(e));
        arr.push(entry);
    }

    if let Err(e) = write_settings(&settings) {
        eprintln!("failed to write settings.json: {e}");
        std::process::exit(1);
    }

    println!("Installed claude-status-bar hooks into {}", crate::paths::settings_file().display());
    println!("Backup written to {}", crate::paths::settings_backup().display());
    println!("Next (KDE panel widget): run scripts/install-plasmoid.sh");
    println!("(Or, for a system-tray fallback instead of the panel widget: {exe} &)");
}

pub fn uninstall() {
    let mut settings = read_settings();
    if let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) {
        for (event, _) in EVENTS {
            if let Some(arr) = hooks.get_mut(*event).and_then(Value::as_array_mut) {
                arr.retain(|e| !is_ours(e));
            }
        }
        // Tidy up any event arrays we emptied.
        hooks.retain(|_, v| v.as_array().map(|a| !a.is_empty()).unwrap_or(true));
    }
    let _ = write_settings(&settings);
    let _ = std::fs::remove_file(crate::paths::autostart_file());
    println!("Removed claude-status-bar hooks (and any legacy autostart entry).");
}
