#!/usr/bin/env bash
# Universal, no-root installer. Works on any distro (Arch, Debian/Ubuntu, Fedora…).
# It installs the binary into ~/.local/bin, wires the Claude Code hooks, and sets up
# the best status UI it can detect for your desktop:
#
#   KDE Plasma 6   -> inline panel widget (full experience)
#   Waybar         -> prints a ready-to-paste module (inline text)
#   GNOME / other  -> system-tray icon via autostart (tooltip only), + tips
#
# Re-running is safe.
set -euo pipefail

REPO="$(cd "$(dirname "$0")" && pwd)"
BIN_NAME="claude-status-bar"
LOCALBIN="$HOME/.local/bin"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }

# --- 1. get a binary -----------------------------------------------------------
if [ -x "$REPO/target/release/$BIN_NAME" ]; then
    SRC_BIN="$REPO/target/release/$BIN_NAME"
elif command -v cargo >/dev/null 2>&1; then
    say "Building (cargo build --release)…"
    ( cd "$REPO" && cargo build --release )
    SRC_BIN="$REPO/target/release/$BIN_NAME"
else
    echo "Need either a prebuilt target/release/$BIN_NAME or 'cargo' to build it." >&2
    echo "Install Rust:  https://rustup.rs   (or your distro's 'rust'/'cargo' package)" >&2
    exit 1
fi

# --- 2. install binary to ~/.local/bin ----------------------------------------
mkdir -p "$LOCALBIN"
install -m755 "$SRC_BIN" "$LOCALBIN/$BIN_NAME"
BIN="$LOCALBIN/$BIN_NAME"
say "Installed binary: $BIN"
case ":$PATH:" in
    *":$LOCALBIN:"*) ;;
    *) say "Add ~/.local/bin to your PATH (e.g. in ~/.profile or ~/.bashrc):"
       echo "      export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac

# --- 3. wire Claude Code hooks -------------------------------------------------
say "Wiring Claude Code hooks…"
"$BIN" install

# --- 4. set up the best UI for this desktop -----------------------------------
DE="$(printf '%s' "${XDG_CURRENT_DESKTOP:-}" | tr '[:upper:]' '[:lower:]')"
say "Desktop: ${XDG_CURRENT_DESKTOP:-unknown}"

setup_tray_autostart() {
    local dir="$HOME/.config/autostart"
    mkdir -p "$dir"
    cat > "$dir/$BIN_NAME.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Claude Status Bar
Comment=Claude Code activity in the system tray
Exec=$BIN
Terminal=false
X-GNOME-Autostart-enabled=true
Categories=Utility;
EOF
    # start it now too
    ( setsid "$BIN" >/dev/null 2>&1 < /dev/null & ) || true
    say "System-tray icon set to autostart (and started now)."
}

if printf '%s' "$DE" | grep -q kde; then
    CSB_BIN="$BIN" bash "$REPO/scripts/install-plasmoid.sh" \
        && say "KDE panel widget installed — if it didn't auto-add, right-click the panel → Add Widgets → 'Claude Status Bar'."
elif command -v waybar >/dev/null 2>&1; then
    say "Waybar detected. Add a custom module to your ~/.config/waybar/config:"
    cat <<EOF

    "custom/claude": {
        "exec": "$BIN waybar",
        "return-type": "json",
        "interval": 1,
        "tooltip": true
    }

  …and put "custom/claude" in one of your modules-* arrays. See packaging/waybar/.
EOF
elif printf '%s' "$DE" | grep -q gnome; then
    say "GNOME: for a tray icon, install the 'AppIndicator and KStatusNotifierItem Support'"
    say "extension (then re-login). Setting up the tray now:"
    setup_tray_autostart
    say "Note: GNOME's top bar can't show always-visible inline text without a custom"
    say "extension — the tray icon shows status in its tooltip."
else
    say "Setting up the system-tray fallback (works on XFCE, Cinnamon, MATE, LXQt, …)."
    setup_tray_autostart
fi

say "Done. Start a NEW Claude Code session for the hooks to take effect."
