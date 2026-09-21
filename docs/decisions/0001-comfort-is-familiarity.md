# 0001 — The comfort zone measures familiarity

**Date**: 2026-08-30 · **Status**: accepted

## Context

The "comfort zone" dial (0 to 5) has to measure a distance. Two definitions
were possible:

- closeness to the **current track** (how well the sequence hangs together);
- closeness to **what the user already knows** (library, history).

They diverge: a big leap towards an artist you love is far from the current
track yet perfectly comfortable.

## Decision

**Comfort = familiarity.** The comfort zone measures how far you are willing
to stray from what you know. How well the sequence hangs together with the
current track is a separate, implicit parameter, handled by the branch
engine.

## Consequences

- The engine needs to know what the user knows: their Spotify library, their
  listening history, their past journeys.
- A "sidestep" branch can be comfortable if it lands on familiar ground; a
  "neighborhood" branch can be uncomfortable if the neighbors are unknown.
  Branch type and comfort level are two independent axes.
