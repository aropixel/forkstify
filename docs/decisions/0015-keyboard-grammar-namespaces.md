# 0015 — Keyboard grammar: four namespaces

**Date**: 2026-09-05 · **Status**: accepted

## Context

The first long `ecouter` sessions (2026-09-05) produced eleven pieces of
usage feedback, including this one: "I'm starting to get mixed up, I don't
want to add more and have it become unusable".

Taking stock proved that worry right. The key table lived in three copies
(the code, `forme-de-l-application.md`, `retours-usage.md`), **eleven
gestures out of some thirty** were wired, and bringing them together
revealed **eight collisions**: `u` meant two things, `n` three, `d` and `p`
were both actions and prefixes, `dt` and `da` duplicated gestures already
decided, `.` duplicated `e`, `h`/`l` duplicated `1 2 3`.

Decision [0013](0013-keyboard-tuning-measure-or-edit.md) already set the
*principle* (every key is a measurement or an edit, each one the shortcut
of a `:` command) but left the table as a direction. It did not survive the
next addition.

## Decision

**The first character says what you act on, the second what you do.**
Four namespaces:

> **`f` the branch · `e` encore · `t` the track · `a` the artist**

The rest of the keyboard only navigates and drives the session (`h`/`l` and
the arrows, space, `u`, `q`, `Q`, `.`, `?`, `/`, `:`).

Three rules:

1. **A key comes from an English word**, vim-style (`y` yank, `c` change) —
   `f` fork, `e` encore, `t` track, `a` artist, then `l` like, `s` skip,
   `b` ban, `m` mark, `t` top, `d` door, `e` edit, `L` link, `p` peek,
   `r` reroll, `w` wander, `u` undo.
2. **The target is a prefix, never a suffix.** `tl` likes the track, `al`
   likes the artist. Two gestures can therefore only clash inside one
   namespace, where the letters are under control.
3. **A frequent gesture gets a key, a setting gets a `:` command.**
   `:size`, `:comfort`, `:sync`, `:fork`.

**The table is authoritative in [`docs/keybindings.md`](../keybindings.md)** —
one copy, never three again.

## Consequences

- **The eight collisions fall**, and by construction: a namespace makes them
  impossible anywhere but inside itself.
- **The bare keyboard empties out** — it keeps only `h l e f t a u q Q` —
  which leaves eighteen letters for the queue mode and whatever comes next.
- **`u` and `fu` split apart**: `u` undoes the last gesture (so 0013 is
  honoured, not revised), `fu` steps one notch back up the journey.
- **Frequent gestures cost two keystrokes** (`tl`, `ts`) where vim keeps
  one. That is the price of regularity, and only use will tell. `1` `2` `3`
  remain the shortcut for `f1` `f2` `f3` — the deliberate exception, since
  choosing a branch is *the* gesture of the product. If another gesture
  turns out to be constant, the bare keyboard is empty enough to promote it
  there.
- **The count comes after the verb**: `f3` branch 3, `e3` three encores. The
  vim rule (`3dd`) would want the opposite, but it is **incompatible** with
  the `1 2 3` shortcut: pressing `3` would be both "branch 3" and "the start
  of a count", and the parser cannot decide without waiting for the next
  key — which would make the most frequent gesture slow. So the count
  follows the namespace, which has the advantage of making `f` and `e`
  symmetric. Corollary: `e` alone is not a command, the count is mandatory.
- **Input goes raw** (no Enter), except `/` and `:` which open a line — that
  is feedback item no. 1, and it is what makes the grammar typable.
- 0013 is not revised: its principle (measurement / edit, `u`, `:` commands)
  is carried over as is. Only its **table**, which was only a direction, is
  replaced.
