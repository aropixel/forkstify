# 0029 — Soundtracks, several MBIDs in one discography, and `ad`

- **Date**: 2026-10-20
- **Status**: accepted · **Amends** the filter of
  [0027](0027-the-discography-is-catalog-material.md) §2 and the format of
  [0028](0028-the-discography-file-and-its-match.md) §1

## Context

Joel listens to many small artists, and 0028 §4 gives an artist with no
release kept by MusicBrainz no file — their tops only, where Spotify gave
`ad` a whole discography until now. Measured on 2026-10-20 on the 273
artists Joel played that have a card (`learned/artists/`), plus three Czech
bands he named (Boo, Traband, Tara Fuki):

- **No artist without a release**: 0 of 276. Six have singles only
  (Amixem, JOYCA, Mastu, Trei Degete, Maceo & the Macks, Hope Sandoval);
  Spotify has no more for the first four. The Czech bands are complete —
  Boo 2 albums, Traband 8, Tara Fuki 6 — and on Boo MusicBrainz is cleaner
  than Spotify, whose page carries a 2022 single by a namesake.
- **The large gaps are expected**: Queen 17 albums against 47 on Spotify,
  Blur 9 against 23 — Spotify counts compilations, lives and deluxe
  editions as albums, which 0027 leaves out on purpose.
- **Two real holes.** The filter drops every secondary type, and
  *Soundtrack* is one: Lionel Limiñana loses four of his five albums
  (*Thatcher's Not Dead*, *Heureux gagnants*, *Les règles de l'art*, *The
  Devil Inside Me*), The Limiñanas theirs. And MusicBrainz splits projects
  Spotify gathers on one page: Michel Cloup has 2 albums under his MBID,
  the six others (*Notre silence*, *Minuit dans tes bras*, *Ici et
  là-bas*…) are under **Michel Cloup Duo**, another MusicBrainz artist.

A soundtrack is a work, not a repackaging like a compilation or a live.
And the small artists are precisely the ones with such projects.

Separately, 0027 and 0028 do not say what `ad`
([artist-exploration.md](../design/artist-exploration.md)) does on a card
that has no `discography/` file yet — a fork that has not run the harvest,
a card written by hand.

## Decision

1. **Soundtracks are kept.** A release group whose primary type is album,
   EP or single is kept when it has no secondary type **or only
   `Soundtrack`**. Compilation, live, remix, DJ-mix, demo, interview,
   spoken word and the others stay out.
2. **A discography may gather several MBIDs.** An `artists` field, after
   `format`, lists the MusicBrainz artists whose release groups the file
   gathers — the card's own MBID first. Absent, it is the card's MBID
   alone. The harvest reads each of them and keeps a release group once,
   by its MBID.
3. **The harvest proposes the projects itself**: an artist's groups in
   MusicBrainz (the *member of band* relation) whose name begins with the
   artist's — *Michel Cloup Duo*, *Michel Cloup Trio*, *Hope Sandoval & the
   Warm Inventions* — are added to `artists`, and named in the commit's
   body so the pull request shows them
   ([0025](0025-the-review-is-the-pull-request.md)). A fork removes one,
   or adds another, by hand.
4. **`ad` on a card with no discography file harvests that artist** from
   MusicBrainz — the harvest command at the scale of one — writes the file
   and commits it with `Forkstify: discography`. The modal shows the tops
   at once, and the discography comes behind, as it does today.

## Consequences

```toml
format = 1
artists = [
  "835c8fb2-29e2-475c-8180-b8e79b76ef6b",  # Michel Cloup
  "671970fb-8949-4dc0-946f-2a3b157c206b",  # Michel Cloup Duo
]

[[release]]
mbid = "…"
title = "Notre silence"
…
```

- `artists` is written deterministically like the rest (0027 §5): the
  card's MBID first, then the others sorted.
- The Spotify match still reads the albums of the card's Spotify artist.
  Michel Cloup Duo's albums are on Michel Cloup's Spotify page, so they
  match; a project Spotify keeps on a page of its own does not play until
  the match learns to look there — to be measured when the match is wired.
- A project gathered in a discography is not a card: if it has one, its
  own file lists its own releases, and the two files may share some.
