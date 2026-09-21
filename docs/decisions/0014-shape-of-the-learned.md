# 0014 — Shape of the learned layer: `learned/`, one file per artist, decaying counters

- **Date**: 2026-09-04
- **Status**: accepted

## Context

The base / mine / learned model ([catalog.md](../design/catalog.md)) puts
the *learned* layer — what usage produces — in a separate folder, version
controlled, never contributed upstream. Decisions
[0012](0012-track-rotation.md) (dated cooldown) and
[0013](0013-keyboard-tuning-measure-or-edit.md) (every measurement writes
into that layer) depend on it, but its **shape** was still to be settled:
one file or one per artist, which counters, what decay.

Those earlier notes called the layer `usage/`; this decision settles the
final name and supersedes that incidental usage.

## Decision

- **The learned folder is `learned/`** (not `usage/`). Vocabulary on disk —
  paths, subfolders, fields — is in **English**, just like the card format:
  the repository aims at open source, and those names are a public
  interface.
- **One file per artist**, `learned/artists/<slug>.toml`, mirroring the
  cards: clean diffs, no conflict with upstream (everyone has their own),
  and it scales. **Marks** (the `m` key) live separately and across
  artists: `learned/marks/<name>.toml`.
- **Counters with decay built in.** We do not keep a play history but a
  decayed count: on every play,
  `plays = plays × ½^((now − last)/half-life) + 1`, `last = now`. One float
  and one date per artist and per top is enough; an old play barely weighs
  at all. **Half-life: 6 months.**
- **Silent, never contributed back.** The learned layer changes without
  confirmation (a measurement) and never enters a PR.
- Every field, what each key of 0013 writes, and what the engine reads
  (excluding `blacklisted`, familiarity → comfort, cooldown, weights) are
  detailed in the "Shape of the learned layer" direction in
  [catalog.md](../design/catalog.md).
- `learned/classement.json` (the bootstrap, 741 artists scored from the
  library) becomes the **starting familiarity**, extended by
  `learned/artists/`.

Still tunable along the PoC, with no new decision: the cooldown window
(0012) and the familiarity → comfort zone 0–5 formula (0001).

## Consequences

- The catalog's `usage/` folder is renamed `learned/`; the scripts in
  `outillage/` follow. The bootstrap files still named in French
  (`amis/`, `artistes-*.json`) and their keys will be translated along with
  the scripts (a separate `outillage/` task).
- Decisions 0012 and 0013, being immutable, mention `usage/`: read
  `learned/`.
