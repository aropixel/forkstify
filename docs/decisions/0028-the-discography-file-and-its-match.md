# 0028 — The discography file, its harvest and its Spotify match

- **Date**: 2026-10-20
- **Status**: accepted · **Completes**
  [0027](0027-the-discography-is-catalog-material.md), whose "Left to
  settle" it answers

## Context

[0027](0027-the-discography-is-catalog-material.md) made the discography
catalog material — `discography/<slug>.toml`, MusicBrainz data only, the
Spotify match made and cached on each machine — and left six questions
open: the file's fields, the command that harvests it, the trailer of its
commits, an artist with nothing to write, the title matching rules, and
how long the local match lives. Joel settled them one by one on
2026-10-20.

One of the six was badly framed. "An artist MusicBrainz knows nothing
about" cannot happen: `mbid` is one of a card's three required fields
([0009](0009-mbid-identity.md)), and only artists with a card are
harvested or watched. What can happen is an artist MusicBrainz knows with
no release kept by 0027's filter — no one entered their records (a small
local band, a member generated in cascade for their relations), or only
compilations and lives are left.

## Decision

1. **The file's format.** `format = 1` at the head, then one `[[release]]`
   per release group, its fields in this order:
   - `mbid` — the release group's;
   - `title`;
   - `type` — `album`, `ep` or `single`;
   - `date` — the first release date, as precise as MusicBrainz has it
     (`1967`, `1967-03` or `1967-03-12`);
   - `spotify` — a Spotify album id, **only when MusicBrainz has the
     link**;
   - `tracks = [{ disc, n, title, mbid }]` — `disc` only on a release with
     more than one disc, absent otherwise; `n` the position on its disc.
2. **The harvest command is `forkstify discography [catalog]`**, beside
   `forkstify vectors`. It skips every card that already has its file, so
   running it again after an interruption resumes it, and it commits **by
   batches of 25 artists** — an interruption loses one batch at most, and
   the pull request to the reference reads batch by batch.
3. **Two trailers** ([0017](0017-syncing-the-learned.md)):
   `Forkstify: discography <version>` for a harvest,
   `Forkstify: release <version>` for a new release appended by the watch.
   A card generated on the fly that is born with its discography keeps
   `Forkstify: generate` ([0025](0025-the-review-is-the-pull-request.md)):
   the file comes in the card's commit.
4. **An artist with no release kept has no file.** No empty file, no
   Spotify fallback: the artist plays their tops, and the next harvest
   tries again, so the records someone enters in MusicBrainz come in on
   their own.
5. **The title matching rules**, MusicBrainz ↔ Spotify, on each machine:
   - **Normalization**: lowercase, accents and punctuation removed, `&`
     read as `and`; a parenthesis or a dash suffix that only names an
     edition is dropped — `Remaster*`, `Deluxe*`, `Expanded*`,
     `Anniversary*`, `Bonus*`, `Edition`.
   - **The date only breaks ties**: a normalized title unique in the
     artist's discography is enough, whatever the date — a 2012 remaster
     finds its 1967 album. Between two albums of the same title, the
     nearest date wins.
   - **Which edition**: the original when Spotify has it; a deluxe or a
     remaster only when the original is missing.
   - **Once the album is matched, the whole Spotify edition plays** — its
     tracks, bonus tracks and demos included, not only those of the
     MusicBrainz track list.
6. **The local match is kept 30 days** — one `[tuning]` number
   ([0023](0023-engine-numbers-are-tunable.md)) — then made again lazily,
   the next time the artist plays, never in bulk.

## Consequences

- An album played whole is the Spotify edition chosen by rule 5 — the
  original's order when it exists. The MusicBrainz track list says what
  the album is; Spotify says what plays.
- Bonus tracks and demos of a deluxe enter the tail when the deluxe is the
  only edition. They are drawn like any other track
  ([0012](0012-track-rotation.md)); `ts` lowers one that should not be
  there ([0018](0018-one-gesture-for-taste.md)).
- The thirty days answer the Terms' "do not store Spotify Content
  indefinitely" (IV.3.1.1) at a cost of a few calls per played artist per
  month; nothing is matched for an artist that is not played.
- `docs/tuning.md` gains the match's lifetime when it is wired.
