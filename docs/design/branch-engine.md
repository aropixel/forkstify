# Branch engine

Working note. **Decided** = recorded in `docs/decisions/`; **direction** =
proposed, uncontradicted, not yet recorded; **to settle** = open question.

## Decided

- The engine reasons about **artists**; tracks are chosen within the artist
  through the tops and the usage signals
  ([0003](../decisions/0003-tracks-tops-and-doors.md),
  [0010](../decisions/0010-revised-format-links-without-doors.md)).
- Its raw material is the **catalog** of cards
  ([0002](../decisions/0002-shared-forkable-catalog.md)), not a
  recommendation API.
- **Repetition must never be imposed**: a weighted draw from a pool wider
  than the tops, a dated cooldown, no replacement within a journey, comfort
  = the depth of the draw ([0012](../decisions/0012-track-rotation.md)) —
  detail below.

## Directions

### Two complementary representations

1. **An explicit graph** — the *links* written in the cards, typed
   ("filiation", "same scene", "same producer"…) and annotated. That is what
   **names** the branches: a link has a readable label, a vector does not.
2. **An implicit vector space** — every artist has a vector, derived from
   their card (description, tags, links), possibly enriched with
   co-occurrences (Last.fm, playlists) and with the user's history. That is
   what **fills the holes** where nobody wrote a link, and what gives a
   continuous notion of distance.

Explicit links win wherever they exist; the vector space takes over
otherwise.

### Branches as geometric operations

- **Encore** = stay at the same point, draw other tops.
- **Neighborhood** = nearest neighbors within a radius *r*.
- **Sidestep** = move in a direction (same era other genre, same genre other
  era…).
- **Return** = move back towards the library's center of gravity.
- **Comfort zone** = distance to the center of gravity of what the user
  knows.
- A **journey** is a path through the space: you can avoid going back over
  it, or return to it deliberately.

### Vectors: three sources that stack, nothing to train

1. **An embedding of the text composed by the application** from the card's
   structured fields (tags, origin, dates, links), with the description as
   nuance. Nobody writes "for the embedding" — see
   [catalog.md](catalog.md). It captures "Orelsan is close to Casseurs
   Flowters, a little less to Stupeflip".
2. **Co-occurrence**: artists that show up together (Last.fm similars,
   playlists, tags). It captures the "these go together" that text misses.
3. **The user's history**: what they actually chain. Small in volume, very
   relevant, and it is what personalizes the space.

The vectors are a **regenerable cache**, never a source of truth.

### Choosing the track within the artist

The engine chooses among the tops and the history according to the comfort
zone — a familiar track when you are reassuring yourself, a less played one
when you explore. If the card has a **door** whose tags overlap the
direction taken, that track gets a bonus — an additional criterion, never
the main one ([0011](../decisions/0011-doors-an-additional-criterion.md)).

### Rotation: not suffering repetition

Tops exist to be replayed; repetition is not a bug, it just must never be
**imposed**. Four mechanisms that stack, each explainable in one sentence
(recorded on 2026-09-01,
[0012](../decisions/0012-track-rotation.md)):

1. **A top is a weight, not a closed list.** An artist's pool adds up: the
   tops (heavy weight), the user's liked tracks for that artist
   (`learned/`), the doors, and the rest of the known discography (a cache
   of the extended Deezer/Spotify top, outside the catalog). The engine
   **draws at weighted random** from that pool, it does not take the first
   of the list.
   **Implemented on 2026-09-05** (`engine::reservoir`) for the **first
   three** sources: tops 1.0, liked 0.8, doors 0.4 with a ×2.5 when the
   branch's direction overlaps their tags. The fourth — the long tail —
   awaits the API cache. Every track shown carries its provenance
   (`♪ ♥ ↳ + ~`).
2. **Freshness (cooldown)** — wired on 2026-09-08: a tenth of the weight on
   the day the track played, recovered with a half-life of one week
   (`Learned::freshness`). Every play is dated in `learned/`; a recently
   played track is penalized, and the penalty decays over time. "Already
   played on Tuesday, I'll let it rest."
3. **No replacement within a journey.** Never the same track twice in one
   journey; **encore** draws without replacement, so the second encore
   mechanically goes down towards the less known — two encores become a
   gesture of exploring the artist.
4. **The comfort zone sets the depth of the draw.** High comfort: a tight
   draw on the tops (repetition is *chosen*); low comfort: the long tail
   weighs more. That is the role the dial already has in
   [0001](../decisions/0001-comfort-is-familiarity.md) — no second setting.

## To settle

- The length of a segment: fixed (3 tracks?), variable by branch type, or by
  comfort?
- Which embedding model, local or remote? Constraint: everything is
  dockerized, and offline use is desirable for the catalog side.
- How we **name** a branch that comes from the vector space and not from an
  explicit link: by the dominant tags of the neighbors? by an LLM?
- Do we need named directions in the space ("older", "more electronic") and
  how would they be built?
- Co-occurrence sources: Last.fm is the obvious candidate; check the state of
  its API and its terms of use.
- Rotation: how fast the cooldown decays, and where the extended discography
  lives (an API cache, outside the catalog) — which joins the "shape of the
  learned layer" question in [catalog.md](catalog.md).
