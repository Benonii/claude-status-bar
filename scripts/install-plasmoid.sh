#!/usr/bin/env bash
# Deploy the Plasma 6 plasmoid (Layer B) and add it to the primary panel.
# Idempotent: re-running upgrades the package in place.
set -euo pipefail

ID="com.abyot.claudestatusbar"
SRC_DIR="$(cd "$(dirname "$0")/../plasmoid" && pwd)"
BIN="$(cd "$(dirname "$0")/.." && pwd)/target/release/claude-status-bar"
SESSIONS_CMD="$BIN sessions"

# Stage a copy with the absolute 'sessions' command baked in and the spark bundled.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cp -r "$SRC_DIR/." "$TMP/"
sed -i "s|__SESSIONS_CMD__|$SESSIONS_CMD|g" "$TMP/contents/ui/main.qml"
mkdir -p "$TMP/contents/icons"
[ -f "$HOME/.claude/claude-icon.png" ] && cp "$HOME/.claude/claude-icon.png" "$TMP/contents/icons/claude.png"

# Install or upgrade the KPackage.
if kpackagetool6 -t Plasma/Applet -l 2>/dev/null | grep -qx "$ID"; then
    kpackagetool6 -t Plasma/Applet -u "$TMP"
else
    kpackagetool6 -t Plasma/Applet -i "$TMP"
fi

# Add it to the first panel if not already present (live, no logout needed).
qdbus6 org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript "
var ps = panels();
if (ps.length) {
    var panel = ps[0];
    var have = false;
    var ws = panel.widgets();
    for (var i = 0; i < ws.length; i++) {
        if (ws[i].type == '$ID') have = true;
    }
    if (!have) { panel.addWidget('$ID'); }
}
" >/dev/null 2>&1 || echo "Note: add the 'Claude Status Bar' widget to your panel manually (right-click panel -> Add Widgets)."

echo "Plasmoid $ID installed."
