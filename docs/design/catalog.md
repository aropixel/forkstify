# Catalog

Working note. **Decided** = recorded in `docs/decisions/`; **direction** =
proposed, uncontradicted, not yet recorded; **to settle** = open question.

## Decided

- **Version-controlled text files, one per artist**, shared and forkable
  ([0002](../decisions/0002-shared-forkable-catalog.md)).
- **No separate overlay: the fork is the overlay.** The active catalog is a
  git clone, and personal changes are commits inside it
  ([0008](../decisions/0008-the-fork-is-the-overlay.md)).
- **TOML**, `format = 1` at the head of the card
  ([0007](../decisions/0007-cards-in-toml.md)).
- Every card carries **tops**
  ([0003](../decisions/0003-tracks-tops-and-doors.md)); and, occasionally,
  **`doors`** — tracks targeted as an exit towards a direction (tags), **an
  additional criterion, never the main one**
  ([0011](../decisions/0011-doors-an-additional-criterion.md)).
- **English fields, typed links on one line, default proximities in
  `catalog.toml`**
  ([0010](../decisions/0010-revised-format-links-without-doors.md)).
- **The format is a public interface**: stable, documented, editable by
  hand.
- **Two repositories**: the application on one side, the catalog on the
  other. **Importing = cloning** a catalog and declaring it active; one
  active at a time, switch whenever you like
  ([0004](../decisions/0004-two-repositories-targetable-catalog.md)).
- **Every card carries its format's version** (`format: 1`), not its
  content's ([0005](../decisions/0005-version-in-the-cards.md)).

## Directions

### A git repository

The catalog is a git repository: portable (cloning is enough), diffable,
version controlled, forkable in the literal sense. Making a card your own =
a commit. Contributing an improvement back = a PR.

### The shape of a card

Format 1, as applied to the first batch:

```toml
# cards/the-cure.toml
format = 1
generated = true          # goes away on human review
name = "The Cure"
mbid = "69ee3720-a7cb-4402-b48d-a02c366f2bcf"
spotify = "7bu3H8JO7d0UbMoVzbo70s"
begin = "1977"
origin = "Crawley"

tags = ["post-punk", "new-wave", "gothic", "uk", "80s"]

tops = [
  "Boys Don't Cry",
  "A Forest",
]

links = [
  { to = "siouxsie-and-the-banshees", type = "member", note = "Robert Smith played guitar there in 1983" },
  { to = "depeche-mode", type = "scene", note = "new wave, the synth side", proximity = 2 },
]

# optional: a track-choice bonus when heading in this direction
doors = [
  { track = "A Forest", to = ["post-punk", "atmospheric"], note = "the door towards the dark" },
]

description = """
Post-punk then dark pop, Crawley, since 1977. ...
"""
```

The *content* — descriptions, notes, tags — follows each catalog's own
language ([0010](../decisions/0010-revised-format-links-without-doors.md));
only the format's keys are fixed, and in English.

And at the root of the catalog, `catalog.toml` carries the identity and the
settings — including the type → proximity grid, which everyone adjusts in
their fork.

The format's semantics, clarified when reviewing the first batch:

- **A minimal card: `format`, `name`, `mbid`. Everything else is
  optional**, with graceful degradation: with no `spotify`, resolution goes
  through MusicBrainz; with no `tags` and no `links`, the artist floats in
  space; with no `tops`, the engine draws from usage. Prose (description,
  notes) completes the experience, never required.
- **`begin` / `end`**: free-precision strings (`"1977"`, `"1960-03-27"`),
  like MusicBrainz. The active period is the pair; no `end` = still active.
  The engine only reads the year.
- **`origin` is informational** (the human layer): the city crosses nothing.
  Geographic crossing goes through the tags (`fr`, `uk`, `belgique`…), at
  the right grain.
- **The order of the `links` means nothing.** Priority is `proximity` (per
  type in `catalog.toml`, with a local override) — no invisible semantics,
  no fragility on merge.
