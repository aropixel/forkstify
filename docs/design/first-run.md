# The first install, and the life of the catalog

Note opened on **2026-09-06**, on a series of questions from Joel: what
happens when somebody else installs forkstify? should their Spotify be
scanned? should the missing cards be created? and above all — "how do we
reconcile a base catalog common to everyone with the user's own additions?".

The last one is already settled in the repository, but scattered across four
decisions. This note gathers it, then says what is really missing.

## Where the column's 780 names come from

The starting question: "there are more artists than I had on Spotify". Yes,
and that is correct:

| | |
|---|---|
| The ranking, drawn from the Spotify library | **741** |
| Cards in the catalog | **214** |
| Cards for artists **absent** from the ranking | **39** |
| **The column = the union of the two** | **780** |

Those 39 do not come from Spotify: they are artists the **catalog called in
itself**. The generator follows a card's `links` towards the artists it
names, so Cult Hero comes in because The Cure names them. The catalog is
therefore not a copy of the library: it is a **neighborhood** around it, and
it overflows on purpose.

## The model is already decided — three layers, one repository

Four decisions say it, one piece each:

- **[0002](../decisions/0002-shared-forkable-catalog.md)** — the catalog is
  a repository of text files, shared and **forkable**.
- **[0004](../decisions/0004-two-repositories-targetable-catalog.md)** —
  **importing = cloning**, then declaring "this is the one I use". You can
  have several and switch.
- **[0008](../decisions/0008-the-fork-is-the-overlay.md)** — "there is no
  separate overlay. The active catalog is a git clone, and personal changes
  are **commits inside it**".
- **[0014](../decisions/0014-shape-of-the-learned.md)** — usage lives apart,
  in `learned/`, version controlled but **never contributed back**.

Hence the answer to "how do we reconcile the base and my additions": **we do
not reconcile them, we stack them in the same repository, and git does the
work.**

| Layer | Where | Shareable? |
|---|---|---|
| **The base** | the cards as they come from upstream | it *is* upstream |
| **Mine** | my commits on those cards — `tt`, `td`, `aL`, `ae` | **yes**, that is what a PR proposes |
| **The learned** | `learned/` — counters, dates, weights, bans | **never** |

Updating the base is a `git pull` from upstream: my commits sit on top,
theirs underneath, and a conflict only happens if we both touched the same
line of the same card — which is rare, one file per artist being made
precisely for that. Contributing is a **PR** containing nothing but cards.
`learned/` never enters it, and the application knows that.

**Promotion** is the bridge between the last two layers: "you took the
Cocteau Twins branch from The Cure six times, shall I add the link?". A
usage signal becomes readable knowledge, and it is a commit you can read
back and revert.

## What is really missing

The model holds; it is its **bootstrap** that does not exist.

1. **Nothing clones the catalog.** The application reads
   `~/Work/forkstify-catalog` if it is there and fails otherwise. A new user
   has to clone by hand. `forkstify` should offer to import the reference
   catalog on first run — that is 0004, never written.
2. **Nothing scans the user's Spotify.** `classement.json` was produced by
   seven Python scripts in `tools/`, run by hand by Joel, with his session.
   For somebody else, that file does not exist: they would have **no
   starting familiarity**, so a home screen with no regulars and no
   neglected artists, and a comfort zone that leans towards nothing.
3. **Nothing generates a card on the fly.** `tools/generate-cards.py` knows
   how (MusicBrainz for the facts, Deezer for the tops and the similars),
   but it is a script outside the application. Yet
   [catalog.md](catalog.md) makes it a central mechanism: "arriving at an
   artist with no card → generating a card, flagged as generated until
   reviewed".

## The hard point, which is written nowhere

**The current base is not neutral: it is Joel's universe.** The 214 cards
were generated from his ranking and outwards from there. A user who loves
jazz or US rap would find a catalog that does not speak about them — and
since **a seed without a card cannot start**, they could barely start
anything.

Three ways out, to be settled:

- **(a) A neutral, broad base**, generated upstream over a few thousand
  common artists. Costly to produce, but the install works for everyone
  right away. That is what "reference catalog" implicitly assumes.
- **(b) A thin base, and on-the-fly generation becomes mandatory.**
  Everyone's catalog grows towards their universe from the first listen.
  Faithful to "the catalog covers the user's universe and grows with their
  listening", but it makes the application depend on the APIs at startup.
- **(c) Several bases, by family of taste**, imported to suit (0004 already
  allows it: "somebody else's because it looks cool"). The most faithful to
  the project's spirit, the heaviest to bootstrap.

Nothing forces a choice now — **as long as forkstify has one user, the
question does not arise**. But it decides what "reference catalog" means,
and therefore what goes into the first public repository.

## Settled: (a) **and** (b) — decision [0016](../decisions/0016-broad-base-and-on-the-fly-generation.md)

