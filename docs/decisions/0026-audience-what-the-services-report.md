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

But the service is often right. Joel, the same day: "there are many
`similar` links that were coherent on other cards". Measured on the 963
unannotated `similar` whose target has a card, the genre tags (neither a
country nor a decade, as `genre_tags` in `src/engine.rs` reads them) tell
the two apart:

| Case | Links | For instance |
|---|---|---|
| The two cards share a genre tag | 616 | Nina Simone → Ella Fitzgerald, Ray Charles, Amy Winehouse; IAM → JoeyStarr |
| One of the two cards has no genre tag | 120 | Expérience's four, Arthur Satàn's |
| Both have genres, none in common | 227 | |

The other 239 point at an artist with no card: nothing to compare them
with.

The vectors do not correct it, they echo it: a card with no genre tag is
vectorized mostly from its links, so the cosine from Expérience to Les
Ogres de Barback is **0.87** — higher than Nina Simone to Nat King Cole
(0.70).

## Decision

1. **A seventh link type, `audience`**: "a service reports these two
   artists together, and nothing in the catalog confirms it" — Deezer's
   related artists, Last.fm's similar ones, whatever the pipeline
   harvests. It says where the link comes from, not what it means.
2. **Its default closeness is 2** in `catalog.toml`, beside `influence`. A
   fork that trusts the services more raises it in one commit; that is the
   grid's purpose ([0010](0010-revised-format-links-without-doors.md)).
3. **The selective rule**: *a service's neighbour stays `similar` when the
   two cards share a genre tag; otherwise it is `audience`.* One sentence,
   checked against the catalog, no judgment invented. `similar` therefore
   means: written by hand (through `ae`), or reported by a service **and**
   confirmed by a shared genre, or an `audience` someone read and promoted.
4. **The generator applies the rule at birth.** A neighbour whose card
   already exists is typed by the rule; a neighbour with no card yet is
   `audience`. When that neighbour's card is born later, the same commit
   re-applies the rule to the `audience` links pointing at it — so the
   verdict follows the catalog as it fills in.
5. **The branch says it as it is**: an `audience` link's reason reads
   "same listeners", not "similar".
6. **`audience` stays out of the vectorized text.** The vector says what
   the card asserts; a service's neighbourhood would only push it towards
   the mistake the adventurous side is there to escape — the 0.87 above. A
   card with nothing but `audience` links gets a thin vector, and `check`
   shows it, which is the truth about that card. The links still give
   directions through the graph: the vectors do not decide whether a link
   comes up, its closeness does.
7. **The existing cards are retyped once, by the same rule.** In Joel's
   fork, one commit applies it to every `similar` that has no note and that
   no hand edit has touched since the card's birth (a `Forkstify: edit`
   commit, or any commit by hand on the line, keeps it `similar`). On the
   2026-10-06 catalog: 616 stay `similar`, 347 become `audience`, and the
   239 with no card become `audience` until their card is born. The commit
   is proposed to the reference through `Cp`; the diff is the review
   ([0025](0025-the-review-is-the-pull-request.md)).
8. **The format stays at version 1.** A type is added, nothing is removed,
   and the reference's `catalog.toml` declares `audience = 2` under
   `[proximity]`: a binary that predates this decision already accepts any
   type the grid declares (`src/validate.rs`), and the engine walks an
   unknown type at its declared closeness.

## Consequences

- **The weight of a service's word drops where nothing confirms it.**
  Expérience, with Michel Cloup at 5 and four `audience` at 2: the graph
  heads go 39 % to the four, against 72 % today (closeness², since the draw
  of 2026-10-06). A card whose only links are `audience` still has only
  those through the graph: the rule demotes them, the agent's review or a
  hand edit is what replaces them.
- **The rule is only as sharp as the genre tags.** They come from
  MusicBrainz and are sometimes broad: two cards tagged `rock` pass without
  sounding alike. The rule catches the flagrant cases, not all of them.
- `src/generate.rs` types by the rule, re-applies it at a neighbour's
  birth; its cap of four counts `similar` and `audience` together.
- `src/validate.rs` adds `audience` to its known types, `src/engine.rs` its
  label ("same listeners"), `src/fork.rs` counts it in the `Cp` summary.
- `src/embed.rs` gives it no phrase; the vector index is regenerated in
  the retyping commit, since the texts of the retyped cards change.
- `docs/design/catalog.md`: the table of types gains its line, and its
  sources table names `audience` for Deezer `/artist/related`.
- The agent's suggestions ([agent.md](../design/agent.md)) gain a natural
  move: **promote** an `audience` to `similar` (or `scene`, `influence`)
  with a note, or drop it — proposal by proposal, the listener's call.
