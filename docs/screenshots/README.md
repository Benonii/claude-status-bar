# Screenshots

These three files are referenced by the root `README.md`:

| File | What to capture |
|------|-----------------|
| `plasma-bar.png`   | The KDE Plasma 6 panel widget inline — spark + `web-frontend · Editing  43s   +2`. |
| `plasma-popup.png` | The click-to-expand popup listing the three posed sessions (project · activity · timer). |
| `tray.png`         | The system-tray fallback icon (the Claude spark) with its tooltip, on KDE or any SNI host. |
| `demo.gif`         | The hero clip — the bar animating through activities. Record with `scripts/demo-pose.sh loop` + Spectacle, convert with `scripts/gif-from.sh` (see below). |

## Capturing them (Arch / KDE)

Pose a clean, reproducible scene with [`scripts/demo-pose.sh`](../../scripts/demo-pose.sh)
— it writes fake session state and **hides your real sessions** so the shots stay tidy:

```sh
scripts/demo-pose.sh                       # pose the 3-session scene
```

Then capture each (Spectacle region mode = `spectacle -r`, or PrintScreen → Rectangular Region):

1. **`plasma-bar.png`** — region around the panel widget.
   Shortcut: `scripts/demo-pose.sh --shot docs/screenshots/plasma-bar.png` poses + opens Spectacle for you.
2. **`plasma-popup.png`** — left-click the widget to open the popup, then region-capture it.
3. **`tray.png`** — run `claude-status-bar &`, expand the system tray (˄), region-capture the Claude icon + tooltip. This is the GNOME/XFCE look too — the tray renders identically everywhere.

Restore your real sessions when done:

```sh
scripts/demo-pose.sh --clean
kill %1 2>/dev/null    # if you started the tray
```

Tips: capture at 2× scale for crisp panels, crop tight, keep each under ~300 KB.

## Demo GIF (`demo.gif`)

```sh
scripts/demo-pose.sh loop                         # animate the bar (Ctrl-C restores)
spectacle -R region                               # drag around the bar → record ~12s → save .webm
scripts/gif-from.sh ~/claude-demo.webm demo.gif   # two-pass palette → optimized GIF
# Ctrl-C the loop when done
```

`gif-from.sh INPUT [OUTPUT] [MAX_WIDTH] [FPS]` never upscales; defaults 760px / 15 fps.
