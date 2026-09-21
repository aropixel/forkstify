# 0012 — Track rotation: repetition must never be imposed

**Date**: 2026-09-01 · **Status**: accepted

## Context

Tracks are picked from tops ([0003](0003-tracks-tops-and-doors.md)). If the
engine always takes the head of the tops, the same tracks keep coming back.
Yet tops exist precisely *to* be replayed: repetition is not a bug, it just
must never be **imposed**.

## Decision

Four mechanisms that stack, each explainable in one sentence:

1. **A top is a weight, not a closed list.** An artist's pool adds up the
   tops (heavy weight), the user's liked tracks for that artist (`usage/`),
   the doors and the rest of the known discography (API cache, outside the
   catalog). The engine **draws at weighted random** from that pool.
2. **Freshness (cooldown).** Every play is dated in `usage/`; a recently
   played track is penalized, and the penalty decays over time.
3. **No replacement within a journey.** Never the same track twice in one
   journey; encore draws without replacement — the second encore goes down
   towards the less known.
4. **The comfort zone sets the depth of the draw.** High comfort: a tight
   draw on the tops (repetition *chosen*); low comfort: the long tail
   weighs more. That is the role the dial already has in
   [0001](0001-comfort-is-familiarity.md) — no second setting.

## Consequences

- `usage/` carries play dates, not just counters.
- The extended discography lives in an API cache, never in the catalog.
- Left to settle along the PoC: how fast the cooldown decays and the exact
  shape of the learned data (open question in
  `docs/design/catalog.md`).
