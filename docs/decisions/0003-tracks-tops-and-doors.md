# 0003 — Tracks are picked from tops, with manual targeting possible

**Date**: 2026-08-30 · **Status**: accepted — the "doors" part is
withdrawn by [0010](0010-revised-format-links-without-doors.md)

## Context

The branch engine reasons about **artists** (a few hundred of them, which
can be described and linked). It then has to choose which **track** to play.
Describing every track individually would make the cards far too heavy.

## Decision

- **Tops first**: each card lists the tracks that make up the artist's
  default pool. That is what plays on encore or on arrival.
- **Targeting specific tracks by hand stays possible**: a card may name
  **doors**, tracks hand-picked as an exit towards a given direction
  (*A Forest* → post-punk, *Friday I'm in Love* → pop). A door is optional,
  a top is the norm.

## Consequences

- Cards stay light: a list of titles, plus a few annotated doors when there
  is something to say.
- The engine can prefer a door to a top when changing direction, and a top
  when staying put.
- Only artists are vectorized; tracks are not.
