# Keybindings

**The single reference for forkstify's keys.** This document is
authoritative: the table lived in three copies (the code,
`forme-de-l-application.md`, `retours-usage.md`) until 2026-09-05, and that
is what let eight collisions settle in. The other notes now point here.

The grammar is set by decision
[0015](decisions/0015-keyboard-grammar-namespaces.md); this document alone
rules on the table itself, which is adjusted without reopening the
decision.

## The rule, in one line

> **`f` the branch · `e` encore · `t` the track · `a` the artist** —
> the rest of the keyboard only navigates and drives the session.

The first character says *what* you act on, the second *what you do*.
**Space is the leader**: outside the grammar, it opens the **input hints** —
everything you can type, or only the namespace being typed, like which-key
in LazyVim. It is not a poster: the sequence is still running, and the key
pressed in the hints performs the action (`space`, `e`, `3` = `e3`). `⌫`
goes one level up, `escape` closes (Joel, 2026-09-07). **A namespace typed
opens its level on its own** (Joel, 2026-09-23): `t` shows `tl`, `ts`…
without space, and `⌫` closes it — space is for the entry level, the help
of the beginning. **`:` is a level of its own**: the line opens the list
of the commands, narrowed to the word being typed (`:c` leaves `catalog`
and `comfort`), and the entry table holds one `:` row instead of every
command. In every level the **keys come first**, then the two lines, `/`
and `:`, then the legend. **One table for both screens** (Joel,
2026-09-23), and each screen shows only what means something on it: `f`,
`e`, `J`/`K` appear while listening, `s`, `v`, `r` at the home; everything
else works on both and is listed on both.

Every key comes from an **English word**, vim-style (`y` yank, `c` change):
`f` fork, `e` encore, `t` track, `a` artist, then `l` like, `s` skip,
`b` ban, `m` mark, `t` top, `d` door, `e` edit, `L` link, `p` peek/pause,
`r` reroll, `w` wander, `u` undo.

> **Paths renamed on 2026-09-06.** The catalog speaks English on disk, as
> `AGENTS.md` and [0010](decisions/0010-revised-format-links-without-doors.md)
> require: `fiches/` → `cards/`, `outillage/` → `tools/`, `vecteurs/` →
> `vectors/`, `catalogue.toml` → `catalog.toml`. Earlier decisions are
> **immutable** and mention the old names: read the new ones there.

## Legend

| Mark | Meaning |
|---|---|
| ✅ | Wired, usable in `listen` |
| 📋 | Decided (0015), **not wired yet** — the key answers "not wired yet" instead of doing nothing |

## `f` — the branch

| Key | Word | Action | |
|---|---|---|---|
| `f<n>` | **fork** | Branch n, **appended after whatever is already decided** — choices chain up and the evening builds itself | ✅ |
| `fn<n>` | fork **now** | …after the current track, the rest kept | ✅ |
| `f!<n>` | fork now, **force** | …after the current track, the rest dropped | ✅ |
| `fg<n>` | fork **generate** | **Generate the card for a gap without taking it**: the "○ no card yet" row becomes a **playable branch**, appended after the branches, the others do not move; then the gaps refresh around the fresh card — its own links into the void appear in grey. `f<n>` takes it afterwards, `⏎` may draw it (Joel, 2026-09-19/20) | ✅ |
| `fp` | fork **peek** | Unused since 09-06: branches are **shown permanently**, on the right. The key says so rather than doing nothing | ✅ |
| `fr` | fork **reroll** | Propose three other branches, **from the end of the list as it stands**: a track inserted by `ti`, queued by the discography's `e` or moved by `J`/`K` counts — both as context and as already played (Joel, 2026-09-17) | ✅ |
| `fu` | fork **undo** | Back to the previous branch | ✅ |
| `fw` | fork **wander** | Go far away, out of the current universe: opens the `:wander ` line already filled in — **enter** alone draws a head among the artists furthest from the journey (below the comfort floor, outside the journey and its graph neighbors), `fw <artist>` sets off from that catalog artist. The branch goes at the end of what is decided, like `f<n>` (Joel, 2026-09-11) | ✅ |
| `1`…`9` | | Shortcut for `f1`…`f9` | ✅ |
| enter | | Auto: draws at random among the branches shown | ✅ |

