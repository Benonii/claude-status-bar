# Using claude-status-bar with any panel

The binary exposes three read commands, so almost any status bar can show Claude
activity. Assume `claude-status-bar` is on your `PATH` (it is after `install.sh`,
or use the absolute path).

| command                    | output                              | use with |
|----------------------------|-------------------------------------|----------|
| `claude-status-bar status` | one plain-text line (empty if idle) | polybar, i3blocks, xfce4-genmon, tint2, dwmblocks, eww |
| `claude-status-bar waybar` | Waybar JSON (`text`/`tooltip`/`class`) | Waybar (sway, Hyprland, wlroots) |
| `claude-status-bar sessions` | JSON array of all sessions        | custom scripts, the KDE plasmoid |

Example `status` output: `✳ web-frontend · Editing 43s (+2)`

## Waybar (sway / Hyprland / wlroots)

`~/.config/waybar/config`:

```json
"custom/claude": {
    "exec": "claude-status-bar waybar",
    "return-type": "json",
    "interval": 1,
    "tooltip": true
}
```

Add `"custom/claude"` to a `modules-left/center/right` array. Style by state in
`style.css`:

```css
#custom-claude.busy       { color: #d97757; }
#custom-claude.permission { color: #e6b800; }
#custom-claude.waiting    { color: #e6b800; }
#custom-claude.idle       { opacity: 0.6; }
```

## Polybar

```ini
[module/claude]
type = custom/script
exec = claude-status-bar status
interval = 1
```

Add `claude` to one of your `modules-*` lines.

## i3blocks

`~/.config/i3blocks/config`:

```ini
[claude]
command=claude-status-bar status
interval=1
```

## XFCE (xfce4-genmon-plugin)

Add a "Generic Monitor" panel item, set the command to:

```sh
echo "<txt>$(claude-status-bar status)</txt>"
```

with a 1s period. (Install `xfce4-genmon-plugin` first.)

## tint2 / dwmblocks / eww / others

Anything that runs a command on a timer can use `claude-status-bar status`.
It prints one line and exits; empty output means no active session.

## System-tray icon (any SNI-capable desktop)

If you'd rather have a tray icon than inline text (XFCE, Cinnamon, MATE, LXQt,
GNOME-with-AppIndicator, Waybar's `tray` module):

```sh
claude-status-bar &     # or let install.sh add it to ~/.config/autostart
```

The icon shows the spark; status is in the hover tooltip (the StatusNotifierItem
protocol has no inline-text field — that's why KDE uses a panel widget instead).
