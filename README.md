# claude-status-bar (Linux / KDE)

Shows Claude Code activity in the KDE Plasma panel, ported from the macOS project
[m1ckc3s/claude-status-bar](https://github.com/m1ckc3s/claude-status-bar).

## Architecture

Two decoupled layers communicating through files under `~/.claude/statusbar/`,
identical in spirit to the macOS original:

- **Layer A — hooks (Rust `claude-status-bar hook <event>`).** Claude Code fires
  lifecycle/tool events; each runs the binary, which receives a JSON payload on
  stdin and writes **one state file per session** at `sessions.d/<id>.json` (each
  session keeps its own sticky fields — e.g. its own elapsed-timer start), plus an
  aggregate `state.json` (most-recently-active wins) for the tray fallback. Sticky
  fields are loaded from that session's own file, so concurrent sessions never
  clobber each other. `claude-status-bar sessions` prints all live sessions as a
  JSON array (freshest first) for the panel widget. Pure Rust, single binary,
  links only `libc` (+ `libdbus-1` for the optional tray, below).
- **Layer B — UI. Two options, both read the same `state.json`:**
  - **Plasma panel widget (recommended)** — a QML plasmoid in `plasmoid/`. Shows
    the Claude spark **plus an always-visible status label** ("Editing  43s") at
    full panel height, like the macOS menu bar. With several Claude sessions live,
    the bar shows the busiest one with a `+N` badge; **click it for a popup listing
    every session** (project · activity · timer). The popup width auto-matches the
    bar.
  - **System-tray icon (fallback, `claude-status-bar` with no subcommand)** — a
    Rust `ksni`/StatusNotifierItem indicator. Works on any SNI host (XFCE, etc.),
    but a tray icon **cannot show inline text** (SNI has no such field) — the
    status is only in the hover tooltip. Use this only off KDE/where a panel
    widget isn't an option.

`state.json` schema (field names match the macOS app, so the two are
wire-compatible):

```json
{
  "state": "idle|thinking|tool|permission|waiting|done",
  "label": "Editing",
  "tool": "Edit",
  "project": "my-repo",
  "sessionId": "abc123",
  "transcript": "/path/to/transcript.jsonl",
  "startedAt": 1750000000,
  "ts": 1750000000
}
```

### Source layout

| File | Responsibility |
|------|----------------|
| `src/main.rs`    | argument dispatch (`hook` / `install` / `uninstall` / tray) |
| `src/paths.rs`   | filesystem locations |
| `src/state.rs`   | `state.json` schema, atomic load/save, elapsed timer |
| `src/hook.rs`    | Layer A — event → state translation, session tracking |
| `src/icon.rs`    | procedural ARGB32 "spark" rendering for the tray fallback |
| `src/tray.rs`    | Layer B fallback — ksni `Tray` impl + animation/reload loop |
| `src/install.rs` | additive `settings.json` hook merge |
| `plasmoid/metadata.json`        | Plasma 6 applet manifest |
| `plasmoid/contents/ui/main.qml` | Layer B (KDE) — panel widget: spark + inline label |
| `plasmoid/contents/icons/claude.png` | the Claude spark asset |
| `scripts/install-plasmoid.sh`   | deploy plasmoid + add to panel |

## Requirements

- Rust toolchain (build) and `dbus` (runtime).
- KDE Plasma 6 for the panel widget. Tray fallback also works on XFCE/Cinnamon/MATE;
  on GNOME install the `gnome-shell-extension-appindicator` extension first.

## Install via package (Arch / Manjaro)

```sh
makepkg -si          # builds + installs the binary and the plasmoid system-wide
claude-status-bar install   # wire the hooks into ~/.claude/settings.json (per-user)
# then: panel -> Add Widgets -> "Claude Status Bar", and start a new Claude session
```

The package installs `/usr/bin/claude-status-bar` and the plasmoid under
`/usr/share/plasma/plasmoids/`. The hooks and the panel widget are per-user, so
those two steps stay manual.

## Install from source (KDE — dev)

```sh
cargo build --release
./target/release/claude-status-bar install   # Layer A: wire up the hooks
bash scripts/install-plasmoid.sh             # Layer B: deploy + add the panel widget
```

`install` backs up `~/.claude/settings.json` to `settings.json.bak-statusbar` and
additively merges the hooks (existing hooks preserved; re-running is idempotent —
it replaces only its own entries). Then start a **new** Claude Code session, since
hook changes are picked up only by newly started sessions.

`install-plasmoid.sh` installs the KPackage to
`~/.local/share/plasma/plasmoids/com.abyot.claudestatusbar/` (baking the absolute
`state.json` path into the QML) and adds the widget to your first panel. It lands
at the panel's right end — **drag it where you want** (left, for the menu-bar look)
via right-click → Enter Edit Mode.

## Tray fallback (non-KDE)

```sh
./target/release/claude-status-bar &   # StatusNotifierItem tray icon (tooltip only)
```

## Uninstall

```sh
./target/release/claude-status-bar uninstall          # remove hooks
kpackagetool6 -t Plasma/Applet -r com.abyot.claudestatusbar   # remove widget
```

## States & appearance

| state        | trigger (hook)        | panel widget |
|--------------|-----------------------|--------------|
| `thinking`   | UserPromptSubmit / PostToolUse | spinning spark + "Thinking…  m:ss" |
| `tool`       | PreToolUse            | spinning spark + labelled ("Editing", "Running command", …) + timer |
| `permission` | Notification (permission) | spark + "Awaiting permission" |
| `waiting`    | Notification (other)  | spark + "Waiting for you" |
| `done`/`idle`| Stop / no session     | dimmed spark, no text |

Left-click the widget for a small popup with project + elapsed detail.

## Plasma 6 gotchas (learned the hard way)

1. **QML `XMLHttpRequest` cannot read `file://`** without the global
   `QML_XHR_ALLOW_FILE_READ=1` env flag. The widget reads `state.json` via the
   `org.kde.plasma.plasma5support` *executable* datasource (`cat`) instead.
2. **A panel applet with only a `compactRepresentation` renders blank** — Plasma 6
   won't instantiate it. You must also define a `fullRepresentation` (even a small
   popup). This is non-obvious and silent (no error in the journal).
3. The compact representation reads its size from `Layout.*`/implicit sizes; a new
   widget appended to a panel lands at the far right and can be clipped at the
   screen edge until you reposition it.
