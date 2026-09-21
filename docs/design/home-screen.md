# The home screen

Note opened on **2026-09-05**: Joel wants the subcommands (`parcours`,
`check`, and eventually `ecouter`) to disappear in favor of a plain
`forkstify` that opens the application. That calls for a home screen, which
he will mock up with Claude Design.

Nothing is coded, nothing is decided. This note gathers what the repository
already constrains, what the data really allows, and the questions to
settle.

## What is already written

`forme-de-l-application.md` settled **two entry points** for the seed:

- **`/` then some text** — a merged catalog + Spotify search.
  **Implemented** on 2026-09-04.
- **A list** — "the user's library, artists and albums liked on Spotify,
  walked from the keyboard, filtered with `/`". Never written.

The home screen is therefore where that list finally exists. It does not
start from nothing.

## What the data allows — the figures

Measured on 2026-09-05 on the real catalog.

| | |
|---|---|
| Artists in `classement.json` (Joel's library) | **741** |
| Cards in the catalog | **214** |
| Ranked artists **that have a card** | **175** |
| Cards for artists **absent** from the ranking | **39** |

And the coverage by band of the ranking:

| Band | Have a card |
|---|---|
| top 20 | 20/20 |
| top 50 | 48/50 |
| top 100 | 83/100 |
| score ≥ 20 | 25/25 (100 %) |
| score ≥ 5 | 82/89 (92 %) |
| score ≥ 2 | 122/289 (42 %) |
| everything (score ≥ 1) | 175/741 (23 %) |

**Three conclusions that govern the mockup:**

1. **The top of the listening is fully covered.** A "your regulars" list
   will always find a card. No risk of proposing an artist you cannot branch
   from.
2. **The bottom is not.** Below score 2, fewer than one artist in two has a
   card — and **a seed without a card cannot start a journey** (`resolve()`
   requires a card, and the branches come from the card's links and
   vectors). So the "push what you rarely listen to" axis runs into the edge
   of the catalog.
3. **The ranking's median score is 1.** The ranking's tail is noise (an
   artist met once). The useful pool is score ≥ 5: 89 artists, almost all of
   them with a card.

## The blind spot: there is no usage yet

`learned/artists/` **does not exist** — no real session has been played yet.
On first run, the home screen will therefore only have `classement.json`,
that is, **a snapshot of the Spotify library**, not forkstify usage.

That is the same constraint as the comfort zone: the screen has to be **good
on day one with zero history**, and better afterwards. A mockup assuming
rich listening data would describe a screen nobody will see for weeks.

## On the idea of listing the Spotify account

**Do not do it, and that is good news.** `classement.json` *is* already
Joel's library, harvested by `tools/`: liked tracks, liked albums,
playlists, followed artists, #fipway, road trip. It is richer than what the
API would return, and it is **local, instant, offline**.

Going back through the API would also cost a **re-authorization**:
`/v1/me/top/artists` needs the `user-top-read` scope, absent from our
current five (`spotify.rs`). That would be paying an OAuth for data we
already have, in a worse form.

## The strong idea: "neglected" beats "rarely played"

Joel proposes contrasting often played / rarely played so as not to always
circle the same artists. The `learned/` layer allows better than that.

The counters of [0014](../decisions/0014-shape-of-the-learned.md) **decay by
themselves** (six-month half-life) and every artist carries a `last`. So we
can distinguish:

- **rarely played** — low familiarity, never really spent time with them;
- **neglected** — familiarity that *was* high and has decayed, with an old
  `last`: "you loved them, you do not listen to them any more".

The second is infinitely more accurate as a prompt: it is a reminder, not a
discovery. And it is computable as soon as there is usage, with no new data.
On day one, only "rarely played" will exist (through the ranking).

## The proposal: every block is a reason

The project's signature rule — *any automatic decision is explainable in one
sentence* — applies to home. A screen showing six artists has to say **why**
each one is there. So: no undifferentiated grid, but a few **entry doors**,
each carrying its reason in one line.

Leads, from the safest to the most debatable:

1. **Search** (`/`) — the first line, always. Already implemented, it is the
   escape hatch that makes all the rest optional.
2. **Resume** — the last seed and where you left off. Needs one new piece of
   data (the last journey), tiny.
3. **Your regulars** — high familiarity. Works on day one.
4. **Neglected** — decayed familiarity, old `last`. Only works after usage;
   on day one, replace with "from the catalog, never played" (the 39 cards
   absent from the ranking are exactly that).
5. **At random** — a weighted draw, the door that asks you to choose
   nothing.

**And the mix should be governed by the comfort zone**, not by a new
setting. The 0–5 dial already says "I'm staying with what I know" or "I'm
heading for what I don't"; that is exactly the axis between "regulars" and
"neglected". At the cocoon, home puts the regulars first; at exploration, it
puts the neglected. **One axis in the product** —
[0012](../decisions/0012-track-rotation.md) §4 already says "no second
setting".

## One more door, very much in the project's spirit

**Artists you play a lot and who have no card** — 17 in the top 100. Showing
them means offering to grow the catalog where use is asking for it, which is
the promise of
[0002](../decisions/0002-shared-forkable-catalog.md) and the *promotion* of
the vocabulary.

But that is an **edit** (creating a card), and no edit is wired. To be kept
for later, and not mocked up as if it worked.

## Settled by Joel on 2026-09-05

- **The seed can be either**: an artist (we start a segment on them, native
  branches) or a track (we play it, then branch from its artist if they have
  a card). That is already what the `/` search does since 09-04; home
  applies the same rule, and the vocabulary contradiction falls away:
  `vision.md` says "the starting track", the code seeds on an artist — both
  are true.
- **Two screens, depending on the connection state.** A specific screen when
  not connected, the real home when connected. Home therefore does not
  degrade: it only exists once the authorizations are in place.

A consequence not to be missed: **there are two distinct authorizations**,
and the disconnected screen must handle them separately —

1. **librespot**: the credentials come from the phone over zeroconf (you tap
   the device name in Spotify). Without them, no sound.
2. **the Web API**: browser OAuth, ncspot client id. Without it, we resolve
   no title.

And two very different situations hide behind "not connected": **never
authorized** (a first-run home, both gestures have to be explained) and
**authorization lost** (the token has expired — a real case, fixed on 09-05:
the application now asks for authorization again by itself). The second must
not look like the first.

Finally: **the catalog is local**. Even with no connection at all, the
cards, the vectors and `learned/` are readable — dry navigation stays
possible. The disconnected screen is therefore not a dead end.

**The collection's view** (Joel, 2026-09-09): by default it only shows
**the liked** — an artist or track liked here, a track, an album or a follow
on Spotify (`classement.json`) — and `v` switches to **all** the catalog's
artists. The `s` sort applies to the view.

## The mockups (Claude Design, 2026-09-05)

Project **"Accueil Forkstify"**, two boards:

- **`Raccourcis.dc.html`** — the leader menu, a half-typed namespace, the
  media keys. **It is not a proposal**: it is a faithful rendering of
  `help()`, checked line by line against `src/listen.rs`. It stands as a
  visual specification of what exists.
- **`Accueil.dc.html`** — three screens: **A1** not connected, first run,
  **A2** authorization lost, **B** connected home. Adjustable props: the
  comfort (0–5, which genuinely reorders the blocks), the screen shown, the
  Omarchy theme.

What the mockup adopts, and what holds: every block carries its reason in
one line · the `1-5` numbering runs **across** the blocks, so choosing a
seed is the same gesture as choosing a branch · the seed mixes artists and
tracks in one list (entry 3 is a track, with its reason: "it plays, then the
branches set off from New Order") · comfort moves the neglected to the front
from 4 up · A2 says librespot still holds, only titles no longer resolve —
which is exactly the real behavior.

### Four points of friction — settled and fixed on 2026-09-05

1. **`p` stays pause.** "Browse dry" moves to **`b`** (*browse*), a free
   letter and an English word as 0015 requires.
2. **"At random" gets no new key**: it is **enter**, which means "choose for
   me" everywhere else. The mockup's `*` goes away.
3. **`:comfort`**, in English, as the code already writes it.
4. **The zeroconf name became "forkstify (omarchy)"** in the code
   (`src/sound.rs`): here it was the mockup that was right.

Added to the keyboard: **`r`** (*resume*) to resume. Both new keys are
prefix-free, and `keys.rs`'s exhaustive test verifies it.

### The points of friction, as they were found

1. **`p` is already pause.** The mockup gives it "browse dry" on the
   disconnected screens. There is no playback at that point, so no
   *technical* collision — but the user learns a key, not one key per
   screen. And "parcourir" is not an English word, which 0015 requires.
   `b` (*browse*) or `d` (*dry*) are free.
2. **`*` for "at random"** is not an English word either. And the gesture
   already exists: **enter** means "choose for me" everywhere else. Reusing
   it here costs no new key.
3. **`:confort`** appears on screen B's prompt line — the real command is
   `:comfort`, in English like every public interface of the repository (the
   other board writes it correctly).
4. **The zeroconf device name** shown is "forkstify (omarchy)"; the binary
   today announces **"forkstify (spike)"** (`src/bin/spike-connect.rs`). The
   mockup's name is better — it is the code that will have to change, not
   the mockup.

### Two behaviors the mockup assumes and that do not exist

Both are right, but they are workstreams, not display.

- **forkstify announces itself over zeroconf while screen A1 is shown**
  ("waiting — no device has announced itself yet"). Today discovery lives in
  the separate `spike-connect` binary; `Sound::connect()` only re-reads the
  cache and fails if it is empty. The discovery loop has to move into the
  application.
- **The OAuth authorization is deferred**: the mockup waits for a key press
  to open the browser. Today `WebApi::new()` opens it **by itself** at
  launch. The mockup's choice is better — we do not want a browser popping
  up on every start — but it is a flow change.

## To settle

1. ~~**Seed = an artist or a track?**~~ — settled: both. `vision.md` says
   "the starting track", the code seeds on an **artist** (`resolve()`
   returns a card slug). The home screen forces a choice — or an acceptance
   of both, as the search already does: an artist starts a segment, a track
   plays then branches from its artist.
2. **What becomes of `parcours` and `check`?** `check` (a card's neighbors)
   is useful and could become a gesture on a selected artist. `parcours` is
   a development tool — a flag, or nothing.
3. **Does home render before the Spotify connection?** Today `ecouter` opens
   librespot **and** the OAuth before showing anything. A home screen should
   appear **instantly** from the catalog and `learned/` — both local — and
   only connect when it is time to play. That is a change of initialization
   order, not a display detail.
4. **How many entries per block?** Three branches at a fork point; the same
   readability constraint probably applies here.
5. ~~**Removing an artist from the liked, and having it stick.**~~ —
   settled and wired on 2026-09-09: `al`, `as`, `ab` on home on the
   highlighted row; `as` sets `unliked = true` in
   `learned/artists/<slug>.toml`, `al` clears it, and the 0017 merge treats
   it as a flag. The context: on 2026-09-09, the "liked" view showed Bosh,
   Bossikan and Bow Wow: tracks liked on Spotify… by Joel's son. The source
   cannot be fixed from the application, and the next harvest would bring
   them back. So a gesture on home is needed, **written into `learned/`**,
   that outranks Spotify. Two possible strengths, and the key table already
   has them while listening:
   - **`as`** on the highlighted row — "less often": the artist leaves the
     "liked" view and stays proposable. That would be an explicit
     `liked = false` flag in `learned/artists/<slug>.toml`, because the
     weight alone is not enough (it climbs back); `al` clears it.
   - **`ab`** — "never again": the existing ban, which excludes them from
     everything.
   Direction: wire both on home, on the highlighted row, with rule 0020 (the
   highlighted row, otherwise nothing on home). The flag merges like the
   others (`merge_flag`), so it survives synchronization.
