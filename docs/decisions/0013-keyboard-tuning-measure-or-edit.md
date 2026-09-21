# 0013 — Keyboard tuning: every key is either a measurement or an edit

**Date**: 2026-09-01 · **Status**: accepted

## Context

While listening, the user has to be able to tune their algorithm without
leaving the flow: promote a track to top, set it aside, make it a door…
The interface is keyboard-first, neovim-style.

## Decision

- **Every key is either a measurement or an edit.** A **measurement**
  changes `usage/` silently (skip, like) — that is the *learned* layer. An
  **edit** changes a card and produces **a readable commit** ("top: + A
  Forest") — that is *mine*, shareable at once.
- **`u` undoes the last action**, whatever it was: a revert for an edit, an
  erasure for a measurement. The tuning history *is* the git log.
- **Every key is merely the shortcut of a `:` command** (`:top`,
  `:door post-punk`, `:comfort 2`): everything is discoverable and
  scriptable, and the keybinds are a lookup table remappable in the config.

## Consequences

- The key table (`t`/`T`, `x`/`X`, `a`, `d`, `m`, `e`, `-`, `z`, `?`,
  `y`/`n`) is a direction in `docs/design/application-shape.md` — it gets
  adjusted along the PoC without revisiting this decision.
- Nothing the engine learns is a black box: every edit is a diff you can
  read back and undo, consistent with "the fork is the overlay"
  ([0008](0008-the-fork-is-the-overlay.md)).
