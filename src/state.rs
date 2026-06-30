//! The `state.json` schema — the contract between the hook writer (Layer A) and
//! the tray reader (Layer B). Field names match the macOS project exactly.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Coarse activity classification. Drives the icon colour/animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Activity {
    Idle,
    Thinking,
    Tool,
    Permission,
    Waiting,
    Done,
}

impl Default for Activity {
    fn default() -> Self {
        Activity::Idle
    }
}

impl Activity {
    /// True while Claude is actively working (animated, timer running).
    pub fn is_busy(self) -> bool {
        matches!(self, Activity::Thinking | Activity::Tool)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub state: Activity,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub tool: String,
    #[serde(default)]
    pub project: String,
    #[serde(default, rename = "sessionId")]
    pub session_id: String,
    #[serde(default)]
    pub transcript: String,
    /// Unix seconds the current "busy" stretch began (drives the elapsed timer).
    #[serde(default, rename = "startedAt")]
    pub started_at: u64,
    /// Unix seconds this state was written (freshness / staleness check).
    #[serde(default)]
    pub ts: u64,
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl State {
    /// Best-effort load of the aggregate state. A missing or malformed file
    /// yields the idle default rather than an error.
    pub fn load() -> State {
        State::load_from(&crate::paths::state_file())
    }

    /// Best-effort load from an arbitrary path (e.g. a per-session file).
    pub fn load_from(path: &std::path::Path) -> State {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => State::default(),
        }
    }

    /// Write the aggregate `state.json` (used by the bar and the tray fallback).
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&crate::paths::state_file())
    }

    /// Atomic write to `path`: serialize to a temp file in the same directory,
    /// then rename over the target so a reader never sees a half-written file.
    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join(format!(
            ".tmp.{}.{}",
            std::process::id(),
            path.file_name().and_then(|s| s.to_str()).unwrap_or("state")
        ));
        let json = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into());
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Seconds elapsed in the current busy stretch, or None when not applicable.
    pub fn elapsed(&self) -> Option<u64> {
        if self.state.is_busy() && self.started_at > 0 {
            Some(now_secs().saturating_sub(self.started_at))
        } else {
            None
        }
    }
}
