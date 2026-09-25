# Progress

Updated on **2026-09-24**. This file is the entry point for picking the work
back up: what is done, what is waiting on Joel, what comes next.

## Done

- **Design**: vision and philosophy ("take back the algorithm"), vocabulary,
  13 decisions (`docs/decisions/`), 5 living notes (`docs/design/`, 12
  today). Rust, TOML, MBID, two repositories, card format v1 (typed English
  links + cascading proximity, doors as an additional criterion, everything
  optional but `format`/`name`/`mbid`). 14 decisions; the last on
  2026-09-04: **the shape of the learned layer** — a `learned/` folder, one
  file per artist, decayed counters (six-month half-life) (0014).
  Vocabulary on disk (paths, fields) is in English like the code.
- **Catalog bootstrapped** (`~/Work/forkstify-catalog`, private GitHub):
  `catalog.toml` (the type → proximity grid), **30 cards** written
  (`generated = true`, the top of Joel's ranking), **tools/** — 7 Python
  bootstrap scripts (reading the Spotify library/playlists through the
  Omarchy-Spotify session, harvesting Spotify/Deezer friends, MBID
  resolution, ranking), **learned/** — 741 scored artists (liked tracks,
  albums, #fipway, road trip BDX//ATX, follows), 728 MBIDs resolved.
- **Analysis of Omarchy-Spotify** (code read): two browser OAuth PKCE flows
  with no dashboard (ncspot client id for the Web API, Spotify desktop
  client id for librespot), a ~800-line Rust backend around librespot. See
  `docs/design/spotify.md`.
- **Cold start designed**: the card generation pipeline (MusicBrainz facts,
  Deezer `/artist/top` tops, Deezer `/artist/related` similars — verified
  with no key), the base's three workstreams. See
  `docs/design/catalog.md`.
- **Card generator prototyped** (2026-09-01, `tools/generate-cards.py`) and
  **batch 2 generated: 58 cards** — the slugs called in by batch 1's links
  (56 real ones, not 61) plus 2 members of Destiny's Child called in
  cascade. Facts, dates, origin and typed relations from MusicBrainz; tops
  and similars from Deezer; genre + country + decade tags (bands only — a
  person's begin is their birth); scene links by cross-referencing.
  Descriptions absent from the generated cards: review is what enriches
  them.
- **Widened batch 3 generated** (2026-09-02): **126 cards** — the 89 slugs
  called in by batch 2's links, plus the ranking's artists with a score ≥ 5
  and no card. A fix along the way: "Experience" (Joel's library) wrongly
  resolved to The Jimi Hendrix Experience — it is **Expérience** (Michel
  Cloup, Toulouse), the MBID fixed in `learned/mbid.json`. The catalog holds
  **214 cards**; batch 3 in turn calls in **100 slugs** (batch 4, not
  generated — the ranking's tail at score 1–4 is also set aside: those
  artists will come in when a link calls them).
- **Vectors prototyped** (2026-09-02): `tools/vectoriser.py` composes each
  card's text from its structure (tags, dates, origin, outgoing and incoming
  links, description if present) and computes the vectors in a container —
  model `paraphrase-multilingual-MiniLM-L12-v2` (fastembed, 384 dimensions,
  mean pooling, available in Python and in Rust). The derived index
  committed: `vectors/vectors.jsonl` (214 cards) + `meta.toml`.
  `tools/voisins.py` (stdlib) = a prototype of `forkstify check`: coherent
  neighbors (The Cure → Joy Division/Siouxsie; IAM → French rap; Nina Simone
  → Ella/Nat King Cole); thin cards have blurry neighbors at low scores,
  which is exactly what `check` is meant to reveal.
- **The sound wired onto the navigation** (2026-09-04): `forkstify ecouter
  <seed>` — the same engine and the same menus as `parcours`, but **it
  plays**. Modules `sound.rs` (the embedded librespot player), `spotify.rs`
  (ncspot Web API: title → `spotify:track:` resolution, disk cache, 429
  backoff) and `listen.rs` (the async loop: playback in the background, the
  menu over it, segment finished → auto-advance so it never stops; `1-3`
  jumps to a branch, `j`/`k` next/previous track (a classic player, a
  past/current/queue timeline, events filtered by `play_request_id`),
  `e`/`<n>e` slots in, `b<n>` the size, `u` previous branch, `q` quits).
  **Media keys** ⏮ ⏭ ⏯ supported through **MPRIS** (D-Bus, `mediakeys.rs`
  module, `mpris-server` crate) — like `playerctl`; the loop moved to a
  current-thread runtime + LocalSet to host the MPRIS server.
  Display (2026-09-04): the **queue of upcoming tracks** is shown (the
  current one on the `▶` line), the **branches only show on the segment's
  last track**, and `p` previews them on demand (the real "preview +
  choosing ahead" will come with the interface).
  Search `/text` (2026-09-04): searches the catalog **and** the Spotify API,
  a merged `[catalogue]`/`[spotify]` list — a catalog artist starts a
  segment, a Spotify track plays and hooks onto its artist's card if they
  have one (otherwise outside the catalog). Choosing a branch **does not cut
  off the current track**: it is set pending and starts at the end of the
  track (`j`/⏭ forces it right away). **Prefetching** (2026-09-04): the
  title → `spotify:track:` resolution of the next track (a pending branch or
  the head of the queue) is done in advance, cached, for a transition with
  no API wait (librespot audio prefetching stays in reserve if needed).
  **Configuration** (2026-09-04, `src/config.rs`):
  `~/.config/forkstify/config.toml` (created on first run), the
  `[playback] prefer_studio` option (default true) — on resolution, we fetch
  several results and set aside the live versions (title or album marked
  live/unplugged/concert), unless the requested title is itself live. The
  engine stays untouched — it produces tracks, `sound`/`spotify` play them.
  `parcours` stays the dry mode (fast, no Premium needed, for iterating on
  the engine). Built through the **`forkstify-build`** image (`Dockerfile`:
  rust + pkg-config + libasound2-dev).
- **Dry navigation prototyped** (2026-09-03) — the first Rust code in this
  repository: `forkstify parcours <seed>` proposes 3 readable branches with
  their reasons (the graph first — typed links both ways, cascading
  proximity —, the vectors to fill in and for the adventurous branch), `1-3`
  to choose, enter = a weighted auto draw, `u` to go back, `q` to quit;
  segments = 3 tops drawn without replacement (the spirit of 0012).
  `forkstify check <artist>` = the neighbors in the space, with graph links
  marked. It builds in a container (`docker run --rm -v "$PWD":/app -w /app
  -v forkstify-cargo:/usr/local/cargo/registry rust:1-slim cargo build
  --release`) and runs on the host. Journeys confirmed coherent: IAM → Zebda
  → Fabulous Trobadors → La Rue Kétanou → Camille; The Cure → Siouxsie →
  Cult Hero → The Fall → Joy Division. **Code and comments in English**
  (aiming at open source — the rule is in AGENTS.md).
  Touch-ups from Joel's test: **a branch is a segment** — grind the current
  artist, or a walk of n tracks across several artists (one per artist
  crossed), size adjustable with `b<n>` — the directions are proposed from
  **the whole branch** (graph neighbors taken together, the vectors'
  centroid), not from the last artist alone — **nothing is deterministic**
  (heads and jumps drawn at weighted random, 0012 applied to the branches) —
  a **"stay in the journey's universe"** branch circles inside the cluster,
  with revisits allowed as long as unplayed tracks remain — the adventurous
  branch has a **floor** (cosine ≥ 0.72 + a shared genre tag, constants to
  be driven by comfort) — and **grinding is no longer a branch but the `e` /
  `<n>e` key** (the `:encore` command) which slots in n tracks by the
  current artist (see `docs/design/application-shape.md`).
- **Spotify Connect spike done** (2026-09-03, `src/bin/spike-connect.rs`,
  run by Joel on his Premium account). **Incoming zeroconf discovery: ✓** —
  the device shows up on the phone, the credentials arrive, the librespot
  session opens (so the sound is validated end to end, with nothing
  installed on the host, without Omarchy). **A session token for the Web
  API: ✗** — keymaster answers 403, login5 returns a token the API refuses
  with a persistent 429 (a desktop client id in restricted quota). Verdict:
  the sound goes through embedded librespot, the Web API will go through
  browser OAuth + ncspot's client id (the whole ecosystem's route). Detail
  in `docs/design/spotify.md`.
- **Web API validated** (2026-09-03, `src/bin/spike-webapi.rs`): browser
  OAuth PKCE with ncspot's client id (`librespot-oauth`), refresh token
  cached. `/v1/me`, `/v1/search` (title → `spotify:track:`) and
  `/v1/me/albums` (247 albums) answer. The lesson: the 429s we hit were a
  temporary **account/IP** throttle (a decreasing Retry-After, clearing with
  rest), not a client id block — the real client must respect `Retry-After`
  (the spike does).
- **Playback validated** (2026-09-03, `src/bin/spike-play.rs`): an embedded
  librespot player (`librespot-playback`, rodio → alsa backend), loads a
  `spotify:track:` and **the sound comes out of the binary** — tested by
  Joel, "it works very well". **The sound workstream's four bricks are
  validated** (zeroconf, session, Web API, playback); forkstify is itself
  the device, we drive no other device through the API.
- **Silent authentication failure fixed** (2026-09-05, Joel's first long
  `ecouter` test): after ~2 h, every track became "not found on Spotify" and
  the journey stopped. Cause: Spotify's PKCE flow **rotates the refresh
  tokens**, and `refresh_if_needed` only kept the access token — neither in
  memory nor on disk — and `WebApi::new` took the refresh from the
  *response*, which is empty when it carries none. The stored token
  therefore went stale, and `resolve()` drowned the error in a `None`
  indistinguishable from a missing track. Three fixes: (a) `resolve()`
  returns a **`Resolved`** (`Track` / `Absent` / `Failed`) — a failed call is
  no longer an absence, and `search_tracks` likewise; (b) the renewed
  refresh is **kept and rewritten** on every rotation (`keep_refresh`), and a
  dead refresh **asks for browser authorization again** instead of failing;
  (c) the disk cache only remembers **real** absences — a call that did not
  go through does not enter it (one already poisoned entry purged: The
  Limiñanas — "Au début c'était le début"). On the navigation side, a
  failure **stops the journey** instead of burning through the queue: the
  track stays at the head, `j` retries. **And "enter/auto" now draws among
  the branches shown** — `auto_advance` was recomputing three new branches
  and therefore playing what Joel had not seen (Joel's call, 2026-09-05).
- **Repositories** (moved on 2026-09-08): `aropixel/forkstify` and
  `aropixel/forkstify-catalog`, the reference, private, `main` branch;
  `kbyjoel/forkstify-catalog` is Joel's **fork**, the one the application
  reads (`origin`), with upstream as `upstream`. The chorizo recovery plan
  is up to date.

## Waiting on Joel

- **No card-by-card review** (Joel's decision, 2026-09-02): he looked at the
  whole, and the tuning will happen **in use** (keybinds, decision 0013).
  The known points are kept on record: uncertain MBIDs (28 in batch 2, 14 in
  batch 3 — the generator's reports), empty tops (cabadzi, le-motel,
  la-ruda-salska), duplicate versions for J.P. Nataf, a stray Russian top
  for Expérience.
- Deezer/Spotify identifiers of **consenting friends** to widen the base
  (`tools/amis-*.py`).
## The review is the pull request (2026-09-24)

Jarvis Cocker's similar links looked odd: they were Deezer's neighbourhood
filtered by what Joel's library held on 2026-09-01, the four survivors of
a twenty-long list. Joel rewrote them by hand, and the agent pointed out
that the card still said `generated = true`. So did 381 others: 382 out of
383, nothing in the application lifting the flag, nothing in the engine
reading it. Joel: "I would like to remove every `generated`. If there are
changes to be made, they will be proposed and examined in a PR with `Cp`."

Decision [0025](decisions/0025-the-review-is-the-pull-request.md): the
review is the pull request, the flag goes, who wrote a card is a question
for git. In the code, `Card.generated` leaves with every label built on it
(the seed line, the finder, the explore header, the `Cd` overlay), the
generator stops writing the line, the commit bodies lose "to review", and
the `Cp` title and body stop sorting the new cards into generated and
written: "Propose 2 cards (1 new, 1 edited)", and the reviewer reads the
diff. The reader still tolerates the field, so an older fork loads.

In the catalog, the 382 lines go in one commit on Joel's fork, for him to
propose through `Cp` — the first real pull request the gesture makes, on
a change that is one line per card.

The first `Cp` on it failed: "fatal: not a git repository: (null)". The
proposal worktree in `~/.local/state/forkstify/proposal` dated from
2026-09-20 and its `.git` file pointed into `~/Work/forkstify-catalog`,
the clone the catalog lived in before it moved. `propose` now checks that
the worktree belongs to this clone (`git rev-parse --git-common-dir`) and
rebuilds it otherwise; the fork test breaks the worktree on purpose and
proposes again.


After a listening session that generated three cards, `Cd` answered that
nothing lay beyond the reference. The clone on this machine declares
`upstream` but had never fetched it — only `Cp` and `Cu` did — so
`upstream/main` did not exist locally and the diff's base fell back on
`origin/main`: the fork itself, where the cards were already committed and
pushed. An empty diff, honestly computed against the wrong thing.

Two changes in `src/fork.rs`. `diff` fetches `upstream` first, as `Cp` and
`Cu` do, and `propose` no longer fetches on its own since the diff it asks
for does; offline, the last fetch serves, and a fork that was never fetched
says so. And `base` no longer lets `origin` stand in for the reference when
an `upstream` remote is declared: `origin` is the reference only in local
mode, where it is the only remote. The fork test drops its explicit fetch
so the diff is seen doing it.

## v0.1.0 released, and the AUR package ready but held (2026-09-21)

The release channel is live. The `v0.1.0` tag ran the workflow through in
6 min 42 s and attached `forkstify-0.1.0-x86_64-linux.tar.gz` (17.4 MB) and
`SHA256SUMS`. **Installing forkstify no longer needs Docker**: the archive
was downloaded and checked against the published digest, and it holds
exactly the binary and `LICENSE`, which is what `omarchy/install.sh` expects.

The AUR package is finished and proven. `sha256sums` carries the real digest
now, `options=('!strip' '!debug')` stops makepkg carving an empty debug
package out of an already-stripped binary, and a real build produced one
clean `forkstify-bin-0.1.0-1-x86_64.pkg.tar.zst` of 16 MB holding
`/usr/bin/forkstify` and its licence, nothing else.

**What holds it is not ours**: AUR account registration is temporarily
closed while they deal with a wave of automated sign-ups. There is no manual
queue and retries tell you nothing; the reopening is announced on the Arch
news feed and on `aur-general`. Nothing is blocked by it — the release is
the main road anyway, and any Arch user can already install the very same
package with `makepkg -si` from a clone. A personal pacman repository stays
the fallback if the wait drags on. All of it in `packaging/aur/README.md`.

A `.gitignore` now keeps makepkg's `src/`, `pkg/` and archives out of the
repository: it is the Omarchy plugin, cloned by every user, and a stray
build left 118 MB sitting in it.

## Three scripts for the maintainer (2026-09-21)

Everything we had been doing by hand, and getting wrong, is now a script in
`bin/`. All three are maintainer tooling, not secrets: they belong in the
open repository.

- **`bin/dev-install`** makes the clone you stand in the forkstify that
  runs: build, link on the `PATH`, link into Omarchy, enable, reload. It
  refuses to `ln -s` over a leftover plugin **directory** — that is what
  swallowed the link on 21/09/2026 and made the icon vanish — and it names
  the package when one shadows the dev build, since `/usr/bin` comes before
  `~/.local/bin` on most paths.
- **`bin/release <version>`** sets the version in `Cargo.toml`,
  `manifest.json` and `Cargo.lock`, checks they agree, commits, tags and
  pushes. The tag is what fires the workflow.
- **`bin/release-aur [version] [--publish]`** fetches the published
  archive, **checks it against `SHA256SUMS` rather than trusting it**,
  writes the digest into the PKGBUILD, bumps `pkgrel` when the version has
  not moved, regenerates `.SRCINFO`, and builds the package to prove it —
  all in a temporary directory, because makepkg leaves 100+ MB next to the
  PKGBUILD and this repository is the plugin everyone clones. Proven
  end-to-end against `v0.1.0`.

`.gitignore` also gains `/scratch/`, a drawer for throwaway scripts and
notes that is never committed. What has to follow you between machines
belongs in a private repository instead: a gitignored folder dies with the
clone, as this one nearly did.

## The index is regenerated only where it moved (2026-09-21)

Joel, seeing a commit rewrite `vectors/vectors.jsonl` after a two-line
translation: "is versioning the vectors really a good thing?"

**What we measured.** Same cards in, 358 vectors out of 360 different:
97.4 % of the 138 240 components moved, by 1.3e-5 median and 1.29e-4 at
worst, on components whose own amplitude is 0.035. So the embedding is not
reproducible to the last digit — ONNX Runtime, thread by thread, machine by
machine — and every regeneration rewrote 1.5 MB into the history for
nothing. The workflow's "commit only if changed" guard could never fire.

**Rounding, which we first proposed, does not work.** Simulated on the two
real indexes: 4 decimals leaves 2 lines out of 360 stable, 3 decimals the
same. Two decimals stabilises 195, but quantizes to a cosine error near
0.06 — the order of the proximities the engine reads. Dropped.

**What is wired instead.** Each line carries the fingerprint of the text it
came from (`h`, FNV-1a over the model name and the text; `DefaultHasher` is
not stable across Rust releases and this goes into a versioned file).
`regenerate` composes every text — cheap — hashes it, and runs the model
**only over the cards whose fingerprint moved**, keeping every other line
byte for byte. A card's text carries its neighbours' links, so a new link
still makes both sides stale, which is right. An index written before today
has no fingerprints: it regenerates once, wholesale, then holds still.

**And the trigger was wrong.** The catalog workflow fired on `catalog.toml`,
which carries the proximity grid the engine reads and the embedding never
does. Removed: `cards/**` only. The pull-request check is untouched, its
trigger has no path filter.

Versioning the vectors stays right: a listener would otherwise download a
241 MB model to compute 366 of them. It was regenerating them wholesale
that was wrong.

## Tracks resolved but never played: the missing market (2026-09-22)

Joel: "several times in the day, I started tracks, they did not play and
moved on to the next, which often did not play either."

**Where it was not.** Resolution works: of 185 entries in the cache, 179
carry a Spotify uri and 6 are genuine misses. So the tracks were found, and
they still would not play.

**Where it was.** **No Spotify call passed `market`.** Without it the API
answers with tracks that exist somewhere and play nowhere here: the uri is
valid, librespot cannot play it, and forkstify skips on — to another track
chosen the same way, which fails the same way. The `MAX_DRY_ADVANCES` guard
of 2026-09-11 stops the runaway; it never addressed the cause.

Four endpoints were blind: both searches, the artist's albums, and the
albums themselves. The last two matter most — a tail track goes to the
queue **without being resolved**, so an unplayable one there is a silence
later. All four now pass `market=from_token`, which also makes Spotify
relink to the copy that does play here. And a hit Spotify marks
`is_playable: false` is dropped, in the resolution and in the harvest.

**Why it surfaced now.** The clean reinstall of 2026-09-21 wiped the
resolve cache built over weeks. Every track is resolved afresh, so a flaw
that only bit on new resolutions started biting on all of them.

**Consequence for anyone running before today**: the caches hold entries
chosen without a market. `~/.local/state/forkstify/resolve-cache.json` and
`~/.cache/forkstify/discography/` have to go once, and fill again on their
own.

## `tg` — the track in the browser (2026-09-23)

Joel: a `tg` for track google, as `ag` is for artists, searching **the
artist's name and the song's**. Wired: `tg` searches `<artist> <title>`,
and the toast names it the other way round, title then artist, as the rest
of the interface does.

Like `ag`, it aims at the highlighted row otherwise at what plays, and it
**needs no card** — a name and a title are enough, so a track outside the
catalog is searchable too. The two now share one `google` helper instead of
each spawning `xdg-open` on its own. The `ta` row of the table also gets
back the column it was missing.

**And at the home**, asked for right after. There the collection holds
artists and the cursor never lands on a track, so the only track to aim at
is the one sounding underneath — what the playback foot shows. `tg` says so
when nothing plays. The key helper gains its own `t` level at the home,
holding that single key, rather than falling back to the full home list:
the menu never promises what the code does not do.

## `x` removes, wherever you are (2026-09-23)

Joel: make `tx` into `x`, and let `x` be **the** removal gesture — a track
out of the queue, an artist out of the liked at the home.

Removing is not a verb of the track alone, so `x` leaves the `t` namespace
and stands on the bare keyboard. While listening it does what `tx` did: the
highlighted track leaves the queue, and can still be proposed — it is not a
ban. At the home it takes the highlighted artist **out of the liked**, and
only that: `learned::unlike_artist` sets the flag without moving the weight,
where `as` also pushes the artist down. `al` brings them back. It is the
same shape as the `tl` toggle of 2026-09-14: leaving a list is not the same
statement as stating a taste.

**`u` and `.` leave the grammar**, asked for in the same breath: undo and
repeat are not being built for now, and a helper that lists them promises
what the code does not do. `fu` still steps back one branch, and `u` keeps
its meaning **inside the discography**, where it undoes a pending edit —
that one was wired all along. `Cmd::Repeat` is gone; `Cmd::Undo` stays for
the modal. The free-letter list and the 2026-09-05 collision table are
updated, the latter with a note rather than a rewrite: a record is not
corrected after the fact.

## A paused player no longer moves on by itself (2026-09-23)

Joel: "when the bluetooth of the speaker or the headphones cuts out and the
music was already paused, the music starts again on its own."

The loop reacted to the end of the current track without ever asking
whether we were paused. `track_over` covers `EndOfTrack`, `Stopped` and
`Unavailable`; losing the output makes librespot **stop** the stream, which
read as "the track ended", so forkstify advanced — and advancing plays. One
condition short: the end of a track is only acted on while something is
actually playing.

Nothing else relied on that event while paused: `tb` and `ts` call `next()`
themselves, they do not wait for librespot to say so.

**Left open, and untested here for want of a bluetooth device**: what `p`
does after the output has gone. The stream librespot stopped may not
resume, in which case the track has to be reloaded — and the position
kept, which `Sound` cannot do today, it only knows how to go back to zero.

## The heart shows up where the like is made (2026-09-23)

Joel: liking a track should change its glyph in the list, and that glyph
should show in `ta` too.

A stop carries the source it was **drawn** with, so liking one afterwards
left `♪` or `·` sitting there — the list disagreed with `learned/`.
`engine::current_source` now says what a track deserves as things stand:
the like outranks the rest (0018), **a door keeps its arrow** since that
says where it leads rather than how it was picked, and otherwise it is a
top of the card, a track of its tail, or neither. `tl` puts that glyph back
on every line holding the track — played, playing, still to come — and so
does the discography's `tl`, since the same track may be sitting in the
queue behind the modal.

`ta` opens on ` mark: ♥ liked`, the same glyph the list carries with the
word behind it. The two helpers live in `engine.rs`, where the sources are,
rather than in the session: a test pins the like, the door and the
off-catalog cases.

## The evening drifts, and the branches follow (2026-09-23)

Joel, showing a journey of 31 artists: started at Can, ended at Metronomy,
M83 and Archive by way of Paul McCartney and Lou Reed — and the first
branch on offer was still *Lou Reed → Eagles → The Rolling Stones*. "The
last tracks should weigh more than the start."

**Which of the three was at fault.** Only one. The graph branch and the
adventurous one already work from `context`, the **last** round, so they
were following him: they proposed Air → Sébastien Tellier → Flavien Berger
and The Dø → Beirut → Girls in Hawaii. The third, "stay within the
journey's universe", took the **plain average of every artist played**, so
a 31-artist evening was represented by a middle that sat back near its
beginning.

**What changed.** `drifting_centroid` weights each artist of the universe by
how far back it is, halving every `universe_half_life` artists — a new
setting, default **5**, so roughly the last two branches say where the
evening stands. A very large value restores the old flat average. The other
two branches are untouched: they were never the problem.

Picking the default took a measurement rather than a guess: at 8 artists,
on a journey of 31, the 25 older ones still outweighed the 6 recent. At 5
they do not. A test pins the direction, not the number — a recent cluster
leads, and reversing the order reverses the centre.

## `aL` asks how close (2026-09-23)

Joel, finding that some of the connections he draws depend on his own ear:
the format has carried a per-link `proximity` since 0010, the gesture never
set one, so every link written by `aL` fell back to the grid's 4 for
`similar`.

Enter on a target now opens one question — `1` a distant echo to `5` almost
the same universe, **⏎** for the grid, **esc** writes nothing — on the
model of the comfort dial: the digits mean closeness while it waits, not
which branch to take. The number is written into the link, the commit says
it, and the in-memory catalog gets it too, so the engine follows without a
relaunch. Removing a link is untouched: that path returns before the
question is ever asked.

## `ac` — a connection of your own (2026-09-23)

Joel, after some use: "certain connections in my catalog hold for me, they
depend on my tastes and my own history — I'd link artists that nothing
naturally links, but that I like to hear follow one another. What is the
gesture? Does it cause a problem on `Cp`?"

It did. `Cp` checks out the **whole** of `cards/` onto a branch from the
reference: no selection, no filter, so a personal link left with the rest
and arrived in a pull request dressed as shared knowledge.

The answer was not a new kind of link but the line the project already
draws: **`cards/` is knowledge, `learned/` is taste**. So `ac` writes into
`learned/artists/<slug>.toml`, under `[connections]`, slug to closeness.
Three things fall out of that and none had to be built: `Cp` structurally
cannot carry it, it follows him between machines through 0017, and the
three-way merge keeps it — added on one side it stays, undrawn on one side
it goes, moved on both the side that left the base wins. A test pins that.

The engine needed no new parameter either. `weave_into` puts the
connections into the catalog **in memory** as links of a kind no card
carries, so branches follow them both ways with their closeness, while
`cards/` on disk never moves. `text_of` ignores the kind on its own, so the
vectors do not drift either.

**`aL` is retired**, as Joel asked: writing a link into a card is rare
enough to go through `ae`, which opens it in `$EDITOR`. `add_link`,
`remove_link` and `link_line` go with it rather than sit unused.

## The key helper opens on its own (2026-09-23)

Two touches to the which-key, asked for by Joel. **A namespace typed opens
its level**: `t` shows `tl`, `ts`… at once, and `⌫` closes it; space is
only for the help of the beginning, the entry table. In the code, both
screens follow what is typed through one method, `follow_typing`: helper
open, it goes down to the level and back up on `⌫` as before; closed, a
namespace with a level of its own (`f`, `e`, `t`, `a`, `C` — not `c` or
`g`, which wait for a second key but are not namespaces) opens it. A
helper that opened on its own closes on `⌫` instead of going up to an entry
level nobody asked for, and its footer says so.

**And the rows are ordered**: the keys first, then the lines typed after `/`
and `:`, the legend last — both entry tables were mixing them.

**Then, the same evening, the `:` commands fold into a level of their own.**
Joel found the entry help heavy and the two screens' helpers too different.
The `:` line is now followed like a namespace: `follow_line` opens the
commands level as soon as the line starts, narrowed to the word typed, and
the line's end closes it — so the entry table keeps one `:` row. For the
level to be honest on both screens, **every command now works at the
home**: the home's own three-command table is gone, the session runs the
line wherever it was typed (`run_colon`, one path for both screens), and
`:warm` / `:discography` aim at the highlighted artist of the collection
(`Home::highlighted`), generating the card first when there is none, as
`ad` does there.

**Last, one table for both screens.** Joel wanted the helper to be the same
everywhere. One entry table, where each row says which screen it belongs
to, and **each screen shows only its own rows** — a key that acts on a
list the screen does not show (`f`, `e`, `J`/`K` on the branches and the
queue) is not made to act blind. A first cut marked those rows `·` and
said where they lived; Joel saw it and preferred them left out. What could
mean something on both now does: the whole **`t`
namespace at the home** acts on what plays underneath (0020 lands on the
playback foot, since the collection never highlights a track, and a
highlight left behind by `q` is ignored there — `target` and the insert's
anchor look at the screen), and **`h`/`l`** move along the journey playing
underneath. Typing `f` or `e` at the home answers with a word. The `t`
level of the helper is the same on both screens too; the home's own `t`
table is gone.

`Cp` and `Cu` stay as they are: Joel weighed `CP` / `Cp` for the lazygit
reflex (push / pull) and kept the words, since a proposal is not a push and
an update is not a pull.

## `:catalog shell` — a terminal in the fork (2026-09-23)

Joel asked for a way into the catalog fork's directory from the player.
`:catalog shell` launches the desktop's default terminal there:
`xdg-terminal-exec --dir` — the default-terminal specification, which
Omarchy's own launcher uses and sets `$TERMINAL` to — and, failing that,
`$TERMINAL` started from the directory; with neither, the toast gives the
path. The terminal gets its own process group, so closing the one forkstify
runs in does not take it down. No key: the catalog namespace keeps its
three gestures, and a shell is not a gesture on the catalog.

## Seeing and setting the connections (2026-09-23)

`ac` became a gesture Joel uses a lot, and he asked how to see an artist's
connections, set their closeness, or remove them. Two steps were proposed:
the modal of `ac` as the one place to see, draw, set and undraw; and later,
if the web grows, a read-only `:connections` overlay of every pair. Joel
took the first, with one change: not digits alone but **`h`/`l`, the
neovim way**, to move the closeness.

So the modal now lists the connections **on either side** — `→` drawn from
here, `←` drawn from the other artist, which it never showed although the
engine follows both — straight from `learned/`, and each row remembers
where it is held (`Finder::drawn`). **Enter on one reopens the question**
on its current closeness instead of undrawing on the spot: `h`/`l` or the
arrows move it a notch as the comfort dial does, a digit jumps, ⏎ sets,
**`x` undraws** — the same key as everywhere —, esc leaves it. A new
connection starts at 4 and takes the same keys. The question is one struct,
`LinkPending`, that knows whether it is drawn, and the toast is derived from
it. A branch reached through a connection already says "yours" as its
reason, so the connection is visible where it acts. The modal's title
still said `aL`; it says `ac`.

**The second step followed the same day.** `:connections` lists every
connection drawn, grouped by the artist it was drawn from, sorted by name,
with its closeness, in a block that scrolls like `Cd` on both screens.
Read-only by design: setting or undrawing goes through `ac` on either
artist, and the block's last line says so. No key — it is a look, not a
gesture, and the `:` level of the helper lists it.

**And the arrows in the list.** Joel wanted to move a closeness without
enter then a digit: **← → on a drawn row** move it a notch, the number
changes under the cursor, and it is written at once — a measurement, as
`tl` in the discography. The reader swallowed ← → in text mode; they now
reach the modal as `Next`/`Prev`, which only the drawn rows answer. Writing
a connection is one method, `set_connection`, shared with the question.

## The closeness question waits (2026-09-23)

Joel used `ac` and found the question toast gone after its few seconds,
and the scale unclear. The toast is now **derived from the state**: as long
as `link_pending` holds a target, `toast()` returns the question, whatever
the timer says, so it stays until a digit, ⏎ or esc — the same way the
loading toast stays while loading. And the question names its ends: **1 the
farthest, 5 the closest**. One function composes it, `closeness_question`,
for the log line and the toast.

## Release notes, and a way to keep them honest (2026-09-23)

Joel asked for a document recording what changes from one version to the
next. `CHANGELOG.md`, in the repository's front matter rather than in
`docs/`: it is for whoever **uses** forkstify, where `docs/progress.md` is
the working log — why a thing was done, what was measured, what was left
open. The two say different things about the same work and should not be
confused.

It opens on `0.1.0` and on everything since, grouped the way a user meets
it: what is fixed, what changed under the fingers, what listening gained,
how one installs, what the catalog gains. The market fix carries the two
commands that clear the caches filled before it — notes that hide a
required action are worse than none.

**It cannot fall behind**: `bin/release` promotes `## Unreleased` to the
version being cut, dates it, opens a fresh empty one above, and commits it
with the version bump. It refuses to cut a release if the section is gone,
and says so when it is empty.

**And it is what the release publishes.** The workflow no longer generates
notes from the commits: it cuts the section of the version out of
`CHANGELOG.md` and passes it to `gh release create`. The cutting happens in
the job that has the repository checked out — the publishing job has none,
and the notes must match the commit that was built. With no section for
that version it warns in the log and falls back to the commits, so a
re-release never fails for want of prose.

## A silent session, and two indicators that lied (2026-09-25)

Joel: a session started from a Sparks track, nothing played, every track
went by one after another — and **nothing was displayed**.

**What the evidence said.** The learned layer recorded 27 artists played the
evening before and **none** that morning, on the same code, so the commits
pulled from the other machine were not at fault. Resolution was working:
166 fresh cache entries, 6 misses, and the Sparks track tried resolved to a
valid uri. Every harvested discography was younger than the market fix and
held well-formed uris. The process was alive and **idle** — two seconds of
cpu in seven minutes — with its https connections up and the ALSA and
PipeWire threads running. So the failure was at playback, and librespot was
ending tracks without playing them.

**Why it was so hard to see.** Two indicators lied by construction.
`Status::read()` only checked that a credentials **file** existed, and a
file stays where it is when a session is long gone — the header said "✓
librespot" all morning. And a track ended by librespot rather than by its
own length passed **in silence**: `on_track_over` counted the listen when it
was one and said nothing when it was not.

**Both are fixed.** `Sound` keeps its `Session` and answers `alive()` from
`is_invalid()`, so the header asks rather than reads; `✓ api web` follows
what the API last answered. And a track that ends without having played
says so, telling apart Spotify refusing it here (`Unavailable`, which we
never cause) from nothing having come out at all. A stop we caused ourselves
lands there too, but always with a position behind it, so a manual skip
stays quiet.

**The cause itself is still open**: the account is not in use elsewhere, and
until the session runs again with these two in place, what invalidated it is
a guess.

## The search adds, it no longer takes over (2026-09-25)

Joel, on searching a track by an artist with no card: the only way out was
enter, which creates the card. He first asked for `e` to queue it, as in the
discography — `e` cannot serve there, the modal is a **typing field** and the
letter belongs to the query, so it went to `⌃e`. Then, on reflection: make
it **enter**, and change what enter does. "I did not like that it wiped the
rest of the playlist to come, and that it fired on its own."

Both grievances were the same arm. `:search` on a track cleared the queue
and started playing; on an artist it replaced the segment. Enter now
**adds at the end** and the modal **stays open**, so several rows can be
picked in a row and `esc` closes on what has been queued. An artist stands
for their best unplayed track, as `ti` already read them, and a track by an
artist with no card is queued at once with the card generated behind
(0016). `⌃e` goes, now that enter does it.

The three destructive arms of `take_found` are gone, replaced by one that
delegates to the same queueing: enter is routed before it ever reaches
them, and the arm is what stops another door leading back to a cleared
queue. The home is untouched — enter starts a journey there, and there is
nothing to disturb. So are `ti`, which inserts at its anchor, and `ac`,
which picks a target; the modal's footer now says which of the three it is.

## A gesture that wandered off to another artist (2026-09-25)

Joel: a `ti` inserted a Kanye West track, then `en2` on it "did not work" —
"it did not take the artist of the track under the cursor but the last
artist of the list, Orelsan".

**One line did it.** `encore` and `aimed_artist` both read the row under the
needle (0020), kept its slug only if the catalog had that card, and
otherwise fell back on `state().1` — the journey's last artist. A track
inserted from Spotify carries **no slug** until its card lands (0016), so
the gesture silently changed target. It was written for the branches, where
an anchor in the graph is needed; for "more of this artist" it is plainly
wrong.

**Both now resolve the card properly and admit failure.** `engine::card_of`
takes the stop's own slug, else the slug its artist's **name** resolves to
— which is what was missing here: the card exists, filed as `kanye-west`
though it is named `Ye`, so the lookup finds it and `en2` would have worked
outright. With no card at all, the gesture says so instead of aiming
elsewhere; `ti` has one on the way anyway. A test pins the three cases.

## One of yours, beside their name (2026-09-25)

Joel: what indicator would show, from the listening page, whether an artist
is liked — "a heart would be confusing with the track likes, no?"

**Answered wrong once.** Counted on the catalog, a "liked" mark would land
on **187 of 388 cards**, **110 of them from the imported Spotify library
alone**; an indicator on one row in two seemed to say nothing, so the row
got `↑` / `↓` on the artist's **weight** (`al` / `as`) instead. Joel, on
seeing it: "what I would like to see at a glance is whether the artist in
the list is in the home's *liked* list. With the change made I do not see
it." The density was a real measurement and the wrong argument — he is not
asking to be told something rare, he is asking to recognise **his own
artists** while they play, and half the rows being his is the point.

So the row carries an `↑` after the name when `learned.liked` says the
artist is one of yours: asked for with `al`, or a liked track of theirs, or
your Spotify library — minus anyone set aside with `as` or banned. It is
the **same call the home's `v` filter makes**, so the two screens cannot
disagree, and it works off-catalog too, `liked` falling back on the name
when there is no slug yet. The weight arrows go with it: `al` already puts
the mark there, and `as` takes it away.

**Turned down twice over.** A `★` shipped first and Joel: "too visible."
Which follows from the density rather than contradicting it — a mark on
half the rows has to be quiet or it becomes the page. So: the arrow back
instead of the star, and **blue only on what plays and what is to come**,
grey behind, following the artist's name into `MUTED` once the track has
played. The past does not need to be read.

The confusion with the track's heart is settled by **place**: the track's
mark is at the head of the row, the artist's after their name. A test pins
the mark, its absence, the tone in each of the three slots, and that the
head mark has not moved.

## Keyboard grammar wired (2026-09-05)

**Decision [0015](decisions/0015-keyboard-grammar-namespaces.md)**: four
namespaces — `f` the branch, `e` encore, `t` the track, `a` the artist — the
target is a prefix, a frequent gesture gets a key and a setting gets a `:`
command. The single table is in
[`docs/keybindings.md`](keybindings.md).

Wired the same day (`src/keys.rs`, `src/listen.rs`):

- **Raw-mode input, no Enter** (feedback no. 1): termios through `libc`, an
  RAII guard that gives the terminal back even on a panic, ← → arrows
  recognized, `/` and `:` opening an editable line.
- **A prefix-free grammar**: no complete command is the beginning of a
  longer one, so everything fires with no delay. That is what moved the
  modifier **before** the count (`fn3`, `f!3`, `en2`, `e!2`). An exhaustive
  test over every three-key sequence verifies it.
- **The three variants** (feedback nos. 2 and 3) for the branches and for
  encore: end of the branch, `n` now, `!` now with what followed dropped.
  **It fixes along the way** the reported bug: choosing a branch no longer
  throws away the rest of the segment — the `!` variant was serving as the
  default.
- **`fp` peek, `fr` reroll** (feedback no. 5), **`fu`** (previous branch),
  **space** (the keyboard pause that was missing), **`h`/`l`** and the
  arrows.

Not yet wired, and saying so on screen: `t` and `a` (the tuning, blocked by
`learned/` — step 2 below), `fw`, `u`, `.`, `?`, `Q`, `:`.

## The long tail, the fourth source (2026-09-05)

`src/discography.rs` + `WebApi::discography()`. 0012 §1's pool is complete:
tops, liked, doors, **and the rest of the discography**.

The source is **Spotify**, through the cards' `spotify` field — present from
the start, never read until now. It avoids the per-artist `search` Deezer
would require (no card carries a Deezer identifier) and returns
`spotify:track:` directly, so a tail track can never become a "not found on
Spotify".

Cached in `~/.cache/forkstify/discography/<slug>.json` — outside the
repository, as 0012 and `catalog.md` require: regenerable, never committed,
not synced. Harvested **at the moment of need** (when `e<n>` asks for more
than the card has) or by `:warm`. No expiry.

**The comfort dial finally has its third lever**: the tail's share *is* the
comfort's openness (0012 §4) — zero at the cocoon, full at exploration.

Deduplication by normalized title: Spotify delivers the same song in ten
guises, and a title already in the tops does not enter the tail. The `·`
mark.

**Not verified over the network**: the harvest needs a Spotify session, so
`:warm` has never run for real. The rest is covered by 14 tests.

## The comfort zone wired up (2026-09-05)

`engine::Comfort` implements
[0001](decisions/0001-comfort-is-familiarity.md): 0 = cocoon, 5 =
exploration, read from `[journey] comfort` in the config file and adjustable
while listening with `:comfort <n>` (with a word beside the digit: cocoon,
cautious, balanced, curious, adventurous, exploration).

Two levers, the ones `avancement.md` already called "constants to be driven
by comfort": the **adventurous branch's floor** (cosine ≥ 0.80 at the
cocoon, ≥ 0.60 wide open — **comfort 2 reproduces exactly the old 0.72 /
0.80**) and the **familiarity that tilts the draw of the heads**, bounded to
[0.25, 2.0]: we discourage, we do not forbid.

**A polarity trap recorded** in
[`comfort-zone.md`](design/comfort-zone.md) and pinned down by a test:
0012 §4's "high comfort" means the *feeling* of comfort, so the value **0**,
not 5. Read literally, the whole dial inverts.

0001's open question **settled de facto**: what the application knows of
what you know is `learned/` — our decayed, saturating plays, and failing
that `classement.json` brought onto the same scale by its maximum.

**The dial is missing its third lever**: the depth of the draw within the
pool (0012 §4), which will have nothing to set as long as the long tail does
not exist — see [`long-tail.md`](design/long-tail.md).

## The pool opened up, the doors woken (2026-09-05)

A question from Joel — "did we plan for tracks to be played without being
tops?" — which brought a gap to light: **[0012](decisions/0012-track-rotation.md)
§1 called for four sources, the engine drew from only one**. Worse, the
`doors` field had been written in **13 cards** since 09-02 and the word
appeared nowhere in `src/`:
[0011](decisions/0011-doors-an-additional-criterion.md) was asleep.

`engine::reservoir()` now adds up three of the four sources, each with its
weight: the **tops** (1.0), the **liked tracks** from `learned/` (0.8), the
**doors** (0.4, ×2.5 when the branch's direction overlaps their tags —
0011's bonus). A frequently skipped track steps back
(`weight ÷ (1 + skipped)`), a banned one leaves. The draw is weighted,
without replacement within a journey.

**Every track shown carries its provenance**: `♪` top · `♥` liked · `↳` door
· `+` outside the tops · `~` outside the catalog. Verified dry — from Joy
Division or The Fall, `↳ A Forest — The Cure` shows up.

**The fourth source is missing**: the discography's long tail (a Deezer/
Spotify API cache), which needs a cache layer that does not exist. 0012 §2's
dated cooldown is not applied either, nor 0001's comfort zone which is to
set the depth of the draw.

## The learning loop opened (2026-09-05)

`src/learned.rs` implements
[0014](decisions/0014-shape-of-the-learned.md):
`learned/artists/<slug>.toml` in the catalog, one file per artist,
**counters with decay built in**
(`plays = plays × ½^((now−last)/6 months) + 1`), `learned/marks/inbox.toml`
for the marks. Written on every gesture, silent, never contributed back.
`classement.json` (741 artists) serves as the starting familiarity.

**Seven measurements wired**: `tl` like, `ts` skip (notes and moves on),
`tb` ban the track, `tm` mark, `al`/`as` the artist's weight, `ab` ban the
artist. Plus the **automatic counting** of a full play — only `EndOfTrack`
counts, a skip is not a play.

**Three reads by the engine**: excluding the banned (artists and tracks),
the artist's weight applied to the branches that start from them, and `?`
which shows familiarity and weight. The bans go through the exclusion
channels the engine already has (`visited` by slug, `played` by title), so
without touching its signature.

**What is still missing on the read side**: familiarity does not yet feed
the comfort zone ([0001](decisions/0001-comfort-is-familiarity.md)), and
[0012](decisions/0012-track-rotation.md)'s cooldown is not applied. On the
write side, the **edits** (`tt`, `tT`, `td`, `ae`, `aL`) touch the cards and
need the layer that writes and commits the catalog.

## Additions merge by themselves (2026-09-20)

Joel: "let's make it so that PRs with only card additions are validated
automatically". Auto-merge, kept in reserve on the morning of 09-20, was
settled that evening. On the reference,
`.github/workflows/automerge.yml`:

- triggered by **`workflow_run`** when `catalog` completes on a PR — in the
  reference's context, with the right to merge, where the token of a PR
  coming from a fork is read-only;
- finds the PR by its head sha (`commits/<sha>/pulls`, since
  `workflow_run.pull_requests` is empty for a fork), reads its files: **all
  `added` in `cards/`**, otherwise "a reader decides" and nothing moves;
- merges (`gh pr merge --merge`), says so in a comment, then **regenerates
  the index on `main`** itself — a push made with the workflow's token
  triggers no other workflow, so `catalog`'s `index` job would not see it
  (it stays for merges done by hand).

`CONTRIBUTING.md` states the rule; so does `Cp`'s confirmation. Joel's PR
no. 1 touches up three cards: it waits for his reading.

## `aL` also removes, and `ae` finally opens the card (2026-09-20)

Joel, reading his PR back: King Hannah "was linked to Beirut by mistake. If
I want to remove the link from forkstify, how do I do it?" — nothing did.
Two gestures wired the same day:

- **`aL` both ways.** The modal first lists the links the card already has
  (✓, type, name, provenance note), filtered as you type; **enter on one
  removes it** — `edit::remove_link`, the line alone goes away, commit "King
  Hannah — unlink: → Beirut (similar)"; below it, the search to add one, as
  before. One gesture for links, after the model of `tl` which likes and
  unlikes. And **the engine follows immediately**: the card in memory loses
  or gains the link, and the branches recompute — no need to relaunch.
- **`ae` — the card in `$EDITOR`, in place.** What the key reader had
  prevented since 09-05: it **parks itself** (`keys::suspend_reader`, the
  reader waits for `poll` before reading, no keystroke stolen), the terminal
  leaves raw mode (`keys::raw_pause`) and the alternate screen
  (`Tui::suspend`), `$VISUAL` or `$EDITOR` opens on the card, then
  everything comes back (`raw_resume`, `Tui::resume`, a full redraw). The
  card is read again: changed and readable, it replaces the session's and is
  **committed** ("… — edited by hand", trailer `edit`); unreadable, the
  toast says so, nothing is committed, `ae` again. With no editor named,
  `xdg-open`. The loop waits for the editor: the sound carries on in its
  thread, and the end of a track waits for it to close.
- 93 tests green (`a_link_is_removed_and_the_others_stay`).
- **Joel's first try**: "`ae` works well but the screen does not redraw
  after the `:wq`". Checked with a spike (`src/bin/spike-editor.rs`, in a
  pty with `stty rows 24 cols 80` — with no size, ratatui draws nothing,
  which is what made the previous smoke tests silent about the display): the
  give back / resume / clear / redraw sequence **is indeed emitted** after
  nvim exits. Two things fixed alongside: nvim asks the terminal who it is
  on the way out (`ESC [ c`) and the answer arrived on `stdin` **after** it,
  read as keystrokes — "62;1;4c" would have taken branches 6, 2, 1, 4 —
  hence `keys::drain_input` before handing the keyboard back; and the screen
  is repainted **in place** on resume (`paint()` in `edit_card`, resuming
  with styles reset and `2J`), without waiting for the next gesture. To be
  tried again; if the screen stays empty, look at whether a key brings it
  back (the repaint) or not (the terminal).

For PR no. 1: `aL` on King Hannah, enter on Beirut, then `Cp` — the
`proposal` branch rewrites itself and the PR updates. **Done by Joel that
same evening**, with one hitch: `Cp` asked for `y` again and `gh pr create`
refused, "a pull request already exists". Detecting the open PR went through
`gh pr list --head kbyjoel:proposal`, which returns nothing — `--head` wants
the bare branch name; fixed (`--head proposal`, the owner filtered on the
answer), and the open PR now receives **the rewritten title and body**
(`gh pr edit`) on top of the branch.

## The detached fork: `y` refused by GitHub, the fork remade (2026-09-20)

Joel, on `Cp`'s `y`: "pull request create failed: GraphQL: Head sha can't be
blank, Base sha can't be blank, Head repository can't be blank, No commits
between aropixel:main and kbyjoel:proposal…". Diagnosed through the API:
`kbyjoel/forkstify-catalog` was **no longer a fork** as far as GitHub was
concerned (`fork: false`, no parent), and both catalog repositories had gone
public. **When a private repository goes public, its private forks are
detached** and become independent repositories; and a PR only opens between
repositories in the same fork network.

Repaired without touching the local clone or the other machine: the detached
repository renamed `kbyjoel/forkstify-catalog-detached`, the reference
forked again under the freed name (`gh repo fork`), `main` and `proposal`
pushed to it (same history, same `origin` URL).
`compare/main...kbyjoel:proposal` answers "ahead 1". **The detached
repository can be deleted by Joel**, it holds nothing the new fork does not.

And `Cp` now says it in one sentence instead of GraphQL's list: before
proposing, it asks GitHub for `origin`'s parent and, if it is not
`upstream`, explains the detachment and exits.

## The first real `Cp`: `gh` not found out of raw mode, and the colors (2026-09-20)

Joel ran his first `Cp`: the `proposal` branch went out ("Propose 49 cards
(46 generated, 3 edited)"), but "the confirmation prompt is missing", and
"the colors like on the mockup".
- **`gh` was not found.** It only exists on this machine through mise's
  shims; launched from Omarchy's bar, forkstify does not have a mise
  shell's `PATH`, `gh auth status` fails, and `Cp` takes the browser route —
  with no confirmation, since the click stands in for it.
  `fork::gh_command` now looks for `gh` on the `PATH`, then in
  `~/.local/share/mise/shims`, `~/.local/bin`, `/usr/local/bin`, `/usr/bin`;
  the setup uses it too. And the browser route's toast now says why ("gh not
  found or not logged in").
- **The confirmation, like screen 4**: the steps done at the head (fetch,
  worktree, commit, push), `from` → `to`, the title, **the whole body** (the
  overlay scrolls), the `gh pr create` command, the question with the `gh`
  account, and what the action will check.
- **The colors** (`tui::info_line`): a `##` title in bright, a new card `+`
  in green, a touched-up one `~` in yellow, a PR slug in blue, a key
  (`from`, `title`, `origin`…) in blue and its value in bright, the trailer
  dimmed, a glyph line (`✓ ⏹ ⊘ ↻ → ⇅`) in the toast's color, the keys of a
  hint line (`Cd`, `Cp`, `Cu`, `o`) in magenta. They apply to `Cd`,
  `:catalog` and `Cu` too. Test
  `a_catalog_line_is_coloured_by_its_shape`.

Joel's second try, binary up to date, `gh` found: "the confirmation with the
y is still missing". It was there — **below the fold**: the PR's whole body
(49 cards) pushed it to the bottom of an overlay that scrolls, and the
screen only showed the top. Now the **question is at the head**, right under
`from`/`title`, and the body is **abridged as on screen 4** — two new cards
then "… 44 more, one line each", the touched-up ones in full
(`abridged_body`, test). `Cd` has all the detail. The pushed `proposal`
branch is still there: the next `Cp` rewrites it and asks for `y`.

## Overlays scroll (2026-09-20)

The first piece of feedback on `Cd`, Joel: "if my `Cd` is large, I cannot
scroll to see everything the diff holds". The overlay cut off at twelve new
cards and the block at the screen's height. Now **every overlay scrolls** —
`Cd`, `:catalog`, `ta`, `Cp`'s PR, `Cu`'s conflicts: `j`/`k` or ↑↓ by one
line, `gg`/`G` to the two ends, the title says what lies above and below
("↑ 10 · ↓ 12 · j/k"), and the axis's selection does not move while an
overlay is open (`overlay_scroll` in `listen.rs`, `render_block` with
`scroll`). `Cd` shows the whole diff. One rendering test
(`a_long_overlay_scrolls_and_says_so`).

## README, LICENSE, and a history with no personal note (2026-09-20)

Joel wants to make the application public. Checked beforehand: no token, no
password, no address in the 188 commits; the history is kept as it is — the
dated decisions and the commits that point at them are the project's memory,
and a squash would say the opposite of what the repository is. Done:

- **`README.md`** in English (0022): what it is, what it needs (Linux,
  Premium, git, docker), installation (Omarchy plugin or `bin/build`), the
  first run (the seven steps), the card, the configuration, where the docs
  are. **`LICENSE`**: MIT, as the manifest announced.
- **A personal working note removed from the history** (Joel, 2026-09-20),
  through `git filter-branch`, and `main` force-pushed. It now lives outside
  any repository, in `~/Work/forkstify-private/`. **The other machine has to
  re-clone**, or `git fetch && git reset --hard origin/main` — its local
  history no longer matches.
- Not done, judged unnecessary: the old `amis-*.py` scripts stay in the
  history of both catalog repositories (Joel: "no big deal"), as do the
  reference's eight learned files; to be reconsidered together the day the
  reference goes public.

## The reference's action and its CONTRIBUTING (2026-09-20)

Joel: "set up the GitHub action and the CONTRIBUTING.md on the reference".
The three pieces of "Review on the reference side" are there:

- **`forkstify validate [catalog]`** (`src/validate.rs`) — every card read
  as raw TOML and as a `Card`: `format = 1`, a `name`, an `mbid` **unique
  across the whole catalog** (the same person under two files is refused), a
  file name in slug form, `links` whose target is a slug (a target with no
  card is a proposal, 0016) and whose type is in 0010's closed list or
  declared in `catalog.toml`. A name that no longer matches its file (`Ye`
  against `kanye-west`) is a warning, not an error. Exit 1 on an error. Both
  catalogs pass: 0 errors, 18 name warnings.
- **`.github/workflows/catalog.yml`** on `aropixel/forkstify-catalog`:
  `check` on every PR — only cards may change, then `forkstify validate .`;
  `index` on every push to `main` that touches the cards — `forkstify
  vectors .` and the index committed by the bot. A composite action
  `.github/actions/forkstify` builds the binary from `aropixel/forkstify`
  (cargo cache, alsa); the embedding model is cached between runs.
- **`CONTRIBUTING.md`** (English, 0022): how a proposal is made (`Cp`), what
  the action checks, how it is read — generated ones: a glance; touch-ups:
  the facts are taken, a `similar` with its note, a top if it fixes an
  error. The README points at it.

~~**For Joel to do**: the `FORKSTIFY_TOKEN` secret~~ — moot:
**`aropixel/forkstify` has been public since 2026-09-20** (Joel), and the
workflow's token is enough for the action to clone the application. The
README now installs over `https://`.

## The catalog without tooling, the reference without learned data (2026-09-20)

Joel: "remove the Python scripts in `tools/` from the catalog and clean the
reference's learned data". Done on both repositories:

- **`kbyjoel/forkstify-catalog`** (the fork, the one the application reads):
  `tools/` removed — the ten scripts each have their replacement in
  forkstify (the setup and `:library`, on-the-fly generation, `forkstify
  vectors`, `forkstify check`); `amis-*.py`, with no equivalent, go with
  them. `learned/` **stays**: it is Joel's learned data, and
  `classement.json` is read as long as `library.toml` does not exist.
- **`aropixel/forkstify-catalog`** (the reference): `tools/` and **the whole
  of `learned/`** removed — a fresh fork no longer inherits Joel's library.
  What is left is `cards/`, `vectors/`, `catalog.toml`, `.gitattributes`
  (the merge driver) and the README.
- The fork's first `Cu` will see `learned/` changed on one side and deleted
  on the other: the merge keeps his without asking (fork.rs).

## The `C` namespace — diff, propose, update (2026-09-20)

Workstream B of [`design/before-release.md`](design/before-release.md),
coded after `Catalogue.dc.html` (Claude Design, seven screens). Everything
settled on 09-19 and 09-20 is wired, in `src/fork.rs`:

- **`:catalog`** — the state in one line (origin, ahead/behind, last update,
  cards beyond the reference), and in local mode it says so.
- **`Cd`** — `:mine`'s overlay, renamed: the count and the date, the new
  cards (generated / written, tags, links) then the touched-up ones (+n −m,
  the sections affected read from the diff — tops, tags, description, link
  types —, the provenance note).
- **`Cp`** — fetch, a `~/.local/state/forkstify/proposal` worktree on a
  `proposal` branch from `upstream/main`, the state of `cards/` copied from
  `main`, a `Propose N cards (a generated, b edited)` commit whose body is
  two lists for the reviewer, `push --force`. With `gh` connected: the
  overlay shows the PR and **`y`** opens it (`gh pr create`), any other key
  sends nothing; a PR already open is updated by the push. Without `gh`: the
  GitHub comparison page, title and body in the URL.
- **`Cu`** — the learned layer committed, fetch, **merge** `upstream/main`;
  conflicts settled alone on `vectors/` (upstream, regenerated), `learned/`
  (yours), `tools/` (upstream); a card changed on both sides **stops** it:
  the merge stays in progress, the overlay names the cards and what each
  side changed, `o` opens the first one, `:catalog` resumes after `git add`
  (and `git commit`, or not). Then the index is regenerated if cards
  changed, the date is remembered, and **the session reloads its catalog**.
- **`:catalog fork <url>`** — leaving local mode.
- The gestures run in `spawn_blocking`, one at a time, and playback carries
  on. Departures: `:catalog` in an overlay, not in the flow; `Cd` does not
  scroll; `o` goes through `xdg-open`.
- 86 tests green, including an **integration test over three temporary git
  repositories** (reference, fork, clone) chaining diff, propose twice, the
  update that merges, the update that stops on a card and the resume —
  `git` entered the `forkstify-build` image for that (`Dockerfile`, image to
  rebuild: `docker build -t forkstify-build .`). **Not proved for real**:
  `Cp` through to `gh pr create`, `Cu` on Joel's fork.

## The setup after installation, after the mockup (2026-09-20)

Workstream A of [`design/before-release.md`](design/before-release.md),
coded the same day after `Installation.dc.html` (Claude Design, project
"Accueil Forkstify" — nine screens, imported by the agent). The mockup
settled the four open points: local mode kept, a single `library.toml`,
threshold ≥ 5 and at most 30 cards, both scopes accepted.

- **The first run**: `forkstify` with no readable catalog opens the setup
  instead of failing (`main::accueil`). Screen 0 lists the seven steps, `⏎`
  starts.
- **`src/setup.rs`** — the seven steps and the two exit screens.
  1 the catalog: the URL of a fork pasted in, `gh repo fork` + `gh repo
  clone` if `gh` is there and connected, or the reference cloned in **local
  mode** (`git config forkstify.local`, the sync commits without pushing,
  home says `⇅ local`); the clone goes into
  `~/.local/share/forkstify/catalog` (XDG), `upstream` added,
  `[catalog] path` written into `config.toml` (`config::set_catalog_path`,
  textual, the comments stay). 2 the git identity, only if the global config
  has none, written `--local`. 3 the connection: the phone (zeroconf in the
  background, escape skips) and the browser (`o`); **seven scopes** now
  (`user-follow-read`, `playlist-read-private`), and a token granted with
  the previous five **goes through the browser once** —
  `spotify::needs_reauthorization`, the scopes remembered in the state, and
  the screen says it is not a failure. 4 the library: `/me/tracks`,
  `/me/albums`, `/me/following`, one bar per source, the main artist alone.
  5 the playlists: their own first, space ticks, `/` filters, `⏎` harvests
  the ticked ones, remembered. 6 the comfort on the gauge. 7 the coverage:
  the ranking crossed with the catalog by bands (≥ 20, 10–19, 5–9), `o`
  generates the missing cards with a score ≥ 5 (30 at most) — one at a time
  in the background, vectors at the end, **one single commit**
  `library: N cards generated`. Screen 8 recaps in toasts, screen 9
  (`:setup`) lists the ticked steps and replays one; `:library` replays 4,
  5, 7.
- **`src/library.rs`** — `learned/library.toml`, a file in English (`name`,
  `spotify`, `liked_tracks`, `liked_albums`, `followed`, `playlist_tracks`,
  `score`, `sources`, the date and the ticked playlists), with
  `classement.py`'s formula unchanged; `learned.rs` reads it first, and
  `classement.json` as long as there is no `library.toml`.
- **`keys::parse_setup`** (digits, `j`/`k`, `o`, space, `⏎`, escape) and
  **`tui::render_setup`** (the step and its gauge to the right of the
  header, the lines: choice, field, checkbox, bar, step, key/value, names).
  A field opened after another no longer keeps the previous one's line (a
  text-mode generation — a bug seen in the smoke test).
- **Accepted departures**: the generation (7) and the harvest (4) are
  watched, escape stops them; `:setup` and `:library` close the session (the
  sound stops) and home comes back on the replayed catalog. The `tools/`
  scripts stay in the catalog until `:library` has run on Joel's fork.
- 81 tests green, no warnings; a smoke test of the whole thread (clone by
  URL, identity, comfort, summary) on temporary XDG folders. **Not proved in
  a real session**: the harvest, the playlists, the coverage generation,
  `gh`.

**On Joel's next launch**: the browser opens once for the two extra scopes,
then `:library` from home harvests his library and replaces
`classement.json`.

## `fg<n>` — generating a gap without taking it, and a gap's branch walks (2026-09-20)

Workstream C of [`design/before-release.md`](design/before-release.md),
coded on the day of the call. Joel: artists without a card proposed as gaps
forced you to queue them to get the card; he wants "to generate them, and
have it propose new branches accordingly", and "a branch regenerated like
the others, with one track by the generated artist and other tracks by other
artists".

- **`fg<n>`** (`Cmd::ForkGenerate`, `After::Gap`): gap n's card is born —
  composed, vectorized, committed, adopted by the session as today — and
  **nothing is queued**. On arrival, the "○ no card yet" row becomes a
  playable branch after the branches shown, the others do not move (the
  reason `⏎` draws among what is shown), and the toast gives its number. On
  a branch number, `fg<n>` answers "f<n> takes it".
- **The gaps refresh** around the context *and* the fresh card
  (`refresh_gaps`): its own links into the void appear in grey in their turn
  — the catalog grows along its links, one notch further. The next recompute
  starts from the list as it stands, like `fr`.
- **A gap's branch is a walk** (`engine::branch_from`, the `walk` behind
  `propose` and `wander`, the fresh card at the head, the link's reason and
  proximity as reason and weight). A fix along the way: `branch_to` — the
  path of `f<n>` on a gap — built the branch with `encore`, hence n tracks
  by the generated artist alone. `f<n>` and `fg<n>` both go through the
  walk.
- Tests: `fg2` parses, the grammar stays prefix-free; a fresh head's branch
  crosses more than one artist, an unknown head gives nothing. 72 tests
  green, no warnings. **Not proved in a real session.**
## The workbook of the last three workstreams before release (2026-09-19)

Joel: "refine the last things before we can release the project" — the setup
after installation, a "catalog" namespace (`:mine` renamed diff, a PR of the
new cards towards the reference, an update of the fork from the reference),
and generating a gap without taking it (`fg<n>`), with the branches then
proposed using the fresh card. The workbook is
[`design/before-release.md`](design/before-release.md): for each
workstream, what is asked for, what is proposed, what is left to settle; the
proposed order (C, then B, then A); and what release needs on top.

Proposals to approve, in short: the setup in **seven steps** (a forked
catalog or local mode, the git identity, the connection, the library, the
playlists to tick, the comfort, the coverage by generation),
`learned/library.toml` in English in place of `classement.json`, two more
OAuth scopes; the **`C`** namespace for the catalog (`Cd` diff, `Cp`
propose, `Cu` update — the letter settled by Joel that same day, after
ruling out a `c` shared with comfort), a PR that carries **the state of the
cards** on a `proposal` branch from `upstream/main` (never the learned
layer, never the vectors) and opens in the browser, a **merge** rather than
a rebase because `main` is shared by two machines; `fg<n>` which turns the
gap into **a branch at its number** without redrawing the others, then
refreshes the gaps. Settled the same day: `C`, the merge for `Cu`, and for
`Cp` the PR through `gh` with confirmation when it is there and connected,
the browser otherwise. On 2026-09-20, against the fear of laborious
validations: on Joel's fork, 46 new cards all generated against 3 touched
up — hence a **GitHub action** on the reference (checks, index regenerated
on merge), a PR **composed for the reviewer** in two lists, and a **merge
rule** written into a `CONTRIBUTING.md` (generated: a glance; touch-ups: the
facts are taken, a `similar` with its note, a top if it fixes an error).
Then one single `proposal` branch, rewritten, and the name `Cp` kept:
**workstream B has nothing left to settle**. Workstream C, on 2026-09-20:
the branch born of a gap is a **walk** like the others (`engine::walk`, the
fresh card at the head, other artists afterwards), not an encore of the
generated artist — which fixes the current `f<n>` on a gap along the way.
Noted along the way: the reference still carries Joel's learned data, to be
removed before it goes public.

## The engine's numbers are tunable in `config.toml` (2026-09-17)

Joel: "push the logic of *taking back the algorithm* all the way and give
whoever installed forkstify the ability to change the value of every number
(like `ARTIST_COOLDOWN_HALF_LIFE`, `LESS_OFTEN`, `WEIGHT_FLOOR`…) through
the configuration file". **Decision
[0023](decisions/0023-engine-numbers-are-tunable.md).**

- **A `[tuning]` section** in `~/.config/forkstify/config.toml`, twenty
  named settings, all at their former default value: the cooldowns
  (`track_cooldown_floor` / `_half_life`, `artist_cooldown_floor` /
  `_half_life`), taste (`less_often`, `more_often` — `0` = the mirror of
  `less_often` —, `weight_floor`, `weight_ceiling`), familiarity
  (`plays_reference`, `plays_half_life`), the pool (`top_weight`,
  `liked_weight_cocoon` / `_open`, `door_weight`, `door_bonus`,
  `tail_weight`) and the adventurous leap (`leap_floor_cocoon` / `_open`,
  `leap_trust_cocoon` / `_open`). The first run's template lists them all,
  commented.
- `config::Tuning` + `config::tuning()` (loaded once, at launch);
  `engine.rs` and `learned.rs` no longer hold a numeric constant. An absurd
  value is reported on stderr and reset to its default, alone.
- Out of scope, deliberately: the interface delays and the shapes of the
  draw (`take(6)`, the powers). See the decision.
- Two config tests (parsing, guard rails). 71 tests.

**On an existing machine**: the configuration file is not rewritten; with no
`[tuning]` section, everything is at its default. Copy the section from the
template (`src/config.rs`, `TEMPLATE`) to have it at hand.

## `fr` starts from the list as it stands (2026-09-17)

Joel: "When I do an `fr`, the songs added to the playlist through a `ti` or
through an `e` from a discography are not taken into account. It should take
the playlist's full current state into account. Same for the titles added
with `fw`."

The cause: the engine's state (`state()`) only read the rounds register —
branches taken, `e<n>` encores, `fw`. A `ti`, an `e` in the discography or a
`J`/`K` touch the queue without writing a round: `fr` started from the last
round, and could propose titles already in the queue again.

- **The list is authoritative.** `state()` now bases the state on the axis
  (past, current, upcoming): the **context** is the list's last segment
  (`engine::segment_of` — from the last segment head, a branch or "inserted
  (ti)", to the end; the whole list if there is no head, the seed's case),
  its artists join the **universe** and the visited, and **every title on
  the axis counts as played**. The rounds stay the fallback when the axis
  holds nothing known (a track outside the catalog), and the register `fu`
  pops from.
- An accepted consequence: after `fn<n>` (a branch inserted after the
  current track, the rest kept), the directions set off from what **ends**
  the queue, not from the inserted branch — that is what the list says.
- `fu` empties the queue **before** re-reading the state, and takes the
  previous artist from the register, not from the list.
- Test `the_last_segment_of_the_playlist` (69 tests).

## The whole repository goes English (2026-09-21)

Joel: "convert every document of the project into English, and state in
AGENTS.md that everything generated must be generated in English."
**Decision [0024](decisions/0024-everything-in-english.md)**, which
supersedes the "Language" part of
[0022](decisions/0022-english-interface.md): the interface went English on
09-10, the prose follows.

- **The 44 documents are translated and renamed.** `docs/conception/` →
  `docs/design/`, `avancement.md` → `progress.md`, `atouts.md` →
  `strengths.md`, `reglages.md` → `tuning.md`, and every decision keeps its
  number with an English slug. `git mv` throughout, so `git log --follow`
  still reads on every file.
- **A decision is translated, not revised**: same content, same date, same
  status. Names on disk quoted in an old decision keep the spelling they had
  that day (`catalogue.toml`, `fiches/`, `usage/`), and 0010 says in a line
  that 0022 renamed them.
- **The cross-links are rewritten**, in the documents and **in the code**:
  some thirty `docs/conception/…` references and French decision slugs in
  `src/`, including the one in `config.rs`'s `TEMPLATE`, which the user
  reads in their `config.toml`. Zero broken links, checked over every `.md`.
- **The last French comments** in the code are translated (`embed.rs`,
  `validate.rs`, `listen.rs`, `engine.rs`, and "chantier" → "workstream"
  everywhere). What does not move: `embed.rs`'s vectorized text (the index
  depends on it, 0022 §5), artist names, track titles and Joel's quoted
  words.
- **`AGENTS.md` is in English** and carries the rule in one line, under "How
  the agent works here": everything generated is generated in English,
  including the commit messages the agent writes. French remains the
  language of the exchange in the terminal.
- Two duplicated 2026-09-14 headings in this file are merged along the way.
  93 tests green, no warnings.

**Left to do**: the edit commits the *application* produces still speak
French ("Cat Power — tops : +2 −0") — see the pending fixes below.

## Installing without Docker: a GitHub release and an AUR package (2026-09-21)

Joel, while trying a clean install: "can we ship a binary so the user does
not have to have Docker?". The binary lends itself to it: 56 MB, and only
ALSA, libstdc++, libgcc, libm and libc as dynamic dependencies — ONNX
Runtime is already statically linked.

- **`.github/workflows/release.yml`**: on a `v*` tag, checks that the tag,
  `Cargo.toml` and `manifest.json` agree, runs the tests, compiles in
  `rust:1-slim` (the same image as the `Dockerfile`, so CI publishes what
  `bin/build` produces), strips the binary and attaches
  `forkstify-<version>-x86_64-linux.tar.gz` and `SHA256SUMS` to the release.
  It is the application repository's first CI.
- **`omarchy/install.sh` rewritten**: the published binary first, with its
  digest verified — never an install past a mismatch —, and building in a
  container only as a fallback or on `--from-source`. It does not touch a
  `forkstify` put there by a package manager.
- **`packaging/aur/forkstify-bin/`**: `PKGBUILD` and `.SRCINFO` for Arch.
  Depends on `alsa-lib`, `gcc-libs`, `glibc` and `git` (the catalog is a git
  repository); `github-cli` and `xdg-utils` optional. `sha256sums` is at
  `SKIP` as long as no release exists: `updpkgsums` fills it before
  publishing. The procedure is in `packaging/aur/README.md`.

The package puts `/usr/bin/forkstify` in place but not the widget, which
stays `omarchy plugin add` (0021); the two complement each other, the widget
testing `command -v forkstify`. **Left to do: tag `v0.1.0` to try the
workflow for real.** The fastembed model (241 MB) remains the big cost of
the first run, independent of all this.

## Artist freshness and a tail modulated by familiarity (2026-09-14)

The small-circle feedback (usage-feedback no. 13), wired.

- **Freshness at the artist level** (`Learned::artist_freshness`, half-life
  4 days, floor 0.3): an artist heard recently steps back as a branch head,
  and recovers over a few days. Applied to the graph's heads, to the
  adventurous branch and to the `stay` branch. It breaks the reinforcement
  loop that was tightening the circle.
- **A tail modulated by the artist's familiarity**: in `reservoir`,
  `share = tail_share() × familiarity`. A new artist is led by their tops, a
  known artist opens up their long tail. Lowering comfort widens the artists
  without bringing in deep cuts at random.
- The tail test is rewritten (familiar ⇒ tail, new ⇒ tops), with an
  `artist_freshness` test added. 68 tests.
- **The data is already synced**: `plays`/`last` live in `learned/`, version
  controlled in the fork and synced by 0017. Freshness travels between
  machines with no new file (Joel's question, 2026-09-14).

## `:warm` on an empty tail, and a card with the wrong identifier (2026-09-14)

Joel: impossible to get Jarvis Cocker's discography. Two causes.

- **The card pointed at the wrong Spotify.** `2kTHIUipN0SYKBbmcTCLfQ` is
  "Jarvis Branson Cocker", an almost empty homonym (2,176 listeners, with no
  discography of their own); the real one is `13W7XLRXdWeLmIu9vacE1w` (the
  profile verified). Checked on open.spotify.com, fixed in the catalog. The
  API was answering, then, but with zero albums.
- **An empty tail was cached and blocked the retry.** `:warm` saw `[]` in
  the cache and said "already cached, 0 tracks" without ever trying again.
  `:warm` now **forgets the cache before harvesting**, so it always makes
  the call again; and a harvest with zero tracks says so clearly ("its
  Spotify identifier may be wrong") instead of passing for a success. The
  empty cache on disk was deleted.

## `:warm` harvests the artist under the needle (2026-09-14)

Joel: `:warm` was taking the context's last artist (`state().1`), not what
is playing or the highlighted row. It now uses the same target as
`e`/`t`/`a` (0020): `target()` — the selection, otherwise the current track
— falling back on the context if the target is outside the catalog.

## A little color on the track info panel (2026-09-14)

Joel: "put a little color on the track information window". `info_line` in
`tui.rs` tints the labels of the `ta` panel (featuring, album, tags, from
here, off-catalog) in bold cyan, brightens their value, and turns the
metrics line (familiarity, weight, links) yellow with discreet separators.
Lines that are not "label: value" — the rows of the help menu, which shares
the same panel — stay grey, as before. Cover art in `ta` is still waiting
for the terminal's graphics protocol.

## Resume resumes the whole journey, and nothing goes to stdout any more (2026-09-14)

Joel, two points.

- **"Resume" (`r`) picks the whole current journey back up**: history,
  current track and upcoming tracks, instead of starting from the last track
  alone. `LastSession` now carries `past`, `current`, `queue` and `rounds`
  (Stop/Source/Head/Round derive serde); `remember` records them on exit,
  `restore` puts them back and replays the track that was playing. An older
  `last.json`, without those fields, falls back on the old behavior
  (starting from the last track).
- **No journey written to stdout on exit any more.** `run` no longer returns
  the artist list and neither caller prints "Journey: …": the alternate
  screen is given back, and nothing shows behind it.

## Comfort is kept from one launch to the next (2026-09-14)

Joel: "I'd like forkstify to remember the last comfort zone chosen". A
`~/.local/state/forkstify/comfort` file, written on every change
(`config::remember_comfort` on `c<n>`, `:comfort`, a confirmed `cc`, home
and listening) and read back at startup (`config::comfort_at_start`, called
everywhere the old code read `config.journey.comfort`). The state outranks
`config.toml`, which becomes the first run's seed.

## Four pieces of usage feedback: taste, info, gauge, album (2026-09-14)

Joel, after a few days.

- **`tl` toggles liked / not liked.** On an already liked track, `tl`
  removes the like **with no penalty** — `ts` remained "less often + remove
  the like", and the plain removal was missing. `learned.track_liked` /
  `unlike_track`; it holds while listening and in the discography.
- **`ta` — track about**, a new key (an English word, not "fiche"): album,
  featuring, year when the discography has them, plus tags, familiarity,
  weight and the first branch. **It replaces `?`/why**, whose reason is
  folded into it. Featuring read from the title (`feat.`/`ft.`/`with`),
  album and year from the discography cache, with no network call.
- **The comfort gauge** is now **at the top right on both screens, with the
  editing look permanently** (`comfort_spans`), with `cc` adding "↑↓".
  **Home's status only shows when degraded** (Joel doubted its usefulness
  continuously): red, second line, otherwise nothing.
- **⏎ on an album row** in the discography **listens to the whole album**:
  its tracks open a new seed in order, then the branches set off from the
  artist (`start_album`). On a track, ⏎ keeps the 09-11 meaning (set off
  from it).

Cover art in `ta` is left for later: it needs the terminal's graphics
protocol (sixel/kitty), a separate workstream.

## `aL` links to an artist chosen from the search (2026-09-14)

Joel: "I don't understand the gesture for linking one artist to another… I
was listening to King Hannah and I wanted to link them to Peter Kernel".
`aL` linked to the artist **you had come from**, an implicit target no
screen announced, and one that could not reach an artist outside the
journey.

`aL` now opens the search modal, with the header "link <artist>"; enter on a
row writes a `similar` link in the current card towards the chosen artist. A
catalog target (`Hit::Artist` or a title with a card) or one outside the
catalog (a slugified name — a link towards a missing card is a proposal,
0016). A `link_from` carried by the `Finder`, resolved at the head of
`take_found`; the edit commits as before (it counts at the next launch). It
works on home as while listening. A current artist with no card cannot be
linked: the toast points at `:generate`.
## `:generate <name> <mbid>` offers instead of overwriting (2026-09-11)

Joel: with an mbid, `:generate` started a new list and overwrote what was
playing. Now, if there is a list in progress (a track playing or a non-empty
queue) and an mbid is given, the card is made but **the journey does not
start**: a success toast that lingers longer (`SEED_OFFER_SECONDS`, 12 s)
offers to set off from the artist — **⏎** accepts and replaces the list, any
other key keeps the list and does its job. A new `After::Offer` branch, a
pending seed `pending_seed` consumed by enter in `on_cmd`. With no mbid, on
home or with nothing playing, the previous behavior does not move. The toast
now carries a duration of its own (`linger`).

## Branches no longer scroll past into the void, and the card reopens (2026-09-11)

Two pieces of feedback from Joel after a real session.

- **Branches could be chosen endlessly without playing.** Since the tail is
  harvested for the proposed artists, a branch can draw a tail track (at
  comfort 3, the tail weighs). A track unavailable in the region ends the
  instant it starts: the branch runs out, `auto_advance` draws another, with
  no sound — and nothing stopped it. A safeguard: `MAX_DRY_ADVANCES` (4)
  counts the branches chained without a track reaching the speakers; past
  that threshold, playback stops and hands control back ("tracks may be
  unavailable in your region"). The counter falls back to zero as soon as a
  `Playing` arrives.
- **The Omarchy card did not reopen forkstify when it was off.** The "Show
  forkstify" button pointed at the bare word `forkstify`, which also caught
  a terminal left in `~/Work/forkstify`: focus went to that shell instead of
  launching. Back to `omarchy-launch-or-focus-tui forkstify`, on the app-id
  `org.omarchy.forkstify` — precise, as before the button was added.

## Enter in the discography sets off from the track (2026-09-11)

Joel: "I want to be able to start a new seed from a song on the discography
screen (with enter?)". Enter kept the 09-08 meaning, writing the batch. The
two stack: enter writes the batch if there is one (a commit), then, **on a
track, sets off from it** — `start_journey` on `Choice::Track`, like the
search. On an album, enter only writes; a banned track does not set off. The
modal's handler is synchronous, so the seed goes out through
`start_requested` after it, like `:search` and `:wander`. Detail in
[artist-exploration.md](design/artist-exploration.md).

## `fw` — going far away, or to someone (2026-09-11)

Joel: "let's implement fw. I also want to be able to do `fw <artist name>`
to aim at a particular universe". Feedback no. 6 is settled in its (a)
reading, leaving the universe, with an optional target.

- **`engine::wander`**: with no target, the head is drawn among the artists
  furthest from the journey's center — outside the journey and everything
  one link away from it, below the comfort floor (what the adventurous
  branch refuses), with unplayed tops; the furthest weighs the most, and the
  dial leans as for any head. With a target, the head is that artist,
  whatever the distance. Then the same walk as a branch. A test pins both
  cases down.
- **The key**: `fw` opens the `:wander ` line already filled in (`read_line`
  takes the start of a line); enter alone goes far away, a name goes to
  them. `:wander [artist]` is the spelled-out command. The name resolves
  through `search_names` on the catalog; if absent, the toast points at
  `:generate`.
- **Where**: the branch goes at the end of what is decided, like `f<n>`. To
  settle in use: if "go off to something completely different" means now,
  `fn`/`f!` have the gesture, and `fw` could take it.

Joel, before relaunching: "the home page mentions a random with enter, which
is not true" and "cc + arrows does not work".

- **Enter on home** took the page's first door, whereas the screen and
  [home-screen.md](design/home-screen.md) promise "at random — a weighted
  draw, the door that asks you to choose nothing". It is now true: a draw
  over all the page's doors, weighted by what the dial does to the artist's
  familiarity (`Comfort::favours`, the engine's own) — the familiar at the
  cocoon, the unknown wide open, never a zero weight. A highlighted row
  keeps priority.
- **`cc` on home** did nothing: home took the key before the dial was
  consulted, and did not know about `ComfortMode`. The dial now goes ahead
  of home in `on_cmd`, `cc` opens it from home, the gauge lights up as while
  listening, and confirming only recomputes the branches if there is a
  session.
- **The gauge looked "odd" while setting it** (Joel, right after): dial mode
  turned the whole gauge black on cyan, so the full blocks went black and
  the empty ones colored — the reverse. The blocks keep their color, only
  the words light up, on both screens.
- **The blocks in reverse order** (Joel: "when I go to cocoon, the never
  played come first… that's counter-intuitive, no?"):
  [home-screen.md](design/home-screen.md) says "at the cocoon the regulars
  first, at exploration the neglected", and the code tested `comfort ≥ 4` to
  put the neglected first — a leftover of the dial's old meaning (5 = cocoon
  since 2026-09-06). Inverted, and a test pins the meaning down at the four
  corners (5, 3, 1, 0). `home.rs`'s first test.

## The tail follows the branches (2026-09-11)

Joel, "very happy" after a few days of listening (noted in
[strengths.md](strengths.md)), but at comfort 3 "never any long tail".
Checked in the code: a branch never harvested the discography — only `e<n>`,
`:warm` and `ad` did, and 16 artists out of 314 had one cached. The weight
itself was right.

Option 1 chosen by Joel: after every `recompute`, the artists of the
proposed branches with no tail are **harvested in the background**, without
a word, as soon as comfort opens the tail up. `harvesting` becomes a map
slug → silent, so that the "discography — loading" toast only serves
requested harvests. Detail in [long-tail.md](design/long-tail.md).

A first wiring redrew the branches still proposed when the tail arrived; in
use, "the songs in the branches change immediately", and Joel does not want
that. Between waiting for the harvest before showing and showing the tops
then letting the cache serve the next draws, **the second** is wired (the
simplest, reversible): a proposal once shown does not move. `engine::redraw`
and its test are removed. **To be tried: the `·` mark should appear from the
next draws on, not on an unknown artist's first one.**

## The bar's card finally follows the needle (2026-09-11)

Joel: the Omarchy card's progress bar "stays at 0:00 while a track is
clearly playing" — the return of the 09-10 bug. This time the bus is not at
fault: `busctl` gives the position, the length, "Playing", and `Seeked` does
go out on every track. The fault is in Quickshell: `MprisPlayer.position` is
**computed on every read** (last sample + elapsed time) but the
`positionChanged` signal is never emitted during playback — a QML binding
therefore keeps the value read when the player was discovered, or the one
from the last `Seeked`: 0 at the start of the track. Verified in a separate
Quickshell instance: the bound property stays at 152.91 s while a direct
read advances; a call to `player.positionChanged()` resynchronizes the
binding. The widget gains a one-second `Timer`, active only **while the card
is open and the track is playing**, that asks for that signal. The 09-10
`Seeked` stays useful for seeks (`h`, a correction). Recorded in
[omarchy-bar.md](design/omarchy-bar.md).

## Ye and Kanye West are one (2026-09-10)

After the previous fix, `enter` on "Kanye West" answered "Ye already has a
card" and stopped. Two causes:

- **The collection doubled the artist.** The `kanye-west` card carries the
  name MusicBrainz gives them today, "Ye"; the Spotify library still says
  "Kanye West". The collection matched the ranking against the cards **by
  name**: two rows, "Ye" with a card but zero familiarity, "Kanye West" with
  no card. The `learned/` seed is now **indexed by slug**, and familiarity
  and likes are looked up by the card's slug or by the displayed name's
  slug: one row, the right familiarity. Test
  `le_seed_se_retrouve_par_le_slug_quand_le_nom_a_change`.
- **An existing card blocked the gesture.** `:generate` on an artist who has
  a card said "already has a card" and did none of what was wanted of it.
  Now the intent (`After`) runs anyway, through the same path as a fresh
  card (`Job::Existing`, `after_card`): enter sets off from them, `ad` opens
  their discography.

The card is still called "Ye": that is MusicBrainz's name of the moment, and
the card is Joel's — an `ae` renames it if he prefers "Kanye West".

## Enter on an artist with no card generates, and everything is said in a toast (2026-09-10)

Joel, on Kanye West highlighted in the collection: "it tells me 'Kanye West
has no card: nothing to branch from'". Yet arriving at an artist means
making them a card (0016), as `ad` has since that morning and as the search
modal does: `enter` on a row with no card goes through `:generate` — the
card is born, and the journey sets off from them.

And "the notification appears at the bottom and is not very visible. Note it
as a rule: every notification must appear as a toast." The 09-08 rule
already said "no more status line", but home kept its bottom line for what
it had to say (`(unknown: …)`, "nothing highlighted", whatever `tell` handed
it). Now **everything goes through the toast**, under home as while
listening: home drops what it has to say, the session picks it up after
every key and lays it out as a panel; `tell` no longer distinguishes the
screens. The rule is recorded in
[`application-shape.md`](design/application-shape.md) § "Everything is said
in a toast", test `the_home_says_everything_in_a_toast`.

## The next track moves up into the playback line (2026-09-10)

Mockups **4a** and **4a′** of `Lecture.dc.html` (Claude Design, project
"Accueil Forkstify"): the "up next" line under the bar cost a footer line to
repeat what the list shows two lines higher, and as long as a line followed
the bar, the eye took it for a separator. Wired as is:

- **The footer goes from three lines to two**: the playback line carries the
  whole time axis — `▶ what is playing — artist  (2 / 10) │ then ♪ next —
  artist` on the left, `♪ top │ 1:48 / 3:49 -2:00` on the right —, then the
  bar, which closes the footer. Same thing under home.
- **`→ fork in 8`** moves up into the title bar with the segment counters,
  in place of "n ahead": it is a state of the journey, not of playback.
  `→ fork next` when the queue is empty.
- **4a′'s order of sacrifice** as the window shrinks, with the right-hand
  block untouchable: 1. the next track's artist · 2. `then` and the counter,
  the `│` is enough · 3. the provenance and the remaining `-20:44` · 4. the
  current title cut with an ellipsis, never below 16 characters · 5. the
  next track leaves the line and the list marks it **`▸` in the gutter**
  (4b's fallback: ` 2 ▸↻ ♪ Israel`). One `…` per line at most; the next
  track's title is never truncated. No fixed thresholds: every step is taken
  as soon as the previous one does not fit.
- **Settled pending better**: if even the title alone does not fit, the
  current artist disappears entirely rather than take a second ellipsis (the
  mockup showed both cut, but its note forbids two cuts). Reversible.

Tests: the line at widths 170 / 145 / 138 / 120 / 96 / 72; the `▸` gutter at
60 columns. `Bar` loses `ahead`.

## The interface goes English (2026-09-10)

Joel wants to publish a first version soon, usable by as many people as
possible: **everything shown is now in English**
([0022](decisions/0022-english-interface.md)) — TUI, toasts, key tables,
command-line output, the `config.toml` template, the Omarchy widget and its
`install.sh`, the engine's labels (`liked` · `tail` · `non-top` ·
`off-catalog`, `shared members`, `close to the branch's center`…). The
subcommands follow: **`journey`** (formerly `parcours`) and **`listen`**
(formerly `ecouter`). The configuration's `[catalogue]` section becomes
`[catalog]`, with the old name still read. The notes written into the cards
by `td`/`aL` are in English (`set while listening, <date>`). What does not
move: `embed.rs`'s vectorized text (the index depends on it), the spikes in
`src/bin/`, and the documentation, which stays in French. In the same pass,
**all the code comments are translated** (~1,000 lines, only comments
moved, tests green), then the **51 test names** and their assertion
messages; only the spikes' output is left in French.

## The agent: an AI driving forkstify from the outside (2026-09-10)

An idea of Joel's: an `:agent` command that passes a prompt to a connected
AI (the Claude Code on his machine) — "build me a playlist of 20 tracks in
the mood of Kanye West, Drake, Kendrick Lamar" —, with the AI using
forkstify's tools, the catalog and the listening session, after an exchange
if it needs clearing up. Discussed, nothing coded; the direction is recorded
in [`agent.md`](design/agent.md).

In short: **the agent drives, it does not choose in its head** — it
searches, generates cards (`:generate`), asks for branches, reads their
reasons and queues; the playlist is the by-product, and the catalog has
grown. And **the agent is outside**: no LLM client and no key in the binary.
Floor 1, with no `:agent`: a control socket and `forkstify cmd ':…'` plus a
few JSON reads — the agent's API is 0013's `:` commands, and the
conversation happens in Claude Code. Floor 2, `:agent` in the TUI with an
external command configured, only if floor 1 turns out to be too heavy in
use. Six questions to settle in the note (the generation ceiling, the trace
in the queue, "off the top of its head" tracks, the name, the trailer,
whether floor 2 is necessary).
## A day of listening: the queue, the target, generation, the liked (2026-09-09)

Joel's first real long listening session, and his feedback handled as it
came — step 1 of the next steps is **under way**. In order:

- **The playlist showed a truncated queue.** The queue is a `VecDeque`;
  after a `push_front` (a branch taken "now", going back, `ti`) the buffer
  wraps and `as_slices().0` only returned its first half: a chosen branch
  was missing, or came back a few tracks later. `make_contiguous()` before
  drawing.
- **`tx` existed** but was missing from `t`'s hints; added, and its index
  now follows the same axis as the movement.
- **The encore targeted the end of the chain.** Since choosing a branch
  appends to the queue, the "current" drawn from the rounds is the last
  artist stacked, not what is playing: an `en3` served the wrong artist.
  Hence **decision [0020](decisions/0020-the-target-of-a-gesture.md)**: a
  gesture targets the **highlighted row, otherwise what is playing**, for
  `t`, `a` and `e`; `ts` and `tb` only move the music forward if they target
  what is playing; `en<n>` and `e!<n>` land **behind the highlighted row**
  when it is still to come. Feedback no. 12 (the target of `t`/`a`) is
  closed by this.
- **La Ruda**: five tops set by hand, and the card renamed "La Ruda Salska"
  — the MusicBrainz name is not Spotify's, and resolving a track searches by
  name. The question is noted in
  [on-the-fly-generation.md](design/on-the-fly-generation.md).
- **`:generate <name> <mbid>`**: an identifier found by hand replaces the
  search by name; if MusicBrainz stays silent, a minimal card flagged for
  review; an artist proposed as a gap takes their branch. Read from home as
  from listening.
- **"Lojo not found"**, two causes: MusicBrainz answers 503 in bursts (we
  wait through six attempts over half a minute, and say "busy" rather than
  "not found"); and a name coming from a slug has lost its apostrophes
  ("Lojo" for Lo’Jo) — **Deezer lends the spelling**, verified against the
  cards' key. A network test, ignored by default.
- **`docs/atouts.md`**: Joel's positive impressions, dated and quoted, to
  list the strengths when the time comes. First entry: rediscovering what
  Spotify never offered, coherence under control, unexpected all the same.
- **Home shows the liked by default**, `v` switches to the whole catalog.
  Liked = a "more often" or a ♥ here, a track, an album or a follow on
  Spotify (`classement.json`).
- **The guests on a liked track are not liked** (Bosh, Bossikan, Bow Wow —
  Joel's son): the harvest scripts now count only the main artist; the
  harvest relaunched by Joel, the ranking recomputed: 605 ranked artists
  instead of 741.
- **`al` / `as` / `ab` on home**, on the highlighted row. `as` sets an
  `unliked` flag in `learned/artists/<slug>.toml` that outranks Spotify and
  survives the harvests; `al` clears it; the 0017 merge treats it like a
  ban. An artist with no card is written and read back under the slug of
  their name.

Two design questions noted, partly settled: **removing an artist from the
liked** (done, above) and **a smooth setup** — connection, library import,
playlists to tick — settled as "first run *and* replayable", with a Claude
Design mockup to come from Joel before coding
([first-run.md](design/first-run.md)).

## The vector is born with the card (2026-09-09)

Joel chose (b): **the application does its own vectorizing**
([0019](decisions/0019-the-application-vectorizes.md)).

- **`embed.rs`**: the text composition carried over word for word from
  `vectoriser.py` (verified identical over the reference's 316 texts,
  `forkstify vectors --texts` against `vectoriser.py --textes`), the model
  through `fastembed` (rustls features, the quantized variant,
  `max_length = 128`, cache in `$XDG_CACHE_HOME/forkstify/fastembed`),
  writing one line into `vectors.jsonl` in slug order, and full
  regeneration. `Card` now carries `begin`, `end`, `origin`, `description`.
- **In session**: `Job::Generated` composes the text and sends the model to
  the background (`spawn_blocking`), `Job::Vectorized` adopts the card
  **and** its vector in the same commit (`Edit.also`). If the model is
  missing, the card comes in anyway and the toast says "without vector". On
  the first computation, the toast warns that the model is downloading.
- **`forkstify vectors [catalog] [--texts]`** regenerates the index and
  `meta.toml` (English keys, `max_length`, `normalized`). The import does it
  within its commit, with no docker.
- **Build image**: `g++` added to the Dockerfile. Binary: 23 → 58 MB.
- **The reference index regenerated** by the application and pushed to the
  fork (`dd82e8d` on the catalog side): cosine ≥ 0.999999 against the old
  one, norms at 1. `tools/vectoriser.py` marked as replaced, kept on record.

**Left**: editing a link in session does not recompute the vectors (it only
counts at the next launch) — noted in the design note. The regenerated index
is on the fork, not yet on the `aropixel` reference: to be pushed.

## The fastembed trial in Rust (2026-09-09)

To settle how a generated card gets vectorized, a throwaway spike in
`~/Work/tries/fastembed-spike` (outside the repository) vectorized the
reference's 316 texts with the `fastembed` 6 crate and compared them to
`vectors/vectors.jsonl`. Result: **cosine ≥ 0.999999 everywhere at
`max_length = 128`** (at 512, the crate's default, the rich cards diverge
down to 0.82), cold build 26 s, binary +35 MB, `g++` required in the build
image, a 241 MB model downloaded on first use, 14 ms per text. The default
features pull OpenSSL in: take the `rustls` variants. Found along the way:
the Python reference **is not normalized** (norms 2.5–3.6), which the
engine's centroid suffers from. Figures, conditions and the direction (b,
with a cargo-feature fallback) in
[design/on-the-fly-generation.md](design/on-the-fly-generation.md).
**Joel's call.**

## A title's versions are told apart in the search (2026-09-09)

Feedback from Joel: "when the song appears several times (*Quand on n'a que
l'amour*), I cannot tell which is which". The modal's `[spotify]` rows
carried only title, artist and a generic mention: five Brel versions
(studio, Olympia, a best-of…) made five identical rows.

- `search_tracks` now brings back **album, year and length** (`SearchHit` in
  `spotify.rs`, in place of the title/artist/uri triple).
- A Spotify row's note says them **first** — `Olympia 64 · 1964 · 3:07 ·
  branch next` — then what enter does: `branch next` when the artist has a
  card, `⏎ generates the card` otherwise. The old parentheses
  `(branch next) the card exists` / `(off-catalog — ⏎ generates the card)`
  go away; the group's header already says "off-catalog unless stated".
- The catalog is untouched: its titles are already deduplicated by (title,
  card), and the artist is enough to tell them apart.

On a narrow terminal, it is the end of the note that gets cut — so the
mention, never the album. Verified dry (tests); to be confirmed in use on a
title with several versions.

## Generating a card on the fly (2026-09-09)

Joel, that morning: "how do I add an artist? I feel like listening to
Jacques Brel but he is not in the catalog. I could find him through the
Spotify search, but I cannot play the track and it does not create the
artist card." Then: "yes, I want to be able to generate on the fly."

That was the unwritten half of
[0016](decisions/0016-broad-base-and-on-the-fly-generation.md). The point
the decision had left open — **when** generation fires — is settled:
**both**, the search and arrival. The subject has its own note,
[design/on-the-fly-generation.md](design/on-the-fly-generation.md).

- **`src/generate.rs`** — `tools/generate-cards.py`'s pipeline ported to
  Rust: MusicBrainz (identity, dates, origin, genres, typed relations) then
  Deezer with no key (five tops, four similars). Four calls, about three
  seconds, in `spawn_blocking` like the discography harvest. MusicBrainz's
  one-request-per-second limit is respected, and the 503 — frequent — is
  retried instead of passing for an absence.
- **One deliberate difference from the script**: a `similar` link towards an
  artist **with no card** is kept instead of being dropped. That is
  precisely what grows the catalog along its links.
- **The search brings somebody new in**: `enter` on a result outside the
  catalog generates the card, commits it, and sets off from them — from home
  as from listening. `:generate <name>` does the same without going through
  a track. No more "nothing to branch from".
- **Arrival grows the catalog**: `graph_neighbors` was throwing away links
  whose card is missing (Brel had three). `engine::missing_neighbors`
  returns them, the column shows them in grey — "○ no card yet" — and they
  are taken with the digit after the branches.
- **The session now owns its catalog** (`Live { catalog: Catalog }` instead
  of `&'a Catalog`) and inserts the fresh card into it: a generation you
  would have to relaunch to see would not be one. `listen::run` loads the
  catalog itself; the caller no longer lends it.
- **A generated card is an edit** (0013): `edit::create_card` writes,
  refuses to overwrite, and commits with the `Forkstify: edit` trailer.

**Still open, and Joel will decide**: the **vectorizing** of a generated
card. It is born with no vector — it navigates through its links and its
tags, and the screen says so — and catching up still goes through
`tools/vectoriser.py`'s docker command. Both ways out (a `:vectors` command,
or `fastembed` in Rust) are measured in the note; neither has code to undo.

**Not proved in a real session**: the pipeline is (network test
`le_pipeline_compose_une_vraie_fiche`, ignored by default), the two triggers
are not.

## `:search` opens from home too (2026-09-09)

Joel: "when you do `:search` from home, the search window does not open";
"on home, there is still some old text saying `/` does a search, whereas `/`
now filters".

- **The modal only existed on the listening screen.** On home,
  `:search <text>` still did the old gesture — resolve a name in the catalog
  and start — and `:search` alone did nothing at all. It is now **the same
  modal on both screens**: the keyboard is given to it **before** the screen
  switch, and it draws over home's body, collection included; the playback
  footer and the prompt stay.
- **Enter starts a journey**, from home: on the artist, or on the track —
  the title first, then its artist's branches. That is home's rule, and a
  digit already does the same there. A Spotify title **with no card** has
  nothing to branch from: home says so and points at listening, where
  `:search` plays it anyway.
- **Home's text still spoke of the old `/`**: the "search" block announced
  `/` for "catalog and spotify". It now announces `:search`, and a second
  line says what `/` really does — filter the collection, escape clears. The
  disconnected screen too.
- **`:search bowie` opens the modal filled in, and typing carries the word
  on**: the key reader's text mode was emptying its line on the way, so the
  first key erased "bowie". The starting line is set along with the mode
  (`keys::set_text(true, "bowie")`).

## Batch 4: 100 cards, the reference reaches 314 (2026-09-08)

Joel: "are there cards left to create? do we import them onto aropixel?"
Taking stock: 100 slugs called in by links with no card (the batch that
batch 3 had been calling since 09-02), 22 called by at least two cards; 561
ranked with no card, all below score 5, left to the tail as planned. A
generated card is knowledge, not taste: it goes to the **reference**
`aropixel` (0016), and the fork receives it through `upstream`.

**The tooling scripts had been broken since the 09-06 rename**:
`generate-cards.py`, `vectoriser.py` and `voisins.py` still read `fiches/`
and `vecteurs/vecteurs.jsonl`. Fixed on the `lot-4` branch off
`upstream/main`. The generator now retries on a timeout — the first run had
died on the first artist with a MusicBrainz `TimeoutError`.

**Generated in a `python:3.12-slim` container** (standard library only,
MusicBrainz at one request per second): **100 cards written**, and the
catalog holds **314 cards**. Report: to review, uncertain MBID — blundetto,
dalle-beton, lej, ozuna, palatine; with no Deezer, hence no tops —
mahmoud-ahmed. Vectors recomputed in the `fastembed` container (the model
downloaded, ~220 MB, cached in `tools/cache/`, now ignored by git).

Pushed to `aropixel/main`, then the `kbyjoel` fork rebased onto it: Joel's
learned commits do not go out towards the reference.

## Escape gets through on the first press (2026-09-08)

Joel: "when I want to close with escape, I often have to press several
times." After an `ESC`, the key reader read **two more bytes** to recognize
an arrow (`ESC [ A`), blocking — but the escape key alone only sends one: it
took two more keystrokes for it to get through. Fixed: after an `ESC`, the
reader **polls the descriptor** (`poll`, twenty milliseconds) — a sequence
arrives in one block, a lone escape has no sequel. The reader now reads
input **unbuffered** (`libc::read`), because `std::io::stdin`'s buffer would
have hidden the rest of a sequence from the `poll`. Same thing in text mode
and in the `/` and `:` lines, where an arrow no longer falls back into the
grammar. `ESC O A` (application mode) is recognized too.

## No more status line: everything in a toast (2026-09-08)

Joel: "I don't want any notification below 'up next' any more, they must all
appear as toasts; 'up next' must always be followed by the shortcut hints."
The "last thing said" line disappears from the footer, in session as under
home; the keys follow "up next" directly. Everything said goes through a
toast, parentheses included (in grey, four seconds). The log stays in memory
for the blocks (`?`).
## `J` / `K` move a track in the list (2026-09-08)

Joel: "how do I select a track in the playlist and move it?" Three routes
proposed — `J`/`K` one notch right away, `tg` grab-and-drop with the arrows,
`tm<n>` to position n — and the first one chosen: one notch covers the real
use, moving up a track you want to hear sooner, with no mode and no
confirmation, and the opposite key undoes it. Only what is still to come
moves; the branch name travels with its track; the move shows in the
numbering and is not said out loud. `tg` will come if long moves turn out to
be frequent.

## The search modal, and `ti` (2026-09-08)

Joel: ":search runs with no argument, a modal opens, with a line separating
the text area from the results area; ti — track insert — inserts a track
where you are in the list, opening the same modal." Mockup
`Recherche.dc.html` (Claude Design): 1a `:search`, 1b `ti` anchored, 1c both
edges.

- **The modal is modal, and that is new**: while it is open, the prefix-free
  grammar no longer applies — otherwise typing "cros" would fire c, r, o, s.
  The key reader has a **text mode** (`keys::set_text`): everything is
  typing, except ↑↓, enter, tab and escape. Hence the `⟩` prompt, to say
  "here, you write".
- **A single rule separates the input from the results**, and carries the
  count: `── catalogue 3 · spotify 5 ──`. No boxed field.
- **The catalog before Spotify, always**, in two groups never mixed (blue
  written by a human, cyan guessed). The catalog answers on every character
  — artists by name, titles known to the cards and the learned layer, with
  the `♪ ♥` provenance and the play counts; Spotify answers **behind**
  (`Job::Searched`), the cyan group says "… querying" until then, and an
  answer to an older keystroke is thrown away: the list never jumps under
  the cursor.
- **Enter acts according to the door**: through `:search`, an artist
  branches there (a segment at theirs), a title plays now and the branches
  set off again from its artist if they have a card; through `ti`, the title
  **enters the queue at the anchor** — before the highlighted row if it is
  still to come, otherwise right after what is playing — marked "inserted
  (ti)" in grey beside it; a chosen artist inserts their best unplayed
  track. The toast says "→ inserted at 4: title — artist" or "→ … — via
  :search".
- **`ti` states its anchor at the top** ("the insertion lands at 4 — between
  X and Y"), as the mockup wanted: inserting blind into a queue you can no
  longer see is the easiest gesture to get wrong.
- **Tab** hides Spotify. Empty, the modal says what to type; with no result,
  it says that too.
- The old log-based search (numbered results, a digit chooses) and its
  `pending` state go away.

**Set aside from the mockup**: `e` (queue), `tb`, `A`/`N` inside the modal —
those are letters, they get typed; `ti`'s "cards from the journey" scope
(the whole catalog answers, with Spotify behind); the durations; the last
five searches. One rendering test. Not verified in a real session.

## `ag` — the artist in the browser (2026-09-08)

Joel: "add a command, ag?, to google the current artist in the default
browser". `ag` opens `https://www.google.com/search?q=…` through
`xdg-open`, detached (inputs and outputs closed, so nothing prints under the
TUI). It targets like `ad`: the highlighted row if there is one, otherwise
what is playing. The toast says "→ artist — in the browser".

## Four keyboard touch-ups (2026-09-08)

Joel: remove queue mode from the hints; "I no longer see the shortcut for
slotting a track in, is it me?"; a `c<n>` shortcut for the comfort; `/`
becomes filter, the search becomes `:search`.

- **`Q` goes away** — from the hints, the grammar and the table. Queue mode
  fell on 09-06, and the queue chains up in the listening screen.
- **Slotting in**: it is not him. What exists: `e<n>` / `en<n>` (encore,
  same artist, at the end of the branch or right away), `fn<n>` (a branch
  after this track), and `e` in the discography (queued, at the end).
  **Slotting a specific track into a specific place does not exist** — it
  was queue mode's `i`, never wired. To be designed along with `:search`: a
  result could be slotted in after the current track rather than playing
  right away.
- **`c` becomes a namespace**: `c<n>` sets the comfort in one go (0 to 5, on
  home too), `cc` opens the gauge with the arrows. `c` alone could no longer
  be complete without breaking the prefix-free grammar.
- **`/text` filters, `:search <text>` searches.** On home, `/` filters the
  collection (escape clears, ↑↓ and enter start); in the discography it
  already filtered; while listening, there is no list to filter and it says
  so. `:search` does what `/` used to: catalog + Spotify while listening,
  with a digit to choose; the catalog alone on home, and the artist found
  starts.

## No more `tt` in the modal, and toasts (2026-09-08)

Joel: "the tt and tT shortcuts are still there in the discography modal, I
want to remove them. The loading messages should be more visible — toasts at
the bottom of one of the two columns, with some color?"

- **`tt` / `tT` leave the modal** (the `parse_modal` grammar, the keys, the
  legend, `Explore::top` / `untop` and their tests). What is left is `A`,
  which promotes an album's most played titles in one go; a single title
  gets liked (`tl`), it is no longer promoted. The write test goes through
  `A`.
- **The toasts**: a panel framed in the message's color, bold text, at the
  bottom right of the body, above the branch column's keys — over the modal
  too. **Sticky while something is loading** ("⏳ in progress — loading —
  title — artist", "X's discography — loading", in cyan, the color of a
  network wait), otherwise **four seconds** for the last thing said (`✓`
  green, `⏹` red, `↻` yellow, `→` magenta — `tui::tone_of`, the same reading
  as the footer's line). The parentheses stay on the footer line, with no
  toast. The one-second tick clears it.

## "Previous" restarts first (2026-09-08)

Joel: "when I press the arrow for the previous track and a track is playing,
I want it to restart the track from the beginning; you press a second time
to go back to the track before." Like any player: past three seconds
(`RESTART_AFTER_MS`), ← and ⏮ put the needle back to zero
(`Sound::restart`, a librespot `seek(0)` — the progress follows its
`Seeked`); within the first three seconds, or by pressing twice, you go back
to the previous track as before.

## Home becomes a session screen (2026-09-08)

Joel: "the two screens, home and playback, are independent: when I leave the
playback screen, my session stops and is lost. I'd like to start a session,
go back to home, keep listening and the bar at the bottom, and come back to
my session screen without ever losing my session."

Before, `main.rs` looped: home returned a choice, the session was born
(sound, API, MPRIS), lived, died, and home came back. Now **the session is
the application**, and home is one of its two screens (`Screen::Home` /
`Screen::Session`):

- **`q` while listening gives home back**, and listening carries on
  underneath: the sound, the queue, the branches, the learned layer, all of
  it stays. **`r`** (or escape with no cursor) **brings the session screen
  back**. `q` on home quits for good, with 0017's commit and push.
- **The playback footer shows under home** — what is playing, its bar, what
  follows, the last thing said — on the four lines above the prompt
  (`tui::Bar`, the same footer as in session, `render_bar`). `p` holds the
  pause there; the media keys work everywhere.
- **Choosing a seed on home while a session is playing starts a new
  journey** that replaces the old one (`Live::start_journey`) — with nothing
  to reconnect, so with no waiting screen.
- `home::run` becomes `home::Home` (a state: what is typed, the sort, the
  cursor) with `draw` and `on_cmd` → `Outcome::{Stay, Start, Back, Quit}`;
  `listen.rs`'s single loop routes the keys by screen. The sound and the web
  API only connect once per launch.

One rendering test of home with the footer. Not verified in a real session.

## The screen first, Spotify behind (2026-09-08)

Joel: "when starting a listening session, or opening and closing the
discography modal, there is often a lot of latency. I'd like to handle that
better: through cache where possible, and by showing first then loading
afterwards, with an indication that something is loading."

**The cause**: every call to the Spotify API — resolving a title into a
`spotify:track:` address, an artist's discography — was **awaited in the
command loop**, which only redraws once the gesture is finished. The backoff
on a 429 (an account/IP quota, frequent right after a discography's burst)
sleeps for up to sixty seconds inside that same wait: that is the freeze on
closing the modal, when `prefetch_next` was resolving the next track. The
cache already existed (resolutions in `target/resolve-cache.json`,
discographies in `~/.cache/forkstify/`), it only helped the second time.

**What changes**: the web API is shared behind a lock
(`Arc<Mutex<WebApi>>`) and the calls go out as **background tasks**
(`spawn_local`), which hand their result back to the loop through a channel
(`Job::Resolved`, `Job::Harvested`), like 0017's push.

- **Playing a track**: if the cache knows its address, it plays right away;
  otherwise it **shows right away** with "· loading…" in the footer, and
  plays when the answer arrives. A track skipped during the wait does not
  enter the past; a not-found moves on to the next; a failure puts it back
  at the head of the queue, as before. Prefetching the next one no longer
  blocks anything.
- **The discography** opens instantly on what we have — the tops and the
  learned layer — with "… discography loading" and the word "loading…" in
  its title; the albums arrive behind and the screen rebuilds without losing
  the sort, the filter or the pending edits (`Explore::reload`). A harvest
  never fires twice.
- **`e<n>` and `:warm`** start the harvest behind and say so: "its tail is
  on the way — try e<n> again in a moment".

**Still awaited in the loop**: the `/text` search, whose result you wait for
by nature, and the connection at startup (librespot, the token), which the
waiting screen shows step by step. Not verified in a real session: it is for
you to say whether the freezes are gone.

## The catalog becomes a fork, the reference moves to aropixel (2026-09-08)

Joel: "to solve the problem of the catalog not being a fork because I am the
designer, I'd like to move forkstify and forkstify-catalog into my aropixel
GitHub. That way I'll publish as aropixel, which lets me fork with my
kbyjoel account."

Done through the GitHub API: both repositories are **transferred** to the
`aropixel` organization (they stay private), and
`aropixel/forkstify-catalog` is **forked** into `kbyjoel/forkstify-catalog`,
which takes the freed name. Locally, `~/Work/forkstify` points at
`aropixel/forkstify`; `~/Work/forkstify-catalog` points at the fork as
`origin` — that is what the startup pull pulls from and what the learned
layer joins — and at the reference as `upstream`, which `:mine` already
compares against first. Joel is at last a user like any other (0016): his
edits live on his fork, and go up to the reference through a PR.

The two Cat Power commits stayed on the reference; to be carried over to the
fork or left there, Joel's choice. The chorizo recovery plan now clones the
right repositories.

## The dated cooldown (2026-09-08)

Joel: "apply 0012's dated cooldown then". §2 said: every play is dated, a
recently played track is penalized, and the penalty decays over time. The
dates had been there since 0014 (`last` per top), but the pool did not read
them.

`Learned::freshness`: **a tenth of its weight on the day it played**,
recovered with a **one-week half-life** — 55 % at seven days, 78 % at
fifteen, 94 % at a month; full if it has never played here. The pool
multiplies by that factor, after the "less often". A liked track played
yesterday (6.8 × 0.13 ≈ 0.9) therefore weighs like a top never played, and
takes its place back over the week. Two constants (`COOLDOWN_FLOOR`,
`COOLDOWN_HALF_LIFE`), "to be tuned along the PoC" as 0012 intended.
## One gesture for taste (2026-09-08)

Joel, after a day of listening: "I had trouble knowing whether I should like
the track or make it a top: to me it was the same thing." Then: "I wouldn't
even put a tt shortcut. We leave the ability to change the tops on your
fork, but we leave only one simple gesture to say I want to see this track
more often. In exchange, you also need to be able to say this track does not
interest me, and the likes must fully outrank the tops, which become nothing
more than entry doors on a fresh fork."

Decision [0018](decisions/0018-one-gesture-for-taste.md), wired:

- **`tl` more often, `ts` less often**, and each undoes the other (liking
  resets the skips, skipping removes the like). `tb` remains the never
  again.
- **A liked track outranks the tops** in the pool: ×10 at the cocoon, ×2
  wide open, the dial in between (`liked_weight`). A liked top takes that
  weight and carries `♥`. Before, a liked track weighed 0.8 and a liked top
  stayed an ordinary top — which is what made `tl` inaudible.
- **No more `tt` / `tT` while listening**: the session refuses them and
  points at the discography; the grammar still parses them, because that is
  where, in `ad`, they serve (`edit::set_tops`) — a first version had
  removed them from the parser and broke the modal, fixed right after.
  `add_top` / `remove_top`, which only served `tt` / `tT`, are removed.

Three tests (pool, learned layer, grammar). The subject of the reference
repository against the personal fork (a GitHub organization for upstream,
your clone becoming your fork) is still for you to do by hand; the
application needs nothing for that, except later an `:upstream`.

## Building it yourself, and counting usage (2026-09-08)

Two wrappers in `bin/` — the machine's convention, and mise puts them on the
`PATH` as soon as you enter the folder: **`build`** (cargo build --release
in the `forkstify-build` container, built on first call, the cargo registry
in the `forkstify-cargo` volume) and **`test`**. The three commits brought
back from the other machine (exploring an artist, "ad") compile with no
warning, 41 tests green.

The commands that count the commits forkstify produced across public forks
(the `Forkstify:` trailer) are noted in
[`first-run.md`](design/first-run.md), section "Measuring usage across the
forks".

## The learned layer syncs itself (2026-09-07)

Joel, switching machines: "learned filled up, but with no commit at all; I
no longer have my records. Shall we set up automatic commits and an
automatic pull on open? A better solution?" Then: "I approve, but I want the
message in English. How do we identify the number of commits in the
different users' public repositories?"

Decision [0017](decisions/0017-syncing-the-learned.md), wired the same day
(`src/sync.rs`, `learned::merge_artist`, the `merge-learned` subcommand):

- **Pull on start** (home and `ecouter`), after committing what was learned
  here; home's header says `⇅ up to date` / `⇅ learned committed, catalog
  updated` / `⇅ offline`. **A commit every ten minutes** if `learned/`
  moved, a push in the background, the result in the footer. **Commit and
  push on exit**, shown for a second. **`:sync`** on demand. Short network
  timeouts: offline, nothing hangs.
- **Merging by counter**: the git driver `merge=learned` calls
  `forkstify merge-learned`, which adds up what each side counted since the
  ancestor (decayed to the day), keeps a ban or a like set on either side,
  follows the weight that moved, and lets new tops in. Three unit tests,
  **and a real git scenario**: two clones, two concurrent listens to the
  same artist, `pull --rebase` with no conflict, `plays` gone from 3 to 6.01
  (5 + 4 − 3 decayed by a day).
- **The tops are written sorted** (`BTreeMap`): hash ordering made every
  write a spurious diff.
- **English messages, trailer `Forkstify: <kind> <version>`** on the learned
  layer's, the edits' and the imports' commits. To count usage:
  `gh api search/commits -f q='"Forkstify:"' --jq .total_count` (public
  repositories, default branch) and the reference repository's
  `forks_count`.

**Left for Joel to do by hand**: launch forkstify on the other machine — it
will commit its learned data, rebase onto what this machine pushed, and the
driver will merge. This machine pushed its own along with this commit.

## The discography opens as a modal (2026-09-07)

Joel, after a Cat Power seed: "I like almost nothing but tracks from the
album *What Would the Community Think*; I would have liked a command to get
a visual list of the tracks sorted by album, and to be able to `tt` the ones
I like and `tT` the tops I want to remove." Then, on the mockup: "let's go
with `ad` and a modal", form **1a** of `Discographie.dc.html`.

Designed in
[`design/artist-exploration.md`](design/artist-exploration.md), wired the
same day:

- **`ad` (or `:discography`) lays a modal over the listening screen** — it
  does not replace it, the sound does not stop, and the header and footer
  stay. It targets the artist of the **highlighted** row, of the current
  track failing that.
- **The albums are folded**: one hundred and eighty-seven titles become
  twelve rows, the one under the cursor opens by itself, with its share of
  the plays as a gauge. The header answers the question you came with — "2
  albums carry 79 % of the 118 plays — 6 albums never opened".
- **The modal has its own table** (`keys::parse_modal`, the product's
  first): `j`/`k` go down, `h`/`l` fold and unfold, `tt`/`tT` fix the tops,
  `A` promotes the album's four most played titles, `tl`/`tb` measure, `e`
  queues, `s` changes the order, `v` the view (all, ♪, ♥, ⊘), `/` filters,
  `u` undoes, ⏎ writes, escape closes.
- **One batch, one commit** (`edit::set_tops`): the edits accumulate at the
  bottom with the subject of the commit to come, and go out as one write.
  The first escape warns if any are left. That does not contradict 0017,
  which covers the learned layer: measurements and edits have never had the
  same rule.
- **The tops the discography does not return** fall at the end of the list
  ("tops outside the discography") and stay removable: that is where a
  generated card gets reviewed.
- **The tail cache** gains the date, the position, the length and the type
  (album/single); an older harvest is silently redone — the cache is
  regenerable and outside the repository.

Ten more tests (deduplicating reissues, matching tops by normalized title,
opposite gestures cancelling out, a cursor surviving a fold, and a full
rendering of the modal). **Not verified in a real session.**

## The footer never grows, the gesture shows in the list (2026-09-07)

Joel, after an `e3`: "it only added one track; I'd like to remove the
notification of the action: the footer never grows, and the gesture is
verifiable — you see what was added through a special icon in front of the
track, ↻ in place of →. An action with no visible effect in the list (a ban,
a card edit, a playback error) must still show; that notification needs a
bit more formatting."

- **What shows is no longer said**: taking a branch (it appears with its
  reason), an encore (its tracks carry **`↻`** in yellow in place of `→` —
  `Stop::encore`), removing from the queue, pause and resume (the footer's
  glyph). Nine `say!` fall away.
- **What does not show is shown better**: the "last thing said" line is
  formatted by nature, by the glyph that opens it, like the design system's
  Notice component — `✓` green, `⏹` `⊘` red, `↻` `⚑` yellow, `→` magenta, a
  parenthesis in grey, "not wired yet" in dimmed italics; the detail after
  the "—" or in final parentheses is dimmed. Tested.
- **The encore that only adds one track.** Two possible causes, both
  handled. Harvesting the tail was decided on the card's number of tops
  minus a global ceiling, not on **this artist's unplayed** tops — fixed.
  And when the tail was missing (no Spotify identifier, an unreachable API,
  a tail closed at the cocoon or exhausted), the encore served what it could
  **without saying so** — the footer's single line being immediately
  overwritten by "↻ encore…". Now `harvest` returns its result instead of
  speaking, and the encore only speaks **when** it does not serve the
  request: "(only 1 at X — their tail is exhausted)". `:warm` says
  `✓ X's discography — n titles cached` or `⏹ …`.

## The input hints (2026-09-07)

Joel: "I want to change how the shortcuts window works: you open it with
space and close it with escape; if I open it and press e, it shows e's
shortcuts window, and I must be able to go back; if I then press 3, it fires
the intended action — it is no longer a plain shortcuts window, but input
hints."

That is which-key for real. The leader was no longer **emptying** anything:
it erased the sequence in progress to show a menu, and everything had to be
typed again. Now:

- **space** opens the hints on the current level (everything, or the
  half-typed namespace) **and leaves the sequence in progress**; space at
  the entry level closes it again, **escape** closes it from anywhere.
- **Every key pressed in the hints goes through the grammar**: `e` takes the
  hints down to e's level (the session follows `Cmd::Pending`), `3`
  completes `e3` — the command goes out and the hints close.
- **⌫ goes one level up**: the key reader erases the sequence's last key and
  says so; outside the hints, it is simply erasing.

Each level's footer is a reminder: "one key = the action · ⌫ back · escape
close". The entry level's rows say "press f for its keys" instead of "space
for the detail".

On the code side, `Live::help_open` is the only state added; a block laid
over the screen fell on the next gesture, and the hints are the exception
until the sequence completes. Not verified in a real session.

## The playback screen after mockup 2b (2026-09-07)

Joel, with `Lecture.dc.html` enriched with two header and footer variants:
"rework the playback screen after mockup **2b**. In the end we drop the
vertical rule. As for the branch column, the presentation we have suits me,
do not change it. Just put the reasons in grey."

2b: "the seed as a block — and a full-width progress bar". The artist
journey disappears from the top (the played list already says it); in its
place, a header line and the seed's block; at the bottom, what is playing
and what follows, then the prompt. The screen now fits like this:

    forkstify ecouter the-cure     segment 2 · 6 tracks · 3 upcoming · comfort 3 ███░░ balanced

    ── seed ────────────────
    The Cure  [catalogue]  card written · 41 links · 12 tops        last played -3s
    1 fork point since — 2 artists crossed
          ♪ A Forest — The Cure  seed: the-cure          1 play · yesterday │ ── branches 2
     1 ▶  ♪ Cities in Dust — Siouxsie and the Banshees        never played │   1  The Creatures
     2 →  ♪ Israel — Siouxsie and the Banshees                never played │      shared members — …
                                                                           │      ♪ Right Now — The Creatures
     3    horizon  nothing drawn beyond — 1-3 to add a branch              │      █████ shared members
    ▶ Cities in Dust — Siouxsie and the Banshees  (2 / 3)                  ♪ top │ 2:34 / 3:47 -1:13
    ████████████████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
    up next  Israel — Siouxsie and the Banshees                  → fork point in 1 track
    (the last thing said)
    [1-3 branch · h/l · p · space = the keys · q] █

- **The header on one line**: the command on the left, the state on the
  right — segment, tracks, upcoming, and the comfort gauge, which leaves the
  prompt (it inverts there when `c` sets it).
- **The seed as a block**, under its rule: the name, `[catalogue]`, what its
  card says (written or generated, links, tops — `Card` now reads
  `generated`), its last play according to `learned/`, and the count of fork
  points taken and artists crossed.
- **The list with no rule**: the morning's half-rule falls, the tracks
  follow each other; the number goes grey, only the arrow keeps its color; a
  blank before the horizon.
- **The footer**: what is playing with its position in the list and its
  provenance spelled out (`♪ top`, `♥ liked`, `↳ door`, `· tail`… —
  `Source::word()`), then what follows and "fork point in n tracks". The
  last thing said keeps its line, and the prompt stays last with a block
  cursor.
- **The branch column does not change**, except the word under the gauge
  (`shared members`, `0.78`) which goes grey like the reason: only the
  gauge's cells still say the nature of the link.

**The progress bar, in the end** (Joel: "can we really not have a progress
bar?"). We can: librespot gives the position on every start, pause and seek
(`Playing`, `Paused`, `Seeked`, `PositionCorrection`) and the length on
every track change (`TrackChanged`). `Live::follow_needle` follows them —
for the current request only, since a skipped track still talks — and **a
tick every second** redraws while it plays, with the position extrapolated
from the last sample. The footer gains `♪ top │ 2:34 / 3:47 -1:13` and a
**full-width bar** in cyan, like waybar's media module. With no data yet
(before the first `Playing`), the bar is empty and the times absent.

**Not done, for want of data**: "1 door set aside by the comfort" — the
engine does not count what it sets aside.
## The playlist on the chain's grid (2026-09-07)

Joel, with the `File d'attente.dc.html` mockup: "can we present the playlist
a bit like mockup **3a**? It was an old mockup from the aborted queue
project, but I like the graphics and I want to get them back for the
playlist." Then, on a first attempt that made one link per branch: "I'd like
to take back the idea of numbering in front of the current and upcoming
tracks, and the separation with the vertical dash. If we can find some
interesting little stats or info to put on the side in grey like on the
mockup, that would be nice."

3a drew "the chain" on a four-column grid: the number and the arrow, the
name, the reason in grey, a piece of info on the right; a `│` rule between
the lines; the horizon at the end. Queue mode fell on 09-06, but the grid
suits what the axis has become. **Every track is one row there**:

          ♪ A Forest — The Cure  seed: the-cure              1 play · yesterday
     ╵
          ♪ Push — The Cure                                        never played
     ╵
     1 ▶  ♪ Cities in Dust — Siouxsie and the Banshees   links…    3 plays · -2s
     ╵
     2 →  ♪ Israel — Siouxsie and the Banshees                     never played
     ╵
     3 →  ♪ Right Now — The Creatures  shared members              never played
     ╵
     4    horizon  nothing drawn beyond — 1-3 to add a branch

- **What is playing is 1, and what comes counts from it** — `▶` (or `⏸`) in
  green on the current one, `→` in magenta on what follows. What has played
  is not numbered and is dimmed.
- ~~A rule between each track~~ — tried as `│` then as a half-dash `╵`,
  removed that same evening along with mockup 2b.
- **The branch's reason reads in grey beside the track that opens it** (in
  cyan when it comes from the vectors), and the seed beside the first one. A
  branch's head therefore now carries its reason along with its name
  (`engine::Head { label, reason }`); `tx` on a head passes them to the next
  track.
- **On the right, what listening knows of the track**: `3 plays · -2s`,
  `never played`, `skipped 2×`, `off-catalog` — read from `learned/`
  (`track_stats`, counters decayed to today), with the age written as in
  home's collection. Those are the only per-track stats the application
  holds: no length (the API does not return it), no set-aside branches (the
  engine does not count them).
- **The horizon** replaces "nothing more to follow". The header says
  "n tracks · k upcoming".
- **Nothing changes in the gestures**: the ↑↓ selection, enter and `tx` act
  on the same rows as before.

The columns tighten by themselves: the reason takes what is left between the
track and the note, and disappears below eight cells. At 100 columns (59 for
the axis), the note fits, the reason rarely; at 160, everything reads.
Verified by a rendering test at 160 columns; not yet run in a real session.

## The branches unfold in their column (2026-09-07)

Joel, with the `Lecture.dc.html` mockup (Claude Design) in hand: "put the
forks in a column on the right like on mockup **1a** instead. But I want
each branch's tracks to be shown so I can choose knowing what I'm getting."

The 09-06 panel had 1a's permanence but **1b's placement** — a framed block,
laid at the bottom of its column. It becomes the column itself: **the whole
height**, a `│` rule on the left to separate it from the axis (the only rule
it draws — the system forbids frames), `── branches 3` at the head, and the
keys at the foot (`1-3 take · fr propose again`, `fn1 without waiting for
the end`), which do not move.

**Each branch is unfolded** as the design system's `Branch` component writes
it: the number and the name, the reason folded to five cells, then **its
tracks** — they are already drawn at the moment of the proposal, and there
was no reason to hide them — with their provenance (`♪` `♥` `↳` `·`), the
title in front, the artist behind in blue. Below, the mockup's **proximity
gauge**: `█████ shared members` for a graph link (blue, catalog),
`████░ 0.78` for the adventurous branch (cyan, vectors) — the weight the
engine already carries, on the 1–5 scale. "Stay in the journey's universe"
is said once only, by the reason; the gauge stays silent.

A **60/40 split, the same as home** (Joel, 2026-09-07) — the `LEFT_SHARE`
constant serves both screens, and below 60 columns the right disappears on
both. The mockup gave the right 38 fixed columns; proportionally, a
100-column terminal gives it 40, a 160-column one gives it 64 and long
tracks no longer get cut. A list gets cut, a reason gets folded — the fold
is done by hand, ratatui does not touch it.

**Set aside, for want of data**: the mockup's "↳ 1 door set aside by the
comfort" line. The engine does not count what it sets aside; inventing it
would be lying.

**Verified by three rendering tests** (`TestBackend`, 100×30 and 70×30) —
the column, its tracks, its gauges, its keys at the foot; and a narrow
terminal that keeps the axis. Not yet run in a real session.

## The catalog speaks English, and imports (2026-09-06)

**Renaming.** `AGENTS.md` requires English for "everything that is a public
interface of the repository — paths, subfolders", and
[0010](decisions/0010-revised-format-links-without-doors.md) says the same
of the format. Yet the paths had stayed in French; only `learned/` (0014)
and the cards' fields followed the rule. Fixed: `fiches/` → **`cards/`**
(the code already calls that a `Card`), `outillage/` → **`tools/`**,
`vecteurs/` → **`vectors/`**, `catalogue.toml` → **`catalog.toml`**.

The **decisions stay untouched**: they mention the old names and are
immutable. A note at the head of [`keybindings.md`](keybindings.md) says to
read the new ones there, as 0014 did for `usage/` → `learned/`.

**`forkstify import <url>`.** Taking the cards of another catalog: it adds
the remote, fetches, takes **the cards we do not have** — never the ones we
do, their corrections to our artists belonging in a PR where they get
discussed — commits the lot in one go, then regenerates the vectors.

It is a **subcommand, not a listening gesture**: vectorizing needs a
container and several minutes. If docker is missing, the exact command is
shown instead of failing silently — with no up-to-date vectors, the cards
taken in would only exist for the graph.

## The first install, an open question (2026-09-06)

Joel: "what happens when a new user installs forkstify for the first time?"
and above all "how do we reconcile a common base catalog with the user's
changes?".

The second is **already settled**, but scattered across 0002, 0004, 0008 and
0014: we do not reconcile, we **stack in the same repository** and git does
the work — the base comes from upstream, mine is my commits on top, and the
learned layer lives in `learned/` and is never contributed back. Gathered in
[`first-run.md`](design/first-run.md).

What is missing is the **bootstrap**: nothing clones the catalog, nothing
scans the user's library, nothing generates a card on the fly. And a hard
point that was written nowhere: **the current base is not neutral, it is
Joel's universe** — 214 cards born of his ranking. A new user with distant
taste could barely start anything, since a seed with no card does not start.

## The whole collection, to the right of home (2026-09-06)

A request from Joel, after the column added to `Accueil.dc.html`: on the
left what forkstify **proposes**, on the right what it **holds**. "The
column proposes nothing: it lists."

The split is **proportional, 60/40** (Joel, 2026-09-06): the left carries
reasons and tracks, the right a list. One constant to change to go
half-and-half (`LEFT_SHARE`). Below 60 columns the list disappears — better
one readable column than two unreadable ones.

`gg` and `G` jump to the two ends, as in vim — in home's collection, and in
the axis while listening, since it is the same gesture on the same kind of
list. `g` alone is nothing: it waits for its second, and `keys.rs`'s
exhaustive test verifies the grammar stays prefix-free.

It brings the catalog **and** the ranking together — 780 names, 214 of them
with a card — each with a familiarity gauge, their name, and how long since
they last played (`today`, `yesterday`, `-3w`, `-7m`, `never`, in orange
past six months). `s` cycles the order: familiarity, a-z, last played.

**Two departures from the mockup, both deliberate.**

The mockup enters the column with `c` — but `c` has been the comfort setting
since the day before. Rather than moving either one, **there is no key to
enter**: the ↑↓ arrows move a cursor there, and `enter` starts the artist
under it. That is exactly the session's gesture (select then enter), and
with no cursor `enter` keeps its usual meaning — "choose for me". One less
key to learn.

The mockup says that "an artist with no card starts anyway: the first branch
comes from the vectors". **That is not true here**: the engine starts from
the card (links, tags, vector), and an artist in the ranking alone has none.
The column lists them — it is indeed the whole collection — but choosing
them answers that the catalog grows with use, rather than failing without
saying so.

## The edits finally write into the cards (2026-09-06)

`src/edit.rs` closes the last third of
[0013](decisions/0013-keyboard-tuning-measure-or-edit.md): four of the five
edits change a card **and produce a readable commit** (`tt`, `tT`, `td`,
`aL`). The commit message and what shows on screen are **the same
sentence** — what the user reads is what git will keep.

**The cards are patched textually, never rewritten.** A round trip through
serde would lose everything the code does not model — `format`, `generated`,
`mbid`, `spotify`, `begin`, `origin`, `description`, the order of the keys,
the quotes chosen by hand. A card is a **public interface**
([0002](decisions/0002-shared-forkable-catalog.md)) that a human reads and
fixes: we insert a line into it, we do not regenerate the whole. Six tests
cover what would silently corrupt, including two that verify that after
every insertion **the card still reads back as a card** — a quoted title
included.

Choices made for want of an input method: `td` takes as its direction the
tags of the **next artist** in the queue (0011: a door points at tags, where
you are going), and `aL` links to the artist **you came from**. Both read
back and get fixed in the card.

**`ae` remains the fifth**, for an architectural reason: opening `$EDITOR`
means giving the input back to the terminal, and the key reader holds
`stdin` permanently and would steal its keystrokes. It shows the card's path
while awaiting a prompted input rather than a blocked thread.

**A limit stated on screen**: the in-memory catalog does not move, so an
edit only counts for the engine at the next launch.

## Home brought closer to its mockup (2026-09-06)

Graphic touch-ups asked for by Joel, all taken from `Accueil.dc.html`:

- **The wordmark and the state on a single line** — `forkstify` on the left,
  `✓ librespot · ✓ api web` on the right. For want of cell justification,
  the gap is computed from the area's width.
- **The rules run to the end of the measure** (66 columns at most). They are
  what separates the blocks, since the system forbids cards.
- **The title in front, the artist behind**, everywhere: under an entry
  (`♪ Crystal Frontier — Calexico`), on a seed that is a track, and on the
  "resume" line — which still had them the other way round.
- **The colors are roles**: the number and the keys in magenta (branch), the
  artist in blue (the catalog, written by a human), the `♪` in green, the
  reasons in grey, the punctuation dimmed.
## The queue chains up: the journey becomes a playlist (2026-09-06)

Joel, after living with the TUI: "I'd like choosing a new branch to be added
to what was decided before. Right now it replaces it. That way you can build
a playlist quickly with a few branch choices."

It is a change of model, and a more accurate one: **choosing a branch
appends it to the queue** instead of replacing it, and the next branches are
proposed from the **end** of the queue, not from what is playing. It is the
"chain" of the queue mockups, obtained with no separate mode.

Consequences:

- **There is no "pending" branch any more.** Everything decided is in the
  queue — `pending_branch` goes away, and `next()` reduces to "move on,
  otherwise draw". `fn<n>` and `f!<n>` keep their meaning: insert after the
  current track, or with the rest dropped.
- **Every branch opens with its name in the queue**: the first of its tracks
  carries the label (`Stop::head`), the following ones a magenta rule. So
  you see several branches stacked, each one delimited.
- **The whole past stays on screen** — it is the playlist being made, not a
  history to forget. The axis scrolls to follow what is playing.
- **`tx` removes from the queue** the selected track. It stays proposable:
  it is not a ban, it is a "not this evening". If it was a branch's head,
  the next one takes over its name.
- **No more line wrapping on the axis**: a list gets cut, it does not wrap.
  That was the reported bug — the column having narrowed with the branch
  panel, long labels were wrapping.

**Still open**: saving the playlist. Everything needed is there — the past,
the queue, the branch names — but where to write it and in what format is
not settled (a Spotify playlist? a catalog file?).

## The chosen branch unfolds into "up next" (2026-09-06)

Joel: "when I choose a branch, the journey shows below the upcoming songs
but on one line with no detail". It did indeed fit on a `→ label`, whereas
its tracks are **already drawn** at the moment of choosing — there was no
reason to hide them.

They now line up after the queue, separated from it by a **magenta rule**
(`│`) that replaces the indentation: you see at a glance where the branch
starts and how far it goes. They are dimmed, because they have not played
yet, and the head line says **when** the branch will take over — at the end
of the branch, at the end of the track, or with the rest dropped.

**An accepted limit**: the selection (↑↓) stops at the end of the queue. The
branch's tracks show but cannot be chosen yet — they are not in the queue
until the branch has taken over, and pretending otherwise would make a
highlight that lies.

## The branches are always on screen (2026-09-06)

A request from Joel: "I'd like the panel of upcoming branches to be shown
all the time". That is variant **1a** of the mockups — the permanent panel —
with 1b's placement, at the bottom right.

One nuance was added to the request: **the panel now has its own reserved
place** instead of being laid over the axis. A panel floating permanently
would hide the bottom of the queue forever; a reserved column hides nothing.
The axis takes what is left on the left, the panel 54 columns on the right —
and below 48 columns wide, it disappears rather than crush the axis.

Consequences: `fp` (*peek*) loses its purpose and says so instead of doing
nothing; a dead end shows in the panel instead of leaving it empty.

## Nothing prints under the TUI any more (2026-09-06)

Joel, at the next test: "when I press 2, the bottom still moves", with texts
overlapping ("▶ ♪ Vilaine — Odezenne forkstify (media keys active…)").

The cause was broader than startup. **Three families of prints were writing
into the alternate screen behind ratatui's back**, and it only redraws what
it believes has changed — hence the leftovers:

1. a session's four startup messages (learned layer, connection, MPRIS);
2. `spotify.rs`'s two authorization messages;
3. and above all **the key reader itself**: the echo of half-typed
   sequences, the line edited after `/` or `:`, the erasures.

The reader no longer says anything on screen: it **reports its state** —
`Cmd::Pending` for a sequence in progress, `Cmd::Typing` for a line,
`Cmd::Unknown` for a sequence with no purpose — and it is the TUI that shows
it, on the prompt line. So the bottom of the screen only moves to show **the
command in progress**, which was the exact request.

Starting a session now has its waiting screen, with its steps ticked as they
go (`Tui::splash`), and `Tui::clear()` starts from an empty screen on every
view change.

## First usage feedback on the TUI (2026-09-06)

Five pieces of feedback from Joel after the first real hands-on, all
applied.

- **The comfort scale is flipped**: **5 = cocoon, 0 = exploration**.
  "Comfort is what you know well." The repository was the odd one out:
  [0012](decisions/0012-track-rotation.md) §4 already wrote "high comfort: a
  tight draw on the tops", which now reads literally. Only
  `zone-de-confort.md` said the opposite, and a design note yields to use.
  Default default: 3.
- **The gauge is set from the keyboard**: `c` opens the setting, ↑↓ move,
  enter confirms, escape gives back the previous value. Nothing is applied
  before confirmation.
- **The bottom of the screen no longer moves.** The log grew downwards and
  became messy; it now fits on **one fixed line** (the last thing said), and
  what is long — the leader menu, `?` — is **laid over the screen** as a
  block instead of going down.
- **Navigation becomes vertical and deferred.** The axis shows vertically,
  so ↑↓ move a **selection** highlighted in yellow there; **enter** plays
  what is selected, escape cancels. With no selection, enter keeps its usual
  meaning — draw a branch. ←→ and `h`/`l` stay the gesture of immediate
  transport, like ⏮ ⏭.

**And a bug found by reading the screen**: a single real play *replaced* the
ranking, so playing your favorite artist once made them drop from 100 % to
13 % familiarity. The ranking and listening are now combined by the maximum
— listening can only add.

## The TUI, first version (2026-09-06)

`ratatui` enters the project (decision [0006](decisions/0006-rust.md), which
already named it). `src/tui.rs` draws the session; **input stays `keys.rs`'s**
— raw termios, a prefix-free grammar — because it is proven and ratatui does
not need to own the input.

**Variant 1b of the mockups**: a full-width column for the playback axis,
and a panel that lays over it at a fork point then goes away (`Clear` +
`Block::bordered`). Moving to 1a — the permanent panel — will only change a
`Layout`; that was left open on purpose.

The screen fits in four areas: the header (the journey, the seed, the
segment, the comfort), **the axis** (what has played, what is playing
inverted, what follows), the **log** of what forkstify has just said, and
the prompt on the last line with the comfort gauge — the place
[`home-screen.md`](design/home-screen.md) gave it.

A consequence in the code: the session's **69 prints** became log lines
(`say!`), cleared on every command so that a block — the leader menu, `?` —
shows alone and in full. The log is a `RefCell`: saying something does not
require an exclusive borrow, which avoided making thirty methods mutable
that are not.

**The seam was closed the same day** (Joel saw it on the first launch: "when
I launch it, I get a plain terminal"). Home goes through the TUI too: **one
alternate screen for the whole application**, opened by `accueil()` and lent
to the session. Home decides *what* to say — a list of `Row` — and the TUI
*how*: the same separation as between the engine and the sound. The final
journey goes back up to home instead of printing onto a screen that
disappears.

**Not verified**: none of this has run in a real session. The TUI is
verified at compile time; home, though, does run.

## Usage feedback (2026-09-05)

The first long `ecouter` sessions produced **11 pieces of feedback** from
Joel, recorded and worked through in
[`docs/design/usage-feedback.md`](design/usage-feedback.md) — along with the
**real inventory of the shortcuts** (what works vs the projected table,
largely unimplemented) and **5 points to settle** before adding anything.
They are handled as they come.

## Next steps, in order

Read back on the evening of 2026-09-05, against the code. Steps 2 and 4 of
the previous list are largely done; what follows is what is left.

1. **Try `ecouter` for real.** Started on 2026-09-09: a long session from
   Joel, ten pieces of feedback handled the same day (the section above),
   and the first strengths in `docs/atouts.md`. Left to try: the
   measurements that write into the cards, the pool, `:warm`.
2. ~~**The five edits**~~ — `tt`/`tT` (tops), `td` (door), `aL` (link, and
   unlink since 2026-09-20), `ae` ($EDITOR, wired on 2026-09-20: the key
   reader parks itself). They all write and commit (`src/edit.rs`). That was
   the last third of
   [0013](decisions/0013-keyboard-tuning-measure-or-edit.md) and Joel's
   feedback no. 8.
3. ~~**The dated cooldown**~~ — done on 2026-09-08 (a tenth on the same day,
   a one-week half-life).
4. **`u` — undo the last gesture**
   ([0013](decisions/0013-keyboard-tuning-measure-or-edit.md)). Without it,
   a mistaken `tb` can only be taken back by hand in the TOML. It becomes
   necessary as soon as the edits arrive (reverting a commit).
5. **Queue mode** (`Q`, feedback no. 11). The biggest piece: `rounds` is a
   **flat list**, whereas "remove all of a branch's depth" assumes a tree
   you can manipulate.
6. ~~**Git synchronization**~~ — done on 2026-09-07 (0017).
7. ~~**Settle how a generated card is vectorized**~~ — settled on
   2026-09-09 ([0019](decisions/0019-the-application-vectorizes.md)): the
   application vectorizes itself, done the same day.
8. ~~**`fw` — leaving the universe**~~ (feedback no. 6) — settled and done
   on 2026-09-11: leave the cluster, with `fw <artist>` to aim at a
   universe.
13. ~~**The smooth setup**~~ (Joel, 2026-09-09) — done on 2026-09-20 after
    the `Installation.dc.html` mockup (the section above); the Python
    scripts step down after the first real `:library`.
14. **The Omarchy bar** (Joel, 2026-09-10): the `▂▄▆` animation in the bar,
    and on click a popover with title / artist / progress / next track. Two
    floors proposed in [omarchy-bar.md](design/omarchy-bar.md): forkstify
    first publishes real MPRIS metadata (portable), then a small Omarchy
    plugin shows them. **Done on 2026-09-10**
    ([0021](decisions/0021-the-repository-is-the-omarchy-plugin.md)): the
    repository is the plugin, the state lives under
    `~/.local/state/forkstify`, and the widget installs and launches the
    binary. Tried: the card's progress, frozen at 0:00, has been refreshing
    since 2026-09-11.
9. **The GNOME keyring** for the tokens, instead of the `target/` caches — a
   `cargo clean` today erases the authentication.
10. **The real TUI**: the screen does not redraw, everything scrolls.
   Key-by-key input is done, the display is still a terminal that scrolls.
11. ~~**Translate the `tools/` scripts into English**~~ — removed on
    2026-09-20: forkstify harvests, ranks, generates and vectorizes by
    itself.
15. **The three release workstreams** (Joel, 2026-09-19) — the workbook is
    in [design/before-release.md](design/before-release.md). ~~All three~~
    done on 2026-09-20. The same day, `tools/` removed from both catalog
    repositories and Joel's learned data removed from the reference (Joel,
    2026-09-20). Left to try for real: `:library`, `Cp`, `Cu` — the fork's
    first `Cu` will settle the `learned/` conflicts by itself (his are
    kept). The GitHub action and the `CONTRIBUTING.md` are in place the same
    day, and the application has been **public** since (README, LICENSE).
12. ~~**Explore an artist's discography**~~ — done on 2026-09-07 (`ad`,
    modal 1a). The target of `t`/`a`/`e` has been unified since 2026-09-09
    ([0020](decisions/0020-the-target-of-a-gesture.md)).

## Pending fixes (small)

- **Les Thugs**' MBID not found (a likely homonym) and an entry with an
  empty name in `learned/mbid.json`; 110 MBIDs resolved "by name" to review.
- **The edit commits speak French** ("Cat Power — tops : +2 −0", "Jacques
  Brel — fiche générée") whereas `AGENTS.md` wants English for the messages
  the application produces — `learned:` and `import:` are.
  `edit::Edit.summary` serves both as the sentence on screen and as the
  commit subject: separate them, or settle the rule.
- `resoudre-mbid.py` only reads the Spotify files — to be adapted to the
  Deezer harvests (`learned/amis/*-deezer.json`).
- **The MusicBrainz name is not always Spotify's** (La Ruda / La Ruda
  Salska) and resolving a track searches by name: an alias, checking the
  results' Spotify identifier, or the Spotify name at generation time — to
  be settled.

## Session rules

- **Commits and pushes as we go** on `forkstify` and `forkstify-catalog`
  (asked for by Joel on 2026-08-31). Chorizo: always ask before pushing.
- The scripts that read the GNOME keyring are run **by Joel** with the `!`
  prefix.

## A card born under the wrong Boo (2026-09-23)

Joel: the `boo` card is Boo! from South Africa, and he wanted the Czech
group, Spotify `75aF8TBGAxDZlcFPDEhIIK`. "How do we allow regenerating a
card that was generated wrong?"

Diagnosed, not coded. The generator identifies by **name alone** even when
the search or the collection had the Spotify id in hand, and "Boo!"
slugifies to `boo`. The right one is on MusicBrainz
(`a15ba7c3-e02e-440b-bc7e-cc60c328e34a`) with no Spotify or Deezer link,
Deezer does not have the band, and Spotify's top-tracks endpoint is marked
deprecated today: a regeneration by MBID would be thin, and needs the
Spotify id from Joel. The proposal — `:generate <name> <mbid> [spotify-id]`
regenerates an existing card, and the name search rejects a MusicBrainz
candidate whose Spotify link contradicts the id we hold — is in
`docs/design/on-the-fly-generation.md`.

**Joel took all three points the same evening, and they are wired.**
`generate::draft` takes the Spotify id: MusicBrainz is asked who it links
first, then the name search passes over a candidate whose link contradicts
it, accepting an unlinked one with a caveat. The search modal's hits carry
the artist's id, the collection reads it from the library by slug, the
setup's coverage step passes it too. `edit::regenerate_card` rewrites an
existing card and names who it was in the commit; the session takes that
path when `:generate` comes with an MBID over a card that exists, and says
"card regenerated (was Boo!)". A bare name over an existing card now spells
the form instead of stopping mute. Two ignored network tests pin Boo (passed
over) and The Cure (settled by link). What stays as it was: Deezer's tops
are still found by name, so a band Deezer lacks gets a namesake's tops
until edited.

**And the learned goes with the old artist** (Joel, the same evening: "drop
the learned in the same commit"). When the regeneration changes the MBID,
`Learned::forget` drops the slug's counters in memory and deletes its file,
and the edit's commit stages the removal with `git rm --cached
--ignore-unmatch`, which is silent when git never tracked the file. Under
the same MBID nothing is dropped. The toast and the commit body both say
it. Pinned by a test on whether the identity moved.

## The resolution holds to the card's Spotify id (2026-09-23)

Joel, the card redone: "Stones" plays the right song, "Listen" and "The
Answer" do not — "is it the cache to clean?"

Not the cache: it remembered what the resolution found, and it had found
**"Listen" by Snakes in the Boot and "Find the Answer Within" by The Boo
Radleys**. `resolve` searched `track:<title> artist:<name>` and took the
first studio hit without looking at whose it was; with a three-letter name
and generic titles, two out of three went to namesakes. The card's id was
never read at that point — the point left "to settle" since La Ruda on
2026-09-09.

`resolve` and `cached` now take the artist's Spotify id, read from the
stop's card. The hits are held to it when any carries it; otherwise the
name alone decides, so a stale id does not empty the answer. The id is in
the cache key: the addresses found by name alone are dead for an identified
card, nothing to clean by hand. The prefetch goes through the same door.
The choice is a pure function, `pick`, with three tests: the right artist
over the first hit, studio over live within that artist, and the fallback.

**Not enough for "The Answer"** (Joel, right after: "Listen" is right now,
"The Answer" still is not). Spotify's eight hits for `track:The Answer
artist:Boo` hold nothing by BOO at all, so the fallback on the name played
The Boo Radleys again. Holding the search to the id can only choose among
what the search returns.

**The artist's own discography is the authority now.** It is harvested by
the card's Spotify id, so a hit in it cannot be a namesake's. A top is
resolved from the tail first (`discography::find`: same title once the
version noise is off, the plain copy ahead of a live or a remaster, the
earliest release first — tested), then from the resolve cache. When an
identified card has no tail yet, `load_stop` **harvests it before
searching**: the track shows, the discography lands (two calls, cached for
good), and `Job::Harvested` resolves the waiting track from it — the title
search only if the tail does not name it. The prefetch fetches the tail
ahead by the same rule. A card without a Spotify id keeps the old path.
The cache needs no cleaning: the tail comes before it.
