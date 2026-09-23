# forkstify

A music player for Linux, wired to Spotify, that works **by branches**: you
start from a track, the application plays a few, then proposes several
directions; you pick one — or let it pick — and so on. The pitch, the
principles and the vocabulary are in [`docs/vision.md`](docs/vision.md).

**Provisional name.** A personal project of Joel Gomez Caballe, in the
**design phase**: nothing is coded yet. Its philosophy fits in one line —
**take back the algorithm** — and its rule in one sentence: any automatic
decision must be explainable in one sentence and changeable in one commit.

This file is the agent's contract on this repository. It is authoritative
here; `~/Work/chorizo/AGENTS.md` is authoritative on the machine.

## Where things are

| Path               | Contents                                                                |
|--------------------|-------------------------------------------------------------------------|
| `README.md`, `LICENSE` | The public front door and the MIT licence.                          |
| `AGENTS.md`        | This file — the agent's contract.                                        |
| `docs/vision.md`   | What the product is: pitch, principles, vocabulary.                      |
| `docs/decisions/`  | One decision per file, numbered and dated. **A decision is never changed**: to go back on one, you write a new one that supersedes it. |
| `docs/design/`     | Working notes by subject, living, rewritten as the exchanges go. Each one distinguishes **decided**, **direction** (proposed, uncontradicted) and **to settle**. |
| `docs/progress.md` | The current state: done, waiting, next steps. A session's entry point.   |
| `CHANGELOG.md`     | What each version brings, for whoever **uses** forkstify — never the working log above. `bin/release` promotes its `## Unreleased` section to the version being cut, so it cannot fall behind. |
| `docs/strengths.md` | Joel's impressions from use, dated: what works and what sets the project apart, so the list is there when the time comes. |
| `docs/keybindings.md` | The key table: wired, decided, proposed, and the collisions. The single reference. |
| `docs/tuning.md`   | The engine's settings (`[tuning]`): what each one does, its default value, the effect of raising or lowering it. |
| `manifest.json`, `omarchy/` | The repository is also an Omarchy plugin ([0021](docs/decisions/0021-the-repository-is-the-omarchy-plugin.md)): the bar widget, and the script that builds and installs the binary. |
| `bin/dev-install`, `bin/release`, `bin/release-aur` | The maintainer's three: make this clone the forkstify that runs; cut a release (version, tag, push — the workflow does the rest); refresh the Arch package from a published archive and optionally push it to the AUR. |
| `bin/build`, `bin/test` | Build and test in the `forkstify-build` container (mise puts `bin/` on the `PATH`: `build`, `test`). The binary lands in `target/release/forkstify` and runs on the host. The image has had `git` since 2026-09-20 (`fork.rs`'s tests): `docker build -t forkstify-build .` if it is older. |

The rest comes with the decisions.

## What is decided

The detail is in `docs/decisions/`. In summary:

- **Target**: Linux, personal use first, but the catalog is meant to be
  shared by the Linux ecosystem.
- **Music source**: Spotify. No local files and no other service.
- **The catalog is the heart of the product**, and the branch engine brings
  it to life. We supply a base (cards + vectors), you make it your own, you
  improve it, and it **learns from use** — the base / mine / learned model
  in [`docs/design/catalog.md`](docs/design/catalog.md).
- **Comfort zone** = familiarity, from 0 to 5, set at launch; it chooses on
  its own when the user does not
  ([0001](docs/decisions/0001-comfort-is-familiarity.md)).
- **Catalog** = version-controlled text files, one per artist, shared and
  forkable ([0002](docs/decisions/0002-shared-forkable-catalog.md)).
- **Tracks**: tops by default
  ([0003](docs/decisions/0003-tracks-tops-and-doors.md)); doors withdrawn,
  typed links in English with a proximity, English fields, settings in
  `catalog.toml`
  ([0010](docs/decisions/0010-revised-format-links-without-doors.md)).
- **Two repositories**, the application and the catalog; importing =
  cloning, one active catalog, switch whenever you like
  ([0004](docs/decisions/0004-two-repositories-targetable-catalog.md)).
- **Cards carry their format's version**
  ([0005](docs/decisions/0005-version-in-the-cards.md)).
- **An artist's identity is their MBID**; Spotify is one implementation
  among others ([0009](docs/decisions/0009-mbid-identity.md)).
- **Concept and PoC before the interface**: no mockup for now.
- **Rust** ([0006](docs/decisions/0006-rust.md)).
- **Cards in TOML**, `format = 1` at the head
  ([0007](docs/decisions/0007-cards-in-toml.md)).
- **No separate overlay: the fork is the overlay.** The active catalog is a
  git clone, and you commit your changes into it
  ([0008](docs/decisions/0008-the-fork-is-the-overlay.md)).
- **Repetition must never be imposed**: a weighted draw, a cooldown, no
  replacement, comfort = depth
  ([0012](docs/decisions/0012-track-rotation.md)).
- **Keyboard tuning: every key is a measurement or an edit**, `u` undoes,
  everything is a `:` command
  ([0013](docs/decisions/0013-keyboard-tuning-measure-or-edit.md)).
- **A broad base *and* on-the-fly generation** — the reference catalog aims
  for breadth, and the application generates a card when you arrive at an
  artist that has none; a user has a **fork** of the repository, not a
  repository of differences
  ([0016](docs/decisions/0016-broad-base-and-on-the-fly-generation.md)).
- **Keyboard grammar: four namespaces** — `f` the branch, `e` encore, `t`
  the track, `a` the artist; the target is a prefix, the count follows the
  namespace, and the table is authoritative in `docs/keybindings.md`
  ([0015](docs/decisions/0015-keyboard-grammar-namespaces.md)).
- **One gesture for taste**: `tl` "more often", `ts` "less often"; liked
  tracks outrank the tops (×10 at the cocoon, ×2 wide open), the tops are
  only the entry doors of a fresh fork, no more `tt` while listening
  ([0018](docs/decisions/0018-one-gesture-for-taste.md)).
- **The engine's numbers are tunable in the configuration**: a `[tuning]`
  section in `config.toml`, one name per number, defaults = the previous
  code, read at launch
  ([0023](docs/decisions/0023-engine-numbers-are-tunable.md)).
- **The learned layer lives in `learned/`** (one file per artist, decayed
  counters, six-month half-life)
  ([0014](docs/decisions/0014-shape-of-the-learned.md)), **and syncs
  itself**: pull on start, a commit every ten minutes and on exit, a push in
  the background, merging **by counter** through a git driver; the
  application's commits are in English and carry the trailer
  `Forkstify: <kind> <version>`
  ([0017](docs/decisions/0017-syncing-the-learned.md)).
- **The application does its own vectorizing**: a generated card is born
  with its vector, in the same commit; `forkstify vectors` regenerates the
  index and replaces the Python script; same model, 128 tokens, normalized
  vectors ([0019](docs/decisions/0019-the-application-vectorizes.md)).
- **The target of a gesture is the highlighted row, otherwise what is
  playing** — for `t`, `a` and `e`; one rule on the axis
  ([0020](docs/decisions/0020-the-target-of-a-gesture.md)).
- **The repository is also the Omarchy plugin**: `manifest.json` at the
  root, `omarchy/` for the widget, the plugin installs and launches the
  binary; and the player publishes real MPRIS metadata, `forkstify:next`
  included
  ([0021](docs/decisions/0021-the-repository-is-the-omarchy-plugin.md)).
- **Language: everything in this repository is in English**, prose
  included — the documentation, the decisions, the design notes, this file,
  the code (identifiers, comments), the vocabulary on disk (paths,
  subfolders, format fields), the messages of the commits the application
  produces, and the interface itself (screens, messages, command line,
  widget). Aimed at open source; the interface went first on 2026-09-10
  ([0022](docs/decisions/0022-english-interface.md)), the prose followed on
  2026-09-21 ([0024](docs/decisions/0024-everything-in-english.md)).
  Subcommands `journey` and `listen`.

## What is left to settle

No large decision is pending any more: the concept, the language, the format
and the catalog model are all fixed. The fine questions are listed at the
end of each note in `docs/design/` and get settled along the PoC.

**The current state and the next steps are in
[`docs/progress.md`](docs/progress.md)** — that is a new session's entry
point, to be kept up to date with every advance.

## Spotify constraints

Verified on 2026-08-30 in the official documentation and in librespot's and
spotify-player's READMEs, unless stated otherwise. Detail and sources in
[`docs/design/spotify.md`](docs/design/spotify.md).

- **Spotify Premium** required, for the Web API as for librespot.
- **Web API**: every application needs a *client id* registered on the
  developer dashboard. An application in **development mode** is limited to
  **5 users** declared by hand. **Extended mode** has only been granted
  since May 2025 to established **organizations** with at least **250,000
  monthly active users**. A free project therefore cannot obtain a shareable
  client id. The free ecosystem (spotify-player, ncspot) reuses a historical
  client id already in extended mode.
- **Recommendations, related artists, audio features**: closed to new
  applications since late 2024. The project does not depend on them by
  design ([0002](docs/decisions/0002-shared-forkable-catalog.md)).
- **librespot** (Rust) makes the application a Spotify Connect device,
  discovered from the phone's app with no password to type. It does not use
  the developer dashboard. It is not officially supported by Spotify.

## How the agent works here

The rules in `~/Work/chorizo/AGENTS.md` apply (French with Joel, everything
dockerized, no `sudo`, ask before any `push`, any deletion, any root
action). On this repository, in addition:

- **Everything generated is generated in English.** Documents, decisions,
  design notes, code and comments, vocabulary on disk, the interface, and
  the commit messages the agent writes — the whole repository is in English,
  prose included
  ([0024](docs/decisions/0024-everything-in-english.md)). The only French
  left is the conversation between Joel and the agent in the terminal.
- **Design before coding.** As long as a decision a piece of code depends on
  has not been made, the agent does not write it. It proposes, compares,
  recommends, and waits for Joel's call.
- **Record in the right place.** A decision made → a file in
  `docs/decisions/` and a line here. A direction or a question → the note in
  `docs/design/` concerned. A new word → `docs/vision.md`. This file stays
  short.
- **The catalog first.** Faced with a choice, favor what makes the catalog
  more accurate and the branches more readable.
- **Verify before asserting**, especially anything to do with the Spotify
  API: read the official documentation of the moment, and say what could not
  be verified.
- **No secret in the repository.** Spotify credentials, tokens and
  third-party API keys go in a file ignored by git.
- **Sobriety.** No "just in case" file, no dependency without an established
  need, no abstraction before the second use.
- **Commits and pushes as we go** on both forkstify repositories (asked for
  by Joel, 2026-08-31), messages in English
  ([0024](docs/decisions/0024-everything-in-english.md)). The chorizo
  repository keeps its own rule: ask before pushing.
- **Keep `docs/progress.md` up to date** with every notable advance — that
  is what makes it possible to pick the work back up in a new session.