**The numbers carry on over the gaps** (2026-09-09,
[0016](decisions/0016-broad-base-and-on-the-fly-generation.md)). A link from
the card towards an artist **who has no card yet** is no longer thrown away:
it shows in grey at the bottom of the column, marked "○ no card yet", and is
taken with the next digit. Taking it **generates the card** — MusicBrainz
then Deezer, a few seconds — then the branch sets off like the others;
`fg<n>` generates **without taking**. A branch born from a gap is a **walk**
like any proposed branch — the fresh card at the head, then one track per
artist crossed — not an encore of the generated artist (Joel, 2026-09-20;
until then `f<n>` on a gap gave n tracks by that one artist). Generation is
an **edit**: it commits, and the card carries `generated = true`. See
[design/on-the-fly-generation.md](design/on-the-fly-generation.md).

`enter` never draws a gap: it chooses among what can play right away.

The `!` is vim's *force* (`:w!`): "and never mind what followed". The `n` is
*now*. It reads out loud: "fork now 3", "fork force 3".

After `f`, a **digit** names a branch, a **letter** an operation.

## `e` — encore

| Key | Word | Action | |
|---|---|---|---|
| `e<n>` | **encore** | n more tracks by the **targeted** artist, at the end of the queue | ✅ |
| `en<n>` | encore **now** | …behind the highlighted row if it is still to come, otherwise after the current track; the rest kept | ✅ |
| `e!<n>` | encore now, **force** | …in the same place, the rest dropped | ✅ |

The targeted artist is the one on the **highlighted** row, otherwise the one
of the track that is playing ([0020](decisions/0020-the-target-of-a-gesture.md))
— not the end of the branch chain, which has not been what is playing since
choosing a branch started appending to the queue (Joel, 2026-09-09).

`e` alone is not a command: the count is mandatory.

`f` and `e` are the **playback** namespaces — they decide what will play.
`t` and `a` are the **tuning** ones — they decide what the engine keeps.

## `t` — the targeted track

**The target of a gesture** ([0020](decisions/0020-the-target-of-a-gesture.md)):
the **highlighted** row if there is one (↑↓, `gg`, `G`), otherwise the track
that is playing. True for `t`, `a` and `e`. `ts` and `tb` only move the
music forward if they target what is playing; on an upcoming row, `ts` pulls
it out of the queue. Escape clears the highlight. **At the home, the whole
`t` namespace works on what plays underneath** (Joel, 2026-09-23): the
collection holds artists and never highlights a track, so the rule lands on
the playback foot — a highlight left behind by `q` does not count there.

