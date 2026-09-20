# forkstify

> Take back the algorithm.

A music player for Linux, wired to Spotify, that plays **by branches**:
start from a track, let it play a few, then pick one of the directions it
proposes — or let it pick — and so on. You re-choose as the music and
your mood move, so you never drop out.

Streaming services recommend with an algorithm you cannot see, understand
or correct. forkstify is not a smarter recommender: it is one **whose
every reason you can read and change**. Its reasons live in text files —
one TOML card per artist: tags, top tracks, typed links to other artists
— in a git repository you fork, improve and share. The rule behind every
feature: *any automatic decision must be explainable in one sentence and
changeable in one commit.*

## What it does

- **Branches.** From what is playing, three directions with their reasons
  — shared members, a collaboration, the same scene, a similar sound —
  drawn from the catalog's links first, from a vector space when the
  links run out. `1`–`3` takes one, `⏎` lets it choose.
- **A comfort dial**, 5 cocoon → 0 exploration: how far it strays from
  what you know when it chooses for you.
- **It learns from what you play**, silently, into `learned/` — plays,
  likes, skips, bans — and never pushes that upstream.
- **A catalog that grows.** Arrive at an artist without a card and one is
  generated on the spot (MusicBrainz, then Deezer), with its vector, in a
  commit. Fix a top, draw a link: a commit too, readable, revertible.
- **A fork, not a copy.** Your catalog is a fork of the reference. `Cu`
  brings the reference in, `Cp` proposes your cards back as one pull
  request. What is yours is your commits.

The interface is a terminal one, driven by a vim-like grammar: `f` the
branch, `e` encore, `t` the track, `a` the artist, `C` the catalog; space
is the leader and shows what you can type. The whole table is in
[`docs/keybindings.md`](docs/keybindings.md).

## What it needs

- **Linux.** Built and used on [Omarchy](https://omarchy.org); anything
  with ALSA should do.
- **Spotify Premium.** forkstify is a Spotify Connect device (librespot)
  and reads your library through the Web API. It is not affiliated with
  Spotify, and it uses an unofficial client the way the free ecosystem
  does — see [`docs/conception/spotify.md`](docs/conception/spotify.md).
- **git**, and a GitHub account if you want your catalog to be a fork
  you can propose from (`gh` makes that one keystroke).
- **docker**, to build: there is no toolchain to install.

## Install

As an Omarchy plugin — the repository is one — the bar widget builds and
launches it:

```sh
omarchy plugin add https://github.com/aropixel/forkstify.git
```

Or by hand, from a clone:

```sh
bin/build                      # cargo build --release, in a container
ln -s "$PWD/target/release/forkstify" ~/.local/bin/forkstify
forkstify
```

The binary runs on the host and needs only `libasound.so.2` and
`libstdc++.so.6`.

## First launch

`forkstify` without a catalog opens the **setup**: seven steps, nothing to
type you do not already know, each one skippable and replayable later
(`:setup`, or `:library` for the library alone).

1. **The catalog** — paste the url of your fork of
   [forkstify-catalog](https://github.com/aropixel/forkstify-catalog),
   let `gh` fork it for you, or clone the reference in local mode.
2. **The git identity**, only if your machine has none.
3. **The connection** — the phone announces the device over zeroconf,
   the browser grants the Web API.
4. **The library** — liked tracks, liked albums, followed artists.
5. **The playlists** to count, ticked and remembered.
6. **The comfort** dial.
7. **The coverage** — generate the cards your most played artists lack.

Then home: your artists, liked first; `⏎` starts, `/` searches the
catalog and Spotify, space shows the keys.

## The catalog

One file per artist, `cards/<slug>.toml`:

```toml
format = 1
name = "The Cure"
mbid = "69ee3720-a7cb-4402-b48d-a02c366f2bcf"
tags = ["post-punk", "80s"]
tops = ["A Forest", "Lullaby"]
links = [
  { to = "siouxsie-and-the-banshees", type = "member", note = "Robert Smith, 1982–84" },
  { to = "joy-division", type = "scene" },
]
```

Everything but `format`, `name` and `mbid` is optional. Link types are a
closed list — `member`, `collab`, `similar`, `family`, `scene`,
`influence` — each with a proximity the engine reads. The reference
catalog's [CONTRIBUTING](https://github.com/aropixel/forkstify-catalog/blob/main/CONTRIBUTING.md)
says how a proposal is made and read.

## Configuration

`~/.config/forkstify/config.toml`, written with its defaults on the first
run: the catalog's path, the comfort to open with, and a `[tuning]`
section with **every number the engine reasons with** — what a like
weighs, how long a track steps back after playing, how far the
adventurous branch may leap. Each one is explained in
[`docs/reglages.md`](docs/reglages.md).

## The documentation

The code and the interface are in English. The design notes are in
French, and they are the project's memory: [`docs/vision.md`](docs/vision.md)
for what it is, [`docs/decisions/`](docs/decisions/) for every decision
taken (one file each, dated, never rewritten), [`docs/conception/`](docs/conception/)
for the living notes by subject, [`docs/avancement.md`](docs/avancement.md)
for where things stand.

## License

MIT — see [`LICENSE`](LICENSE). The catalog's own license is decided in
its repository.
