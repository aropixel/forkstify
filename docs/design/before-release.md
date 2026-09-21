# Before release: the last three workstreams

Note opened on **2026-09-19**, at Joel's request: "refine the last things
before we can release the project". Three subjects, one workbook. Each
workstream distinguishes what is **asked for** (Joel, 2026-09-19, unless
stated), what is **proposed** (the agent, to be approved) and what is left
**to settle**. Nothing gets coded until the workstream's "to settle" section
is empty.

The background notes stay where they are, and this note points at them
rather than copying them: [first-run.md](first-run.md) for the base / mine /
learned model and the bootstrap, [catalog.md](catalog.md) for what is
shareable, [on-the-fly-generation.md](on-the-fly-generation.md) for the
gaps.

## The starting state, read in the code on 2026-09-19

- **No setup.** `forkstify` reads `[catalog] path`, empty =
  `~/Work/forkstify-catalog`, and `Catalog::load` fails if the folder is
  missing. The Spotify library comes in through five Python scripts in
  `tools/` run by hand, which read the GNOME keyring, then `classement.py`
  writes `learned/classement.json` (French keys, read by `learned.rs`).
  What already exists and will serve: the **disconnected** screen
  (`home::disconnected_rows`) that guides the two authorizations, the
  browser OAuth PKCE (`spotify.rs`, five scopes), zeroconf discovery of the
  phone, `WebApi` which paginates, `import.rs` which knows how to add a
  remote and take cards, `generate.rs` and `embed.rs` for the missing cards.