| Key | Word | Action | Kind | |
|---|---|---|---|---|
| `tl` | track **like** | **Toggles liked / not liked.** Liked = more often (outranks the tops in the draw, ×10 at the cocoon, ×2 wide open, and erases the "less often"); on an already liked track, `tl` **removes the like**, with no penalty — unlike `ts` (Joel, 2026-09-14) | measurement | ✅ |
| `ts` | track **skip** | **Less often** — it does not interest me: takes note, removes the like, and skips | measurement | ✅ |
| `tb` | track **ban** | "Never that one again" — also removes it from the queue | measurement | ✅ |
| `tm` | track **mark** | Put into `learned/marks/inbox.toml` | measurement | ✅ |
| `J` / `K` | | **Move** the highlighted row one notch down / up, right away — only what is still to come moves, the branch name travels with its track, the opposite key undoes it (Joel, 2026-09-08) | queue | ✅ |
| `td` | track **door** | Make it a door towards the direction we are heading in (the next artist's tags) | edit | ✅ |
| `ta` | track **about** | **Its glyph and what it means**, album, featuring, year (when the discography has them), tags, familiarity, weight, and the first branch out of here — replaces `?` (Joel, 2026-09-14, the glyph on 2026-09-23) | session | ✅ |
| `tg` | track **google** | Search the targeted track in the default browser — **the artist's name then the title** — through `xdg-open`. Like `ag`, it needs no card: a track outside the catalog is searchable too. **At the home too** (Joel, 2026-09-23), where the collection holds artists and the cursor never lands on a track: there it searches what sounds underneath | session | ✅ |
| `ti` | track **insert** | **Insert a track** where you are: the same modal, anchored — before the highlighted row if it is still to come, otherwise right after what is playing. The anchor is written at the top; the inserted track carries "inserted (ti)" in the list. Choosing an artist inserts their best unplayed track. A track **outside the catalog** is inserted at once and its card is generated behind it; when it arrives, the track — even if it is already playing — is attached to the card, and `tl` has somewhere to write (Joel, 2026-09-10) | queue | ✅ |

**No more `tt` / `tT` while listening** (Joel, 2026-09-08,
[0018](decisions/0018-one-gesture-for-taste.md)): tops are only the entry
doors of a fresh fork, and are fixed in the discography (`ad`) or by hand in
the card. While listening, one simple gesture says "I want to hear this
track more often", and its opposite.

## `a` — the current artist

**On the home screen too**: the whole `a` namespace targets the highlighted
row of the collection (Joel, 2026-09-10) — `ad` opens the modal over home,
measurements write into the learned layer, `ag` works even without a card,
`ae` and `ac` need one.

| Key | Word | Action | Kind | |
|---|---|---|---|---|
| `al` | artist **like** | This artist, more often (weight ×1.43, ceiling 3) — and back among the liked. **On home too**, on the highlighted row (Joel, 2026-09-09) | measurement | ✅ |
| `as` | artist **skip** | This artist, less often (weight ×0.7, floor 0.1) — and **out of the liked**: an `unliked` flag in `learned/`, which outranks the Spotify likes and survives the harvests. On home too | measurement | ✅ |
| `ab` | artist **ban** | Never this artist again — also empties the queue. On home too | measurement | ✅ |
| `ae` | artist **edit** | **The card in `$EDITOR`, in place** (Joel, 2026-09-20): the key reader parks itself, the terminal and the screen go back to the editor; on close the card is read again — changed and readable, it is **committed** ("… — edited by hand") and the engine follows it immediately; unreadable, that is said, nothing is committed, `ae` again. With no `$EDITOR`/`$VISUAL`, the desktop editor (`xdg-open`). The loop waits for the editor: the sound carries on, the end of a track waits | edit | ✅ |
| `ac` | artist **connect** | **A connection of your own** (Joel, 2026-09-23). Some artists go together because of one ear and one life, not because anything links them; that is taste, not knowledge, so it is written in **`learned/`** and never in the card. The modal lists the connections already drawn, marked ✓ with their closeness — **from here (→) and from the other side (←)**, since the engine follows both — and below it the search. Enter on a result **asks how close**: the question starts at 4, **`h`/`l`** (or the arrows) move it a notch the neovim way, a digit jumps to it, `1` the farthest, a distant echo … `5` the closest, almost the same universe, **⏎** draws it, **esc** draws nothing. **In the list, ← → on a drawn one move its closeness directly**, written at once, the number changing under the cursor (Joel, 2026-09-23). **Enter on it reopens the question** on its closeness: `h`/`l` and ⏎ set it, **`x` undraws it**, esc leaves it — a connection is edited where it is held, whichever side it was opened from. The question **stays on screen until answered**, it does not fade like a toast (Joel, 2026-09-23). **The engine follows immediately**, in both directions, like any link. Nothing is committed on the spot: `learned/` goes with the rest on forkstify's own schedule ([0017](decisions/0017-synchronising-the-learned.md)), it **follows you between machines**, and **`Cp` can never carry it** — that only ever moves `cards/`. Replaces `aL`, retired the same day: writing a link into a card is rare enough to go through `ae` | measurement | ✅ |
| `ad` | artist **discography** | Open the **discography modal**: albums folded, what the card and the learned layer know of each track, `A` to promote an album. It has its own table, below. **On home too**, on the highlighted row of the collection, laid over home; an artist **without a card** gets one first ([0016](decisions/0016-broad-base-and-on-the-fly-generation.md)), and the discography opens as soon as it is there (Joel, 2026-09-10) | edit | ✅ |
| `ag` | artist **google** | Search the targeted artist (highlighted, otherwise current) in the default browser, through `xdg-open` (Joel, 2026-09-08) | session | ✅ |

The three verbs make a **readable scale** on the artist: `al` more often,
`as` less often, `ab` never again.

**The measurements of both namespaces write into `learned/`** since
2026-09-05 ([0014](decisions/0014-shape-of-the-learned.md)): one TOML file
per artist in the catalog, counters with decay built in (half-life six
months), written on every gesture, silent and never contributed back.

**Edits write into the cards since 2026-09-06**: `tt`, `tT`, `td` and
`ae` change a card **and produce a readable commit** (`src/edit.rs`). The
card is patched textually, never rewritten — it is a public interface, and a
round trip through serde would lose everything the code does not model —
except for `ae`, where the editor does the writing. An edit to the tops only
counts for the engine at the **next launch**, and the product says so;
generating a card (2026-09-09), adding or removing a link and editing a card
by hand (2026-09-20) enter the session's catalog **immediately**.

## `C` — the catalog

**The first uppercase namespace** (Joel, 2026-09-19; designed in
[design/before-release.md](design/before-release.md) workstream B, wired on
2026-09-20 after `Catalogue.dc.html`). `c` is comfort; sharing the letter
the way `f` does was ruled out — two meanings under one letter do not read.
The capital marks the **rare and heavy** gesture, as `A` promotes a whole
album. Three gestures, on both screens, running outside the loop: playback
carries on.

| Key | Word | Action | |
|---|---|---|---|
| `Cd` | catalog **diff** | **What this catalog has beyond the reference** — the old `:mine`, renamed. It fetches `upstream` first, like `Cp` and `Cu`; offline, the last fetch serves (2026-09-24). In an overlay, the count and the date of the last update, the **new** cards (generated or written, tags, links) then the **touched** ones (+n −m, the sections affected, the provenance note). Cards only — neither `learned/` nor `vectors/`. **Everything is shown, and it scrolls**: `j`/`k` or ↑↓, `gg`/`G`, the title says what lies above and below; escape closes (Joel, 2026-09-20) | ✅ |
| `Cp` | catalog **propose** | **Propose these cards to the reference**: fetch, a `proposal` branch from `upstream/main` in a separate worktree (the session's clone does not change branch), the **state of the cards** — never the learned layer, never the vectors —, a commit written for the reviewer (the new ones to skim, the touched ones to read), `push --force`. Then, with `gh` connected: the PR as it will go out and **`y`** opens it, any other key sends nothing; without `gh`: the comparison page in the browser, title and body prefilled. A proposal already open is updated by the push, and the toast gives its URL | ✅ |
| `Cu` | catalog **update** | **Bring the reference in**: the learned layer committed first, fetch, **merge** (not a rebase: `main` is shared by two machines), the index regenerated if cards changed, the session's catalog reloaded — a gap can become a branch without relaunching. A card changed **on both sides** stops the merge: the overlay names the cards and what each side changed, `o` opens the first one, `:catalog` resumes after `git add` and `git commit`. `vectors/` takes upstream (regenerated), `learned/` stays yours, `tools/` takes upstream | ✅ |
| `o` | **open** | The first card a merge stopped on, in the desktop editor (`xdg-open` — the key reader holds `stdin`, as for `ae`) | ✅ |

## Navigation and session

| Key | Word | Action | |
|---|---|---|---|
| `h` / `l` | | **Previous / next** track — vim, the horizontal axis. **At the home too** (Joel, 2026-09-23), on the journey playing underneath | ✅ |
| ← / `h` on a playing track | | **Restarts** the track; a second time — or within its first three seconds — goes back to the previous one (Joel, 2026-09-08) | ✅ |
| ← / → | | Likewise, for fingers off the home row | ✅ |
| `p` | **pause** | Pause / play | ✅ |
| ↑ / ↓ | | Move the **selection** along the axis — it highlights, it does not play. **Under an overlay** (`Cd`, `:catalog`, `ta`…), they **scroll** it, like `j`/`k`, `gg`, `G` (2026-09-20) | ✅ |
| enter | | Play the selection; with no selection, draw a branch | ✅ |
| `c<n>` | **comfort** | The comfort zone in one go, 5 cocoon → 0 exploration — on home too (Joel, 2026-09-08) | ✅ |
| `cc` | **comfort** | Set the comfort zone with the arrows: ↑↓ move, enter confirms, escape cancels — on home too since 2026-09-11. The gauge is **always at the top right on both screens, with the editing look**; `cc` adds "↑↓" (Joel, 2026-09-14) | ✅ |
| escape | | Clear the selection, close a block — hints included | ✅ |
| space | | **The leader**: opens the input hints — everything, or the namespace being typed; the sequence carries on inside, one key does the action. Space at the entry level closes it again. **On home too**, with its own table (Joel, 2026-09-10) | ✅ |
| ⌫ | | Erase the last key of the sequence — in the hints, go one level up | ✅ |
| `/text` | | **Filter** a list — the collection on home, the discography; escape clears (Joel, 2026-09-08) | ✅ |
| `:search` | | **The search modal** (mockup `Recherche.dc.html`, 2026-09-08): a `⟩` input line, results recomputed on every character — the catalog first (artists and titles known to the cards and the learned layer), Spotify behind, never mixed. ↑↓ choose, **enter** branches on an artist or plays a track, **tab** hides Spotify, **escape** closes. `:search <text>` opens it already filled, and typing carries the word on. **It opens from home too** (Joel, 2026-09-09), where enter starts a journey — on the artist, or on the track then the branches of its artist; a result **outside the catalog** generates its artist's card before setting off (2026-09-09, [0016](decisions/0016-broad-base-and-on-the-fly-generation.md)). Every Spotify row says **album · year · length**, to tell versions of the same title apart (2026-09-09) | ✅ |
| `q` | **quit** | While listening: **back to home**, listening carries on underneath with its playback footer. On home: quit (prints the journey) — Joel, 2026-09-08 | ✅ |
| `x` | **remove** | **Take this out of the list it is in** (Joel, 2026-09-23, replacing `tx`: removing is not a verb of the track alone). While listening: the highlighted track leaves the queue — it can still be proposed, this is not a ban. At the home: the highlighted artist leaves the liked, **without the weight `as` also moves** — `al` brings them back | ✅ |
| `r` | **resume** | On home: **back to the listening screen** if a session is playing; otherwise **resume the whole last journey** — history, current track and upcoming tracks (Joel, 2026-09-14). Escape with no cursor does the same | ✅ |
| `p` | **pause** | On home too: the session plays underneath | ✅ |
| `s` | **sort** | Change the collection's order: familiarity → a-z → last played — **home only** | ✅ |
| `v` | **view** | The collection shows **the liked** by default — artist or track liked here, track, album or follow on Spotify — or **all** the catalog's artists (Joel, 2026-09-09). Home only; the same word as in the discography | ✅ |
| `gg` / `G` | | The two ends of a list, as in vim — the collection on home, the axis while listening. `g` alone waits for its second | ✅ |
| `b` | **browse** | Browse dry — **disconnected screens only** | 📋 |

**`u` and `fu` are not the same thing**: `u` undoes the last *gesture* (a top
set by mistake, a ban), `fu` steps one notch back up the *journey*.

## Where each track comes from

Every song shown — in the queue, in the branches, on the `▶` line — carries
the mark of its **provenance**. An artist's pool adds up several sources
([0012](decisions/0012-track-rotation.md) §1: "a top is a weight, not a
closed list"), and the mark says which one won the draw.

| Mark | Provenance |
|---|---|
| `♪` | A **top** from the card |
| `♥` | A track **liked** here (`tl`), that is not a top |
| `↳` | A **door** ([0011](decisions/0011-doors-an-additional-criterion.md)) whose direction overlaps the branch's. The glyph is reserved for it: the search says `(branch next)` in plain words, since a glyph carries only one meaning (2026-09-05) |
| `+` | Known artist, track **outside the tops** — the one `tt` would promote |
| `·` | The **long tail**: the rest of the discography, which only weighs as comfort opens up |
| `~` | **Outside the catalog**: played from Spotify, with no card |

A door only takes its mark **when it opens**: outside its direction, it
stays a track like any other. That is what 0011 calls "an additional
criterion, never the main one".

## `:` commands

0013 wants every key to be the shortcut of a `:` command. **Every command
works on both screens** (Joel, 2026-09-23): the home used to answer three of
them with a table of its own; now the session runs the line wherever it was
typed, and the helper lists them all under `:`. What a command aims at
follows the screen: at the home, the highlighted artist of the collection.

| Command | Action | |
|---|---|---|
| `:size <n>` | Branch size, 1 to 9 (with no argument: shows it) | ✅ |
| `:comfort <n>` | Comfort zone, **5 = cocoon → 0 = exploration** ([0001](decisions/0001-comfort-is-familiarity.md)); with no argument, shows it. **Kept from one launch to the next** (Joel, 2026-09-14) | ✅ |
| `:warm` | Harvest the discography of the artist **under the needle** — the highlighted row, otherwise what is playing (0020); at the home, the highlighted artist — the long tail. It wrongly took the last artist of the chain (Joel, 2026-09-14) | ✅ |
| `:wander [artist]` | Go far away — the `fw` shortcut; with a name, to that artist (2026-09-11) | ✅ |
| `:sync` / `:push` | Commit and push the learned layer now — otherwise every ten minutes, on exit, and pull on start ([0017](decisions/0017-syncing-the-learned.md)) | ✅ |
| `:generate <name> [mbid] [spotify-id]` | **Bring in an artist missing** from the catalog, then set off from them — or, if they were proposed **as a gap**, take their branch at the end of the queue. An MBID as the last word replaces the search by name when MusicBrainz does not find them; if it does not answer at all, the card is born minimal (name, id, Deezer tops), flagged for review. **Over an existing card, the MBID regenerates it** — rewritten from the sources, hand edits included, one commit that says who it was; a Spotify id as a further word goes into the card ahead of MusicBrainz's and holds the search by name to it, a candidate linked to another id being passed over; when the MBID moved, what was learned about the old artist is dropped in the same commit (Joel, 2026-09-23, the `boo` card that was Boo! of South Africa) ([0016](decisions/0016-broad-base-and-on-the-fly-generation.md)): a card composed from MusicBrainz and Deezer, **its vector computed** ([0019](decisions/0019-the-application-vectorizes.md)), one single commit, and the session's catalog has it right away. Works on home as while listening. **With an mbid, over ongoing playback, it no longer starts on its own** (Joel, 2026-09-11): the success toast lingers and offers — **⏎** sets off from the artist and replaces the list, any other key keeps it. The search modal does the same on a result outside the catalog, with a plain `enter` — and so does **`enter` on a collection row without a card** (Joel, 2026-09-10, on Kanye West) | ✅ |
| `:catalog` | **The state of the fork in one line**: where we pull from, where we push to, commits ahead and behind, the date of the last update, cards beyond the reference; in local mode, it says so. And, after a `Cu` stopped on some cards: **resumes** once the cards are added (or committed) — the merge finishes, the index regenerates | ✅ |
| `:catalog diff` / `propose` / `update` | The three gestures, as a command — `Cd`, `Cp`, `Cu` | ✅ |
| `:catalog shell` | **A terminal in the fork** (Joel, 2026-09-23), for what the gestures do not cover — a `git log`, a card by hand. The desktop's default terminal (`xdg-terminal-exec --dir`), else `$TERMINAL` launched from the directory; with neither, the toast gives the path | ✅ |
| `:catalog fork <url>` | **Leave local mode**: the reference becomes `upstream`, your fork's URL becomes `origin`, `main` is pushed, and the learned layer pushes there from then on — rare, no key | ✅ |
| `:discography` | The artist's discography, as a modal — the `ad` shortcut (Joel, 2026-09-07); at the home, the highlighted artist's, generated first when they have no card (2026-09-23) | ✅ |
| `:connections` | **Every connection drawn with `ac`**, by artist, with its closeness — a block that scrolls like `Cd`, on both screens (Joel, 2026-09-23). Read-only: to set or undraw one, `ac` on either of its artists. It answers "where did I draw what" | ✅ |
| `:setup` | **Replay a setup step** — the list of seven, ticked or not, `⏎` or `1-7` replays one; the session closes and home comes back on the catalog (2026-09-20, [design/before-release.md](design/before-release.md) workstream A) | ✅ |
| `:library` | **Re-harvest the library** — steps 4, 5 and 7 of the setup in a row: tracks, albums, follows, ticked playlists (remembered), missing cards from the top of the ranking | ✅ |

And as a subcommand, because it has no place in the middle of a listening
session: `forkstify import <url>` takes in the cards of another catalog —
the ones you do not have, never the ones you do — then regenerates the
vectors.

## The setup screens

A table of its own (`keys::parse_setup`), prefix-free, vertical like the
modal, with the digits on top — a step's choices are numbered — and `o` to
open (the browser, the generation). After `Installation.dc.html`
(2026-09-20).

| Key | Action | |
|---|---|---|
| `1`…`7`, `0` | Choose — one of the step's choices, a step in the `:setup` list, the comfort at step 6 | ✅ |
| `j` / `k`, ↑ / ↓ | Up, down — the playlist list, the step list | ✅ |
| `h` / `l` | The comfort, one notch | ✅ |
| space | **Tick** a playlist | ✅ |
| `/text` | Filter the playlists; escape clears | ✅ |
| `o` | **Open**: the authorization page (step 3), the generation (step 7) | ✅ |
| ⏎ | Confirm the step, the field, the batch | ✅ |
| escape | **Skip** the step (or cancel a harvest, stop a generation after the current card) | ✅ |
| `q` | Quit | ✅ |

## The discography modal (`ad`)

**A mode of its own, with its own table** — the first one, and the pattern
for the next (`keys::parse_modal`). It is **prefix-free** like the listening
grammar, and it borrows from vim what that one left free: its axis is
vertical, so `j`/`k` go down and up, and `h`/`l` fold and unfold. Wired on
2026-09-07, after mockup **1a** of `Discographie.dc.html`.

What is playing keeps playing: the modal lays over the listening screen, it
does not replace it. `ad` targets the artist of the **highlighted** row if
there is one, of the current track otherwise — and the header names the
artist that is open, so any doubt is settled on screen.

| Key | Action | |
|---|---|---|
| `j` / `k`, ↑ / ↓ | Down, up — the album under the cursor opens by itself | ✅ |
| `h` / `l` | Fold everything (twelve rows), reopen the album under the cursor | ✅ |
| `gg` / `G` | The two ends of the list | ✅ |
| `A` | **Album**: promote the album's four most played tracks, outside the tops — put **pending**. No more `tt` / `tT` here either (Joel, 2026-09-08): a single track gets liked, it does not get promoted | ✅ |
| `tb` | Ban the row — a **measurement**, written right away (like / unlike: `tl`, a toggle) | ✅ |
| `e` | Put the track **in the queue**, without closing | ✅ |
| `tl` | Like / **unlike** (toggle) the row — a measurement written right away (Joel, 2026-09-14) | ✅ |
| `s` | The order: chronological ⇄ my plays first | ✅ |
| `v` | The **view**: all → ♪ tops → ♥ liked → ⊘ banned | ✅ |
| `/text` | Filter on a title or an album | ✅ |
| `u` | Undo the last pending edit — free, nothing has been written | ✅ |
| ⏎ | **Write the batch** — one write, **one single commit** — then, **on a track, set off from it**; **on an album row, listen to the whole album**: its tracks open a new seed in order, then the branches set off from the artist (Joel, 2026-09-14) | ✅ |
| escape | Close. With pending edits, the first escape warns | ✅ |

**Why a batch and one single commit**: you fix five tops in one single
thought, and five commits do not read back.  That does not contradict
[0017](decisions/0017-syncing-the-learned.md), which commits **the learned
layer** every ten minutes and on exit: the learned layer is measured and
silent, an edit is written and readable. The modal's `tl`/`tb` follow 0017's
rule, its `tt`/`tT` follow 0013's.

The screen also shows the **tops the discography does not return** — a typo,
a live version, a compilation title — at the end of the list: `tT` works
there, and that is where a generated card gets reviewed.

## Media keys (MPRIS / D-Bus)

Active as soon as MPRIS registers, as for `playerctl`.

| Key | Action | |
|---|---|---|
| ⏭ | Next (= `l`) | ✅ |
| ⏮ | Previous (= `h`) | ✅ |
| ⏯ | Pause / play (= `p`) | ✅ |
| ⏹ | Stop | ✅ |

## Queue mode

A mode of its own, with its own table, still to be designed: prepare
branches in advance, remove a track, remove a branch (**alone, or all the
depth that follows from it**), slot something in.

A structural point spotted: `rounds` is today a **flat list**, whereas
"remove all the depth" assumes a tree you can manipulate.

## How input works

Since 2026-09-05, `listen` (formerly `ecouter`) reads the keyboard in **raw
mode**: every key acts without Enter (`src/keys.rs`, termios through `libc`,
an RAII guard that gives the terminal back even on a panic). `/` and `:`
leave raw mode for an editable line, where a query belongs.

The grammar is **prefix-free**: no complete command is the beginning of a
longer one. That is what allows firing **with no delay and no timeout**,
where vim falls back on `timeoutlen`. An exhaustive test over every
three-key sequence checks the property (`grammar_is_prefix_free`), so a
future addition cannot break it silently.

That constraint decided two things:

- **The modifier comes before the count** (`fn3`, `f!3`), not the other way
  round: `f3n` would make `f3` both complete and a prefix.
- **The count follows the namespace** (`f3`, `e3`), not `3e` as in vim.
  Otherwise pressing `3` would be both "branch 3" and "the start of a
  count", undecidable without waiting for the next key — which would slow
  down the most frequent gesture. A side benefit: `f` and `e` become
  symmetric.

## The price to pay

**Frequent gestures cost two keystrokes** (`tl` to like, `ts` to skip),
where vim keeps one key for what you do most. That is the cost of
regularity, and only use will tell.

Two safeguards: `1`…`9` remain the shortcut for `f1`…`f9` — the deliberate
exception, since choosing a branch is *the* gesture of the product; and the
bare keyboard is empty enough to promote a gesture there later, should one
turn out to be constant.

## Still to settle

1. **`ts` (skip track) vs `l` (next).** Two gestures to skip a track, with
   an invisible difference: `l` moves on without noting anything, `ts` moves
   on **and** notes it in `learned/`. A distinction that is sound on paper,
   perhaps imperceptible in the fingers.
2. ~~**`aL` or `ac`** to link two artists~~ — settled on 2026-09-23, and
   not on the letter but on the meaning. Drawing two artists together turned
   out to be **taste, not knowledge**: it depends on one ear and one life,
   so it belongs in `learned/` and must never leave with `Cp`. `ac`
   (*connect*) says that act; `aL` is retired, and writing a link into a
   card — rarer, and shared once proposed — goes through `ae`. The awkward
   capital goes with it.

3. ~~**The `C` namespace**~~ (2026-09-19,
   [design/before-release.md](design/before-release.md)), the catalog
   (`Cd` diff, `Cp` propose, `Cu` update) — the letter was settled by Joel
   on 2026-09-19 (sharing `c` with comfort, ruled out), settled and wired on
   2026-09-20 — table above. `fg<n>`, from the same workbook, is too.

## Free letters

The bare keyboard only keeps `h`, `l`, `p`, `e`, `f`, `t`, `a`, `c`, `C`,
`o`, `x`, `q` — outside the modal, where `j`, `k`, `s`, `v`, `e`, `A` and
`u` serve (table above). `Q` left on 2026-09-08 along with queue mode.
Still free: `b`, `d`, `g`, `i`, `j`, `k`, `m`, `n`, `r`, `s`, `v`, `w`,
`y`, `z`, `u` and `.`, and the capitals except `C`, `G`, `J`, `K` (`o` was
taken on 2026-09-20, and `C` too; `x` on 2026-09-23, while `u` and `.` went
back to the pool — undo and repeat are not being built for now, and `u`
keeps its own meaning inside the discography). Inside the namespaces, `d` was taken in `a` on
2026-09-07 (`ad`, discography), and `c` on 2026-09-23 (`ac`, connect) —
`aL` retiring the same day gives `L` back. Queue mode can move in without displacing
anything.

## Where this grammar comes from

On 2026-09-05, taking stock of the keys revealed **eight collisions**, six
of them invisible as long as the table lived in three copies. The namespaces
make them all fall, and by construction: two gestures can only clash inside
one namespace, where the letters are under control.

| The collision then | Resolution |
|---|---|
| `u`: "undo" (0013) vs "previous branch" (the code) | `u` undoes a gesture, `fu` steps back one branch |
| `n`: skip, no, and the "now" modifier | `n` = **now**; `y`/`n` only lives in a modal prompt |
| `d` both an action (door) and a prefix (`da`/`dt`) | `d` leaves the bare keyboard: it is `td` |
| `dt`/`da` overlap `X` and `-`, already decided | Absorbed by `tb`, `as` and `ab` |
| `.` overlaps `e` (two keys for encore) | `.` takes back its vim meaning (repeat) |
| `h`/`l` (reassuring/adventurous) overlap `1 2 3` | `h`/`l` become navigation |
| `p` both an action (plan) and a prefix (`p1`) | `p` leaves the branches: it is `fp`, and `p` becomes pause |

Two of those resolutions have since lapsed: **`u` and `.` left the grammar
on 2026-09-23**, undo and repeat not being built for now. `fu` still steps
back one branch, and `u` keeps its own meaning inside the discography,
where it undoes a pending edit.
| `pr` colliding with `p<n>` | `fr`, inside the namespace |

**"Fork" has two meanings, and that is deliberate** (Joel's call,
2026-09-05): forking the **catalog**
([0008](decisions/0008-the-fork-is-the-overlay.md)) is a rare gesture, once
per machine, and stays `:fork`; forking the **journey** is the constant
gesture of listening, and it is the `f` key. They never cross. The nuance
is in the vocabulary of [`vision.md`](vision.md).
