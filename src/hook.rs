//! Layer A: the hook entry point Claude Code calls.
//!
//! Claude Code fires a hook, runs `claude-status-bar hook <event>`, and pipes a
//! JSON object to our stdin (session_id, cwd, transcript_path, tool_name, …).
//! We translate that event into an updated `state.json` and/or a `sessions.d`
//! file. Everything here is OS-agnostic — it is the exact same data pipeline the
//! macOS app uses, just reimplemented in Rust instead of Node.

use crate::state::{now_secs, Activity, State};
use serde_json::Value;
use std::io::Read;

/// Map a raw Claude Code tool name to a human label (mirrors the original table).
fn tool_label(tool: &str) -> &'static str {
    match tool {
        "Bash" => "Running command",
        "Edit" | "MultiEdit" | "NotebookEdit" | "Write" => "Editing",
        "Read" => "Reading",
        "Grep" | "Glob" => "Searching",
        "WebFetch" => "Browsing web",
        "WebSearch" => "Searching web",
        "Task" => "Delegating",
        "TodoWrite" | "TaskCreate" | "TaskUpdate" => "Planning",
        _ => "Using tool",
    }
}

/// Sanitize a session id into a safe filename: keep `[A-Za-z0-9._-]`, cap at 64.
fn safe_id(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
        .take(64)
        .collect();
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

/// Read the entire hook payload from stdin, parse as JSON (empty object on error).
fn read_payload() -> Value {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    serde_json::from_str(&buf).unwrap_or(Value::Object(Default::default()))
}

fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Derive a project name from a cwd path (its last path component).
fn project_from_cwd(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn run(event: &str) {
    let payload = read_payload();
    let session_id = str_field(&payload, "session_id");
    let cwd = str_field(&payload, "cwd");
    let transcript = str_field(&payload, "transcript_path");

    let sid = safe_id(session_id);
    let session_path = crate::paths::session_state_file(&sid);

    match event {
        // ---- session lifecycle: maintain one state file per live session -----
        "session-start" => {
            let s = State {
                state: Activity::Idle,
                project: project_from_cwd(cwd),
                session_id: session_id.to_string(),
                transcript: transcript.to_string(),
                ts: now_secs(),
                ..Default::default()
            };
            let _ = s.save_to(&session_path);
            return;
        }
        "session-end" => {
            let _ = std::fs::remove_file(&session_path);
            return;
        }
        _ => {}
    }

    // ---- activity events: rewrite this session's state, preserving its own ----
    // sticky fields (timer, project) — loaded from the per-session file, NOT the
    // shared aggregate, so concurrent sessions don't clobber each other's timers.
    let prev = State::load_from(&session_path);
    let mut s = State {
        session_id: if session_id.is_empty() {
            prev.session_id.clone()
        } else {
            session_id.to_string()
        },
        transcript: if transcript.is_empty() {
            prev.transcript.clone()
        } else {
            transcript.to_string()
        },
        project: if cwd.is_empty() {
            prev.project.clone()
        } else {
            project_from_cwd(cwd)
        },
        ts: now_secs(),
        ..Default::default()
    };

    match event {
        // User submitted a prompt → start of a busy stretch; (re)start the timer.
        "prompt" => {
            s.state = Activity::Thinking;
            s.label = "Thinking…".into();
            s.started_at = now_secs();
        }
        // About to run a tool.
        "pre" => {
            let tool = str_field(&payload, "tool_name");
            s.state = Activity::Tool;
            s.label = tool_label(tool).into();
            s.tool = tool.to_string();
            // Keep the timer running from the prompt if we already have one.
            s.started_at = if prev.started_at > 0 {
                prev.started_at
            } else {
                now_secs()
            };
        }
        // Tool finished → back to thinking, timer preserved.
        "post" => {
            s.state = Activity::Thinking;
            s.label = "Thinking…".into();
            s.started_at = if prev.started_at > 0 {
                prev.started_at
            } else {
                now_secs()
            };
        }
        // Claude is blocked on the user (permission prompt or idle wait).
        "notify" => {
            let msg = str_field(&payload, "message").to_lowercase();
            if msg.contains("permission") || msg.contains("approve") {
                s.state = Activity::Permission;
                s.label = "Awaiting permission".into();
            } else {
                s.state = Activity::Waiting;
                s.label = "Waiting for you".into();
            }
            s.started_at = 0;
        }
        // Turn finished.
        "stop" => {
            s.state = Activity::Done;
            s.label = "Done".into();
            s.started_at = 0;
        }
        other => {
            eprintln!("claude-status-bar: unknown hook event '{other}'");
            return;
        }
    }

    // Per-session file feeds the popup's session list; the aggregate feeds the
    // bar (most-recently-active wins, since this event just fired) and the tray.
    let _ = s.save_to(&session_path);
    let _ = s.save();
}
