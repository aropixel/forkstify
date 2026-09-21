# 0016 — A broad base **and** on-the-fly generation

**Date**: 2026-09-06 · **Status**: accepted

## Context

`docs/design/first-run.md` (2026-09-06) raised a hard point nothing had
written down: **the current base is not neutral, it is Joel's universe**.
The 214 cards grew out of his Spotify ranking and then outwards, one
neighbor at a time. And since **a seed without a card cannot start** —
branches come from the card's links, tags and vector — a user with distant
taste could barely start anything at all.

Three ways out were proposed: a broad, neutral base (a), a thin base with
mandatory generation (b), or several bases by family of taste (c).

## Decision

**(a) and (b), together.**

- **The reference catalog aims for breadth.** It is not its author's library
  but a base someone else can use: we widen it upstream, outside sessions,
  with the tooling. That is the one the application offers to import on
  first run.
- **And the application generates a card on the fly** when you arrive at an
  artist that has none. Everyone's catalog thus grows towards their own
  universe from the very first listen, which
  [design/catalog.md](../design/catalog.md) already called "the central
  mechanism".

The two complete each other rather than compete: breadth makes the install
work right away, generation keeps it from ever staying foreign. Neither one
alone is enough — a broad base always ends up missing someone, and a thin
base makes the application unusable offline on day one.

**(c) is not dismissed**, it simply no longer needs deciding:
[0004](0004-two-repositories-targetable-catalog.md) already allows importing
"somebody else's because it looks cool". Bases by family of taste can exist
with no new decision.

## Consequences

- **The active catalog's path is a setting** (`[catalogue] path`), no longer
  a hard-coded path. The command-line argument overrides it.
- **One fork per user, not a repository of differences.** Someone's catalog
  is a **fork of the reference repository** — so it holds *everything* in
  it, plus their commits. That is what allows both `git pull` from upstream
  and a PR back to it. A repository holding only the changes could do
  neither, and would contradict
  [0008](0008-the-fork-is-the-overlay.md): "there is no separate overlay".
- **A generated card is an edit** in the sense of
  [0013](0013-keyboard-tuning-measure-or-edit.md): it produces a commit and
  carries `generated = true` until reviewed, like the bootstrap cards.
- **Generation needs the network.** Offline, arriving at an artist without a
  card stays a dead end — the application says so rather than failing.
- Still to write, and to settle along the way: the **import on first run**
  (does forkstify clone, or ask for a URL?), the **scan of the user's
  library**, and **when** generation fires — on arrival at the artist, or on
  demand.
