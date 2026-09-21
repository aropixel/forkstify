# 0020 — The target of a gesture: the highlighted row, otherwise what is playing

- **Date**: 2026-09-09
- **Status**: accepted

## Context

Since choosing a branch **appends** to the queue (2026-09-06), the "current"
the engine draws from the journey is the last artist of the last branch
stacked — the end of the chain, no longer what is playing. Joel, while
listening, took several branches then typed `en3`: the tracks added were
those of the artist at the end of the chain, not of the one he was
listening to and hovering over.

Two rules also coexisted on that axis: `ad`, `ti` and `tx` followed the
highlighted row, the rest of `t` and `a` followed the playing track (an
open question since 2026-09-07).

## Decision

Joel, 2026-09-09: "by default the track currently playing, but if a row is
highlighted after a move, it takes priority."

- **A gesture targets the highlighted row if there is one, otherwise the
  track that is playing.** One rule, for the three namespaces that speak of
  a track or its artist: `t`, `a` and `e`.
- The selection plays nothing and is visible: that is why it rules whenever
  it exists. Escape clears it and hands the target back to the playing
  track.
- **An encore lands near its target**: `e<n>` at the end of the queue;
  `en<n>` and `e!<n>` **behind the highlighted row** if it is still to come,
  otherwise right after the playing track (Joel, same day). `e!` drops
  whatever follows that point.
- `ts` and `tb` only move the music forward if they target what is playing.
  On an upcoming row, `ts` pulls it out of the queue ("skipping" a planned
  track means not playing it); on a past row, it only takes note.

## Consequences

- `under_needle` and `target` say the same thing; `e` hooks into it.
- The key table ([`keybindings.md`](../keybindings.md)) carries the rule at
  the head of the `t` namespace.
