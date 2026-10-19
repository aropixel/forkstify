# 0027 — The discography is catalog material

- **Date**: 2026-10-19
- **Status**: accepted · **Supersedes** the consequence of
  [0012](0012-track-rotation.md) "the extended discography lives in an API
  cache, never in the catalog"

## Context

The long tail ([0012](0012-track-rotation.md) §1) is harvested from Spotify
by `discography()` (`src/spotify.rs`), on every installation, into a local
cache. Joel, 2026-10-19: the calls are very long and the discographies
often come back incomplete, on big ones (The Velvet Underground, The
Beatles) but not only. The code explains it:

- only the **first page** of `/artists/{id}/albums` is read — 50 releases,
  albums first, so a big discography loses its tail and its singles;
- the tracks come in batches of twenty albums, and the **first batch that
  fails ends the harvest**;
- the shared client id is rate-limited hard: on 2026-10-17, ten artists
  took 225 s with eight 429s, about one call every 20 s
  ([new-releases.md](../design/new-releases.md)).

And what an artist has released does not change: only new releases are
added. Harvesting it again on every installation is paying, for every
user, for a fact that is the same for all of them — the very thing the
catalog is for ([0002](0002-shared-forkable-catalog.md)).

For new releases, Joel the same day: the project is a **Spotify Premium**
project, and leaning on Deezer's free API would make it depend on the free
service of a paid platform no one here subscribes to. Deezer is set aside
for this; MusicBrainz, already the identity of an artist
([0009](0009-mbid-identity.md)), is the source.

Measured on 2026-10-19, MusicBrainz release groups with no secondary type:
Eels 16 albums, 5 EPs, 41 singles; Fontaines D.C. 5, 4, 25; The Velvet
Underground 16 albums out of 127 release groups (the rest compilations and
lives). MusicBrainz rarely links them to Spotify: a Spotify album link on
2 of Eels' 33 release groups seen, 22 of Fontaines D.C.'s 41, 1 of the
Velvet's 49.

What the catalog may carry was read the same day in Spotify's
[Developer Terms](https://developer.spotify.com/terms): "you may not store,
aggregate or create compilations or databases of Spotify Content, other
than as strictly necessary to operate your SDA", "Do not store Spotify
Content indefinitely" (IV.3.1.1), "do not transfer Spotify Content to third
parties" (IV.2.5), metadata included (II.8); local caching only "as strictly
necessary to enhance the performance" (IV.3.2). A public repository of
track lists and uris harvested from the API is such a database. MusicBrainz
data is in the public domain (CC0) and can be redistributed freely.

## Decision

1. **An artist's discography is a file of the catalog**:
   `discography/<slug>.toml`, beside `cards/`, the card's slug, one file per
   artist, in TOML ([0007](0007-cards-in-toml.md)) with `format = 1` at its
   head ([0005](0005-version-in-the-cards.md)). The card is untouched: it
   stays readable, editable by hand, and its diffs keep their meaning.
2. **MusicBrainz says what exists.** One `[[release]]` per release group
   whose primary type is album, EP or single, **with no secondary type** —
   no compilation, live or remix; a fork adds one by hand if it wants it.
   Its identity is the release group's MBID, its date the first release
   date, so a deluxe edition is the same release, not a second one.
3. **The file carries MusicBrainz data only.** Each release has its
   tracks — position, title, recording MBID — from one release of the group
   (the earliest official one), and a Spotify album link **only when
   MusicBrainz itself has one**. Nothing harvested from the Spotify API is
   written into the catalog.
4. **The match with Spotify is made on each machine, and cached there**: the
   artist's Spotify albums, **every page read**, matched by title and date,
   then their track uris — lazily, the first time an artist's tail or its
   discography is needed, resumable when a call fails. A release with no
   match exists and does not play. The cache is local and regenerable, as
   the Terms allow.
5. **The file is written deterministically** — releases by date, then
   title, a fixed field order — so two forks that add the same release
   write the same lines and git merges them.
6. **New releases are found through MusicBrainz**: one query per artist
   for the release groups after the last date of its file, at MusicBrainz's
   1 request per second. A new release is appended and committed in the
   fork, its Spotify match made locally when it is played; it reaches the
   reference like a card, through a pull request
   ([0025](0025-the-review-is-the-pull-request.md)).
7. **The discography stays out of the vectors**
   ([0019](0019-the-application-vectorizes.md)): the vector says what the
   card asserts, not a list of titles.

## Consequences

- The tail cache stays local but changes nature: it no longer says what an
  artist released — `discography/` does — only how each release plays on
  Spotify. Its harvest reads every page and resumes instead of stopping at
  the first failure.
- The Spotify calls per machine drop to the match: a few per artist
  (album pages, then tracks by twenty), made once, when needed, never for
  what is not played.
- The reference has to be harvested once from MusicBrainz: one or two
  calls for the release groups, one per kept release for its tracks — the
  Velvet's 25 or so in under half a minute. A card generated on the fly
  ([0016](0016-broad-base-and-on-the-fly-generation.md)) is born with its
  discography in the same commit, when the harvest succeeds.
- Deezer is not a source of new releases; Spotify's album endpoint is no
  longer a watch, only the local match.
- The `spotify` field of a card — one id per artist — is left as it is; it
  is not a compilation of the catalog's content.
- `docs/design/new-releases.md`: the source question is settled;
  `docs/design/long-tail.md` and `catalog.md`: the discography is no longer
  "outside the catalog", only its Spotify match is.
- Left to settle: the trailer of the commits that add releases, the
  command that harvests the reference, what the file says of an artist
  MusicBrainz knows nothing about, the title matching rules, and how long
  the local match is kept before it is checked again.
