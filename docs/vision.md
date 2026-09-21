# Vision

> **Take back the algorithm.**

Forkstify is a music player for Linux, wired to Spotify, that plays **by
branches**: you start from a track, the application plays a few, then
proposes several directions; you pick one — or let it pick — and so on.
Because you re-choose regularly, as the music and your mood move, you never
drop out.

## The philosophy

Streaming services recommend with an algorithm you cannot see, cannot
understand and cannot correct. Forkstify is not a smarter recommender: it is
a recommender **whose every reason you can read and change**. Its reasons
live in text files, in a repository you fork, improve and share — a **free
artist database**, built up over time by the people who use it.

This is not only about listeners. **Independent labels** suffer from the
opacity of Spotify's and other services' algorithms — right up to the
platforms pushing AI-generated content for the sake of margins, at the
expense of the artists they are supposed to help people discover. A free,
readable, forkable catalog, where a connection is visible, explained and
proposed, is an answer for them too: nobody needs to pay or to guess in
order to exist in the branches.

The rule that follows, applicable to every feature: **any automatic decision
must be explainable in one sentence and changeable in one commit.** A
proposed branch can say "because of the `filiation` link in your The Cure
card", or "because it is a neighbor in vector space, with no written
link" — and in that second case, offers to write it.

## The original pitch

> I'm thinking of building a music application on Linux, wired to my
> Spotify, where you pick a track to start with, and where it then works by
> branches. E.g. I select "The Cure – A Forest", it starts playing the
> track, and offers me 3 branches:
>
> - Branch 1: grind The Cure (more tracks by The Cure)
> - Branch 2: "The Cure, Cocteau Twins, New Order, …"
> - Branch 3: etc.
>
> With possibly a "comfort zone" choice or indicator.
>
> Once you have picked a branch, after a few tracks it offers several fork
> points again.

## Principles

1. **It never stops and it never demands a decision.** If the user does not
   choose, the application chooses for them according to their comfort zone.
   Taking back control is always possible, never mandatory.
2. **The catalog is the heart of the product; the branch engine brings it to
   life.** We supply a base — cards and their vectors — that you make your
   own, that you improve, and that **learns from use**. The interface and
   the audio playback are in service of that. Faced with a choice, we favor
   what makes the catalog more accurate and the branches more readable.
3. **The intelligence is in text files**, not in a service. The catalog can
   be shared, forked, corrected, read back. Spotify is the pipe: it serves
   to find and play the tracks, nothing more — and the database has to be
   able to outlive it.
4. **The name says the program**: *fork* — you fork the catalog to make it
   yours, and you fork a journey at every fork point.

## Vocabulary

The project's words. Use them as they are, in the code as in the
documentation.

| Term                | Meaning                                                                                |
|---------------------|----------------------------------------------------------------------------------------|
| **Seed**            | The starting track chosen by the user.                                                 |
| **Branch**          | A listening direction on offer: a readable title ("Siouxsie → Cult Hero → Joy Division") + a rule for selecting tracks. |
| **Fork point**      | The moment when the application proposes several branches and waits — or does not wait — for a choice. |
| **Segment**         | The few tracks played between two fork points.                                          |
| **Journey**         | The complete listening session: seed, sequence of chosen branches, tracks played.       |
| **Comfort zone**    | A dial from 0 to 5: how far you accept straying from what you know. See [decision 0001](decisions/0001-comfort-is-familiarity.md). |
| **Catalog**         | The whole set of artist cards (version-controlled text files) that feed the branch engine. |
| **Card**            | An artist's file in the catalog: identity (MBID), tags, tops, links, description.       |
| **Top**             | A track from an artist's default pool — what plays when you "grind" them.               |
| **Door** (`doors`)  | A targeted, occasional track that gets a bonus when you leave the artist towards the direction (tags) it points at. An additional criterion, never the main one. |
| **Link** (`links`)  | An explicit link between two cards: a closed type (`member`, `family`, `collab`, `scene`, `similar`, `influence`), a note explaining it, a proximity (by default per type, overridable). |
| **Base / mine / learned** | The three states of a piece of catalog knowledge: come from upstream; written or validated by me; produced by my own use. See [design/catalog.md](design/catalog.md). |
| **Promotion**       | Turning a usage signal (*learned*) into readable knowledge (*mine*): a commit on a card. |
| **Fork**            | Two meanings, never in the same context: **forking the catalog** (cloning it to make it yours, [0008](decisions/0008-the-fork-is-the-overlay.md)) — a rare gesture, the `:fork` command; and **forking the journey**, that is, taking a branch — a constant gesture, the `f` key (Joel, 2026-09-05, see [keybindings.md](keybindings.md)). |

Branch types identified so far (an open list):

- **Encore** — staying on the current artist. No longer a branch since
  2026-09-03: it is a gesture, the `e` key (the `:encore` command). "Poncer"
  — grinding an artist down — remains the project's French slang for it.
- **Neighborhood** — nearby artists (same scene, same era, same influences).
- **Sidestep** — a step to the side: same mood, another genre or another era.
- **Return** — coming back towards the comfort zone once you have strayed.
