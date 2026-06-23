// Layer B (KDE panel variant): a Plasma 6 plasmoid that shows the Claude spark
// plus an always-visible status label, reading the per-session state the Rust hooks
// write under ~/.claude/statusbar/sessions.d/. A panel applet (unlike a system-tray
// icon) can render inline text — which gives the macOS-menu-bar look.
//
// Data source: `claude-status-bar sessions` prints a JSON array of all live sessions
// (freshest first). We use the Plasma5Support "executable" datasource because Qt6
// forbids XMLHttpRequest from reading local files.
//
// NOTE: a fullRepresentation MUST be defined — on Plasma 6 a panel applet with only a
// compactRepresentation fails to instantiate it (renders blank).
//
// __SESSIONS_CMD__ is replaced at install time with "/abs/path/claude-status-bar sessions".

import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PlasmaComponents
import org.kde.plasma.plasma5support as P5Support
import org.kde.kirigami as Kirigami

PlasmoidItem {
    id: root

    readonly property string sessionsCmd: "__SESSIONS_CMD__"

    // All live sessions (freshest first), and the one the bar represents.
    property var sessions: []
    readonly property var idleSt: ({ "state": "idle", "label": "", "project": "", "startedAt": 0, "ts": 0 })
    property var st: idleSt
    property double now: 0

    readonly property bool busy: st.state === "thinking" || st.state === "tool"
    readonly property bool quiet: st.state === "idle" || st.state === "done"
    readonly property int otherCount: Math.max(0, sessions.length - 1)

    function isBusy(s)  { return s && (s.state === "thinking" || s.state === "tool") }
    function needsUser(s) { return s && (s.state === "permission" || s.state === "waiting") }

    // Elapsed string for a given session ("" unless it's busy with a start time).
    function sessElapsed(s) {
        if (!isBusy(s) || !s.startedAt)
            return ""
        var sec = Math.max(0, Math.floor(now - s.startedAt))
        var m = Math.floor(sec / 60)
        return m > 0 ? (m + "m " + (sec % 60) + "s") : (sec + "s")
    }
    function elapsedStr() { return sessElapsed(st) }

    // The bar shows the busiest session (busy > awaiting-user > freshest).
    function pickCurrent(arr) {
        if (!arr || !arr.length)
            return idleSt
        for (var i = 0; i < arr.length; i++) if (isBusy(arr[i])) return arr[i]
        for (var j = 0; j < arr.length; j++) if (needsUser(arr[j])) return arr[j]
        return arr[0]
    }

    // Short activity phrase for a session, used in the popup rows.
    function sessActivity(s) {
        if (isBusy(s)) {
            var e = sessElapsed(s)
            return (s.label || "Working") + (e.length ? "  " + e : "")
        }
        if (needsUser(s)) return s.label || "Waiting"
        if (s.state === "done") return "Done"
        return "Idle"
    }

    // Inline bar text: "<project> · <activity> <elapsed>"  + "  +N" for other sessions.
    readonly property string displayText: {
        if (quiet)
            return otherCount > 0 ? ("+" + sessions.length) : ""
        var who = st.project ? st.project : "Claude"
        var act = st.label ? st.label : ""
        var e = elapsedStr()
        var right = act + (e.length ? " " + e : "")
        var base = right.length ? (who + " · " + right) : who
        return otherCount > 0 ? (base + "   +" + otherCount) : base
    }

    Plasmoid.status: PlasmaCore.Types.ActiveStatus
    preferredRepresentation: compactRepresentation
    toolTipMainText: "Claude"
    toolTipSubText: sessions.length === 0 ? "idle"
                    : (sessions.length + (sessions.length === 1 ? " session" : " sessions"))

    P5Support.DataSource {
        id: reader
        engine: "executable"
        connectedSources: []
        onNewData: function (source, data) {
            reader.disconnectSource(source) // one-shot per poll
            if (data["exit code"] === 0 && data.stdout) {
                try {
                    var arr = JSON.parse(data.stdout)
                    if (Array.isArray(arr)) {
                        root.sessions = arr
                        root.st = root.pickCurrent(arr)
                    }
                } catch (e) {
                    // keep previous good state
                }
            }
        }
        function poll(cmd) {
            disconnectSource(cmd)
            connectSource(cmd)
        }
    }

    Timer {
        interval: 500
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            root.now = Date.now() / 1000 // advances elapsed timers between writes
            reader.poll(root.sessionsCmd)
        }
    }

    // ---- inline panel content -------------------------------------------------
    compactRepresentation: MouseArea {
        id: comp
        readonly property bool horizontal: Plasmoid.formFactor === PlasmaCore.Types.Horizontal
        readonly property int thickness: horizontal ? height : width
        readonly property int iconSize: Math.round(Math.max(15, thickness * 0.72))
        readonly property int fontSize: Math.round(Math.max(10, thickness * 0.38))

        implicitWidth: rowLayout.implicitWidth
        implicitHeight: horizontal ? thickness : rowLayout.implicitHeight
        Layout.minimumWidth: rowLayout.implicitWidth
        Layout.preferredWidth: rowLayout.implicitWidth

        acceptedButtons: Qt.LeftButton
        onClicked: root.expanded = !root.expanded

        RowLayout {
            id: rowLayout
            anchors.fill: parent
            spacing: Math.round(Kirigami.Units.smallSpacing * 0.75)

            // The Claude creature: walks (legs + eyes dart right) while working,
            // stands still (frame 0) when idle. Pixel art, scaled with smooth:false.
            AnimatedSprite {
                id: logo
                source: Qt.resolvedUrl("../icons/claude-walk.png")
                frameCount: 4
                frameWidth: 13
                frameHeight: 12
                frameDuration: 200
                interpolate: false
                loops: AnimatedSprite.Infinite
                running: root.busy
                smooth: false
                Layout.alignment: Qt.AlignVCenter
                Layout.preferredHeight: comp.iconSize
                Layout.preferredWidth: Math.round(comp.iconSize * 13 / 12)
                opacity: root.busy ? 1.0 : (root.quiet ? 0.7 : 0.9)
                Behavior on opacity { NumberAnimation { duration: 200 } }
            }

            PlasmaComponents.Label {
                id: label
                text: root.displayText
                visible: text.length > 0
                Layout.alignment: Qt.AlignVCenter
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
                maximumLineCount: 1
                font.pixelSize: comp.fontSize
            }
        }
    }

    // ---- click-to-expand popup: width locked to the bar, lists every session --
    fullRepresentation: ColumnLayout {
        id: full
        // Match the bar's width, but never collapse below a readable floor.
        readonly property int minWidth: Kirigami.Units.gridUnit * 13
        readonly property int barWidth: root.compactRepresentationItem
                                         ? Math.round(root.compactRepresentationItem.width)
                                         : Kirigami.Units.gridUnit * 10
        readonly property int popWidth: Math.max(barWidth, minWidth)
        Layout.minimumWidth: full.popWidth
        Layout.maximumWidth: full.popWidth
        Layout.preferredWidth: full.popWidth
        Layout.minimumHeight: full.implicitHeight
        Layout.preferredHeight: full.implicitHeight
        spacing: 0

        // Header
        RowLayout {
            Layout.fillWidth: true
            Layout.margins: Kirigami.Units.smallSpacing
            spacing: Kirigami.Units.smallSpacing
            Image {
                source: Qt.resolvedUrl("../icons/claude-still.png")
                smooth: false
                sourceSize.height: 22
                Layout.preferredWidth: 22
                Layout.preferredHeight: 22
            }
            PlasmaComponents.Label {
                text: "Claude"
                font.bold: true
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
            PlasmaComponents.Label {
                text: root.sessions.length
                opacity: 0.6
                visible: root.sessions.length > 1
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.leftMargin: Kirigami.Units.smallSpacing
            Layout.rightMargin: Kirigami.Units.smallSpacing
            height: 1
            color: Kirigami.Theme.textColor
            opacity: 0.15
        }

        PlasmaComponents.Label {
            visible: root.sessions.length === 0
            text: "No active sessions"
            opacity: 0.6
            Layout.fillWidth: true
            Layout.margins: Kirigami.Units.smallSpacing
            horizontalAlignment: Text.AlignHCenter
        }

        // One row per live session (freshest first).
        Repeater {
            model: root.sessions
            delegate: RowLayout {
                required property var modelData
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.smallSpacing
                spacing: Kirigami.Units.smallSpacing

                Image {
                    source: Qt.resolvedUrl("../icons/claude-still.png")
                smooth: false
                    sourceSize.height: 18
                    Layout.preferredWidth: 18
                    Layout.preferredHeight: 18
                    Layout.alignment: Qt.AlignTop
                    opacity: root.isBusy(modelData) ? 1.0
                             : (root.needsUser(modelData) ? 0.9 : 0.55)
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0
                    PlasmaComponents.Label {
                        text: modelData.project ? modelData.project : "Claude"
                        font.bold: true
                        Layout.fillWidth: true
                        elide: Text.ElideMiddle
                    }
                    PlasmaComponents.Label {
                        text: root.sessActivity(modelData)
                        opacity: 0.7
                        font.pointSize: Kirigami.Theme.smallFont.pointSize
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }
                }
            }
        }
    }
}