- **The catalog, on the git side.** Joel's fork is **217 commits ahead** of
  the reference, zero behind: 165 learned files, 49 cards added or touched
  up (`git diff --stat upstream/main -- cards`), and the vector index
  entirely rewritten (0019's normalization). `:mine` computes the diff of
  the cards against `upstream/main` and shows it in an overlay. The
  application's commits are made by `git commit` with no explicit identity:
  they take the machine's global `user.name`.
- **The reference carries Joel's learned data**: `learned/classement.json`,
  his five `artistes-*.json` harvests, `faits-mb.json`, `mbid.json` and 18
  learned files are on `aropixel/forkstify-catalog`. A fresh fork would
  inherit Joel's library. To be cleaned before release (see the final list).
- **The gaps.** `engine::missing_neighbors` returns the context's links that
  point at a missing card; the column shows at most three, in grey "○ no
  card yet", numbered after the branches. `f<n>` on a gap calls
  `generate(…, After::Branch)`: the card is born, **and the branch sets
  off** — there is no path that generates without branching. Gaps are
  frequent because a generated card keeps its four Deezer `similar` even
  into the void, which is intended (0016).

---

## Workstream A — the setup after installation

**Done on 2026-09-20**, after the `Installation.dc.html` mockup (Claude
Design, project "Accueil Forkstify", nine screens): `src/setup.rs` (the
screens and the thread), `src/library.rs` (the harvest,
`learned/library.toml`, the ranking, the coverage), `keys::parse_setup` (the
table: digits, `j`/`k`, `o`, space ticks, `⏎`, escape), `tui::render_setup`
(one column, the step and its gauge on the right, the prompt on the last
line). **The mockup settled the open points**: local mode stays ("it avoids
a wall on the first evening"), a single `library.toml`, threshold ≥ 5 and at
most 30 cards, both scopes accepted along with the re-authorization they
entail. What departs from the mockup, for want of better: at step 7 the
generation **is watched** (escape stops it after the current card, and what
is written is committed) instead of carrying on behind home; step 4 is
watched too (escape cancels). And `:setup` / `:library` **close the
session** — the sound stops — then home comes back on the replayed catalog.
The first run is `forkstify` with no readable catalog. The `tools/` scripts
are not yet removed from the catalog: after Joel has replayed `:library` on
his fork. **Not proved in a real session**: the harvest and the generation
need the network and the tokens; the thread (clone, identity, comfort,
summary) ran in a smoke test on temporary XDG folders.

### Asked for

A setup **on first run and replayable** (Joel, 2026-09-09,
[first-run.md](first-run.md) § To settle, 5): connection, library import,
playlists to tick, the ranking computed by the application, the Python
scripts removed. Joel supplies a Claude Design mockup before we code the
screens. This workstream settles **the steps to present and the information
to gather**, so that the mockup starts from a settled list.

### Proposed: seven steps, in this order

The guiding thread: **there is nothing to type that you do not already
know**, and every step can be skipped then replayed on its own. The screen
is a session screen like home (0021, "home is a session screen"), everything
is said in a toast, and the keyboard grammar stays the modals' one (`j`/`k`,
`space` ticks, `⏎` confirms, `escape` skips).

| # | Step | What we gather | What we write |
|---|---|---|---|
| 1 | **The catalog** | The URL of **their fork** of the reference (or nothing) | the clone in `~/.local/share/forkstify/catalog`, `origin` = the fork, `upstream` = the reference, `[catalog] path` in `config.toml` |
| 2 | **The git identity** | `user.name` / `user.email` if they are missing | `git config --local` in the clone, never global |
| 3 | **The connection** | nothing to type: the phone (zeroconf) and the browser (OAuth) | both tokens, where they already live (`~/.local/state/forkstify`) |
| 4 | **The library** | a "yes" | `learned/library.toml`: liked tracks (main artist only), liked albums, followed artists |
| 5 | **The playlists** | the ones to **tick** in their playlist list | their identifiers in `learned/library.toml`, so that replaying is a single gesture |
| 6 | **The comfort** | a digit 0–5, on the gauge (`cc` exists) — default 3 | the `comfort` state, as today |
| 7 | **The coverage** | a "yes" to generate the missing cards at the top of their library | `generated = true` cards, their vectors, one commit |

Step by step:

1. **The catalog.** The normal case is **a fork**: that is what 0016 and
   0008 assume, and what syncing the learned layer (0017) requires — it
   pushes to `origin`, which a clone of the reference does not allow. Three
   entry points: (a) the URL of their fork, pasted in; (b) if `gh` is
   installed and connected, forkstify offers to **fork it itself**
   (`gh repo fork aropixel/forkstify-catalog --clone`); (c) nothing —
   forkstify clones the reference in **local mode**: everything works, the
   learned layer commits but does not push, and home says so (`⇅ local`).
   `:catalog fork <url>` (workstream B) moves from (c) to (a) later with
   nothing lost: we add the remote and push. The default path leaves
   `~/Work`: `~/.local/share/forkstify/catalog` (XDG), with Joel keeping his
   current `[catalog] path`.
2. **The git identity.** A commit with no `user.name` fails, and forkstify
   commits constantly (0017). If the global config has them, nothing is
   asked. Otherwise we ask for the name and the email and write them
   **locally** into the clone: that is the identity of the catalog's
   commits, not the machine's.
3. **The connection.** That is today's disconnected screen, slotted into the
   sequence: nothing new, except **two more scopes** for steps 4 and 5 —
   `user-follow-read` (followed artists) and `playlist-read-private` (their
   playlists, private ones included). Consequence: already authorized users
   — Joel — will go through the browser **once**, and the product must say
   so instead of letting it look like a failure.
4. **The library.** The scripts' harvest, rewritten on `WebApi`:
   `/me/tracks`, `/me/albums`, `/me/following?type=artist`, paginated
   (several hundred calls for a big library, with `Retry-After` respected —
   ~2 min for 5,000 tracks). One progress bar per source. **The main artist
   alone** counts (Joel, 2026-09-09: the guests on a liked track are not
   liked).
5. **The playlists.** The list of their playlists (name, track count,
   owner), their own first, to tick. A ticked playlist counts its artists ×1
   as today (`#fipway`, road trip). The ticked identifiers are
   **remembered** in `learned/library.toml`, so that "re-harvest" is a
   single gesture with nothing to tick again — the question left open on
   09-09 is settled that way if Joel agrees.
6. **The comfort.** `cc`'s gauge, with its words (cocoon → exploration), and
   0001's sentence: "it chooses on its own when you do not choose".
7. **The coverage.** We cross the ranking with the catalog: "48 of your 50
   most present artists have a card; generate 12 more to cover everything
   with a score ≥ 5? (~3 s each)". That is the broad base **and** generation
   (0016) at the scale of one install: a newcomer whose taste is far from
   the reference has enough to start from the very first evening, without
   waiting to arrive at each artist. Ceiling (30?) and threshold (score ≥ 5,
   home's) to be tuned; **one single commit** for the batch, like `import` —
   not one per card.

**The ranking computed by the application.** `learned/library.toml`
replaces `classement.json` and the five `artistes-*.json`, in **English
vocabulary** (0014 was waiting for it, 0022 requires it): per artist,
`name`, `spotify`, `liked_tracks`, `liked_albums`, `followed`,
`playlist_tracks`, `score`, `sources`; at the head, the harvest date and the
ticked playlists. The score keeps `classement.py`'s formula — tracks ×1,
albums ×3, follow +8, playlists ×1 — as constants in the code, not in
`[tuning]`: 0023 tunes the engine's numbers, not the harvest. `learned.rs`
reads the new file and **still the old one** as long as it exists, so that
Joel's fork does not change behavior on the day of the switch; resolving the
MBIDs (`resoudre-mbid.py`, `mbid.json`) has no purpose any more — generation
resolves by name, with Deezer as backup.

**Replayable.** `:setup` from home replays the sequence, with every step
already done shown ticked and skipped with a `⏎`; `:library` replays steps
4-5-7 alone. On the command line, `forkstify setup` does the same for the
Omarchy plugin, which installs the binary and will be able to launch it. The
first run is simply `forkstify` **with no readable catalog**: it opens the
setup instead of failing.

**What is withdrawn afterwards.** The harvest and ranking scripts
(`bibliotheque-*.py`, `playlist-spotify.py`, `classement.py`,
`resoudre-mbid.py`), `generate-cards.py` and `vectoriser.py` (already
replaced, 0019), `voisins.py` (`forkstify check`). What stays is
`amis-*.py`, which has no equivalent in the application and awaits
consenting friends — to be translated (step 11 of the progress note) or
moved out of the reference catalog.

### To settle

Settled in one block by the 2026-09-20 mockup:

1. ~~**Fork mandatory or not**~~ — local mode is done.
2. ~~**The format of `learned/library.toml`**~~ — one single file.
3. ~~**Ceiling and threshold of step 7**~~ — score ≥ 5, at most 30,
   proposed again on every `:library` (screen 9 marks it "to do").
4. ~~**The two extra scopes**~~ — accepted; screen 3 says it is not a
   failure.
5. ~~The mockup~~ — `Installation.dc.html`.

What is left, in use: whether step 7's generation should one day carry on
behind home as the mockup shows. The `tools/` scripts were removed on
2026-09-20 (Joel); `classement.json` is still read in the fork as long as
`:library` has not written `library.toml`.

---

## Workstream B — the "catalog" namespace

**Done on 2026-09-20**, after the `Catalogue.dc.html` mockup (seven
screens): `src/fork.rs` (`status`, `diff`, `propose`, `update`, `resume`,
`fork`), the table (`C` pending, `Cd` `Cp` `Cu`, `o`), the jobs outside the
loop in `listen.rs`, `:catalog` and its subcommands. `:mine` and
`edit::mine` are gone. What departs from the mockup: `:catalog` shows in an
overlay like `Cd`, not in the flow; `Cd`'s overlay does not open the card
(it **scrolls**, since Joel's first piece of feedback, 2026-09-20: `j`/`k`,
↑↓, `gg`/`G`, the whole diff shown); `o` on a conflict opens the card in the
desktop editor (`xdg-open`), for want of being able to hand `stdin` to
`$EDITOR`. **Not proved for real**: `Cp` through to `gh pr create`, `Cu` on
Joel's real fork; but `diff`, `propose` (twice, the branch rewritten),
`update` that merges, `update` that stops on a card and `resume` are proved
by an **integration test over three temporary git repositories** (the
reference, the fork, the clone) — `git` entered the `forkstify-build` image
for that.

### Asked for

Group the catalog gestures under one letter: `:mine` (renamed *diff*), a
command that makes a **PR of the new cards towards the reference**, a
command that **rebases the fork on the reference**.

### Proposed

**The letter: `C`, uppercase** (Joel's call, 2026-09-19). `c` has been taken
by comfort since 2026-09-08 (`c<n>`, `cc`); sharing it between two subjects,
the way `f` does (digit = branch, letter = operation), was proposed and
**ruled out** by Joel — two meanings under one letter do not read. `m`
(*mine*) was considered, as was `z` to move comfort. `C` keeps the word
*catalog*, stays free (`G`, `J`, `K` are the only bare capitals), and the
capital marks the **rare and heavy** gesture, as `A` promotes a whole album
in the modal. It is the first uppercase namespace; the grammar stays
prefix-free, and the `grammar_is_prefix_free` test verifies it. Three
gestures, from an English word as everywhere:

| Key | Word | Command | Action |
|---|---|---|---|
| `Cd` | catalog **diff** | `:catalog diff` | What this catalog has beyond the reference — the current `:mine`, renamed; cards only |
| `Cp` | catalog **propose** | `:catalog propose` | Propose these cards to the reference: a branch, a push, the PR opened in the browser |
| `Cu` | catalog **update** | `:catalog update` | Bring the reference into the fork, regenerate the index, reload the session's catalog |
| — | | `:catalog fork <url>` | Turn a local clone into a fork (workstream A, leaving local mode) — rare, no key; replaces the 📋 `:fork` in the table |
| — | | `:catalog` | The state in one line: n commits ahead / behind, last update, remotes |

These are **rare** gestures — 0015 gives them a `:` command; the keys are a
convenience for the three everyday ones, and the `C` line of the hints
(`space`, `C`) shows them. `:mine` disappears with no alias (sobriety: one
name).

**`Cd` — diff.** The same computation as today (`git diff upstream/main --
cards/`), the same overlay, with two additions: every card says whether it
is **new** (`+ generated`, `+ written`) or **touched up** (`~ +3 −1`), and
the header gives the count and the date of the last update. Still cards
only: neither `learned/` nor `vectors/`.

**`Cp` — propose.** Joel's fork shows the problem: `main` mixes 165 learned
commits with the cards, a PR from `main` would be unreadable and would
contribute the learned layer back — which 0014 forbids. So we propose **not
commits, but the state of the cards**, as `import` does in the other
direction ("the state of their cards, never their history"):

1. `git fetch upstream`;
2. a `proposal` branch **from `upstream/main`**, in a separate **worktree**
   (`~/.local/state/forkstify/proposal`) — the clone the session reads never
   changes branch, and the learned layer keeps committing on `main` every
   ten minutes;
3. `git checkout main -- cards/` in that worktree: the cards as they are,
   with no `learned/` and no `vectors/`;
4. a `Propose N cards` commit whose body is written **for the reviewer**, in
   two lists (see "Review on the reference side" below): the new generated
   cards first, one line each — name, MBID, tags —, then the touched-up
   cards with the summarized diff and the links' provenance note (catalog.md
   § "Not all of mine is equally shareable"); trailer
   `Forkstify: proposal <version>`;
5. `git push --force origin proposal` — **one open proposal at a time**, the
   branch rewrites itself and the open PR updates;
6. the PR itself, **two routes depending on the machine** (Joel's call,
   2026-09-19): if `gh` is installed **and connected** (`gh auth status`),
   forkstify shows the title, the body and the card count, and **asks for
   confirmation** — `y` creates the PR (`gh pr create --head
   <account>:proposal --title … --body …`), any other key sends nothing and
   the pushed branch stays there; the toast gives the PR's URL. Otherwise,
   the browser opens on the GitHub comparison page, with title and body
   prefilled in the URL, the way `ag` opens an artist (`xdg-open`): you read
   it back, you click. Either way it is catalog.md's "pre-chewed PR", and
   nothing goes out without one more gesture. A proposal already open does
   not create a second one: step 5's push updated it, and the toast says so
   with its URL (`gh pr list --head proposal`, or nothing to do on the
   browser side).

**The vectors do not go into the PR.** The index is derived (0019) and
rewritten in full on every regeneration: in a PR it would be nothing but
noise and conflicts. It is the reference's GitHub action that regenerates it
on merge (below).

**Review on the reference side** (Joel, 2026-09-20: "I'm afraid validating
the PRs will be a bit laborious on my end"). The measurement on his own
fork, after a month of use: **46 new cards, all `generated = true`, 3 cards
touched up by hand** (14 lines added, 4 removed). A generated card is the
pipeline's output, MusicBrainz then Deezer — the reference would have
produced the same one: there is nothing to *read* there, there are things to
**check**, and a machine does that better. What needs an ear are the
touch-ups, which are rare. Hence three pieces, settled by Joel on
2026-09-20:

1. **A GitHub action on the reference** (in place on 2026-09-20,
   `forkstify validate`), which checks every PR: readable TOML and
   `format = 1`; `mbid` present and **unique across the whole catalog** (it
   is what catches a "Ye" proposed while `kanye-west` exists); slug matching
   the name; `links` targets as valid slugs; no file outside `cards/`. On
   merge into `main`, it **regenerates the index** (`forkstify vectors`) and
   commits it — the maintainer never touches it. The action runs the binary
   in the `forkstify-build` container, like `bin/build`.
2. **`Cp` composes the PR for the reviewer**: step 4's two lists. You skim
   the first, you read the second.
3. **A merge rule written into the reference repository**
   (`CONTRIBUTING.md`, in English — 0022): a PR that brings **only generated
   cards** merges at a glance — name and MBID, for the homonym the action
   cannot see, like Les Thugs — as soon as the action is green. A PR that
   **touches up** existing cards gets read: the facts (`member`, `collab`,
   `family`) are taken; a `similar` is taken if it carries its provenance
   note; a change of tops is taken if it **fixes an error** (wrong title,
   live version, wrong Spotify identifier), not if it expresses a taste —
   the reference's tops are only the entry doors of a fresh fork
   ([0018](../decisions/0018-one-gesture-for-taste.md)).

~~Ruled out for now: auto-merge~~ — **settled on 2026-09-20** (Joel: "let's
make it so that PRs with only card additions are validated automatically"):
a second workflow on the reference, `automerge.yml` on `workflow_run`,
merges any PR whose `check` is green and that **brings only new cards**
(additions only in `cards/`), says so in a comment and regenerates the index
right after; a PR that touches up a card waits for a reader. The glance at
the homonym happens afterwards, through a touch-up. In reserve if even that
weighs: `Cp` would by default propose only the new cards.

**`Cu` — update: a merge, not a rebase.** Joel says "rebase"; I propose
**merge**, for a reason from 0017: `main` is shared by two machines that
pull with `--rebase` and push as they go. Rebasing `main` onto
`upstream/main` rewrites commits already pushed, and the other machine ends
up with a history that diverged under its feet. A merge rewrites nothing,
and the catalog's structure makes it almost always trivial: one card per
artist (the reference's new cards arrive with no conflict), the learned
layer apart. The result is the same for the user — the reference's cards are
there — and `Cd` stays accurate since it compares states, not histories. The
steps:

1. the dirty learned layer is committed first, like `:sync`;
2. `git fetch upstream` then `git merge --no-edit upstream/main`;
3. `vectors/` in conflict: take either one and **regenerate** the index in
   place (0019), in a commit that follows; `cards/` in conflict — both sides
   touched the same card — we stop, name the cards, and hand control back
   ("upstream enriched The Cure's description, you changed the tops": the
   field-by-field guidance imagined in catalog.md is a separate workstream,
   not this one);
4. the session **reloads its catalog** — it has owned it since 09-09, so a
   card arriving from the reference can fill a gap already shown without
   relaunching — the learned layer does not move;
5. a toast: "⇅ 41 cards from upstream · index regenerated".

`git rebase` stays possible by hand for whoever has only one machine; the
product does not offer it.

### To settle

1. ~~**The letter**~~ — settled 2026-09-19: `C`.
2. ~~**Merge rather than rebase** for `Cu`~~ — settled 2026-09-19: the
   merge.
3. ~~**One open proposal at a time**~~ — settled 2026-09-20: the `proposal`
   branch, rewritten.
4. ~~**The browser rather than `gh`**~~ — settled 2026-09-19: `gh` with
   confirmation when it is there and connected, the browser otherwise.
5. ~~The name `Cp`~~ — settled 2026-09-20: `Cp`, *propose*.

**Workstream B has nothing left to settle**: the letter `C`, the three
gestures, the merge for `Cu`, `gh` with confirmation otherwise the browser,
one single `proposal` branch, and the review on the reference side.

---

## Workstream C — generating a gap without taking it

**Done on 2026-09-20** (`src/keys.rs`, `src/engine.rs::branch_from`,
`src/listen.rs`): `fg<n>` generates gap n's card with the `After::Gap`
intent; when it arrives, the card becomes a branch **after the branches
shown** — so at the gap's number when it was the first one —, the others do
not move, and the gaps refresh around the context *and* the fresh card. The
branch is a walk (`engine::branch_from` → `walk`), for `f<n>` as for
`fg<n>`; the `f<n>` on a gap no longer gives a disguised encore. Three
tests: `fg2` parses and the grammar stays prefix-free; a fresh head's branch
crosses more than one artist; an unknown head gives nothing. Not proved in a
real session. In reserve: `fga`, and optional generation ahead of time.

### Asked for

Among the proposed branches, an artist **without a card** can today only be
taken by queueing them. Joel wants to **generate the card alone** — `fg<n>`
— and for the branches, once the card arrives, to be proposed **taking the
new card into account**.

### Proposed

**`fg<n>` — fork generate.** After `f`, `g` is an operation, like `r`, `w`,
`u`; `fg` waits for its digit, and the grammar stays prefix-free. On a gap
shown, `fg<n>` starts the generation with a new intent, `After::Gap`,
alongside `After::Branch`: the card is composed, vectorized, committed and
adopted by the session exactly as today (0013: a generation is an edit) —
**but nothing is queued**. On a playable branch number, `fg<n>` answers "n
has a card already"; on a gap already being generated, "already underway"
(the `generating` guard exists).

**On arrival: the gap becomes a branch, in its place — and a real branch.**
Rather than redrawing three branches — which would reshuffle what the user
was in the middle of reading, the very reason `⏎` draws among the branches
shown (Joel, 2026-09-05) — the "○ no card yet" row turns into a **playable
branch at the same number**, with the ○ mark replaced by the track's source
mark. The other two branches do not move.

**What it contains** (Joel, 2026-09-20: "not only tracks by the artist who
has just been generated — a branch regenerated like the others, with one
track by the generated artist and other tracks by other artists"): a
**walk**, like any proposed branch — the fresh card at the head, then one
track per artist crossed, drawn from its graph and vector neighborhood, at
the `:size` size. That is `engine::walk`, the one behind `propose` and
`wander`, with the fresh card as head, the link's reason as reason and its
proximity as weight. And it is a **fix along the way**: today, `branch_to` —
the path of `f<n>` on a gap — builds the branch with `engine::encore`, hence
n tracks by the generated artist alone; a gap taken gave an "encore"
disguised as a branch. `f<n>` and `fg<n>` both go through the walk. A
neighbor of the fresh card that has no card is ignored by the walk, as
everywhere — but it shows up as a gap, below.

Then the gap list **refreshes** from the context: the fresh card's links
into the void appear in grey in their turn — that is the catalog growing
along its links, one notch further. The toast: "✓ Georges Moustaki — card
ready · branch 3". `fr` is still there for whoever wants three other
branches, and the fresh card is then in the pool like the others; `⏎` can
now draw the branch, which it never does on a gap.

What that changes in the code, to size it: one variant of `After`, one case
in `keys::parse` and its test, `walk` exposed (or a `branch_from` on
`wander`'s model) and `branch_to` calling it in place of `encore`,
`recompute` of the gaps alone after adoption, the `fg<n>` line in the hints
and the table. One test: a generated gap's branch crosses more than one
artist when the neighborhood allows it.

**One step further, to discuss: generating ahead of time.** If the gaps are
a nuisance, it is because they wait for a gesture. A
`[generation] prefetch = true` option would generate the gaps shown **in the
background and silently** (three at most, ~3 s each), the way
`harvest_proposed` goes to fetch the proposed branches' tail: the ○ would
disappear by itself, and `fg<n>` would only serve to force it. The price:
MusicBrainz calls on every recompute, and a catalog growing with cards we
have never visited — less of a nuisance than it looks, since `Cp` will
propose them to the reference and every card born enriches the commons
(catalog.md § Pooling). I propose `fg<n>` first, the option afterwards if
use asks for it, **disabled by default**.

### To settle

1. ~~**In place rather than redrawn**~~ — settled 2026-09-20: in place, at
   the gap's number, and the branch is a **walk** like the others, not an
   encore of the generated artist.
2. ~~**The name**~~ — settled 2026-09-20: `fg`.
3. **`fga` — generate everything** (all three gaps): not before the need
   shows itself twice.
4. **Generating ahead of time**, as an option, later.

---

## The proposed order

1. **Workstream C** — the smallest, no heavy decision, it makes listening
   smoother right away and Joel can try it the next day.
2. **Workstream B** — `Cd` and `Cu` first (Joel needs them to follow the
   reference once it is cleaned), `Cp` afterwards: it requires the reference
   to be ready to receive.
3. **Workstream A** — the biggest; it is waiting for the mockup and it
   touches the learned layer's format. Its steps 1-3 (catalog, identity,
   connection) can be done before the mockup: they have no screen to draw,
   they are questions asked one after the other.

## What release needs on top, outside these three workstreams

Noted along the way, so as not to lose it — every point is one line, to be
settled elsewhere:

- ~~**The reference carries Joel's learned data**~~ — removed on
  2026-09-20, along with `tools/` from both repositories. The fork's first
  `Cu` settles the *modify/delete* conflicts on `learned/` by itself (his
  are kept).
- **The default path** `~/Work/forkstify-catalog` is Joel's machine's —
  workstream A, step 1.
- **A `README.md`** in the application's repository (there is only
  `AGENTS.md`), and the catalog's in English (0022).
- ~~**The reference's GitHub action** and its `CONTRIBUTING.md`~~ — in place
  on 2026-09-20: `.github/workflows/catalog.yml` (`check` on every PR —
  cards only, `forkstify validate`; `index` on `main` — `forkstify vectors`
  committed), the composite action that builds forkstify from
  `aropixel/forkstify`, `CONTRIBUTING.md` in English. The application has
  been public since 2026-09-20: the workflow's token is enough for the
  action.
- ~~**Installing required Docker**~~ — solved on 2026-09-21:
  `.github/workflows/release.yml` publishes, on every `v*` tag, a Linux
  x86_64 binary (`forkstify-<version>-x86_64-linux.tar.gz` + `SHA256SUMS`),
  compiled in the same image as `bin/build`; `omarchy/install.sh` downloads
  it, checks its digest, and only falls back to building in a container
  otherwise. The floor is glibc 2.36 (Debian 12), so Arch and newer. An AUR
  `forkstify-bin` package is ready in `packaging/aur/`, to publish once the
  first release is out.
- **Nothing says to go and click "Install"** (Joel, 2026-09-21, while
  trying a clean install): after `omarchy plugin add`, typing `forkstify` in
  a terminal answers `command not found`, and neither the add command nor
  the shell points at the bar's card. To settle: a line in the `README.md`,
  a word at the end of `omarchy plugin add`, or an installation that fires
  some other way.
- **A `forkstify` already on the PATH hides the button**: the widget decides
  between "Install" and "Launch" on `command -v forkstify`, so a stale link
  makes a valid install look present (seen on 2026-09-21 on Joel's machine,
  with the dev install's link).
- **The catalog's licence** (catalog.md § To settle: ODbL or CC BY-SA) and
  the code's (`manifest.json` says MIT).
- **The edit commits speak French** (fixes pending in the progress note) —
  `Cp` will make them visible to the reference.
- ~~Both repositories are **private**~~ — the application has been public
  since 2026-09-20; the reference stays private, and the fork-counting
  command (first-run.md § Measuring usage) counts nothing until it is
  public.
