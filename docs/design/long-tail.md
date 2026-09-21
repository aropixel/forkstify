# The long tail — the pool's fourth source

Note opened on **2026-09-05** on a question from Joel ("how do we handle the
long tail *behind the scene*?"), after opening up the pool
([0012](../decisions/0012-track-rotation.md) §1), which implements three of
its four sources.

Nothing is coded. This note proposes, compares and recommends; the call is
Joel's.

## Decided — what we do not reopen

- **The tail is the pool's fourth source**, at a low weight: a top is a
  weight, not a closed list (0012 §1).
- **The comfort zone is what sets its depth** — high comfort, a tight draw
  on the tops; low comfort, the tail weighs more (0012 §4). There is **no
  second setting**: [0001](../decisions/0001-comfort-is-familiarity.md)
  already holds that dial.
- **It lives outside the catalog.** 0012 calls it "outside the catalog", and
  [catalog.md](catalog.md) explicitly puts the API caches ("title →
  identifier resolution, cover art") **outside the repository**. The vectors
  set no precedent: they ship because everyone has to have *the same* ones
  for "close" to mean the same thing everywhere, and because they need a
  100 MB model. A discography has neither problem — and redistributing a
  third party's data in a public repository would raise a terms-of-use
  question we have no reason to create for ourselves.

## The fact that settles the source

The cards carry **`mbid` (214/214)** and **`spotify` (212/214)**. **None
carries a Deezer identifier.**

Going through Deezer — as `tools/generate-cards.py` does with
`artist/{id}/top` — would therefore mean a `search/artist` per artist, with
the risk of homonymy the catalog has already paid for once ("Experience"
wrongly resolved to The Jimi Hendrix Experience, fixed on 09-02).

Going through Spotify starts from an identifier **already present and
already verified**, and returns `spotify:track:` directly — so **no title →
identifier resolution**, and none of the "not found on Spotify" that comes
with it.

**Recommendation: the tail comes from Spotify**, through the card's
`spotify` field.

An accepted consequence: `parcours` (the dry mode, without Premium) does not
*harvest* the tail. But it **reads the cache** that listening sessions have
filled, so it keeps working, with the tail of the artists already met.

## Direction — the proposed shape

- **Where**: `~/.cache/forkstify/discography/<slug>.json` (XDG).
  Regenerable, never committed, not synced (feedback item no. 10 does not
  apply to it).
- **What**: title + `spotify:track:` + album, deduplicated by normalized
  title, **with the card's tops removed** — the tail is what is *not*
  already in the pool.
- **The engine stays synchronous.** It reads an already loaded discography;
  filling it is the application's job, in the background. Same rule as
  "separate the brain from the sound": the engine produces, the application
  goes and fetches.
- **A display mark**: a fifth provenance alongside `♪ ♥ ↳ + ~`.

## Wired on 2026-09-05

Joel approved the calls; the two points the note left open were taken the
most sober way, and said so: **harvest at the moment of need** and **no
expiry**.

- `src/discography.rs` — the cache,
  `~/.cache/forkstify/discography/<slug>.json`, loaded once at startup. The
  engine reads it **synchronously**; the application fills it.
- `WebApi::discography()` — albums and singles, then their tracks in batches
  of twenty: a few calls per artist, once. A partial harvest is kept — the
  tail is a pool, not an inventory.
- **When**: automatically when `e<n>` asks for more depth than the card has,
  and `:warm` to force it on the current artist.
- **The dial finds its third lever**: the tail's share of the pool is
  exactly the comfort's openness (0012 §4). **Zero at the cocoon**, full at
  exploration. A test pins it down.
- **Deduplication by normalized title**: Spotify delivers the same song in
  ten guises ("- 2004 Remaster", "(Remastered)"), and a title already in the
  tops does not enter the tail — the tail is what is *not* already in the
  pool.
- **The `·` mark**, alongside `♪ ♥ ↳ + ~`.

The cards' `spotify` field, present from the start and never read by the
code, finally is: it is what opens the door.

## Wired on 2026-09-11 — the harvest follows the branches

Joel, after a few days of listening at comfort 3: "always top or liked
tracks, never any long tail". The diagnosis: **a branch never went looking
for the tail** — only `e<n>`, `:warm` and `ad` harvested it, and 16 artists
out of 314 had one. Where it existed, the weight was right (at comfort 3, a
tail track weighs 0.1 against 1 for a top and 6.8 for a liked one, but two
hundred tracks add up).

The option Joel chose (the first of the two proposed; the other was
preloading at startup): **harvest, in the background, the artists the
proposed branches cross**, as soon as comfort opens the tail up (openness >
0, so anything but the cocoon). One request per artist, cached forever,
silent — no toast and no sticky "loading", which stay with requested
harvests.

The delicate point: **a branch's tracks are drawn when it is proposed**. A
first wiring redrew the branches still on the table when the tail arrived;
in use, "the songs in the branches change immediately" (Joel, 2026-09-11),
and he does not want that. Two options put forward by Joel: wait for the
harvest before showing the branches, or show the first version with the tops
and let the cache serve the next ones. **Chosen: the second**, the simplest
and most reversible — a proposal once shown does not move; an artist's tail
serves from the next draw that goes through them, in this session or the
ones after. The first option stays open if the delay of a session's first
proposal ends up being a nuisance: it would need a `recompute` that waits
for its harvests.

**A consequence found in use (2026-09-11)**: the tail brings in tracks that
Spotify may refuse to play (unavailable in the region). Such a track ends
the instant it starts, and with no intervention `auto_advance` chained
branches without a sound. A guard in `listen.rs` (`MAX_DRY_ADVANCES`) stops
after a few silent branches in a row and hands control back. It does not
filter the tail — availability is only known by trying — it only prevents
the runaway.

## To settle — what is left

*(The three points below are settled; kept as a record of the reasoning.)*

1. **The depth.** ~~The extended top~~ (`/v1/artists/{id}/top-tracks`, 10
   titles, one call) largely overlaps the card's tops and is therefore
   barely a tail at all. The real tail needs the **discography**:
   `/v1/artists/{id}/albums` then `/v1/albums?ids=` in batches of 20 — on
   the order of 3 to 10 calls per artist, once, then it is cached.
2. **When to harvest.** In the background as soon as a branch is shown (the
   proposed artists are known in advance); or only at `e<n>` when an
   artist's tops run out — the moment depth really serves; or a warm-up
   command (`:warm`) that does the whole catalog in one go.
3. **Expiry.** Never (a discography barely moves, and a `:warm` forces the
   update), or a TTL.

## The order, and why it matters

**The comfort zone (0001) should come before the tail.**

0012 §4 is explicit: comfort *is* the tail's volume, and there is no second
setting. Shipping the tail without it means giving it an arbitrary fixed
weight — so either it is never heard, or it floods everything, and in both
cases it cannot be adjusted except by recompiling.

0001 is already decided and `learned/` has, as of today, supplied the
familiarity it needs. Wiring it first gives the tail its volume knob the day
it arrives.

**Done on 2026-09-05** (Joel's call): `engine::Comfort` exists, it drives
the adventurous branch's floor and the draw of the heads. What it lacks is
its third lever — **the depth of the draw within the pool** (0012 §4), which
has nothing to set as long as the tail does not exist. The knob is waiting
for its volume.
