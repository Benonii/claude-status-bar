#!/usr/bin/env bash
# Deploy the Plasma 6 plasmoid (Layer B) and add it to the primary panel.
# Idempotent: re-running upgrades the package in place.
set -euo pipefail

ID="com.abyot.claudestatusbar"
SRC_DIR="$(cd "$(dirname "$0")/../plasmoid" && pwd)"

# Resolve the binary the plasmoid should call: explicit override, then an
# installed copy on PATH, then the repo's release build.
if [ -n "${CSB_BIN:-}" ]; then
    BIN="$CSB_BIN"
elif command -v claude-status-bar >/dev/null 2>&1; then
    BIN="$(command -v claude-status-bar)"
else
    BIN="$(cd "$(dirname "$0")/.." && pwd)/target/release/claude-status-bar"
fi
SESSIONS_CMD="$BIN sessions"

# Stage a copy with the absolute 'sessions' command baked in and the spark bundled.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cp -r "$SRC_DIR/." "$TMP/"
sed -i "s|__SESSIONS_CMD__|$SESSIONS_CMD|g" "$TMP/contents/ui/main.qml"

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
