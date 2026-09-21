# The Omarchy bar — the moving icon, and the track popover

Working note, opened on 2026-09-10. **Decided** = recorded; **direction** =
proposed, uncontradicted, not yet recorded; **to settle** = open question.

## What Joel wants

To get back what he had under waybar: an animation of little bars (`▂▄▆`,
five frames, 100 ms) in the bar when music is playing, nothing when it is
paused. And on click, a popover **under the icon, like every Omarchy
plugin**: title, artist, progress, next track.

The old script:
`dotfiles/.config/waybar/custom_modules/media/media-animation.sh` from the
`kbyjoel/arch-linux` repository — a `playerctl status` loop.

## What already exists, on both sides

- **Omarchy has an MPRIS media widget** (`omarchy.media`, service +
  bar-widget, disabled on Joel's machine): hidden as long as no player has
  metadata, then a ▶/⏸ icon and "title · artist" in the bar, and a
  `PopupCard` on click with cover art, title, artist, album, previous /
  pause / next. **No progress, no next track, no animation.** Quickshell
  exposes a player's `position`, `length`, `isPlaying`, `trackTitle`,
  `trackArtist` and the **raw `metadata` map** — a house key goes through
  it.
- **forkstify is already an MPRIS player** (`src/mediakeys.rs`) — but only
  publishes "playing" on start: **no title, no artist, no position, no state
  change** on pause. Omarchy's widget would see it empty.
- Third-party plugins are installed with `omarchy plugin add <git-url>`,
  which expects a `manifest.json` **at the root of the repository**, and
  live in `~/.config/omarchy/plugins/<id>/` (reloaded on every save). Joel's
  `io.github.sspaeti.neomd` plugin is one example: a `BarWidget.qml`, a
  `PopupCard`, a `Model.js`.

## Decided on 2026-09-10 — [0021](../decisions/0021-the-repository-is-the-omarchy-plugin.md)

Joel settled the four questions: the widget only watches forkstify; **the
repository is itself the plugin** (`manifest.json` at the root, `omarchy/`
for the widget), and it is the plugin that **installs and launches** the
binary, since `omarchy plugin add` only clones, validates and enables; cover
art later; nothing on middle click or scroll. Three floors, **all done on
2026-09-10**: the MPRIS metadata; tokens and caches under
`~/.local/state/forkstify` (moved over from `target/` on first run, with no
re-authorization); the plugin — `manifest.json` at the root,
`omarchy/BarWidget.qml`, `omarchy/install.sh`. One deliberate departure from
the old script: **the icon stays visible** when paused and when forkstify is
not running — the same `▂▄▆` staircase, fixed and dimmed (Joel: flat bars
"look like a bug") — otherwise there would be nothing to click to launch or
install. On Joel's machine, the repository is **linked** into
`~/.config/omarchy/plugins/io.github.aropixel.forkstify`; elsewhere,
`omarchy plugin add git@github.com:aropixel/forkstify.git`.

**Reloading the widget after a QML change**: the shell watches
`~/.config/omarchy/plugins/` with `inotifywait -r`, which does not descend
into a linked folder, and its component cache survives `rescanPlugins` and
even the "Local plugin changed" that recreating the link triggers. Only
**`omarchy restart shell`** picks up new QML (verified on 2026-09-10 by
capturing the bar). One second of flicker.

**Position in Quickshell** (2026-09-11): `MprisPlayer.position` is computed
on every read (last sample + elapsed time), but the `positionChanged` signal
is only emitted on a `Seeked` or a state change — never during playback. A
QML binding (`root.position: player.position`) therefore stays frozen on the
last signalled value, 0 at the start of the track, and the card showed 0:00.
The widget asks for the signal itself: a one-second `Timer`, active while
the card is open and the track is playing, calls
`player.positionChanged()`. Omarchy's media service does not show the
position and does not have this problem.

**Going back to the window** (Joel, 2026-09-11): the card gains a "Show
forkstify" button when forkstify is running, through
`omarchy-launch-or-focus-tui forkstify` — the app-id
`org.omarchy.forkstify`, like "Launch". A first attempt aimed at the bare
word `forkstify` so as to catch a terminal started by hand too: it mostly
caught a terminal open in `~/Work/forkstify`, whose title carries the path,
and focus went to that shell instead of launching when forkstify was off
(Joel, the same day). The app-id is precise, we stick to it.

## Direction: two floors, the first one portable

1. **forkstify publishes everything a desktop expects** (portable, outside
   Omarchy): title, artist, length (`xesam:` / `mpris:length`), position and
   `Seeked`, the state on every pause / resume, and a house key in the
   metadata, **`forkstify:next`** = "title — artist" of the next track in
   the queue. GNOME, KDE, waybar + playerctl benefit right away; Omarchy's
   media widget lights up as is. This is `mpris-server`: `set_metadata`,
   `set_playback_status`, `set_position`. Cover art (`mpris:artUrl`) would
   require the album image of the resolved track — the Spotify search gives
   it, to be kept for later.
2. **An Omarchy `forkstify` plugin**, a bar-widget, modelled on
   `omarchy.media` but **watching only the `forkstify` player**: in the bar,
   the animation's five frames on a 100 ms `Timer` when `isPlaying`, a fixed
   frame when paused, nothing if forkstify is not running. On click, the
   `PopupCard`: title, artist, a `position / length` progress bar, "up
   next: …" read from `metadata["forkstify:next"]`, and the three buttons.
   Other players stay with Omarchy's widget, should he want to enable it.

Why two floors: floor 1 serves everyone and depends on nothing; floor 2 is
the only piece tied to Omarchy, and it stays small.

## Settled (see above) — kept for the record

1. **Does the plugin watch only forkstify**, or every player like the old
   waybar script? Recommendation: only forkstify — "next track" only makes
   sense there, and `omarchy.media` exists for the rest.
2. **Where does the plugin live?** `omarchy plugin add` wants the manifest
   at the root of a git repository: (a) a separate `forkstify-omarchy`
   repository, installable in one command, or (b) an `omarchy/` folder in
   this repository, copied or linked by hand into
   `~/.config/omarchy/plugins/`. (b) to develop, (a) to distribute — the two
   stack (a repository holding only that folder). Recommendation: start with
   (b).
3. **Cover art** in the popover: right away (one more album image to resolve
   per track) or later? Recommendation: later, floor 1 first.
4. **A middle click / scroll** on the icon? The old script had none; Omarchy
   widgets do not use them regularly. Nothing to begin with.
