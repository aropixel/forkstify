# What changed

What each version brings, for whoever uses forkstify. The working log —
why a thing was done, what was measured, what was left open — lives in
[`docs/progress.md`](docs/progress.md); this file only says what you would
notice.

Versions follow the release: a tag `v*` publishes the Linux binary that
`omarchy/install.sh` and the Arch package fetch, and **the section below
becomes that release's notes**. `bin/release` moves `## Unreleased` down to
the version it cuts, so this file cannot fall behind.

## Unreleased

### Added

- **A card can be regenerated.** `:generate <name> <mbid>` over an artist
  who already has a card rewrites it from the sources, in one commit that
  says who the card used to be. For a card born under the wrong artist of
  the same name — and what was learned about that wrong artist goes with
  it, in the same commit.
- **A Spotify id in `:generate`.** `:generate <name> <mbid> <spotify-id>`
  puts that id in the card, ahead of the one MusicBrainz carries.

### Fixed

- **`Cd` said "nothing beyond the reference" on a fork that had cards to
  show.** On a fresh clone the reference was never fetched, and the diff
  quietly compared the fork to itself. `Cd` now fetches `upstream` first,
  like `Cp` and `Cu`, and a fork never takes its own remote for the
  reference: offline and never fetched, it says so instead.
- **A track played by a namesake.** A card's tops were looked up on
  Spotify by title and artist name, and the first hit played whoever it was
  by. A top is now taken from the artist's own discography, fetched by the
  card's Spotify id, and the title search is only the last resort — where
  the card's id now decides among the hits.
- **A card generated under a namesake.** Generating from a Spotify result
  or from your collection identified the artist by name alone, so "Boo"
  could land on another Boo. The artist's Spotify id now travels with the
  request: MusicBrainz is asked who it links, and a namesake linked to
  another id is passed over.

### Fixed

- **Tracks that never played, one after another.** Spotify was asked
  without a market, so it answered with tracks that exist somewhere and
  play nowhere here: the search succeeded, librespot could not play, and
  forkstify moved on — to another track picked the same way. Every call now
  asks for your market, and a result Spotify itself marks unplayable is
  dropped. **Clear the caches once**, they were filled without a market:

  ```sh
  rm -f ~/.local/state/forkstify/resolve-cache.json
  rm -rf ~/.cache/forkstify/discography
  ```

- **A paused player restarting on its own.** Losing the output — a
  bluetooth speaker walking away — makes librespot stop the stream, which
  read as "the track ended". A paused player no longer moves on.

### Listening

- The collection's last-played column drops its leading minus: `2w`, `6d`,
  next to `today` and `yday`.

### Keys

- **The key helper opens on its own.** Type `t`, `a`, `f`, `e` or `C` and
  the keys of that namespace appear at once — no space needed. Space still
  opens the help of the beginning, with everything you can type. In every
  level, the keys come first, then the `/` and `:` lines.
- **The `:` commands have their own level in the helper.** Typing `:`
  lists them, and the list narrows as you type. The entry help shows one
  `:` row instead of a dozen.
- **Every `:` command works at the home too.** `:size`, `:warm`,
  `:discography`, `:wander`, `:catalog`, `:sync`, `:setup` and `:library`
  used to answer only while listening. At the home, `:warm` and
  `:discography` aim at the highlighted artist.
- **One key helper for both screens.** The entry help is the same table
  at the home and while listening, and each screen shows only the keys
  that mean something on it. `f`, `e`, `J`/`K` stay with the listening
  screen, which shows the branches and the queue; `s`, `v`, `r` stay with
  the home.
- **The track keys and `h`/`l` work at the home.** `tl`, `ts`, `ta`,
  `ti`… act on what plays underneath, as `tg` already did; `h`/`l` and the
  arrows move along the journey playing underneath.
- **`x` removes, wherever you are.** It was `tx`; removing is not a verb of
  the track alone. While listening it takes the highlighted track out of the
  queue — still proposable, it is not a ban. At the home it takes the
  highlighted artist out of the liked, without the weight `as` also moves.
- **`ac` — a connection of your own.** Some artists go together because of
  one ear and one life, not because anything links them. `ac` draws that
  connection, asks how close — `h`/`l` move it a notch, a digit jumps, 1
  the farthest, 5 the closest, and the question stays until answered — and
  writes it into `learned/`: it follows you between machines, and `Cp` can
  never carry it upstream. The branches follow it immediately, both ways.
  The modal lists what is drawn on either side; ← → on one move its
  closeness right there, enter reopens its question, and `x` there undraws
  it. And
  **`:connections`** lists every connection drawn, by artist, in one
  block.
- **`aL` is retired.** Writing a link into a card is rarer, and shared once
  proposed: it goes through `ae`, which opens the card in `$EDITOR`.
- **`tg`** — the track and its artist in the browser, as `ag` does for an
  artist. At the home too, on what sounds underneath.
- **`u` and `.` leave the grammar.** Undo and repeat are not being built for
  now, and a helper that lists them promises what the code does not do. `fu`
  still steps back one branch, and `u` keeps undoing a pending edit inside
  the discography.

### Listening

- **The evening drifts, and the branches follow.** "Stay within the
  journey's universe" took the plain average of every artist played, so a
  long evening was still represented by where it began. It now weighs what
  was played lately more heavily — `universe_half_life` in
  [`docs/tuning.md`](docs/tuning.md), in artists.
- **The heart shows up where the like is made.** Liking a track changes its
  glyph in the list, and `ta` opens on that glyph with its word. A door
  keeps its arrow: that says where it leads, not how it was picked.

### Installing

- **No Docker needed.** Each release carries a Linux x86_64 binary;
  `omarchy/install.sh` fetches it and checks it against the published
  digest, falling back to a container build only if it cannot. Docker is
  now only for building from source.
- **An Arch package**, in `packaging/aur/forkstify-bin` — `makepkg -si` from
  a clone installs it today; the AUR itself waits on registration reopening.
- Adding the plugin is **not** installing forkstify: the bar card's
  "Install" is what puts the binary in place. The README says so now.

### For the catalog

- **`:catalog shell`** opens a terminal in the catalog fork, for what the
  gestures do not cover — a `git log`, a card written by hand.
- **The index no longer churns.** Regenerating the vectors rewrote the whole
  file every time, because the embedding is not reproducible to the last
  digit. Each line now carries the fingerprint of the text it came from, and
  only the cards whose text moved go back through the model.

## 0.1.0 — 2026-09-21

First public release: a music player for Linux that plays **by branches**.
It starts from a seed, plays a few tracks, then offers several directions;
you pick one, or let it pick. What it knows of artists lives in a catalog
of TOML cards you fork, edit and propose back, and what it learns of your
listening lives beside it, yours alone.

In this one: the seven-step setup on first launch, the listening screen and
its branches, the comfort dial, the keyboard grammar of four namespaces,
the discography screen, on-the-fly card generation, the catalog namespace
(`Cd`, `Cp`, `Cu`), synchronisation between machines, and the Omarchy bar
widget with its card.