Joel's call, 2026-09-06. The reference base aims for **breadth**, and the
application **generates a card on the fly** when you arrive at an artist
that has none. The two complete each other: breadth makes the install work
right away, generation keeps it from ever staying foreign. (c) is not
dismissed — 0004 already allows importing somebody else's catalog, and no
decision is needed for that.

### A fork, not a repository of differences

A clarification Joel asked for: "does every user have their repo of
changes?". **No — they have a *fork*.**

    aropixel/forkstify-catalog         the reference, upstream
        └── kbyjoel/forkstify-catalog      their fork: the WHOLE catalog, plus their commits
                └── ~/…/forkstify-catalog      their local clone, the one the application reads

Their repository holds **the whole catalog**, not only their changes. That
is what allows both moves: `git pull` from upstream to receive other
people's cards, and a **PR** towards upstream to propose their own. A
repository holding only the differences could do neither — and would
contradict [0008](../decisions/0008-the-fork-is-the-overlay.md), "there is
no separate overlay".

`learned/` lives in that same fork, version controlled so as to be portable
from one machine to another, but **never enters a PR** (0014).

The active catalog's path is now a setting, `[catalogue] path`, with the
command-line argument overriding it.

Since 2026-09-08, that is exactly Joel's situation: the reference is under
the `aropixel` organization, his catalog is a fork in his `kbyjoel` account,
with upstream as `upstream`.

## To settle

1. ~~**(a), (b) or (c)**~~ — settled: (a) and (b), see 0016.
2. **Should the library scan go into the application** — or stay tooling run
   separately? It needs one more OAuth scope (`user-library-read` is already
   there; `user-follow-read` and `user-top-read` are not) and several
   hundred calls.
3. ~~**Generating a card on the fly**~~ — settled on 2026-09-09: **both**,
   the search to bring somebody new in and arrival to grow along the links.
   The subject now has its own note,
   [on-the-fly-generation.md](on-the-fly-generation.md); the **vectorizing**
   of a generated card is settled by
   [0019](../decisions/0019-the-application-vectorizes.md).
5. **A smooth setup on first run** (Joel, 2026-09-09, after the episode of
   the tracks liked by his son): today the Spotify library comes in through
   four Python scripts run by hand, which read the keyring, then
   `classement.py`. What is needed, at install time or on first trigger:
   **connecting** (the OAuth already exists in the application),
   **importing the library** — followed artists, liked albums, liked tracks
   — and **choosing the playlists** to count, ticked from a list. What that
   implies: rewriting the harvest in Rust on `WebApi` (which already knows
   how to paginate), a playlist modal on the discography's model, the
   ranking computed by the application, and the seed file in English
   vocabulary (0014, still pending). The `tools/` scripts then step down.
   **Settled by Joel on 2026-09-09: on first run *and* replayable** for
   re-harvesting; he will supply a Claude Design mockup when we take the
   feature on — we do not code before then. Still open: whether the ticked
   playlists are remembered in `learned/` (likely, so that re-harvesting is
   a single gesture). **The steps and the information to gather are proposed
   on 2026-09-19 in [before-release.md](before-release.md)** (workstream A),
   so that the mockup starts from a settled list. **Done on 2026-09-20**
   after the `Installation.dc.html` mockup: `src/setup.rs`,
   `src/library.rs`, `learned/library.toml`.
4. ~~**The import on first run**~~ — settled by the 2026-09-20 mockup: both,
   and a third one — the URL of your fork pasted in, the fork made by `gh`
   if it is there, or the reference cloned in local mode.

## Measuring usage across the forks

Since [0017](../decisions/0017-syncing-the-learned.md), every commit the
application produces — learned, edit, import — carries a git trailer
`Forkstify: <kind> <version>`. GitHub's commit search indexes the messages
of public repositories, which makes it possible to count those commits
across every fork without asking anyone anything:

```sh
# every commit produced by forkstify, across public repositories
gh api search/commits -f q='"Forkstify:"' --jq .total_count

# by kind: the learned layer, card edits, imports
gh api search/commits -f q='"Forkstify: learned"' --jq .total_count
gh api search/commits -f q='"Forkstify: edit"'    --jq .total_count
gh api search/commits -f q='"Forkstify: import"'  --jq .total_count

# the repositories concerned, one per line
gh api search/commits -f q='"Forkstify:"' --paginate \
  --jq '.items[].repository.full_name' | sort | uniq -c | sort -rn

# and the reference repository's fork count, which counts the users
gh api repos/aropixel/forkstify-catalog --jq .forks_count
```

Limits: only the **default branches** of **public** repositories are
indexed; a private fork is not counted; the trailer says forkstify wrote the
commit, not who. As long as the reference repository is private, these
commands only count what we push to it ourselves.
