# Usage feedback and the keyboard workstream

Living note, opened on **2026-09-05** after the first long `ecouter`
sessions (Joel, several sessions on the morning of 09-05). It holds the list
of feedback items, the inventory of what really exists, and what is left to
settle before adding anything.

It is worked through **as it comes**: every entry carries its status, and
what is done moves down into `docs/progress.md`.

**Status after `learned/` (2026-09-05) — 7 done, 1 half, 3 not started, and
still nothing verified in real listening.** The statuses below are checked
against the code, not against memory.

## The inventory first (feedback no. 4)

Joel: "I'm starting to get mixed up, I don't want to add more and have it
become unusable". So here is the real state, taken from `src/listen.rs`
(`on_input`, `on_control`) on 2026-09-05 — to be distinguished from the
*projected* table in `forme-de-l-application.md`, which remains a largely
unimplemented direction.

### What works today in `ecouter`

Eleven wired gestures, listed with all the rest in
**[`docs/keybindings.md`](../keybindings.md)** — the single reference since
2026-09-05.

### What is missing and is sometimes believed to exist

- **No pause/play key on the keyboard.** ⏯ only goes through the media keys
  (MPRIS). In the terminal, nothing.
- **None of 0013's tuning keys**: `t`/`T` (tops), `x`/`X` (skip/set aside),
  `a` (like), `d` (door), `m` (mark), `E` (edit the card), `-` (less often),
  `z` (comfort), `?` (why), `y`/`n`. The table is written, nothing is wired.
  That is exactly feedback no. 7.
