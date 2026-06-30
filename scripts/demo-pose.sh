#!/usr/bin/env bash
# Pose fake Claude session state so you can screenshot the UIs without running real
# sessions. The plasmoid reads sessions.d/*.json (via `claude-status-bar sessions`);
# the tray reads state.json. Both repaint within ~0.5s of a write.
#
#   demo-pose.sh                Pose a 3-session scene (busy + thinking + waiting)
#   demo-pose.sh <state>        Pose ONE session in <state>
#                               states: thinking tool permission waiting idle done
#   demo-pose.sh loop           Animate the bar (cycle activity) until Ctrl-C — for GIFs
#   demo-pose.sh --clean        Remove all posed demo files
#   demo-pose.sh --shot FILE    Pose the 3-session scene, then Spectacle-capture a
#                               dragged region to FILE (needs spectacle)
#
# Posed files are named demo*.json; --clean only touches those (+ state.json).
# loop/--shot hide your real sessions; they're restored on Ctrl-C / --clean.
set -euo pipefail

SB="$HOME/.claude/statusbar"
SD="$SB/sessions.d"
STASH="$SD/.stash"

usage() {
    cat <<'EOF'
Pose fake Claude session state to screenshot/record the UIs without real sessions.

  demo-pose.sh                Pose a 3-session scene (busy + thinking + waiting)
  demo-pose.sh <state>        Pose ONE session: thinking tool permission waiting idle done
  demo-pose.sh loop           Animate the bar (cycle activity) until Ctrl-C — for GIFs
  demo-pose.sh --clean        Remove posed files and restore your real sessions
  demo-pose.sh --shot FILE    Pose the scene, then Spectacle-capture a region to FILE

Env: DEMO_FRAME_SECS (loop frame duration, default 1.3).
EOF
}

# Hide your real, live sessions so only the posed ones show. Restored by --clean.
stash_real() {
    mkdir -p "$STASH"
    shopt -s nullglob
    for f in "$SD"/*.json; do
        case "$(basename "$f")" in demo*.json) continue ;; esac
        mv "$f" "$STASH/"
    done
}
restore_real() {
    [ -d "$STASH" ] || return 0
    shopt -s nullglob
    for f in "$STASH"/*.json; do mv "$f" "$SD/"; done
    rmdir "$STASH" 2>/dev/null || true
}

# write_session FILE STATE LABEL TOOL PROJECT START_OFF TS_OFF
# START_OFF: seconds-ago the timer started, or -1 for startedAt:0 (no timer).
write_session() {
    local file=$1 state=$2 label=$3 tool=$4 project=$5 start_off=$6 ts_off=$7
    local n started=0
    n=$(date +%s)
    [ "$start_off" -ge 0 ] && started=$((n - start_off))
    cat > "$file" <<EOF
{"state":"$state","label":"$label","tool":"$tool","project":"$project","sessionId":"$(basename "$file" .json)","transcript":"","startedAt":$started,"ts":$((n - ts_off))}
EOF
}

pose_scene() {
    mkdir -p "$SD"
    stash_real
    write_session "$SD/demo1.json" tool     "Editing"          Edit "web-frontend" 43  0
    write_session "$SD/demo2.json" thinking "Scheming…"        ""   "api-server"   129 1
    write_session "$SD/demo3.json" waiting  "Waiting for you"  ""   "docs"         -1  2
    cp "$SD/demo1.json" "$SB/state.json"   # busiest session feeds the tray
    echo "Posed 3-session scene. Bar: 'web-frontend · Editing 43s   +2'."
    echo "Real sessions hidden — run 'demo-pose.sh --clean' to restore them."
}

pose_single() {
    local state=$1 label tool="" project="solo-project" start=12
    case "$state" in
        thinking)   label="Pondering…" ;;
        tool)       label="Editing"; tool="Edit" ;;
        permission) label="Awaiting permission"; start=-1 ;;
        waiting)    label="Waiting for you"; start=-1 ;;
        idle|done)  label=""; start=-1 ;;
        *) echo "unknown state '$state'" >&2; usage; exit 2 ;;
    esac
    mkdir -p "$SD"
    rm -f "$SD"/demo*.json
    stash_real
    write_session "$SD/demo1.json" "$state" "$label" "$tool" "$project" "$start" 0
    cp "$SD/demo1.json" "$SB/state.json"
    echo "Posed single '$state' session."
    echo "Real sessions hidden — run 'demo-pose.sh --clean' to restore them."
}

clean() {
    rm -f "$SD"/demo*.json "$SB/state.json"
    restore_real
    echo "Removed posed demo files; restored your real sessions."
}

# Animate the bar for a screen recording: one "lead" session cycles through
# activities (timer ticking) while two static background sessions supply the +2
# badge without ever stealing the bar. Ctrl-C restores your real sessions.
animate() {
    mkdir -p "$SD"
    stash_real
    trap 'echo; clean; exit 0' INT TERM
    local start; start=$(date +%s)
    # Static, non-busy backgrounds → "+2" but never outrank the lead.
    write_session "$SD/demo2.json" waiting "Waiting for you" "" "api-server" -1 1
    write_session "$SD/demo3.json" idle    ""                "" "docs"        -1 2
    # state|label|tool — project stays web-frontend so the timer stays coherent.
    local frames=(
        "tool|Editing|Edit"
        "tool|Running command|Bash"
        "thinking|Scheming…|"
        "thinking|Reticulating…|"
        "tool|Reading|Read"
        "permission|Awaiting permission|"
        "thinking|Cooking…|"
    )
    echo "Animating the bar (Ctrl-C to stop & restore)…"
    local i=0 st label tool n started
    while true; do
        IFS='|' read -r st label tool <<<"${frames[$((i % ${#frames[@]}))]}"
        n=$(date +%s)
        started=$start
        [ "$st" = permission ] && started=0   # permission shows no timer
        cat > "$SD/demo1.json" <<EOF
{"state":"$st","label":"$label","tool":"$tool","project":"web-frontend","sessionId":"demo1","transcript":"","startedAt":$started,"ts":$n}
EOF
        cp "$SD/demo1.json" "$SB/state.json"
        i=$((i + 1))
        sleep "${DEMO_FRAME_SECS:-1.3}"
    done
}

case "${1:-}" in
    ""|scene)        pose_scene ;;
    --clean|clean)   clean ;;
    -h|--help)       usage ;;
    --shot)
        [ -n "${2:-}" ] || { echo "--shot needs an output FILE" >&2; exit 2; }
        command -v spectacle >/dev/null || { echo "spectacle not found" >&2; exit 1; }
        pose_scene
        echo "Drag a region around the widget…"
        spectacle -rbno "$2"
        echo "Saved $2"
        ;;
    loop|--animate|animate) animate ;;
    thinking|tool|permission|waiting|idle|done) pose_single "$1" ;;
    *) usage; exit 2 ;;
esac
