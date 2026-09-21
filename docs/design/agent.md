# The agent — an AI driving forkstify from the outside

Note opened on **2026-09-10** on an idea of Joel's: an `:agent` command that
passes a prompt to a connected AI — the Claude Code on his machine, for
instance — of the kind "build me a playlist of 20 tracks in the mood of
Kanye West, Drake, Kendrick Lamar". The AI uses forkstify's tools, reaches
the catalog and starts a listening session, after an exchange if the request
needs clearing up.

Nothing is coded. This note records the direction discussed that same day
and lists what is left to settle; the call is Joel's.

## Decided — what we do not reopen

Nothing is recorded in `docs/decisions/`. But the idea inherits rules that
frame it:

- **Any automatic decision must be explainable in one sentence and
  changeable in one commit** ([vision.md](../vision.md)). A track queued by
  the agent is an automatic decision like any other.
- **Every key is merely the shortcut of a `:` command**, and the commands
  "make everything discoverable and scriptable"
  ([0013](../decisions/0013-keyboard-tuning-measure-or-edit.md)).
- **A broad base *and* on-the-fly generation**: arriving at an artist
  without a card means creating one for them
  ([0016](../decisions/0016-broad-base-and-on-the-fly-generation.md)).
- **No secret in the repository**, and **no dependency without an
  established need** ([AGENTS.md](../../AGENTS.md)).
- **Choosing a branch appends it to the queue** instead of replacing it
  ([usage-feedback.md](usage-feedback.md), 2026-09-06).

## The deciding point — the agent drives, it does not choose in its head

An AI that lines up twenty tracks from memory is Spotify's AI DJ: one more
black box, the opposite of the pitch. The same AI **driving forkstify** —
searching, generating cards, asking for branches, reading their reasons and
queueing — stays within the rule: every track carries a readable reason, and
its passage **leaves a trace in the catalog**.

On Joel's example, that gives:

1. The agent looks for Kanye West, Drake and Kendrick Lamar in the catalog.
   They are not there. It runs `:generate` for each: a MusicBrainz + Deezer
   card, similars in cascade, a vector, one commit each.
2. It asks for the branches proposed from those cards, reads the reasons,
   keeps some, appends to the queue. It may fill in with Spotify tracks "off
   the top of its head", but those arrive **labelled**: a branch it named,
   with its sentence of justification — like a `[spotify]` result in the
   search today, which is distinct from a `[catalog]` one.
3. Result: a playlist of twenty tracks, and **three to ten more cards in the
   fork**. The playlist is the by-product; the catalog has grown. That is
   0016 with one more worker.

## Direction — the agent is outside, not inside

**Forkstify embeds no LLM client, no API key and no model choice.** Three
reasons: secrets (none in the repository, none in the binary), sobriety (an
HTTP dependency and an SDK for a convenience feature), and
interchangeability — the AI is a pipe the way Spotify is, it has to be
swappable without touching the core.

Two floors, in this order:

### Floor 1 — forkstify becomes drivable from outside, with no `:agent`

- **A control socket** for the running TUI (in the `XDG_RUNTIME_DIR`
  directory), and a subcommand of the kind
  `forkstify cmd ':generate Drake'` that talks to it — or that runs the
  engine dry if there is no live TUI.
- **A few reads in JSON**: the state (what is playing, the queue, the
  comfort, the size), a card, the neighbors (`check`), the branches proposed
  from an artist or from the end of the queue, the discography.
- **The agent's API is the `:` commands.** Everything the agent does is a
  command the user could have typed; the log reads like a keyboard session.
  That is 0013's "scriptable" promise, kept once and for all — and it serves
  without an AI too (a script, a Hyprland shortcut, something `playerctl`
  -like).
- **The conversation happens in the agent's terminal.** Joel types his
  request into Claude Code, which calls the binary through Bash and already
  knows how to ask a question before acting. A skill or a section of
  `AGENTS.md` documents the commands and the contract (below).
- **No MCP server at this stage.** It only makes sense if a second host
  besides Claude Code shows up — no abstraction before the second use.

### Floor 2 — `:agent <prompt>` in the TUI

- The command launches **in the background** the agent configured in
  `config.toml` (`[agent] command = "claude -p …"`, for instance), with the
  prompt and the socket as a tool. The agent is an external command: Claude
  Code today, something else tomorrow, without changing forkstify.
- **The music does not stop** during the thirty seconds to two minutes it
  takes (principle 1 of the vision). The agent **appends to the end of the
  queue**, it does not replace — the "chaining" queue of 09-06 is made for
  that. The screen footer signals "agent running".
- Its **questions** come back in a modal, on the discography's pattern
  (`keys::parse_modal`); the answer goes back to the same agent (resumed
  session). Floor 2 only gets coded if floor 1 has proved that the round
  trip through Claude Code's terminal is too heavy in practice.

### The agent's contract

- **Every branch it queues carries its name and its reason** in one
  sentence, shown like the engine's reasons. Tracks drawn by the engine keep
  the engine's reasons; those "off the top of its head" are marked as such.
- **`u` undoes its whole contribution at once**, like a single edit (0013) —
  the queue goes back to what it was before the request.
- **Its commits on the catalog are recognizable**: the trailer of
  [0017](../decisions/0017-syncing-the-learned.md) carries a `kind` naming
  it (`Forkstify: agent <version>`, to be specified), so that we always know
  what came from it — and can read it back.
- **A ceiling on the cards generated per request**, otherwise "US rap mood"
  generates forty of them and brings MusicBrainz and Deezer down on our
  backoffs.
- **It prefers the catalog to its memory**: generate and branch before
  quoting. Its memory serves to *choose* between branches and to name a
  mood, not to replace the engine.

## What this reopens

- **Saving the playlist** (question 0 of
  [usage-feedback.md](usage-feedback.md)) comes back on the table: a
  playlist asked for in one sentence will want keeping.
- **Review by an LLM**, which [branch-engine.md](branch-engine.md) raises
  for naming a vector connection with no written link: that may be the
  agent's real job. Reviewing generated cards, writing the missing
  descriptions, proposing a link with its note — commits reviewed by a
  human, exactly what the catalog is waiting for. The playlist is a pleasant
  pretext; the review makes the branches more accurate.

## To settle

1. **The ceiling** on cards generated per request (5? 10?), and whether it
   is set by comfort or by the call.
2. **The trace in the queue**: one branch name per request ("agent: Kanye /
   Drake / Kendrick mood") with the engine's reasons under each track, or
   one branch per direction found?
3. **The fate of the "off the top of its head" tracks**: allowed and marked
   (proposed), or forbidden — the agent can only queue what the engine or
   the search hands it.
4. **The command's name**: `:agent` (proposed) or `:ask`, more neutral about
   who answers.
5. **The trailer's `kind`** for the agent's commits, and whether a card
   generated at its request is distinguished from one generated by hand (the
   `generated = true` field may be enough).
6. **Whether floor 2 is necessary** — to decide after living with floor 1
   from Claude Code.