- **No `:` command**, although 0013 makes them the foundation ("every key is
  merely the shortcut of a `:` command").
- **No write into `learned/`**: the learning loop (0014) is not closed, so
  no measurement can be recorded yet.

### Two conflicts to settle before adding anything

**1. `u` has two meanings.** Decision **0013** (accepted) says: "`u` undoes
the last action, whatever it was". The current implementation made it "go
back to the previous branch". As long as there was neither measurement nor
edit, the ambiguity cost nothing; as soon as `da`/`dt` and the tops arrive
(feedback nos. 8 and 9), a real undo is needed. **To settle**: `u` = undo
(0013) and backwards navigation moves to something else (`h`? already
mentioned in the projected table), or 0013 is revised by a new decision.

**2. The proposed keys overlap already reserved ones.** The `d` of 0013 is
**door** (an edit that produces a commit); the `da`/`dt` of feedback no. 9
would make it a *dislike* prefix. And above all: `X` ("never that one
again") **is already** a track's dislike, `-` ("this artist, less often") is
already the lukewarm on an artist. **To settle**: are `da`/`dt` new
gestures, or the `:` names of the `X` and `-` gestures already planned? The
risk otherwise is two keys for the same thing — precisely what feedback
no. 4 wants to avoid.

## The feedback items, one by one

### 1. Confirming without "Enter", neovim-style

**Status**: **done** (2026-09-05, `src/keys.rs`) — not verified in real
listening.

Today input is line by line (`std::io::stdin().lines()`, a deliberate choice
from 09-04: "real time belongs to the TUI stage").

**A hard point spotted**: this item and items 2, 3, 5 and 9 appear to
contradict each other — you cannot type `2en!`, `p1n!`, `da` or `pr` with a
"one key = one action" reading. **Neovim solves exactly that** and is the
reference cited: raw-mode reading with a **pending buffer**, digits are
*counts*, letters are operators, `!` is a modifier, and the sequence
executes as soon as it is unambiguous. `/` and `:` switch to line mode (with
Enter), which the search needs anyway.

That is what was done: termios through `libc`, an RAII guard that gives the
terminal back even on a panic, ← → arrows recognized, `/` and `:` opening an
editable line. The grammar is **prefix-free**, so everything fires with no
delay and no `timeoutlen` — a property verified by an exhaustive test over
every three-key sequence.

### 2. Encore, in three shades

**Status**: **done** (2026-09-05) — not verified in real listening.

Asked for: add n tracks at the end of the branch · after the current track ·
after the current track, dropping what was planned.

Wired as `e<n>` / `en<n>` / `e!<n>`: the modifier comes before the count, a
constraint of the prefix-free grammar (see 0015). The old behavior was
already the "now" variant; the other two are new.

### 3. Choosing a branch: the same grammar

**Status**: **done** (2026-09-05), **bug understood** — not verified in
real listening.

Joel: "when you choose a branch in advance, it plays after the current
track, not after the current branch".

Checked: `choose()` sets the branch pending for the end of the **track**,
and `start_segment()` does `self.queue = stops.into()` — the rest of the
segment is **thrown away**. So the current behavior is the most destructive
of the three variants, and it is the default.

Wired as `f<n>` / `fn<n>` / `f!<n>` (the prefix is `f`, not `p`: see 0015).
The default is now "after the current branch" — **the segment is no longer
thrown away**, which was the bug.

**An observation that serves feedback no. 4**: items 2 and 3 describe **the
same grammar** — a gesture, then the modifiers `n` ("now") and `n!` ("now,
and never mind what follows"). One rule to learn for two commands, and it
will generalize to the next ones. That is the line to hold so the surface
stays small.

**To settle**: if `p1` exists, should plain `1` be kept? Two ways of doing
the same thing is what we want to avoid. Proposal: `1` stays the quick
gesture (= `p1`), `p` becomes the prefix that accepts the modifiers.

### 4. Taking stock of the shortcuts

**Status**: **done**, and it went further than asked. The inventory
produced [`docs/keybindings.md`](../keybindings.md), the single table, then
the complete overhaul of the grammar (decision
[0015](../decisions/0015-keyboard-grammar-namespaces.md)): four namespaces,
eight collisions resolved, every key backed by an English word.

### 5. Proposing other branches

**Status**: **done** (2026-09-05) — `fr`, in the branch namespace; `pr` is
abandoned, it collided with `p<n>`.

A key that draws three new branches when none of them suits.  Joel proposes
`pr` or `r` (refresh/reload).

**Note**: that is exactly what `auto_advance()` was doing by accident before
the 09-05 fix — the draw already exists (`recompute()`), it only needs
exposing.

**To settle**: `pr` collides with the `p<n>` scheme of feedback no. 3 (`p1`,
`p1n`). Plain `r` is cleaner and keeps `p` consistent.

### 6. "Go off to something completely different"

**Status**: **settled and wired on 2026-09-11** — reading **(a)**, with an
optional target: `fw` alone leaps far (a head below the comfort floor,
outside the journey and its graph neighbors, the furthest weighing the most,
the dial leaning as for any head), `fw <artist>` leaps to that catalog
artist. The key opens the `:wander ` line already filled in, enter sets off.
The branch goes at the end of what is decided.

Two possible readings, and they do not lead to the same work:

- **(a)** a branch/key that **leaves the current universe** — ignoring the
  adventurous branch's floor (cosine ≥ 0.72 + a shared tag) to leap far on
  purpose;
- **(b)** setting off from a **new seed** mid-session, without leaving the
  application (which `/text` half does already).

My reading leans towards **(a)**, given where the note sits in a list of
listening gestures — but I am not making the call for you.

### 7. Wiring the missing shortcuts (tops, card editing…)

**Status**: **half done.** The blocker is lifted, it has moved.

**The measurements work** (2026-09-05, `src/learned.rs`): `tl` like, `ts`
skip, `tb` ban the track, `tm` mark, `al`/`as` the artist's weight, `ab` ban
the artist. They write into `learned/artists/<slug>.toml` on every gesture,
and the engine reads them back (excluding the banned, weights on the
branches).

**The five edits remain to be done** — `tt`/`tT` (tops), `td` (door), `ae`
(open the card), `aL` (link). They do not touch the learned layer but the
**card**, and they must produce a readable commit (0013). That is a
catalog-writing layer that does not exist yet: that is what blocks now, not
`learned/`.

Note: the order proposed that morning — edits first — was reversed, and it
was the right call. The measurements all share the same storage, so wiring
them together cost one module; the edits, on the other hand, need a commit
mechanism no other gesture reuses yet.

### 8. Linking the current artist to another

**Status**: **not started**, and it is now one **edit** out of five (see
feedback no. 7): the `aL` key is decided, what is missing is the layer that
writes and commits a card, plus the way of naming the target.

A key that creates a **typed link** from the current track's artist towards
another artist — so an **edit** in the sense of 0013 (a commit in the
catalog, the link format of 0010).

**To specify**: how the target is named (the `/` search reused?), which link
type by default, and whether the proximity is entered or inferred.

### 9. Saying you do not like something (`da` / `dt`)

**Status**: **done** (2026-09-05) — not verified in real listening.

`tb` (ban track) and `ab` (ban artist) absorb both the `dt`/`da` asked for
and the `X` and `-` already decided: no more duplicate. On the artist, the
three verbs form a scale — `al` more often, `as` less often, `ab` never
again.

Both bans act **right away** on what is planned: `tb` removes the track from
the queue, `ab` removes all of that artist's tracks, and the engine stops
proposing them.

### 10. Syncing through git (`gh`) between machines

**Status**: **not started.** Only the commands are reserved (`:sync`,
`:push`, `:pull`). A different axis from the others — it is infrastructure,
not keyboard.

Joel's idea: regular commits and pushes of usage and cards, `pull` on start
to find your usage again from one machine to the next.

Consistent with 0008 ("the fork is the overlay") and 0002 (a version
controlled catalog). Points to handle: what to do about **conflicts** in
`learned/` (decayed counters, six-month half-life — an additive merge makes
sense, a textual `git merge` does not), at what **frequency** to push
without turning listening into a commit machine, and the **offline**
behavior.

`gh` is installed and authenticated on this machine (the `kbyjoel`
account).

### 11. Queue mode

**Status**: **largely moot** (Joel's observation, 2026-09-06) — and that is
the best news of the day.

Queue mode had to exist because the queue was *imposed*: one branch replaced
another, you only saw one step ahead, and a second view was needed to
prepare. Since **choosing a branch appends it to the queue** (09-06), the
main queue already does most of what the mode was meant to bring:

| What the mode promised | Where it stands |
|---|---|
| prepare branches in advance | **done** — choices chain up, the queue lengthens |
| see all the prepared depth | **done** — it is all on the axis, branches named and threaded |
| move around in the queue | **done** — ↑↓ move a selection |
| remove a track | **done** — `tx`, without banning |
| get back to listening | **moot** — we never left it |
| remove a whole branch (alone or with its depth) | **left** |
| move a track within the queue | **left** |
| slot a chosen track in | **left** — `/` already does part of it |
| undo a removal (`u`) | **left** — it is 0013's global `u` |

So what is left is not a **mode** to write, but **four gestures** to add
where we already are. The `Q` key has no obvious purpose any more; the "flat
list or tree" question no longer arises either, since the chain is a list
and that is enough.

**What really remains of the mockup**, and has no equivalent: board 3b's
observation — **a prepared chain has already consumed the comfort zone**,
which therefore governs nothing beyond the first link. That is true of our
queue today, and it is said nowhere on screen.

A mode in its own right: prepare branches in advance, remove tracks, remove
a whole branch (**it alone, or all the depth that follows from it**), slot a
track in.

It is a **second view** onto the journey's state, with its own key table —
not to be confused with the listening view. The "remove this branch" /
"remove all the depth" distinction assumes the journey's tree can be
manipulated, whereas `rounds` is today a **flat list**. That is the
structural point to look at first.

Linked to the "real preview + choosing in advance" already noted as
belonging to the interface (2026-09-04).

### 12. Exploring an artist's discography

**Status**: **done** on 2026-09-07 — `ad` opens the modal (form 1a of the
mockup), with its own note:
[`artist-exploration.md`](artist-exploration.md).

Joel, after a "Cat Power" seed: "I like almost nothing but tracks from the
album *What Would the Community Think*; I would have liked a command
(`:explore`?) to get a visual list of the tracks sorted by album, and to be
able to `tt` the ones I like and `tT` the tops I want to remove."

What is missing is not the edit — `tt`/`tT` write and commit since 09-06 —
but only being able to perform them on **the track that is playing**:
straightening out an artist would mean grinding them entirely. The
discography is already cached (`:warm`), the glyphs and the learned layer
too: what is missing is a **screen**. Four points await a call — the target
(the current track or the selection, a question that applies to the whole
`t`/`a` namespace), the command's name, the granularity of the commits, and
what enter does.

### 13. The small circle of artists at comfort 3–4 (2026-09-14)

**Status**: **wired on 2026-09-14** (lead A + the tail by familiarity) —
feedback from Joel after a few days.

> After a few days, I alternated between comfort 3 and 4, and the same
> artists come back too often: the feeling of going round in circles, of
> only having a small circle, when my catalog and my liked artists are big.
> It depends on the seed, but still.

**Diagnosis (code read).** Several forces stack, and the main one is a
**reinforcement loop**:

1. `familiarity01` **grows with plays** (the decayed counter in `learned/`).
   At comfort 3–4, `Comfort::favours` **leans towards the familiar** (pull
   0.2 at c3, 0.6 at c4): the more you play an artist, the more familiar
   they are, the more they are drawn as a branch head — hence replayed. The
   circle **tightens by itself**, and the liked ones, being the most
   familiar, dominate.
2. **No rotation of artists over time.** 0012 §2's freshness only penalizes
   recently played **tracks** (`freshness`, per title) and `visited` only
   excludes the **current** journey. Nothing says "you have heard this
   artist a lot lately, ease off". A new seed lands in the same core.
3. **The head draw is very sharp**: the graph in `weight² × favours`, the
   vectors in `(score−0.5)³ × favours`, and only the **first 6** neighbors.
   The closest to a familiar artist almost always win.
4. **The `stay` branch** draws from the neighborhood of the journey's
   universe — it reinforces the current cluster.

A big catalog does not help as long as the engine leans towards that
high-familiarity core.

**Leads (reversible, to be settled).**

- **(A) A freshness at the artist level**, transposed from 0012 §2: an
  artist heard recently is damped as a **branch head**, and the penalty
  decays over a few days. `learned/` already carries `plays` and `last` per
  artist — the data exists. It is the most direct lever, it does not change
  the meaning of comfort, and it is explainable in one sentence (the
  project's rule). **Recommended.**
- **(B) Softening the sharpness of the draw**: reduce the exponents (² and
  ³) and widen the window (`take(6)` → 12–20), so that more neighbors get a
  real chance. Simple, widens the circle without breaking it.
- **(C) Breaking the loop**: cap the effect of **acquired** familiarity on
  the heads' lean (base the lean on the seed's familiarity rather than on
  accumulated plays), so that playing an artist does not make them come back
  more.
- **(D) A novelty budget per session**: guarantee every session a few heads
  not heard recently, even at high comfort.

**Wired on 2026-09-14.** (A) `Learned::artist_freshness`: an artist heard
recently steps back as a branch head (half-life 4 days, floor 0.3), applied
to the graph, to the adventurous branch and to `stay`. And the tail is
**modulated by the artist's familiarity** — `share = tail_share() ×
familiarity`: a new artist (familiarity 0) is led by their tops, a known
artist opens up their deep cuts. Lowering comfort therefore widens the
artists without drowning in the tail. (B), (C), (D) stay in reserve if the
circle tightens further. Still to be proved over time.

**Where the freshness lives, and syncing between machines (2026-09-14).**
Joel: "these freshness notions are tied to one machine; could we consider a
file embedded in the catalog fork?" — that is **already the case**. The
counters and dates (`plays`, `last`, per artist and per title) live in
`learned/artists/*.toml`, **version controlled in the catalog fork and
synced by 0017** (a commit every ten minutes, pull on start, push on exit, a
`merge-learned` merge driver counter by counter when two machines have
learned at the same time). Artist freshness, like track freshness,
therefore travels between the work machine and the laptop, with no new file.
The only setting still **local** is the comfort zone
(`~/.local/state/forkstify/comfort`); whether to sync that too is to be
decided.

**The rest of the feedback (2026-09-14).** Joel: "I was tempted to lower
comfort to get more new proposals, but lowering comfort brings in a lot of
long tail — so tracks that are more or less good, since we are hitting
neither the tops nor the likes, but the rest, somewhat at random."

That is the knot: **comfort mixes two axes Joel wants to set separately.**

- **Width (novelty of artists)** — how far the branches go towards new
  artists. Carried by `favours(familiarity)`.
- **Depth (long tail)** — how far down we go into an artist's discography,
  below the tops and the likes. Carried by `tail_share() = openness()`.

One dial holds both (0001 for familiarity, 0012 §4 for the tail), so **you
cannot have one without the other**: lowering comfort to widen the artists
brings the tail in, and vice versa.

**A deeper lead (reversible), very much in the project's spirit and with no
second dial**: **tie the tail's depth to the artist's familiarity**, not (or
not only) to the global comfort.

- A **familiar** artist (played a lot) → their tail weighs more: we dig into
  their deep cuts, which is what you want from an artist you love.
- A **new** artist → led by their **tops**: we present them at their best,
  not through a track at random.

Then lowering comfort **widens the artists**, and every new artist arrives
**through their tops** — exactly what Joel was after. The deep tail stays
for the artists you know. 0012 §4 ("no second setting") is preserved: it is
the **per-artist** familiarity, already in `learned/`, that modulates the
tail, not a new knob. It stacks with (A), artist freshness: together, width
without drowning in the tail.

## To settle — recap

All five points from that morning were settled on 09-05 (see
[0015](../decisions/0015-keyboard-grammar-namespaces.md) and
[`keybindings.md`](../keybindings.md)): `u` undoes a gesture and `fu` steps
back one branch · `da`/`dt` are absorbed by `tb`/`ab` · `p1` and `1` became
`f<n>` and the `1`…`9` shortcut · proposing again is `fr`.

Still open:

1. **Feedback no. 6**: "go off to something completely different" — leave
   the current universe (a), or set off from a new seed (b)? The `fw` key
   awaits the answer.
0. **Saving the playlist** — everything is there (the past, the queue, the
   branch names), but where to write it is not settled: a Spotify playlist,
   a catalog file, an `.m3u`? That deserves a decision.
2. **`ts` (skip track) vs `l` (next)**: two gestures to skip a track, the
   difference — noting it in `learned/` or not — being invisible in the
   fingers.
3. **`aL` or `ac`** to link two artists.
6. **Removing an artist from the liked** from home, written into `learned/`
   — see [home-screen.md](home-screen.md) § To settle, point 5.
7. **A smooth setup**: connecting, importing the library, playlists to tick
   — see [first-run.md](first-run.md) § To settle, point 5.
4. ~~**The target of `t`/`a`** (feedback no. 12)~~ — settled on 2026-09-09
   ([0020](../decisions/0020-the-target-of-a-gesture.md)): the highlighted
   row wins, otherwise the current track, for `t`, `a` and `e`. The trigger:
   an `en3` that served the artist at the end of the branch chain, not the
   one that was playing.

## What has not been verified

**Nothing that was wired on 09-05 has run in a listening session** — neither
raw-mode input, nor the three variants, nor the seven measurements.
Everything is verified at compile time and by ten unit tests, nothing under
the fingers.

That weighs more since `learned/`: **the measurements write into the
catalog**. The first `ts` will create `learned/artists/` for real, and that
is version controlled content. A test session before stacking the edits
layer on top would be prudent.

Three bets can only be judged at that point:

- the **cost of two keystrokes** on the frequent gestures (`tl`, `ts`);
- the **meaning of `h`/`l`**, navigation having moved to the horizontal
  while the queue displays vertically;
- the real usefulness of **`ts` against `l`**, whose difference — noting or
  not — stays invisible in the fingers.
