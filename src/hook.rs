//! Layer A: the hook entry point Claude Code calls.
//! OS-agnostic: Implemented with Rust instead of Node

use crate::state::{Activity, State, now_secs};
use serde_json::Value;
use std::io::Read;

/// Raw CC tool name => human readable label
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

fn thinking_word(seed: &str) -> String {
    #[rustfmt::skip]
    let words = [
        "Thinking", "Pondering", "Cogitating", "Ruminating", "Noodling",
        "Conjuring", "Musing", "Percolating", "Deliberating", "Scheming",
        "Brewing", "Mulling", "Wrangling", "Reticulating", "Synthesizing",
        "Marinating", "Simmering", "Stewing", "Churning", "Cooking",
        "Crafting", "Forging", "Hatching", "Herding", "Hustling",
        "Ideating", "Inferring", "Manifesting", "Moseying", "Puttering",
        "Schlepping", "Spinning", "Transmuting", "Vibing", "Working", "Beno-ing",
    ];
    // Cheap entropy: clock nanos mixed with the session ids
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    let salt: usize = seed.bytes().map(|b| b as usize).sum();
    let word = words[(nanos.wrapping_add(salt)) % words.len()];
    format!("{word}…")
}

/// Sanitize session id into safe filename
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

/// Read entire hook payload, parse as JSON
fn read_payload() -> Value {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    serde_json::from_str(&buf).unwrap_or(Value::Object(Default::default()))
}

fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn project_name_from_cwd(cwd: &str) -> String {
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
        // maintain one state file per live session
        "session-start" => {
            let state = State {
                state: Activity::Idle,
                project: project_name_from_cwd(cwd),
                session_id: session_id.to_string(),
                transcript: transcript.to_string(),
                ts: now_secs(),
                ..Default::default()
            };
            let _ = state.save_to(&session_path);
            return;
        }
        "session-end" => {
            let _ = std::fs::remove_file(&session_path);
            return;
        }
        _ => {}
    }

    // activity events
    let prev_state = State::load_from(&session_path);
    let mut new_state = State {
        session_id: if session_id.is_empty() {
            prev_state.session_id.clone()
        } else {
            session_id.to_string()
        },
        transcript: if transcript.is_empty() {
            prev_state.transcript.clone()
        } else {
            transcript.to_string()
        },
        project: if cwd.is_empty() {
            prev_state.project.clone()
        } else {
            project_name_from_cwd(cwd)
        },
        ts: now_secs(),
        ..Default::default()
    };

    match event {
        "prompt" => {
            new_state.state = Activity::Thinking;
            new_state.label = thinking_word(&sid);
            new_state.started_at = now_secs();
        }
        // About to run a tool.
        "pre" => {
            let tool = str_field(&payload, "tool_name");
            new_state.state = Activity::Tool;
            new_state.label = tool_label(tool).into();
            new_state.tool = tool.to_string();
            // Keep the timer running from the prompt if we already have one.
            new_state.started_at = if prev_state.started_at > 0 {
                prev_state.started_at
            } else {
                now_secs()
            };
        }
        // Tool finished → back to thinking, timer preserved.
        "post" => {
            new_state.state = Activity::Thinking;
            new_state.label = thinking_word(&sid);
            new_state.started_at = if prev_state.started_at > 0 {
                prev_state.started_at
            } else {
                now_secs()
            };
        }
        // Claude is blocked on the user (permission prompt or idle wait).
        "notify" => {
            let msg = str_field(&payload, "message").to_lowercase();
            if msg.contains("permission") || msg.contains("approve") {
                new_state.state = Activity::Permission;
                new_state.label = "Awaiting permission".into();
            } else {
                new_state.state = Activity::Waiting;
                new_state.label = "Waiting for you".into();
            }
            new_state.started_at = 0;
        }
        // Turn finished.
        "stop" => {
            new_state.state = Activity::Done;
            new_state.label = "Done".into();
            new_state.started_at = 0;
        }
        other => {
            eprintln!("claude-status-bar: unknown hook event '{other}'");
            return;
        }
    }

    // Per-session file feeds the popup's session list; the aggregate feeds the
    // bar (most-recently-active wins, since this event just fired) and the tray.
    let _ = new_state.save_to(&session_path);
    let _ = new_state.save();
}
