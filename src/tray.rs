//! Layer B: the KDE/Plasma tray icon, via the StatusNotifierItem D-Bus protocol
//! (Plasma supports it natively, so `ksni` talks straight to the panel — no GTK).
//!
//! Design: ksni runs its own thread (`service.spawn()`); our main thread is a
//! ~8 Hz loop that (a) reloads `state.json` when its mtime changes and (b) bumps
//! a frame counter so the icon animates. Each tick pushes the current snapshot
//! into the tray via `handle.update`, which makes ksni re-query our `Tray` impl.

use crate::icon::claude_icon;
use crate::state::{Activity, State};
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Tray, TrayService};
use std::time::{Duration, SystemTime};

const ICON_SIZE: i32 = 22;

#[derive(Default)]
struct StatusTray {
    state: State,
    frame: u64,
}

/// Map (activity, frame) → icon opacity. The Claude spark itself never changes;
/// only how brightly it shows. Busy pulses, permission blinks, idle dims.
fn icon_alpha(state: &State, frame: u64) -> f32 {
    // Oscillator in 0..1; `speed` sets the period.
    let osc = |speed: f32| ((frame as f32 * speed).sin() + 1.0) / 2.0;

    match state.state {
        // Working: brisk bright pulse (tool use a little faster than thinking).
        Activity::Tool => 0.80 + 0.20 * osc(0.8),
        Activity::Thinking => 0.82 + 0.18 * osc(0.5),
        // Blocked on the user: slow blink to draw the eye.
        Activity::Permission => 0.35 + 0.65 * osc(0.45),
        Activity::Waiting => 0.7,
        // Nothing happening: dim, static.
        Activity::Idle | Activity::Done => 0.55,
    }
}

/// Build the one-line status string shown in tooltip and menu header.
fn status_line(state: &State) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !state.label.is_empty() {
        parts.push(state.label.clone());
    }
    if !state.project.is_empty() {
        parts.push(state.project.clone());
    }
    if let Some(secs) = state.elapsed() {
        parts.push(format!("{}:{:02}", secs / 60, secs % 60));
    }
    if parts.is_empty() {
        "Claude — idle".to_string()
    } else {
        parts.join("  ·  ")
    }
}

impl Tray for StatusTray {
    fn id(&self) -> String {
        "claude-status-bar".into()
    }

    fn title(&self) -> String {
        "Claude".into()
    }

    // Empty name forces hosts to use our pixmap.
    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![claude_icon(ICON_SIZE, icon_alpha(&self.state, self.frame))]
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Claude".into(),
            description: status_line(&self.state),
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: status_line(&self.state),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_: &mut Self| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn state_mtime() -> Option<SystemTime> {
    std::fs::metadata(crate::paths::state_file())
        .ok()
        .and_then(|m| m.modified().ok())
}

/// Is this state visually animated? Static states don't need per-frame redraws.
fn is_animated(a: Activity) -> bool {
    a.is_busy() || a == Activity::Permission
}

pub fn run() {
    let service = TrayService::new(StatusTray::default());
    let handle = service.handle();
    service.spawn();

    let mut last_mtime: Option<SystemTime> = None;
    let mut current = State::load();
    let mut frame: u64 = 0;
    // Force an initial paint so the icon settles to the current state on launch.
    let mut prev_kind: Option<Activity> = None;

    loop {
        let mtime = state_mtime();
        let changed = mtime != last_mtime;
        if changed {
            last_mtime = mtime;
            current = State::load();
        }

        // Push to the panel only when the state changed or when we're animating —
        // an idle tray then emits zero D-Bus traffic.
        let state_kind = current.state;
        let must_paint = changed || prev_kind != Some(state_kind) || is_animated(state_kind);
        if must_paint {
            prev_kind = Some(state_kind);
            frame = frame.wrapping_add(1);
            let snapshot = current.clone();
            let f = frame;
            handle.update(move |t: &mut StatusTray| {
                t.state = snapshot.clone();
                t.frame = f;
            });
        }

        std::thread::sleep(Duration::from_millis(120));
    }
}
