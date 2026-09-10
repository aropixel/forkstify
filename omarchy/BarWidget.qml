import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Mpris
import qs.Ui
import qs.Commons

// forkstify in the bar (0021): the bars dance while it plays, and stand
// still — the same staircase, dimmed — when it is paused or not running:
// flat bars looked like a bug (Joel, 10/09/2026). One click opens
// the card: the track, its progress, what comes next, and the three
// controls — or « Lancer » / « Installer » when there is nothing to show.
// Only the `forkstify` player is watched: the rest belongs to omarchy.media.
BarWidget {
  id: root
  moduleName: "io.github.aropixel.forkstify"

  readonly property var player: findPlayer()
  readonly property bool running: player !== null
  readonly property bool playing: running && player.isPlaying
  readonly property string title: running ? (player.trackTitle || "") : ""
  readonly property string artist: running ? (player.trackArtist || "") : ""
  readonly property string next: running && player.metadata && player.metadata["forkstify:next"] ? String(player.metadata["forkstify:next"]) : ""
  readonly property real position: running && player.positionSupported ? player.position : 0
  readonly property real length: running && player.lengthSupported ? player.length : 0

  property bool installed: true
  property string installLog: ""
  property bool installing: false
  property bool popupOpen: false
  property int frame: 0

  // the waybar animation Joel had: five frames, 100 ms
  readonly property var frames: ["▂▄▆", "▄▂▆", "▄▆▂", "▆▄▂", "▆▂▄"]
  readonly property string glyph: playing ? frames[frame] : "▂▄▆"

  function findPlayer() {
    var list = Mpris.players ? Mpris.players.values : []
    for (var i = 0; i < list.length; i++) {
      var p = list[i]
      if (p && (p.identity === "forkstify" || p.desktopEntry === "forkstify")) return p
    }
    return null
  }

  function clock(seconds) {
    var s = Math.max(0, Math.floor(seconds))
    var m = Math.floor(s / 60)
    var r = s % 60
    return m + ":" + (r < 10 ? "0" : "") + r
  }

  function close() { popupOpen = false }

  function launch() {
    Quickshell.execDetached(["omarchy-launch-or-focus-tui", "forkstify"])
    popupOpen = false
  }

  function install() {
    if (installing) return
    installing = true
    installLog = "…"
    installProc.running = true
  }

  implicitWidth: label.implicitWidth + Style.space(14)
  implicitHeight: barSize

  Timer {
    interval: 100
    repeat: true
    running: root.playing
    onTriggered: root.frame = (root.frame + 1) % root.frames.length
  }

  // is the binary on the PATH? asked once at load, and again after an install
  Process {
    id: whichProc
    command: ["sh", "-c", "command -v forkstify >/dev/null 2>&1 && echo yes || echo no"]
    running: true
    stdout: StdioCollector {
      waitForEnd: true
      onStreamFinished: root.installed = String(text || "").trim() === "yes"
    }
  }

  Process {
    id: installProc
    command: ["bash", Qt.resolvedUrl("install.sh").toString().replace("file://", "")]
    stdout: SplitParser {
      onRead: function(line) { root.installLog = line }
    }
    stderr: SplitParser {
      onRead: function(line) { root.installLog = line }
    }
    onExited: function(code) {
      root.installing = false
      if (code === 0) { root.installed = true; root.installLog = "installé — « Lancer »" }
      else root.installLog = root.installLog || ("échec (" + code + ")")
    }
  }

  Text {
    id: label
    anchors.centerIn: parent
    textFormat: Text.PlainText
    text: root.glyph
    color: root.playing ? root.bar.barForeground : Qt.darker(root.bar.barForeground, 1.6)
    font.family: root.bar.fontFamily
    // block glyphs fill their em: a size down keeps them level with the
    // bar's icons (Joel, 10/09/2026)
    font.pixelSize: Style.font.caption
  }

  MouseArea {
    anchors.fill: parent
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    acceptedButtons: Qt.LeftButton
    onClicked: root.popupOpen = !root.popupOpen
    onEntered: if (root.bar) root.bar.showTooltip(root, root.running ? (root.title + (root.artist ? " — " + root.artist : "")) : "forkstify")
    onExited: if (root.bar) root.bar.hideTooltip(root)
  }

  PopupCard {
    id: popup
    anchorItem: root
    bar: root.bar
    owner: root
    open: root.popupOpen
    contentWidth: popup.fittedContentWidth(Style.space(300))
    contentHeight: popup.fittedContentHeight(column.implicitHeight)

    Column {
      id: column
      anchors.fill: parent
      spacing: Style.space(8)

      Text {
        textFormat: Text.PlainText
        text: root.running ? (root.title || "rien en cours") : (root.installed ? "forkstify ne tourne pas" : "forkstify n'est pas installé")
        color: root.bar.foreground
        font.family: root.bar.fontFamily
        font.pixelSize: Style.font.subtitle
        font.bold: true
        elide: Text.ElideRight
        width: parent.width
      }

      Text {
        textFormat: Text.PlainText
        text: root.artist
        visible: text !== ""
        color: Qt.darker(root.bar.foreground, 1.3)
        font.family: root.bar.fontFamily
        font.pixelSize: Style.font.bodySmall
        elide: Text.ElideRight
        width: parent.width
      }

      // the progress: a thin bar and the two clocks
      Column {
        width: parent.width
        spacing: Style.space(3)
        visible: root.running && root.length > 0

        Rectangle {
          width: parent.width
          height: Style.space(4)
          radius: height / 2
          color: Qt.darker(root.bar.foreground, 3)

          Rectangle {
            width: root.length > 0 ? parent.width * Math.min(1, root.position / root.length) : 0
            height: parent.height
            radius: height / 2
            color: Color.accent
          }
        }

        Text {
          textFormat: Text.PlainText
          text: root.clock(root.position) + " / " + root.clock(root.length)
          color: Qt.darker(root.bar.foreground, 1.6)
          font.family: root.bar.fontFamily
          font.pixelSize: Style.font.caption
        }
      }

      Text {
        textFormat: Text.PlainText
        text: root.next !== "" ? "à suivre : " + root.next : ""
        visible: text !== ""
        color: Qt.darker(root.bar.foreground, 1.3)
        font.family: root.bar.fontFamily
        font.pixelSize: Style.font.bodySmall
        elide: Text.ElideRight
        width: parent.width
      }

      Row {
        visible: root.running
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.space(6)

        Button {
          iconText: "󰒮"
          foreground: root.bar.foreground
          horizontalPadding: Style.spacing.controlPaddingX
          verticalPadding: Style.spacing.controlPaddingY
          enabled: root.running && root.player.canGoPrevious
          opacity: enabled ? 1.0 : 0.4
          onClicked: root.player.previous()
        }

        Button {
          iconText: root.playing ? "󰏤" : "󰐊"
          foreground: root.bar.foreground
          horizontalPadding: Style.spacing.panelGap
          verticalPadding: Style.spacing.controlPaddingY
          iconSize: Style.font.iconLarge
          enabled: root.running
          onClicked: root.player.togglePlaying()
        }

        Button {
          iconText: "󰒭"
          foreground: root.bar.foreground
          horizontalPadding: Style.spacing.controlPaddingX
          verticalPadding: Style.spacing.controlPaddingY
          enabled: root.running && root.player.canGoNext
          opacity: enabled ? 1.0 : 0.4
          onClicked: root.player.next()
        }
      }

      // nothing to show: launch it, or install it first
      Row {
        visible: !root.running
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Style.space(6)

        Button {
          text: root.installed ? "Lancer" : (root.installing ? "installation…" : "Installer")
          foreground: root.bar.foreground
          horizontalPadding: Style.spacing.controlPaddingX
          verticalPadding: Style.spacing.controlPaddingY
          enabled: !root.installing
          onClicked: root.installed ? root.launch() : root.install()
        }
      }

      Text {
        textFormat: Text.PlainText
        text: root.installLog
        visible: !root.running && text !== ""
        color: Qt.darker(root.bar.foreground, 1.6)
        font.family: root.bar.fontFamily
        font.pixelSize: Style.font.caption
        wrapMode: Text.Wrap
        width: parent.width
      }
    }
  }
}
