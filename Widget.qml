import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui
import qs.Commons
import "SettingsModel.js" as SettingsModel
import "KeyCatalog.js" as KeyCatalog

BarWidget {
    id: root
    moduleName: "dimef.butterfly-guard"
    property string filterState: "not installed / stopped"
    property int delayMs: 40
    property int dropped: 0
    property string lastError: ""
    property bool popupOpen: false
    property bool loaded: false
    property bool dirty: false
    property string savedConfigJson: ""
    property var overrides: ({})
    property string message: ""
    property bool showAllKeys: false
    function close() { popupOpen = false }
    function loadSettings() {
        if (!configProbe.running && !control.running) configProbe.running = true
    }
    function editKey(code, value) {
        var copy = Object.assign({}, overrides)
        if (value === "") delete copy[String(code)]
        else copy[String(code)] = value
        overrides = copy
        dirty = true
    }
    function save() {
        try {
            var config = SettingsModel.buildConfig(globalDelay.text, overrides)
            control.command = ["pkexec", "/usr/local/bin/butterfly-guardctl", "set-config", JSON.stringify(config), savedConfigJson]
            message = "Saving… authenticate if prompted."
            lastError = ""
            root.close() // Release layer-shell focus before the authentication dialog opens.
            control.running = true
        } catch (error) { message = String(error.message) }
    }
    readonly property var visibleKeys: KeyCatalog.keys.filter(function(key) {
        var query = search.text.toLowerCase()
        return (showAllKeys || key.builtIn)
            && (SettingsModel.keyLabel(key.name).toLowerCase().indexOf(query) >= 0
                || key.name.toLowerCase().indexOf(query) >= 0 || String(key.code) === query)
    })
    implicitWidth: label.implicitWidth + Style.space(14)
    implicitHeight: barSize

    function refresh() {
        if (!probe.running) probe.running = true
    }
    function act(action) {
        if (control.running) return
        lastError = ""
        control.command = ["pkexec", "/usr/local/bin/butterfly-guardctl", action]
        root.close() // Authentication must own keyboard focus.
        control.running = true
    }
    Text {
        id: label
        anchors.centerIn: parent
        text: "󰌌"
        color: root.bar ? root.bar.barForeground : "white"
        opacity: root.filterState === "active" ? 1 : 0.5
        font.family: root.bar ? root.bar.fontFamily : "monospace"
        font.pixelSize: Style.font.body
    }
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            root.popupOpen = !root.popupOpen
            if (root.popupOpen && !root.loaded) root.loadSettings()
        }
        onEntered: if (root.bar) root.bar.showTooltip(root,
            "onceKey: " + root.filterState + "\n" + root.delayMs + " ms · " + root.dropped
            + " duplicate presses blocked this session\nClick to configure global and per-key delays"
            + (root.lastError ? "\n" + root.lastError : ""))
        onExited: if (root.bar) root.bar.hideTooltip(root)
    }
    Process {
        id: probe
        command: ["cat", "/run/butterfly-guard/status.json"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    var data = JSON.parse(text)
                    root.filterState = data.state
                    root.delayMs = data.delay_ms
                    root.dropped = data.dropped
                } catch (_) { root.filterState = "not installed / stopped" }
            }
        }
        onExited: function(code) { if (code !== 0) root.filterState = "not installed / stopped" }
    }
    Process {
        id: control
        onExited: function(code) {
            if (code !== 0) {
                if (!root.lastError) root.lastError = "Action failed or authentication cancelled."
                root.message = root.lastError
                Quickshell.execDetached(["notify-send", "onceKey", root.lastError])
            }
            if (code === 0) {
                root.message = "Saved. Settings are persistent."
                root.dirty = false
                root.loaded = false
                root.loadSettings()
            }
            root.refresh()
        }
        stderr: StdioCollector { onStreamFinished: { if (text.trim()) root.lastError = text.trim() } }
    }
    Process {
        id: configProbe
        command: ["cat", "/etc/butterfly-guard.json"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    var config = JSON.parse(text)
                    SettingsModel.buildConfig(String(config.delay_ms), config.key_delays_ms)
                    root.savedConfigJson = JSON.stringify(config)
                    globalDelay.text = String(config.delay_ms)
                    root.overrides = config.key_delays_ms
                    root.loaded = true
                    root.dirty = false
                } catch (_) { root.message = "Could not load settings." }
            }
        }
        onExited: function(code) { if (code !== 0) root.message = "Could not read saved settings." }
    }
    KeyboardPanel {
        id: popup
        anchorItem: root
        bar: root.bar
        owner: root
        open: root.popupOpen
        focusTarget: search
        contentWidth: popup.fittedContentWidth(Style.space(560))
        contentHeight: popup.fittedContentHeight(Style.space(620))

        Flickable {
            anchors.fill: parent
            contentHeight: panel.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            Column {
                id: panel
                width: parent.width
                spacing: Style.space(10)
                Text { text: "onceKey"; color: Color.foreground; font.pixelSize: Style.font.subtitle; font.bold: true }
                Text {
                    text: root.filterState + " · " + root.dropped + " rapid repeats blocked since attachment"
                    color: Color.foreground; font.pixelSize: Style.font.bodySmall
                }
                Row {
                    spacing: Style.space(8)
                    Button {
                        text: root.filterState === "active" ? "Disable filtering" : "Enable filtering"
                        enabled: !control.running && !root.dirty
                        onClicked: root.act(root.filterState === "active" ? "stop" : "start")
                    }
                    Button { text: "Close"; onClicked: root.close() }
                }
                Text { text: "Global delay (ms)"; color: Color.foreground; font.bold: true }
                Row {
                    spacing: Style.space(8)
                    TextField {
                        id: globalDelay
                        width: Style.space(85)
                        placeholderText: "40"
                        enabled: root.loaded && !control.running
                        validator: IntValidator { bottom: 0; top: 200 }
                        onTextEdited: root.dirty = true
                    }
                    Button {
                        text: "Apply to all keys"
                        enabled: root.loaded && !control.running && globalDelay.acceptableInput
                        onClicked: { root.overrides = ({}); root.dirty = true; root.message = "All keys now use the global delay. Save to apply." }
                    }
                    Button {
                        text: "Reset to 40 ms"
                        enabled: root.loaded && !control.running
                        onClicked: { globalDelay.text = "40"; root.overrides = ({}); root.dirty = true }
                    }
                }
                Text {
                    width: parent.width
                    text: "0–200 ms. The first press is immediate. Larger values may suppress intentional double letters."
                    wrapMode: Text.WordWrap; color: Color.foreground; font.pixelSize: Style.font.bodySmall
                }
                Text { text: "Individual keys"; color: Color.foreground; font.bold: true }
                Row {
                    width: parent.width
                    spacing: Style.space(8)
                    TextField { id: search; width: parent.width - allKeys.implicitWidth - parent.spacing; placeholderText: "Search key name…" }
                    Button { id: allKeys; text: root.showAllKeys ? "Built-in keys" : "All Linux keys"; onClicked: root.showAllKeys = !root.showAllKeys }
                }
                Text {
                    text: "Blank = global delay · 0 = no filtering for that key"
                    color: Color.foreground; font.pixelSize: Style.font.bodySmall
                }
                ListView {
                    id: keyList
                    width: parent.width
                    height: Style.space(230)
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    model: root.visibleKeys
                    delegate: Row {
                        id: keyRow
                        required property var modelData
                        width: keyList.width
                        height: Style.space(38)
                        spacing: Style.space(8)
                        Text {
                            width: parent.width - Style.space(175)
                            anchors.verticalCenter: parent.verticalCenter
                            text: SettingsModel.keyLabel(keyRow.modelData.name)
                            elide: Text.ElideRight
                            color: Color.foreground; font.pixelSize: Style.font.body
                        }
                        TextField {
                            width: Style.space(85)
                            anchors.verticalCenter: parent.verticalCenter
                            enabled: root.loaded && !control.running
                            text: root.overrides[String(keyRow.modelData.code)] === undefined ? "" : String(root.overrides[String(keyRow.modelData.code)])
                            placeholderText: globalDelay.text + " ms"
                            validator: IntValidator { bottom: 0; top: 200 }
                            onTextEdited: root.editKey(keyRow.modelData.code, text)
                        }
                        Button {
                            text: "Global"
                            anchors.verticalCenter: parent.verticalCenter
                            enabled: root.loaded && !control.running && root.overrides[String(keyRow.modelData.code)] !== undefined
                            onClicked: root.editKey(keyRow.modelData.code, "")
                        }
                    }
                }
                Text { visible: keyList.count === 0; text: "No matching keys"; color: Color.foreground }
                TextField {
                    width: parent.width
                    placeholderText: "Test typing here: apple, letter, book…"
                    onActiveFocusChanged: if (!activeFocus && !root.popupOpen) text = ""
                }
                Row {
                    spacing: Style.space(8)
                    Button {
                        text: control.running ? "Saving…" : "Save changes"
                        enabled: root.loaded && root.dirty && !control.running
                        onClicked: root.save()
                    }
                    Button {
                        text: "Discard changes / reload"
                        enabled: !control.running
                        onClicked: { root.loadSettings(); root.message = "" }
                    }
                    Text { text: root.dirty ? "Unsaved changes" : ""; color: Color.foreground; anchors.verticalCenter: parent.verticalCenter }
                }
                Text {
                    width: parent.width
                    text: root.message
                    visible: text !== ""
                    wrapMode: Text.WordWrap; color: Color.foreground; font.pixelSize: Style.font.bodySmall
                }
            }
        }
    }
    IpcHandler {
        target: "butterfly-guard"
        function configure(): void { root.popupOpen = true; if (!root.loaded) root.loadSettings() }
        function close(): void { root.close() }
        function settingsStatus(): string { return JSON.stringify({loaded: root.loaded, dirty: root.dirty, message: root.message, visibleKeys: keyList.count, open: root.popupOpen, searchFocused: search.activeFocus}) }
    }
    Timer { interval: 2000; running: true; repeat: true; onTriggered: root.refresh() }
    Component.onCompleted: refresh()
}
