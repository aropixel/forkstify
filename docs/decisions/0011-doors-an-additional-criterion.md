# 0011 — Doors come back (`doors`), as an additional criterion

**Date**: 2026-08-31 · **Status**: accepted · **Amends**
[0010](0010-revised-format-links-without-doors.md) and partially restores
the "doors" part of [0003](0003-tracks-tops-and-doors.md)

## Context

[0010](0010-revised-format-links-without-doors.md) had removed the doors:
choosing the track was entirely up to the engine (tops + usage + comfort
zone). On second thought, Joel came back on it: "occasionally, on a few
targeted tracks, it can be useful; but it will only ever be an additional
criterion, not the main one".

## Decision

- The **`doors`** field comes back into the card, optional, in the inline
  form used by `links`:
  `{ track = "A Forest", to = ["post-punk", "atmospheric"], note = "…" }`.
  `to` points at **tags** (a direction), not at artists.
- **An additional criterion, never the main one**: when the engine leaves an
  artist towards a direction that overlaps a door's tags, that track gets a
  **bonus** — it becomes neither mandatory nor exclusive. Tops, usage and
  the comfort zone remain the primary criteria. A card without a door
  behaves exactly as before.

## Consequences

- The doors written in the first batch are restored in the cards concerned.
- The engine treats `doors` as a weighting on track choice, not as a routing
  rule.
