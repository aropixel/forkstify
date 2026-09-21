# The shape of the application, and the PoC

Working note. **Decided** = recorded in `docs/decisions/`; **direction** =
proposed, uncontradicted, not yet recorded; **to settle** = open question.

## References

Starting points of Joel's thinking, and what we take from them:

- [stappmus/Omarchy-Spotify](https://github.com/stappmus/Omarchy-Spotify) —
  an Omarchy plugin (QML in the bar, JS for the API and the OAuth), sound
  through a local Rust backend or Omarchy's `spotifyd` package, driven
  through the Web API. ~60 MB instead of the official client's ~950 MB.
  **Taken**: the model of integration with Omarchy, and the fact that the
  sound is a local Spotify Connect device driven through the Web API.
- [crmne/fastpotify](https://github.com/crmne/fastpotify) — a full client in
  Rust (egui + librespot), OAuth PKCE, MPRIS, command-line commands
  (`fastpotify next`). **Taken**: librespot is viable; a full client is far
  too big for what we want.
- [ssp-data/neomd](https://github.com/ssp-data/neomd) — a TUI mail client in
  Go (Bubble Tea, Glamour, Lipgloss), vim shortcuts, text-file
  configuration, nothing stored locally. **Taken**: the form. Keyboard
  first, one screen, text, minimal. Joel has already written
  [`omarchy-neomd`](https://github.com/kbyjoel/omarchy-neomd), the bar
  widget that goes with it — the TUI + bar widget pair is a known path.

## Decided

- No mockup for now; **the concept and the PoC come before the interface**.
- **Rust**: `ratatui`, `rspotify`, `tokio`, `fastembed` for the vectors
  ([0006](../decisions/0006-rust.md)).
- **Keyboard tuning: every key is a measurement (`learned/`) or an edit (a
  commit)**, `u` undoes the last one, and every key is merely the shortcut
  of a `:` command
  ([0013](../decisions/0013-keyboard-tuning-measure-or-edit.md)). The key
  table below remains a direction.

## Directions

### Separate the brain from the sound — in the code, not in the binary

The branch engine does not know how the sound gets out: it produces tracks
to play, another component plays them. The first idea was to push into the
queue of an existing Spotify Connect device (`spotifyd`, the official
client). After checking ([spotify.md](spotify.md)), the direction is rather
that **forkstify embeds librespot and is itself the Connect device**: the
user installs nothing else, and connects by choosing "forkstify" in the
device list on their phone. Pushing to another Connect device stays possible
through the Web API, on top.

### A keyboard-first TUI, neomd-style

When there is an interface: one terminal, one screen, vim shortcuts. A fork
point is three lines; you choose with `1` `2` `3` or `h` / `l` (reassuring /
adventurous), you grind with `.`, you skip with `n`. Launched by
`omarchy-launch-or-focus-tui forkstify`, completed later by a bar widget on
the `omarchy-neomd` model (current track, next fork point, one click to
open).

### Tuning the algorithm from the keyboard

The principle was recorded on 2026-09-01
([0013](../decisions/0013-keyboard-tuning-measure-or-edit.md)); the key
table, though, is adjusted along the PoC. **Every key is either a
measurement or an edit.**

- A **measurement** changes `learned/` silently (skip, like) — that is the
  *learned* layer of [catalog.md](catalog.md).
- An **edit** changes a card and produces **a readable commit** ("top: + A
  Forest") — that is *mine*, shareable at once.
- `u` **undoes the last one**, whatever it was — a revert for an edit, an
  erasure for a measurement. The tuning history *is* the git log.

**The key table now lives in
[`docs/keybindings.md`](../keybindings.md)** — actual state, decided,
proposed, and the collisions found. What follows is the reasoning behind the
gestures, not their list.

The `d` happens at the exact moment a door makes sense ("this is the track I
left towards post-punk from") — nobody would write it cold into a file.

The `E` is the ultimate vim gesture: when the shortcuts are not enough any
more, you edit the text — the fork is the overlay, literally. (It was on
`e`, given up to encore on 2026-09-03: lowercase = a light and frequent
gesture, uppercase = a heavy one, like `t`/`T` and `x`/`X`.)

And, very neovim: every key is merely the shortcut of a **`:` command**
(`:top`, `:door post-punk`, `:fiche`, `:confort 2`). The commands make
everything discoverable and scriptable; the keybinds become a lookup table,
remappable in a config file.

### Everything is said in a toast

Joel's rule (2026-09-08, repeated on 2026-09-10 about home): **every
notification shows as a toast** — the panel at the bottom right of the body,
four seconds, sticky while something is loading —, under home as while
listening, modal open or not. The bottom line carries only what you type and
the keys available; nothing is "said" there. What is long is shown in a
block (`?`, the input hints), what is short is said in a toast. On 09-10, a
message from home ("has no card") still went through the bottom line, where
it was not visible: `tell` and everything home has to say now all go through
the same toast.

### A branch is a segment

Asked for by Joel on the first dry navigation test (2026-09-03): a branch
does not propose *one artist* but **a segment of a few tracks, potentially
across several artists**. From The Cure:

1. Siouxsie and the Banshees → Cult Hero → Joy Division;
2. New Order → Depeche Mode → Nouvelle Vague;
3. (and, later in the journey, "stay in the universe").

**Grinding is not a branch** (Joel, 2026-09-03): branches propose futures,
grinding reacts to the moment — "more of *that*". So it is a key, `e` /
`<n>e` (the `:encore` command), which slots n more tracks by the current
track's artist right after it; the chosen segment then resumes. An accepted
consequence: auto mode never grinds any more — lingering is a listener's
desire, not an engine decision (the comfort zone will be able to give auto
that leaning back). A PoC nuance: dry, there is no "current track", so `e`
targets the segment's last artist; the real semantics arrive with the sound.

Display while listening (Joel, 2026-09-04): we show the **queue of upcoming
tracks** (the current track on the `▶` line), and the **branches only appear
on the segment's last track** — at the moment of choosing. `p` lets you see
them in advance at any time (and choose one with `1`-`3`); the real "preview
then choose ahead" will come with the graphical interface.

**Choosing a branch does not cut off the current track** (Joel,
2026-09-04): the selected branch becomes "pending" and starts at the **end
of the current track** — you choose where to go next, the song finishes in
peace. `j` (or ⏭) forces the immediate move. A new start by another route
(search, `u`) cancels the pending one.

A direction is a short **walk** through the graph: you start from a
neighbor, chain on to their closest neighbor, one track per artist crossed
(an artist with no tops is crossed with no track). The **branch size** is
set along the way — provisional shortcut `b<n>`, future command `:taille`.
In real conditions, the seed will be a specific track found by the search,
not just an artist.

And the next directions are proposed from **the whole branch**, not from its
last artist alone (Joel, 2026-09-03): on the graph side, the neighbors of
the n artists taken together — a candidate linked to several of them rises
("linked to 2 artists in the branch"); on the vector side, the neighbor of
the branch's **centroid** — its center of gravity. Grinding and a branch's
internal walk stay on the last artist.

Two pieces of feedback from the second test (Joel, 2026-09-03):

- **Nothing is deterministic.** Two journeys from the same seed differ:
  branch heads and walk jumps are **drawn at weighted random** from a pool
  of good candidates — decision
  [0012](../decisions/0012-track-rotation.md) applied to the branches
  themselves, not only to the tracks.
- **A "stay in the universe" branch**, fourth on the menu when it makes
  sense: a segment drawn from the neighborhood of **the whole journey** (its
  artists and their graph neighbors, ranked by proximity to the journey's
  centroid), where already visited artists **come back** as long as they
  have unplayed tracks. That is the branch that lets you circle inside a
  cluster for as long as you want.

And a safeguard from the third test (The Cure was proposing French chanson
at 0.67 cosine): **the adventurous branch has a floor**. A vector candidate
has to be close enough in absolute terms (≥ 0.72) **and** share at least one
genre tag with the branch — country and decade do not justify a bridge —
unless it is very close (≥ 0.80). Below that, the adventurous branch
disappears and the graph takes the place back. Those two thresholds are
constants pending being driven by the **comfort zone** (0001): low comfort =
high floor.

### Choosing the seed

Two entry points, in the neovim spirit:

- **`/` then some text**: a search, in the active catalog **and** in
  Spotify, presented as one merged list. Implemented in `ecouter` on
  2026-09-04: a `[catalogue]` result starts a segment on that artist (native
  branches); a `[spotify]` result plays the track — and if its artist has a
  card, the journey hooks on there to keep branching, otherwise it is
  outside the catalog (no branch from there until cards are generated on the
  fly). The two searches do not serve the same thing: the cards say *where
  to branch from*, the API lets you play *anything*.
- **A list**: the user's library — artists and albums liked on Spotify —
  walked from the keyboard (`j` / `k`), filtered with `/`.

In the PoC with no interface, the starting seed is the command's argument;
`/` then serves to jump anywhere mid-journey.

### The PoC: playing the application before playing the sound

The criterion is Joel's: "we must be able to play the application before
being able to play the sound — pick a starting song, test the journey by
branches on the local base, and see that the navigation hangs together. At
that point we know we have a working PoC." The sound, Spotify and the
interface come afterwards; no player will save incoherent branches.

1. **The initial base** — Joel's catalog, generated cards (facts from
   MusicBrainz / Wikidata / Last.fm, meaning written by the agent in
   session), reviewed, vectors computed ([catalog.md](catalog.md)).
2. **Dry navigation** — one command: you give a seed, it proposes three
   readable branches with their reasons, you choose from the keyboard, it
   shows the segment (titles, without playing them), and so on. First on the
   links and tags alone; the vectors afterwards, to fill the holes.
   **Success criterion: coherent journeys, confirmed by reading them.**
3. **The sound** — the Spotify spike ([spotify.md](spotify.md)) then playing
   the segments. Can start in parallel with 2, it does not depend on it.
4. **The TUI** — once we know what there is to display.

## To settle

- **Where the PoC runs**: everything is dockerized on Joel's machine; the
  binary is built in a container (`cargo build`) and runs on the host with
  nothing installed there.
- **How you choose with no interface**: a digit in the terminal is enough
  for the PoC; the comfort zone chooses on its own after a delay.
