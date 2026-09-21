# 0021 — The repository is the Omarchy plugin, and the player speaks MPRIS for real

- **Date**: 2026-09-10
- **Status**: accepted

## Context

Joel wants in the Omarchy bar what he had under waybar: a `▂▄▆` animation
when music is playing, and on click a popover with title / artist /
progress / next track, like every Omarchy plugin. And he wants
**`omarchy plugin add <repo>` to install forkstify and make the icon
appear** — the project is not distributed any other way.

What the machinery imposes: `omarchy plugin add` clones the repository,
requires `manifest.json` **at its root**, validates, moves it into
`~/.config/omarchy/plugins/<id>/`, enables it. No install hook. And
forkstify, already an MPRIS player, only published "playing" on start.

## Decision

1. **The forkstify repository is itself the Omarchy plugin.**
   `manifest.json` at the root, the widget code in `omarchy/`. Cloned by
   `omarchy plugin add`, it is the whole repository that lives in
   `~/.config/omarchy/plugins/<id>/` — with `bin/build` inside it.
2. **It is the plugin that installs and launches the binary.** As long as
   `forkstify` is not on the `PATH`, the popover offers "Install" (builds in
   the container, links `target/release/forkstify` into `~/.local/bin`); as
   long as it is not running, it offers "Launch"
   (`omarchy-launch-or-focus-tui forkstify`). Prerequisite: tokens and
   caches move out of `target/` into an XDG folder — launched from the bar,
   forkstify no longer has a working directory.
3. **forkstify publishes real MPRIS metadata**, for any desktop: title,
   artist, duration, position, the state on every pause and resume, and the
   house key **`forkstify:next`** ("title — artist" of the next track).
   Pushed from `paint`, like the screen: derived from the state, only when
   they change.
4. **The widget only watches the `forkstify` player.** Other players stay
   with Omarchy's media widget. No cover art to begin with, nothing on
   middle click or scroll.

## Consequences

- Three floors, in order: the MPRIS metadata (done the same day), the XDG
  paths, then the plugin in `omarchy/`.
- The manifest at the root is a public interface of the repository: in
  English, like the rest.
- The living note: [`docs/design/omarchy-bar.md`](../design/omarchy-bar.md).
