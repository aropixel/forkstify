# 0018 — One gesture for taste: liking outranks the tops

- **Date**: 2026-09-08
- **Status**: accepted

## Context

After a day of listening, Joel: "as a developer, I understand the need for
tops (entry points). As a user, I had trouble knowing whether I should like
the track or make it a top: to me it was the same thing."

Two keys did, to the ear, the same thing — `tl` wrote into the learned
layer, `tt` into the card, for everyone — and the first one had **no effect
at all on a top**: a liked track only entered the pool if it was not
already a top (weight 0.8 against 1.0). A user who liked a top did not hear
it any more often, and switched to `tt`, promoting their taste into shared
knowledge without going through a PR.

## Decision

- **While listening, one simple gesture says "I want to hear this track more
  often"**: `tl`. Its opposite, `ts`, says "this track does not interest
  me": it notes that, removes the like, and skips. `tb` remains the "never
  again". Liking erases the "less often"; each undoes the other.
- **Liked tracks outrank the tops.** In the pool (0012 §1), a liked track
  weighs **ten tops at the cocoon and two wide open**, with the comfort dial
  in between — never less than a top. A liked top takes the weight of the
  liked track and carries its `♥` mark.
- **Tops are now only the entry doors of a fresh fork**: the way into an
  artist you have not listened to yet. They stay editable on your fork — in
  the discography (`ad`, which fixes them in a batch) or by hand in the card
  — but **there is no `tt` / `tT` shortcut while listening any more**.

## Consequences

- 0013 is not revised: every key remains a measurement or an edit; only the
  table changes, and it lives in `keybindings.md` (0015). The edits left
  while listening are `td` and `aL`.
- Taste never silently becomes knowledge: **promotion** ("liked three times,
  not in the tops — promote to top?") is still to be designed, and will ask
  a question, never act on its own.
- The distinction reads where you choose: the input hints say "more often:
  I like it" and "less often: it does not interest me".
