# 0025 — The review is the pull request: no `generated` flag

- **Date**: 2026-09-24
- **Status**: accepted · **Supersedes** the "`generated = true` until
  reviewed" consequence of
  [0016](0016-broad-base-and-on-the-fly-generation.md)

## Context

Every card the pipeline writes has carried `generated = true` since the
bootstrap, "until reviewed" ([0016](0016-broad-base-and-on-the-fly-generation.md)).
On 2026-09-24 the count was 382 flagged cards out of 383: the one exception
was a card Joel had just reread end to end, and the flag only went because
the agent suggested it. Nothing in the application clears it, nothing in the
engine reads it — it only fed a label, "generated card" against "written
card", on the listening screen and in the `Cd` and `Cp` overlays. A "to
review" that nobody lifts says nothing.

Meanwhile the review has a place of its own. A user's catalog is a fork
([0016](0016-broad-base-and-on-the-fly-generation.md)), what is theirs is
their commits ([0008](0008-the-fork-is-the-overlay.md)), and `Cp` turns
those commits into a pull request that the reference reads before merging.
The history already says who wrote what: a card is born in a commit "X —
card generated" with the trailer `Forkstify: generate`, a hand edit in "X —
edited by hand" with `Forkstify: edit`.

Joel, 2026-09-24: "I would like to remove every `generated`. If there are
changes to be made, they will be proposed and examined in a PR with `Cp`."

## Decision

1. **The review is the pull request.** A card in the reference has been
   accepted there; a card in a fork is its owner's. There is no third state
   inside the file.
2. **The `generated` field goes.** The generator no longer writes it, the
   commit bodies no longer say "to review", the labels "generated card" and
   "written card" disappear from the screens and from the proposals. The
   reader keeps tolerating the field in a card that still has it, so an
   older fork still loads.
3. **The base is accepted as it stands.** The 382 cards of the reference
   were merged without review, as the bootstrap of a base; they lose the
   flag in one commit, proposed from Joel's fork through `Cp`. Reviewing
   starts now, one pull request at a time.
4. **Who wrote a card is a question for git**, not for the format: the birth
   commit and its trailer. The format stays at version 1 — an optional
   field is dropped, nothing is added.

## Consequences

- `docs/design/catalog.md` closes its open question "flagging generated
  cards: a field, a separate folder, or both?" — neither. Its "two visible
  quality levels" become the two sides of the pull request.
- The `Cp` body no longer sorts the new cards into "generated" and "written
  by hand": it lists them, and the reviewer reads the diff.
- `Card.generated` leaves the code, with every label built on it.
- What this does not change: a card born from the pipeline is still an
  edit in the sense of [0013](0013-keyboard-tuning-measure-or-edit.md), one
  readable commit, undone by `git revert`.
