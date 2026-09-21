# Exploring an artist's discography

Note opened on **2026-09-07**, on feedback from Joel (item no. 12 of
[`usage-feedback.md`](usage-feedback.md)):

> "I started a 'Cat Power' seed, and it gave me 3 tracks by that artist and
> the first one started. As it happens I like almost nothing but tracks from
> the album *What Would the Community Think*. I would have liked a command
> (`:explore`?) to get a visual list of the available tracks sorted by
> album, and to be able to `tt` the tracks I like and `tT` the existing tops
> I want to remove."

**Wired on 2026-09-07**, in form **1a** of the `Discographie.dc.html`
mockup (Joel's call). What follows keeps the reasoning; what was settled is
marked as such, and the key table lives in
[`keybindings.md`](../keybindings.md).

## What the feedback really says

Three tracks came out because the **branch size** is 3 (`:size`,
`src/listen.rs`), not because the card has three tops — a detail, but it
shifts the question: the problem is not the number, it is that **Cat
Power's pool does not look like what Joel likes about her**.

Yet that is exactly the gesture 0013 promises: `tt` promotes, `tT` removes.
What is missing is not the edit — it has been wired since 09-06 and it
commits — it is **only being able to perform it on the track that is
playing**. To straighten out an artist, you would have to grind them
entirely, one listen per fix. The command asked for flips the ratio: show
everything, fix it at a glance.

It is also the first screen where you **look at the catalog instead of
listening to it**. It had better, then, say everything the engine knows
about a track, not just its title: that is where a card gets read back.

## What it rests on — almost everything is already there

| Brick | Where | State |
|---|---|---|
| The full discography, albums and singles, with each title's album and its `uri` | `src/spotify.rs::discography`, `src/discography.rs` | ✅ harvested by `:warm`, cached outside the repository, regenerable |
| Promoting / removing a top, and the commit that goes with it | `src/edit.rs::add_top`, `remove_top` | ✅ |
| What usage knows of a track (plays, skips, liked, banned) | `src/learned.rs` | ✅ |
| Conflating "A Forest" and "A Forest - 2005 Remaster" | `discography::normalize` | ✅ |
| A list you walk with the arrows, full screen | `tui.rs::render_collection` (home's collection) | ✅ the model is written |

**The feature is therefore a screen, not an engine.** That is what makes it
small — and what argues for doing it before the heavy workstreams (the dated
cooldown, the queue tree).

## The form chosen

**Joel's call, 2026-09-07: the key is `ad`, and it is a modal.** Not a
screen that replaces listening — a **modal laid over the listening
screen**, which takes the keyboard for as long as it is open and gives it
back on `escape`, the way the comfort setting takes over the keys
(`comfort_before`). What is playing stays visible behind: you do not leave
listening to straighten out a card, and the sound never stopped anyway.

`ad` reads **a**rtist **d**iscography; `d` was free in the `a` namespace,
and 0013 wants every key to be the shortcut of a command — it is
`:discography`.

```
 Cat Power — 214 tracks, 19 albums · 3 tops · comfort 2
 ────────────────────────────────────────────────────────
  What Would the Community Think (1996)
   ♪  Nude As the News                        12 plays
   ♥  Good Clean Fun                           4 plays
      They Tell Me                             ·
   ⊘  Enough                                   2 skips
  Moon Pix (1998)
   ♪  Cross Bones Style                        8 plays
      Metal Heart                              ·
 ────────────────────────────────────────────────────────
  ↑↓ browse · tt top · tT remove · tl ♥ · tb ⊘ · escape
```

- **Grouped by album, oldest to newest** — that is how you remember an
  artist, and it is what was asked for.
- **The glyphs are the table's** (`keybindings.md`, "where each track comes
  from"): `♪` a top from the card, `♥` liked here, `↳` a door, `⊘` banned,
  nothing for the rest. A glyph carries one meaning, here as elsewhere.
- **The right-hand column is what the learned layer knows**: plays, skips,
  last time. That is what lets you settle "I think I only like this album"
  by checking it.
- **A final section, "tops not found in the discography"**: the titles the
  card declares and that Spotify does not return under that name — a typo,
  a live version, a compilation. That is the screen's "audit" half, and
  `tT` must work there as elsewhere.

### The gestures, inside the screen

| Key | Effect | Why |
|---|---|---|
| ↑ ↓, `gg`, `G` | Move the current row | As everywhere |
| `tt` / `tT` | Promote / remove **the row** from the tops | The request, and the same fingers as while listening |
| `tl` / `tb` | Like / ban **the row** | Measurements: they only write into `learned/`, nothing to commit |
| `/text` | Filter the list | A habit already formed for searching |
| `escape` | Close | Like a block |

`td` (door) is **not** there: a door points at the direction you are heading
in (0011), and this screen has no "next". The gesture keeps its meaning
while listening, where it has one.

## What the code gained

1. **Four fields on `TailTrack`** (facts): `album_id`, `release_date`,
   `track_number`, `group` (album/single). Without the date, no
   chronological order; without the number, no order within the album. The
   cache is **regenerable and outside the repository** — an entry with no
   date reads back with `#[serde(default)]` and triggers a new harvest, with
   no migration.
2. **Deduplication at display time.** Spotify delivers the same track five
   times (album, single, reissue). `normalize` already knows how to conflate
   them: we keep **the oldest occurrence of album type** and hide the
   others. Without that, Cat Power is three hundred rows of duplicates.
3. **The title written into the card must be clean.** `tt` on "Nude As the
   News - 2015 Remaster" must not write that title: it needs the cut
   `normalize` already makes, but one that leaves the title readable rather
   than a keyword (`clean_title`).
4. **`tT` removes the card's string, not Spotify's.** They are not always
   identical; the row must therefore carry the title of the top it
   recognized, matched by `normalize`. Otherwise `remove_top` finds nothing
   and says "not in the tops" while the `♪` is on screen.
5. **Harvesting on open**: if the artist's tail is not cached, the screen
   harvests it (`harvest`) instead of demanding a prior `:warm`. Offline or
   with no Spotify identifier, it says so and opens only what it has: the
   card's tops.

In volume, that is: `discography.rs` and `spotify.rs` touched up, one more
piece of state in `Live`, a `render_explore` modelled on
`render_collection`, and nothing new in `edit.rs` but the clean title.

## Settled on 2026-09-07, and wired

1. **The target.** `ad` aims at the artist of the **highlighted** row if
   there is one, of the current track otherwise — and the modal's header
   names the artist that is open. The rest of the `t`/`a` namespace acted on
   the current track, except `tx`; **unified on 2026-09-09**
   ([0020](../decisions/0020-the-target-of-a-gesture.md)): every gesture
   targets the highlighted row, otherwise what is playing.
2. **The name and the form.** `ad` / `:discography`, and a **modal** laid
   over the listening screen — not a screen that replaces it. Playback never
   stopped, and you see it behind.
3. **One commit for the batch.** The `tt`/`tT` accumulate at the bottom of
   the modal and go out on ⏎ as **one write, one commit**
   (`edit::set_tops`): five commits for one single thought do not read back.
   `u` undoes the last one as long as nothing is written, and the first
   escape warns if any are left.

   *Joel's question, that day: "for the commits, I thought we had said on
   close, and every ten minutes".* That is
   [0017](../decisions/0017-syncing-the-learned.md), and it covers **the
   learned layer** — measured, silent, never read back line by line.
   **Edits** fall under 0013: written and committed per gesture, because
   they leave a readable, revertible trace. The batch does not change that
   rule, it groups the gestures of one screen.
4. **Enter writes** — and it is `e` that queues, without closing. An edit
   only counts for the engine at the next launch: `e` is the answer to "I
   want to hear it now". **Since 2026-09-11, enter on a track also sets off
   from it** (Joel: "start a new seed from a song on the discography
   screen"): the batch is written first if there is one, then the seed
   replaces the journey — the track plays, the branches set off from its
   artist, like the search modal. On an album row, enter only writes; a
   banned track does not set off. A reversible choice: if writing without
   setting off turns out to be missed on a track, a separate key (`w`) will
   bring it back.
5. **The four additions retained** (Joel): `s` toggles the order
   (chronological ⇄ my plays first), `v` cycles the view (all, ♪ tops,
   ♥ liked, ⊘ banned), `A` promotes the album's four most played titles,
   `e` queues.

## What is still open

- **Folding the albums** can only be judged in use: no per-album fold is
  remembered, `h` folds everything and `l` reopens the one under the cursor.
- **Compilations and guest appearances** stay out of the harvest
  (`include_groups=album,single`): a title that only exists on a compilation
  does not show up — unless it is one of the card's tops, in which case it
  lands in "tops outside the discography".
- **The tail cache** gained four fields (date, position, length, single). An
  older harvest is **redone silently** on open: the cache is regenerable and
  outside the repository, there is nothing to migrate.

## What it does not do

- **Neither rename nor edit a card by hand**: `ae` remains the gesture for
  that, and it still awaits a prompted input.
- **Neither touch the links nor the tags**: this screen is the tracks'.
- **Nor propose upstream**: what gets fixed here goes into the fork, and
  `:mine` already shows it
  ([0008](../decisions/0008-the-fork-is-the-overlay.md)).

`ad`, `:discography` and the modal's table are in
[`keybindings.md`](../keybindings.md), marked ✅. What the code cost: four
more fields in the tail cache, `edit::set_tops` (the batch), `explore.rs`
(the state, ten tests), a modal key table in `keys.rs`, and `render_explore`
in `tui.rs`. **Not verified in a real session** — like everything delivered
over these two days.
