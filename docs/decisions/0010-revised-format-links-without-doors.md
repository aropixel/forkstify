# 0010 — Revised format: English fields, typed links with proximity, no more doors

**Date**: 2026-08-31 · **Status**: accepted — removing the doors was
reversed into an additional criterion by [0011](0011-doors-an-additional-criterion.md) ·
**Amends** [0003](0003-tracks-tops-and-doors.md) (doors are withdrawn) and
refines [0007](0007-cards-in-toml.md)

## Context

Joel's review of the first thirty cards brought out three requests: link
types in English (or a numeric proximity), doubt about the doors ("what
speaks to me is moving from one artist to another, aiming at the tracks I
love — not 'after A Forest I feel like something else'"), and a layout that
makes tuning easier.

## Decision

- **The format's keys are in English** (`name`, `begin`, `origin`, `links`,
  `to`, `generated`…): the format is a public interface, beyond French. The
  *content* (descriptions, notes) stays in each catalog's own language.
- **`links` replaces block-form connections**: one link = one line (inline
  TOML table). **Closed types, in English**: `member`, `family`, `collab`,
  `scene`, `similar`, `influence`.
- **Each type has a default proximity**, set once in `catalogue.toml` at the
  root of the catalog (`member = 5` … `influence = 2`); a link may override
  it locally with `proximity = 1..5` (5 = all but the same universe). The
  type names and explains the branch, the proximity gives the engine the
  distance — never one without the other.
- **Doors disappear from the format.** Choosing the track at the destination
  artist goes back to the engine: tops + usage signals + comfort zone
  (familiar or discovery), not an exit carved by hand.
- `fiches/` stays **flat**; fields are ordered by how often they are edited
  (compact identity, then tags, tops, links, description).

## Consequences

- The 30 cards of the first batch are regenerated in this format;
  `catalogue.toml` created.
- The type → default proximity grid is a catalog setting that anyone can
  change in their fork: "tuning your own algorithm" now has an address.

> Names on disk as of this decision. `catalogue.toml` and `fiches/` were
> later renamed to `catalog.toml` and `cards/` by
> [0022](0022-english-interface.md).