- **Resolving `proximity`, in cascade**: the value on the link if there is
  one; otherwise the `[proximity]` grid of the active catalog's
  `catalog.toml`; otherwise the defaults built into the application
  (identical to the reference catalog's grid). The normal case is to write
  nothing: the type is enough. Changing a value in the grid re-tunes every
  link of that type at once, with no local override.

Making a card your own = editing it and committing: my tops become
`["A Forest", "10:15 Saturday Night"]`, and `generated` goes away.

### We write for humans, the machine reads the structure

The description and the notes are **not** written "for the embedding" —
nobody knows how to do that, and it must not be asked for:

- **The vectorized text is composed by the application** from the structured
  fields: tags, origin, dates, link types and neighbors. The description, if
  it is good, adds nuance; empty or flat, the floor is guaranteed by the
  structure. Editing tags is within everyone's reach.
- **The prose keeps its human role**: the description says who the artist
  is, the note says why the link exists — it is the note that shows when a
  branch explains itself.
- **Feedback replaces effort**: `forkstify check` will show, for a card, its
  neighbors in the space ("The Cure is close to: Siouxsie, Joy Division…
  does that work for you?"). You do not judge your text, you judge its
  effects, and you adjust a tag or a proximity.

### What we ship: the cards and their vectors

- **The TOML cards are the source.** What a human writes, reads, fixes,
  forks: description, tags, tops, doors, links, Spotify identifier.
- **The vectors are an index derived from the cards, shipped with the
  base.** Derived, because it can always be rebuilt from the cards. Shipped,
  because computing an embedding needs a model (~100 MB) and above all
  because everyone has to have *the same* vectors for "close" to mean the
  same thing everywhere. The repository therefore holds the vectors and, in
  its metadata, the name and version of the model that produced them. The
  application recomputes a changed card's vector locally; upstream
  regenerates everything on every model change. **Done on 2026-09-09**
  ([0019](../decisions/0019-the-application-vectorizes.md)): `embed.rs`
  vectorizes inside the application — a generated card is born with its
  vector, `forkstify vectors` regenerates the index and has replaced
  `vectoriser.py` (truncation at 128 tokens, normalized vectors).
  Prototyped on 2026-09-02 (`tools/vectoriser.py` and `voisins.py`): model
  `paraphrase-multilingual-MiniLM-L12-v2` (384 dimensions, mean pooling,
  supported by fastembed in Python as in Rust), index in
  `vectors/vectors.jsonl` + `meta.toml`. The composed text names the
  neighbors of **outgoing and incoming** links (a relation holds both ways,
  only `influence` flips into "influenced").
- **Outside the repository**: Spotify API caches (title → identifier
  resolution, cover art), tokens.

### What the catalog learns

The catalog **modifies itself with use**. We do not pre-fill the whole of
Spotify: the catalog covers the user's universe and grows with their
listening. Signals and effects:

| Signal | What it says | What it changes |
|---|---|---|
| Choosing a branch at a fork point | this direction speaks to me | the weight of the link taken |
| Track skipped | not that one, not now | the weight of the top, or of the artist in this context |
| Track played in full, often | it is one of my tops | the order of the tops, an offer to make it a top |
| A branch born of the vectors (with no link) that pleases | the link deserves to exist | **writing a link** into the card |
| Arriving at an artist with no card | they are part of my universe | **generating a card** (Spotify, Last.fm, LLM), flagged as generated until reviewed |

The last two rows are the central mechanism: **the implicit space feeds the
explicit graph**. What the vectors guess and listening confirms becomes a
readable link, in a text file, readable back and contributable upstream.

### Base, mine, learned — and what is shareable

If the fork modifies itself with use and we contribute it back upstream, we
pollute the base with our own listening. Knowledge (shareable: a link, a
description, a consensual top) has to be separated from usage (personal:
weights, counters, dates). Three states, all in the fork, all in git:

1. **The base** — what comes from upstream.
2. **Mine** — what I explicitly wrote or validated in the cards. That is
   what I can propose upstream.
3. **The learned** — what usage produced, in a separate folder
   (`learned/`), version controlled so as to be portable, which the
   application knows never to include in a PR.

### Not all of "mine" is equally shareable (2026-09-06)

Joel's question: "if I change artists' links, would that be part of the PR?
Would that be a problem?". Yes to the first, **not necessarily** to the
second — and the answer lies in the closed list of types of
[0010](../decisions/0010-revised-format-links-without-doors.md), which
splits in two by itself:

| Type | What it is | Upstream |
|---|---|---|
| `member` `collab` `family` | **facts** — who played where, with whom | valuable to everybody |
| `scene` `influence` | a critical reading, debatable but arguable | defensible, with its note |
| `similar` | **a matching of taste** | the one that pulls the base towards one ear |

`aL` writes `similar`: that is honest — you bring two artists together
because they go well together *for you* — but it means the least shareable
type is the easiest to produce.

**What already protects us**: a PR is a **proposal**, reviewed upstream.
Nothing is contributed back automatically, and the author chooses which
commits to propose. The git model is enough to prevent drift; this is not a
flaw.

**What is really missing** is the means to *sort*. Nothing said why a line
existed, so neither the author nor the reviewer could tell a verified fact
from a one-evening matching. Since 2026-09-06, a line written during a
listening session carries its **provenance and its date** in its note —
`note = "linked while listening, 2026-09-06"` — which 0010 already asked for
in substance: "the type names and explains the branch".

### An overlay by duplication: the need is sound, the mechanism already exists

Joel's proposal (2026-09-06): rather than a fork where everything mixes,
**duplicate the card** — a changed card is copied into a personal space, you
put your doors and links there, and the application **adds the two together,
with the override winning**. "We would no longer be on a pure fork, but it
would be cleaner, no?"

The need behind it is real and poorly served today: **knowing what is
yours**. Nothing in forkstify shows it.

But duplication would pay dearly for it:

- **It freezes the card on the day of the copy.** Upstream fixes an MBID,
  adds a top, rewrites a description: the personal copy never sees it. You
  would then have to merge field by field — that is, reimplement git, worse.
- **It requires a second format.** Either the copy is a whole card, and
  changing one link freezes all the rest; or it is a patch format, to be
  designed, documented, versioned. Yet
  [0002](../decisions/0002-shared-forkable-catalog.md) makes the card format
  a **public interface**: there would be two of them.
- **It pushes away the very debate Joel says he wants.** Proposing upstream
  would require extracting the line from the overlay to carry it into the
  base card — friction exactly where we want fluidity.

**And above all: the overlay already exists, it is simply invisible.** In a
fork, what is yours is **your commits** — `git diff upstream/main` returns
them, card by card and line by line. It is an overlay *computed* rather than
*stored*: nothing to merge, nothing to freeze, nothing to version twice.
What duplication would bring in "clean", git already gives; what is missing
is for forkstify to **show** it.

**Proposal, to be settled**: keep
[0008](../decisions/0008-the-fork-is-the-overlay.md) and add the means to
see your own layer — a `:mine` command listing what differs from upstream,
card by card, and where each line comes from (provenance now being in the
notes). Cost: a few lines around `git diff`.

If duplication is preferred nonetheless, it needs a **decision superseding
0008**, and it will have to settle: the granularity of the copy, the patch
format, the behavior when upstream changes the original card, and the
contribution path.

### Taking part of somebody else's catalog

Joel's question (2026-09-06): "if somebody has a big jazz catalog, I'd be
interested in taking it — and yet I might not be interested in all their
commits".

**That is a case the format already serves**, with nothing to invent:
[0010](../decisions/0010-revised-format-links-without-doors.md) requires
`cards/` to be **flat, one card per artist**. Taking "their jazz" is
therefore not taking *commits* but **files** — and git knows how:

    git remote add someone git@github.com:someone/forkstify-catalog.git
    git fetch someone
    # see what they have that I do not
    git diff --stat HEAD someone/main -- cards/
    # take only what we want, file by file
    git checkout someone/main -- cards/john-coltrane.toml cards/alice-coltrane.toml
    git commit -m "import someone's jazz"

One commit of your own, chosen, that says what it does. None of their
commits comes in — you take the **state** of their cards, not their history.

To find them by family rather than one by one, the `tags` field is what
serves: list the cards in their repository whose `tags` contain `jazz`, then
pass them to `git checkout`. A script in `tools/` would do that in a few
lines.

**Two properties of the engine make a partial import safe**, and that
matters since we are taking a piece of a whole:

- **A dangling link is ignored, not fatal.** `graph_neighbors` looks for a
  `link`'s target with `.get()`: a link towards a card we do not have simply
  sits out. So you can take ten cards out of a set of a hundred without
  breaking anything.
- **A card with no vector breaks nothing either** — it is merely invisible
  to the adventurous branch, since `vector_neighbors` only works on what
  `vectors.jsonl` holds. **An import therefore regenerates the index within
  its commit**
  ([0019](../decisions/0019-the-application-vectorizes.md)); by hand,
  `forkstify vectors`.

That tolerance is no accident: it comes from "one file per artist" and from
the fact that the graph and the vector space are two independent paths to
the same artist.

**Still open**: do we need a **personal** link, one that never goes into a
PR? Two ways, neither settled — a field on the line (`personal = true`), or
nothing at all, with upstream review doing the sorting. The second is more
sober and consistent with "no abstraction before the second use"; the first
says the thing rather than relying on vigilance.

Between the last two states sits **promotion**: turning a usage signal into
knowledge. "You took the Cocteau Twins branch from The Cure 6 times, shall I
add the `neighborhood` link to the card?" A promotion = a readable commit.
That is what keeps the catalog from becoming a black box: everything it
learned on its own is a diff you can read back and revert.

### The shape of the learned layer (recorded 2026-09-04, [0014](../decisions/0014-shape-of-the-learned.md))

How the *learned* layer (the `learned/` folder) stores what listening
teaches, and what the engine reads from it. Vocabulary on disk is in
**English**, like the card format (the repository aims at open source).
Three principles that follow from the rest:

- **One file per artist**, `learned/artists/<slug>.toml`, mirroring the
  cards. Like one card per artist, it diffs cleanly, it **never** creates a
  conflict with upstream (everyone has their own), and it scales. A single
  big file would grow without end and break on every merge.
- **Counters that decay by themselves over time.** Rather than keeping the
  history of every play, we keep **a decayed count**: on every play,
  `plays = plays × ½^((now − last)/half-life) + 1`, and `last = now`. One
  float and one date per artist (and per top), and a play from three years
  ago barely weighs at all — the decay question is settled by construction.
  Default half-life: **6 months**, adjustable.
- **Silent, never contributed back.** The learned layer changes without
  asking (it is measurement), and the application never includes it in a PR.

The shape of a file:

```toml
# learned/artists/the-cure.toml
plays = 12.4          # plays, decayed over time (familiarity)
last  = "2026-09-04"  # last play (recency + cooldown)
weight = 0.8          # the "-" correction (less often); 1.0 = neutral
blacklisted = false   # "X" on the whole artist

[tops."A Forest"]
plays = 5.0
last  = "2026-09-04"
liked = true          # "a"
skipped = 2           # accumulated "x"

[tops."Killing an Arab"]
blacklisted = true    # "X" on this track
```

**Marks** (the `m` key, to be sorted later) live apart, across artists:
`learned/marks/<name>.toml` (a list of `{artist, title, at}`).

**What each key of [0013](../decisions/0013-keyboard-tuning-measure-or-edit.md)
writes** — measurements into `learned/artists/`, edits into the card (a
commit):

| Key | Effect | Where |
|---|---|---|
| full play (auto) | `plays += 1`, `last` | learned (measurement) |
| `x` skip | `tops.<t>.skipped += 1` | learned (measurement) |
| `X` set aside | `blacklisted = true` (top or artist) | learned (measurement) |
| `a` like | `tops.<t>.liked = true` (+ Spotify liked track) | learned (measurement) |
| `-` less often | `weight ×= 0.7` (floor) | learned (measurement) |
| `m` mark | a line in `learned/marks/` | learned (measurement) |
| `t`/`T` top | adds/removes from the `tops` | **card (commit)** |
| `d` door | adds a `door` | **card (commit)** |
| `E` edit | opens the card in `$EDITOR` | **card (commit)** |

**What the engine reads** from the learned layer, on top of the card:

- **exclusion** of the `blacklisted` (artist and top);
- **familiarity** = decayed `plays` (+ the `learned/classement.json`
  bootstrap for an artist with no learned data yet) → feeds the **comfort
  zone** (0001) and the adventurous branch's thresholds;
- **cooldown** (0012): a recent `last` lowers the weight / suspends, so that
  what we have just listened to rotates (a session's "without replacement"
  stays in memory);
- **weight**: `weight` multiplies the artist's branch weight; a top often
  `skipped` moves back in the segment, a `liked` top moves forward.

`classement.json` (the bootstrap, 741 artists scored from the library)
therefore becomes **the starting familiarity**; `learned/artists/` extends
and corrects it as listening goes on.

> Vocabulary: the "learned" layer is the **`learned/`** folder (renamed from
> `usage/` on 2026-09-04 — no French term in the paths or the format). The
> bootstrap files in `learned/` still named in French (`amis/`,
> `artistes-*.json`) and their keys will be translated along with the
> `tools/` scripts.

Direction: the *learned* layer changes on its own, silently (it is
measurement); *promotion* is proposed by default and can be made automatic
by a setting — the application never demands a decision, but it makes its
own visible.

### A free artist database

- **An artist's identity is their MBID**
  ([0009](../decisions/0009-mbid-identity.md)); Spotify is one
  implementation among others, and Deezer or others will come without
  touching the catalog.
- **The reference catalog is offered for download by the application**: on
  first run, forkstify offers to clone it, and everyone builds their own
  version from there
  ([0004](../decisions/0004-two-repositories-targetable-catalog.md),
  [0008](../decisions/0008-the-fork-is-the-overlay.md)).
- **"Complete" is built by use, not by a dump.** We do not pull down the
  whole of MusicBrainz. Every card is born because somebody arrived at that
  artist — library, journey, other people's PRs. The catalog is complete *in
  the sense of its users*.
- **Two visible quality levels**: *generated* and *reviewed*. A card
  reviewed by a human is worth more, and the engine can know it.
- **The facts and the meaning do not come from the same place.** The facts
  (name, MBID, identifiers, country, years, tags, linked artists) come from
  free and verifiable sources — MusicBrainz, Wikidata, Last.fm for the
  similars — never from a language model: no invented facts in a free
  database. Meaning (description, typed and annotated links, doors) is where
  a model helps, and where human review counts.

### The cold start and the available base

A new user must have nothing to write:

1. **On first run, the application offers to clone the reference catalog**
   (already recorded). Nobody starts from zero.
2. **Then the personal import** (their Spotify or their Deezer — the
   `tools/` scripts are the prototype): artists already in the base cost
   nothing (their signals calibrate the comfort zone, in `learned/`); the
   missing ones go through the generation pipeline.

**The generation pipeline** — the same sources verified on 2026-08-31:

| Field | Source |
|---|---|
| identity, dates, origin, tags | MusicBrainz / Wikidata |
| tops | Deezer `/artist/top` (no key) + the user's liked tracks |
| `member` / `collab` / `family` links | MusicBrainz relations (typed, factual) |
| `similar` links | Deezer `/artist/related` (no key), Last.fm as backup |
| `scene` links | cross-referencing era + country + genres |
| notes, description | a language model, or absent (everything is optional) |

Everything is flagged `generated`. "Everything optional" and "we write for
humans" make automatic generation *sufficient* for a usable product, and
review *an improvement* rather than an obligation.

**One pipeline, three moments**: seeding the reference, importing on first
run, generating while listening (a living catalog).

**The available base is worked on three workstreams:**

1. **The reviewed core** — in progress: Joel's library (741 scored artists,
   30 cards written), his friends, batch 2 (the cards called in by the
   links).
2. **Seeding** — the pipeline in batch over the most played artists (Last.fm
   / ListenBrainz charts), to cover any newcomer's library on day 1.
3. **Pooling** — when a user's application generates a card missing from the
   reference, it offers to contribute it upstream (a pre-chewed PR). Every
   cold start enriches the commons.

## To settle

- **Following upstream.** Joel's scenario: "I downloaded the initial
  catalog, I evolved it to my taste; six months later, the initial catalog
  has doubled in volume and quality — how do I benefit?" That is a git merge
  of upstream into the fork, and the structure helps: one card per artist
  (new cards arrive with no conflict at all), the learned layer in
  `learned/` (never in conflict with upstream), "mine" concentrated on the
  cards I touched. Real conflicts are therefore limited to cards changed on
  both sides — and there, a `forkstify catalogue sync` command must guide
  field by field ("upstream enriched The Cure's description, you changed the
  tops: shall I take both?"). To be designed seriously when the time comes;
  in the other direction, make the PR easy for contributing a card back.
- **Import ergonomics**: `forkstify catalogue add <url>`,
  `forkstify catalogue use <name>`, `forkstify catalogue list`? And where
  the clones live (`~/.local/share/forkstify/catalogues/<name>`?).
- **Track identity**: by title (readable, ambiguous — live versions,
  remasters) or by Spotify identifier (precise, unreadable)? Probably the
  title in the card, resolved into an identifier at play time, with caching.
- **Tag vocabulary**: free, with a recommended list to publish? (Link types
  have been closed since
  [0010](../decisions/0010-revised-format-links-without-doors.md).)
- **File name**: the slug (`the-cure.toml`) is also the key of the links
  (`to = "the-cure"`). A slugification rule to settle (accents, articles,
  homonyms).
- **Automatic promotion or with confirmation**: the default setting, and the
  granularity (per promotion type?).
- **The shape of the learned layer**: proposed on 2026-09-04 (see the "shape
  of the learned layer" direction above — one file per artist, decayed
  counters). Still to settle along the PoC: the decay **half-life** (6
  months by default), the **cooldown window** (0012), and the familiarity →
  comfort zone 0–5 formula (0001).
- **Flagging generated cards**: a field (`generated = true`), a separate
  folder, or both?
- **The shared catalog's licence**: a data licence (ODbL like
  OpenStreetMap, or CC BY-SA), and checking the compatibility of the sources
  poured in — MusicBrainz and Wikidata yes, Last.fm blurrier.
- **The language model at runtime** (the card of an unknown artist): which
  provider, which key, which offline fallback?
- **Seeding**: how many artists (a thousand? five thousand?), which chart
  source, and Last.fm requires an API key — Deezer does not.
- **The next concrete step**: prototype the generator in `tools/` and run it
  on the 61 cards called in by batch 1's links — it builds batch 2 and
  validates the cold start.
