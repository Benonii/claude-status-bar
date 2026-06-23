//! Filesystem locations shared by every subcommand.
//!
//! Layout under the user's home (identical to the macOS original so the two are
//! wire-compatible at the state-file level):
//!
//! ```text
//! ~/.claude/settings.json              <- Claude Code hook registration
//! ~/.claude/statusbar/state.json       <- single source of truth the tray reads
//! ~/.claude/statusbar/sessions.d/<id>  <- one empty file per live session
//! ```

use std::path::PathBuf;

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME is not set")
}

pub fn claude_dir() -> PathBuf {
    home().join(".claude")
}

pub fn statusbar_dir() -> PathBuf {
    claude_dir().join("statusbar")
}

pub fn state_file() -> PathBuf {
    statusbar_dir().join("state.json")
}

pub fn sessions_dir() -> PathBuf {
    statusbar_dir().join("sessions.d")
}

/// Per-session state file, keyed by the (already sanitized) session id.
pub fn session_state_file(safe_id: &str) -> PathBuf {
    sessions_dir().join(format!("{safe_id}.json"))
}

pub fn settings_file() -> PathBuf {
    claude_dir().join("settings.json")
}

pub fn settings_backup() -> PathBuf {
    claude_dir().join("settings.json.bak-statusbar")
}

pub fn autostart_file() -> PathBuf {
    home()
        .join(".config")
        .join("autostart")
        .join("claude-status-bar.desktop")
}
