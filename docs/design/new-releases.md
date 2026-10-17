# New releases

Note opened on **2026-10-16** on a request from Joel: see the new albums,
be told about them — the liked artists with the highest score first, but
the others too — play an album that just came out, and, if possible, be
proposed relevant new artists.

Nothing is coded. **Decided** = Joel's call; **direction** = proposed,
uncontradicted; **to settle** = open question.

## Decided on 2026-10-17 (Joel)

The five open questions of the first version, settled one by one:

1. **Banned no, lowered yes.** An artist banned with `ab` ("never again") is
   not watched at all. An artist lowered with `as` is still watched, and
   simply ranks lower, since the list is ordered by score already.
2. **"Seen" is deduced, plus a dismiss.** A release leaves the list when one
   of its tracks has been played (the counters of `learned/` already know)
   or when it is older than a new release's age (below). A key also
   dismisses it ("not interested"): the dismissed releases are written in
   `learned/<artist>` and merged **by union**, which means extending the
   git merge driver ([0017](../decisions/0017-syncing-the-learned.md)).
3. **Spotify watches, MusicBrainz announces.** The watch goes through
   `/artists/{id}/albums`, the call `discography()` already makes: 212 of the
   214 cards carry a verified Spotify id and none a Deezer one, and a
   release found there is playable as is. MusicBrainz is asked for the
   announcements (singles, future dates), for the top of the ranking only.
   **To verify before coding**: that the endpoint returns Joel's three
   examples, and what its rate limit allows in development mode.
4. **A card only if you go there.** The watch only looks at artists that
   have a card — the card carries the Spotify id. A neighbour with no card
   is not watched; its card is generated when you play or open it, as
   [0016](../decisions/0016-broad-base-and-on-the-fly-generation.md) says.
   No commits for candidates you did not choose.
5. **One age, albums and EPs only.** A single `[tuning]` number, the age of
   a new release (6 weeks by default), is both the list's window and the
   length of the freshness boost, which decays to zero at its end. Singles
   show in the list but get no boost in the branches — a lone track is not
   repeated for six weeks.

## The sources — tested on 2026-10-16

Joel's three examples, found that day by searching by hand: Fontaines D.C.
*Dopamine Chamber*, Eels *Cookie Happened*, Drake *HABIBTI (FOMO)*.

| Source | Call | Result |
|---|---|---|
| **Deezer**, public Simple API, no key | `/artist/{id}/albums` | ✅ all three, dated (`2026-10-16`, `2026-10-16`, `2026-10-02`), with `record_type`. The newest is not always first: sort by date. |
| **MusicBrainz**, no key | `release-group?query=arid:{mbid} AND firstreleasedate:[…]` | ✅ all three, **and the singles that announce them**: Fontaines D.C. *Marianne* (08-18), *Tongue* (09-30), *Where Is Gone?* (10-12) before the album on 10-15; Eels three singles from June. Rate limit at 1 req/s, 503 beyond. |
| **Deezer** editorial | `/editorial/{0,85,116,152}/releases` | ❌ `total: 0` for every genre tried — dead. |
| **Spotify** | `GET /browse/new-releases` | Removed for development-mode apps since February 2026 ([migration guide](https://developer.spotify.com/documentation/web-api/tutorials/february-2026-migration-guide)). Not tested with ncspot's client id (extended mode). Global and editorial anyway — not what Joel asks for. |
| **Spotify** | `/artists/{id}/albums` | Already used by `discography()` (`src/spotify.rs`); gives the album's tracks with their `uri`. |

**Conclusion: there is no "new releases for me" feed to consume; we build
it, per artist.** That is the project's line anyway — the list comes from
*your* catalog and *your* scores, not from an editorial.

Drake's case shows the noise to expect: *HABIBTI (FOMO)* is an EP for
MusicBrainz, an album for Deezer, and a re-issue of *HABIBTI* (2026-05-15);
Fontaines D.C. have a *Romance (Deluxe Edition)*. Editions have to be
recognised and ranked below a real new album.

## Direction — a watch, per artist, ranked by score

- **Who is watched**: the ranked artists (`learned`), by score band. The
  top (say score ≥ 5, ~90 artists) checked every day, the rest every week,
  the tail never unless a card links to it. One Deezer call per artist:
  Deezer allows 50 requests / 5 s, so the whole library is about a minute,
  in the background, at start.
- **What is kept**: album / EP / single, date, title, the service ids.
  A cache outside the repository, like the discography — a fact that can
  be re-harvested is not catalog material. Only "seen" is user state.
- **Order of the list**: score first, then the type (album > EP > single >
  edition), then the date. Every row carries its reason: "*Eels — new album
  · you like 14 of their tracks*".
- **Announced, not yet out**: MusicBrainz singles a few weeks before an
  album, and sometimes a future date. "Album announced" is a row of its own.
- **Where it shows**: a block on the home screen
  ([home-screen.md](home-screen.md) — "every block is a reason"), a
  `:releases` command, and a count in the Omarchy bar or a notification at
  start. Never a sound, never a popup mid-listening.

## Direction — playing an album

A new album is the one case where **order matters more than the branch**.
Two gestures:

- **Play it whole**, in track order, then hand back to the branches from
  that artist. The tracks are already reachable: `discography()` returns
  them with album, number and `uri`; the `ad` modal already groups them
  by album.
- **Let it in through the engine**: the album's tracks join the artist's
  tail with a **freshness** boost that decays over a few weeks — a
  `[tuning]` number ([0023](../decisions/0023-engine-numbers-are-tunable.md)).
  Then a branch that crosses Eels this month plays *Cookie Happened* more
  than *Electro-Shock Blues*, and it is said in the reason.

## Direction — discovering artists

What the data already allows, from the closest to the furthest:

1. **A member's new project.** MusicBrainz's member-of relations are already
   in the cards: a band or solo record by a member of a band you love is the
   most legitimate discovery there is.
2. **A release in your neighbourhood.** Watch, beyond the library, the
   artists **linked** to your top (typed links, proximity ≥ some level).
   "*Fontaines D.C. release an album — close to The Fall and Joy Division,
   that you play a lot.*" A new release is the natural time to meet someone.
3. **The near unknowns**: cards with familiarity 0, ranked by their summed
   proximity to your liked artists (graph first, vectors to fill in) — a
   list version of what the adventurous branch does one at a time.
4. **The agent's `:suggest`** ([agent.md](agent.md)), aimed at a liked
   artist: it proposes, you accept, and the catalog keeps the link.

`audience` links ([0026](../decisions/0026-audience-what-the-services-report.md))
are a fair source for 2 and 3 only if the reason says so: "*Deezer listeners
also play…*", never "*similar to…*".

## To settle

- The dismiss key: which one, in the grammar of
  [0015](../decisions/0015-keyboard-grammar-namespaces.md)?
- The shape of the boost's decay (linear, exponential) and its height.
