# 0023 — The engine's numbers are tunable in the configuration

- **Date**: 2026-09-17
- **Status**: accepted

## Context

The project's rule fits in one sentence: any automatic decision must be
explainable in one sentence and changeable in one commit. The decisions
were; their **coefficients** only were for whoever recompiles. A track's
cooldown (`0.1`, seven days), an artist's (`0.3`, four days), what "less
often" is worth (`×0.7`) and "more often" (its mirror), the weight of a
liked track (`×10` at the cocoon, `×2` wide open), of a door, of the tail,
the thresholds of the adventurous leap: all of it lived in Rust constants,
in `engine.rs` and `learned.rs`.

Joel, 2026-09-17: "push the logic of *taking back the algorithm* all the
way and give whoever installed forkstify the ability to change the value of
every number through the configuration file".

## Decision

1. **Every number of the engine and of the learned layer is a setting**, in
   a `[tuning]` section of `~/.config/forkstify/config.toml`, one English
   name per number (`track_cooldown_floor`, `less_often`,
   `liked_weight_cocoon`, `leap_floor_open`…). The default values are the
   ones the code shipped: writing nothing keeps today's behavior. The
   template written on first run lists every setting, commented, at its
   default value.
2. **A setting is read at launch and does not move during a journey**:
   `config::tuning()` gives the values in force, those of the file once
   loaded, the defaults before that (and in the tests). Changing a number
   means editing the file and relaunching.
3. **An absurd number is reported and reset to its default, alone**: a share
   outside 0–1, a negative half-life, a zero weight. The others stay as
   written. The engine must never run with a negative weight, but a typo
   must not erase the other choices.
4. **What is not an engine number is not tuned there**: interface delays (how
   long a toast lasts, restarting a track, the seed offer), screen sizes,
   the number of tops on an album stay constants. They decide nothing about
   what plays.
5. **What is structural stays in the code**: the number of candidates drawn
   (`take(6)`, `truncate(12)`), the powers in the weighting (`weight²`,
   `(score − 0.5)³`), the `4.0` weight of the `stay` branch. Those are
   shapes, not numbers; if one needs opening up, it will be one more line in
   `[tuning]`, not another mechanism.

## Consequences

- `config.rs` carries the `Tuning` struct, its defaults and its guard rails;
  `engine.rs` and `learned.rs` no longer hold a numeric constant. A new
  setting takes three lines: the field, its default, its template line.
- A user who shares their catalog fork does not share their settings: those
  are in their configuration, not in the repository. That is intended — the
  card says what an artist is, the setting says how *you* listen.
- [`docs/tuning.md`](../tuning.md) is the reference for every setting: what
  it does, its default, the effect of raising or lowering it. The
  configuration template gives the short version; `comfort-zone.md` and
  `branch-engine.md` give the mechanics.
