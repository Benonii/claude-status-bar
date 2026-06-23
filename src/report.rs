//! Read the per-session state files and render them in the format a given
//! consumer wants. This is the portability layer: the same session data feeds
//! the KDE plasmoid (`sessions` → JSON array), any text panel (`status` → one
//! line) and Waybar (`waybar` → its JSON object). Anything that can run a
//! command on an interval (polybar, i3blocks, xfce4-genmon, tint2, …) can use
//! `status`.

use crate::state::{now_secs, Activity, State};

/// A session whose last event is older than this is treated as crashed (no
/// `session-end` fired) and pruned, so lists never accumulate orphans.
const STALE_SECS: u64 = 12 * 3600;

/// A session claiming to be busy but silent for this long has almost certainly
/// had its terminal closed or Claude killed mid-turn (so no `stop`/`session-end`
/// ever fired). We demote it to idle so a dead session can't pin the bar to a
/// forever-ticking "Thinking… 35m". A genuine turn fires hooks at every tool
/// boundary, so this only catches the abandoned ones.
const BUSY_STALE_SECS: u64 = 10 * 60;

/// Default text glyph standing in for the Claude spark on text-only panels.
const GLYPH: &str = "✳";

/// Load every live session (freshest first), pruning stale files as we go.
pub fn load_sessions() -> Vec<State> {
    let now = now_secs();
    let dir = crate::paths::sessions_dir();
    let mut sessions: Vec<State> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let mut s = State::load_from(&p);
            if s.ts > 0 && now.saturating_sub(s.ts) > STALE_SECS {
                let _ = std::fs::remove_file(&p);
                continue;
            }
            // Abandoned-but-not-yet-pruned: still a live session, just not busy.
            if s.state.is_busy() && now.saturating_sub(s.ts) > BUSY_STALE_SECS {
                s.state = Activity::Idle;
                s.label = String::new();
                s.started_at = 0;
            }
            sessions.push(s);
        }
    }
    sessions.sort_by(|a, b| b.ts.cmp(&a.ts));
    sessions
}

fn is_busy(s: &State) -> bool {
    s.state.is_busy()
}
fn needs_user(s: &State) -> bool {
    matches!(s.state, Activity::Permission | Activity::Waiting)
}

/// The session the bar represents: busy > awaiting-user > freshest.
fn pick_current(sessions: &[State]) -> Option<&State> {
    sessions
        .iter()
        .find(|s| is_busy(s))
        .or_else(|| sessions.iter().find(|s| needs_user(s)))
        .or_else(|| sessions.first())
}

fn project_of(s: &State) -> &str {
    if s.project.is_empty() {
        "Claude"
    } else {
        &s.project
    }
}

fn fmt_elapsed(s: &State) -> String {
    if is_busy(s) && s.started_at > 0 {
        let sec = now_secs().saturating_sub(s.started_at);
        let (m, ss) = (sec / 60, sec % 60);
        if m > 0 {
            format!("{m}m {ss}s")
        } else {
            format!("{ss}s")
        }
    } else {
        String::new()
    }
}

fn activity_phrase(s: &State) -> String {
    let e = fmt_elapsed(s);
    if e.is_empty() {
        s.label.clone()
    } else {
        format!("{} {e}", s.label)
    }
}

/// One-line human status, e.g. "✳ web-frontend · Editing 43s (+2)".
/// Empty string when there are no sessions (so panels can hide the module).
fn one_line(sessions: &[State]) -> String {
    let cur = match pick_current(sessions) {
        Some(c) => c,
        None => return String::new(),
    };
    let extra = sessions.len().saturating_sub(1);
    let who = project_of(cur);
    let mut body = match cur.state {
        Activity::Idle | Activity::Done => who.to_string(),
        _ => {
            let act = activity_phrase(cur);
            if act.trim().is_empty() {
                who.to_string()
            } else {
                format!("{who} · {act}")
            }
        }
    };
    if extra > 0 {
        body.push_str(&format!(" (+{extra})"));
    }
    format!("{GLYPH} {body}")
}

/// CSS-ish class for the current state — lets Waybar/others style by activity.
fn current_class(sessions: &[State]) -> &'static str {
    match pick_current(sessions).map(|c| c.state) {
        Some(Activity::Thinking) | Some(Activity::Tool) => "busy",
        Some(Activity::Permission) => "permission",
        Some(Activity::Waiting) => "waiting",
        _ => "idle",
    }
}

fn tooltip(sessions: &[State]) -> String {
    if sessions.is_empty() {
        return "No active Claude sessions".to_string();
    }
    sessions
        .iter()
        .map(|s| {
            let act = activity_phrase(s);
            if act.trim().is_empty() {
                project_of(s).to_string()
            } else {
                format!("{}: {act}", project_of(s))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---- subcommand entry points -------------------------------------------------

/// `sessions` — JSON array of all live sessions (consumed by the KDE plasmoid).
pub fn print_sessions() {
    let sessions = load_sessions();
    println!(
        "{}",
        serde_json::to_string(&sessions).unwrap_or_else(|_| "[]".into())
    );
}

/// `status` — a single plain-text line for polybar / i3blocks / genmon / tint2 / etc.
pub fn print_status() {
    let line = one_line(&load_sessions());
    if !line.is_empty() {
        println!("{line}");
    }
}

/// `waybar` — Waybar custom-module JSON (`text`, `tooltip`, `class`).
pub fn print_waybar() {
    let sessions = load_sessions();
    let obj = serde_json::json!({
        "text": one_line(&sessions),
        "tooltip": tooltip(&sessions),
        "class": current_class(&sessions),
        "alt": current_class(&sessions),
    });
    println!("{obj}");
}
