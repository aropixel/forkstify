# 0026 — `audience`: what a service reports is not a similarity

- **Date**: 2026-10-06
- **Status**: proposed (awaiting Joel) · **Amends** the closed list of link types of
  [0010](0010-revised-format-links-without-doors.md)

## Context

Every generated card takes up to four neighbours from Deezer's
`/artist/related` (`src/generate.rs`) and writes them as `similar`, at the
grid's 4 — as strong as a collaboration, as strong as a `similar` written
by hand. In the catalog on 2026-10-06: 1,218 `similar` links on 354 cards,
16 of them with a note.

Joel, 2026-10-06, starting a session on **Expérience** (Michel Cloup's
band, Toulouse, 1999–2009): the band is linked to "the bands of the same
scene of the time, though those bands are not in the same musical universe
at all". The card's four `similar` are Mano Negra, Zebda, Loïc Lantoine and
Les Ogres de Barback. The same day, **Arthur Satàn**'s four were Porridge
Radio, King Hannah, Cate Le Bon and Ghostwoman, where the kinship one hears
is the Beatles and the Kinks.

Deezer does not document how it computes its related artists. What the two
cases show is a shared public — French, the nineties, alternative, the same
city — rather than a shared sound. Whatever it measures, it is what a
service **reports**, not a matching of taste: and `similar` is precisely
"a matching of taste" ([catalog.md](../design/catalog.md)), "the one that
pulls the base towards one ear". Written as `similar`, the service's word
borrows the weight of a judgment no one made, and the branch says "similar"
where it should say where it comes from.

The vectors do not correct it: a card with no genre tag (Expérience has
`fr` and `90s`) is vectorized from its links, so its vector leans the same
way as the links it should have checked.

## Decision

1. **A seventh link type, `audience`**: "a service reports these two
   artists together" — Deezer's related artists, Last.fm's similar ones,
   whatever the pipeline harvests. It says where the link comes from, not
   what it means.
2. **Its default closeness is 2** in `catalog.toml`, beside `influence`. A
   fork that trusts the services more raises it in one commit; that is the
   grid's purpose ([0010](0010-revised-format-links-without-doors.md)).
3. **`similar` is a judgment again**: written by hand through `ae`, or an
   `audience` someone read and promoted. The generator never writes it any
   more.
4. **The branch says it as it is**: an `audience` link's reason reads
   "same listeners", not "similar".
5. **`audience` stays out of the vectorized text.** The vector says what
   the card asserts; a service's neighbourhood would only push it towards
   the mistake the adventurous side is there to escape. A card with nothing
   but `audience` links gets a thin vector, and `check` shows it — which is
   the truth about that card.
6. **The existing cards are retyped once.** In Joel's fork, one commit
   turns into `audience` every `similar` that has no note and was never
   touched since the card's birth (git says so: no `Forkstify: edit` commit
   has changed the line). The others stay `similar`. The commit is proposed
   to the reference through `Cp`; the diff is the review
   ([0025](0025-the-review-is-the-pull-request.md)).
7. **The format stays at version 1.** A type is added, nothing is removed,
   and the reference's `catalog.toml` declares `audience = 2` under
   `[proximity]`: a binary that predates this decision already accepts any
   type the grid declares (`src/validate.rs`), and the engine walks an
   unknown type at its declared closeness.

## Consequences

- `src/generate.rs` writes `audience`; its cap of four counts `audience`.
- `src/validate.rs` adds `audience` to its known types, `src/engine.rs` its
  label ("same listeners"), `src/fork.rs` counts it in the `Cp` summary.
- `src/embed.rs` gives it no phrase; the vector index is regenerated in
  the retyping commit, since the texts of the retyped cards change.
- `docs/design/catalog.md`: the table of types gains its line, and its
  sources table names `audience` for Deezer `/artist/related`.
- The agent's suggestions ([agent.md](../design/agent.md)) gain a natural
  move: **promote** an `audience` to `similar` (or `scene`, `influence`)
  with a note, or drop it — proposal by proposal, the listener's call.
- What this does not change: an `audience` link is still a direction the
  engine can take. It is merely weighed for what it is.
