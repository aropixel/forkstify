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

## Suggested connections — the agent proposes `ac`, you say yes

Discussed on **2026-10-06**, on an idea of Joel's: proposals of `ac`
connections drawn from the artists he likes, so that he does not have to
think of them.

### The problem, on a real case

Joel wanted to connect **Arthur Satàn** to **The Beatles** or **The Kinks**
"without having to think of it": otherwise Arthur Satàn hardly ever comes
up, and listening to the Beatles always leads back to the same four or five
artists. The catalog shows both halves:

- **An orphan.** Arthur Satàn's card has four links, all Deezer similars
  (Porridge Radio, King Hannah, Cate Le Bon, Ghostwoman) that miss the
  point, and one tag (`fr`), so its vector is blurry too. **No card leads
  to it.** Neither the links nor the vectors can bring it up.
- **A closed loop.** The Beatles' card has two links (the Stones, the
  Kinks), the Kinks' four lead back to the Beatles, the Zombies, Donovan
  and Dylan: five or six artists going round.

Joel had drawn `the-beatles → arthur-satan` and `the-kinks → arthur-satan`
at 3 by hand. The feature is that those two be **offered** to him.

The kinship here is one of sound — sixties English pop, melody, a Ray
Davies irony. No database carries it, any ear hears it: it is neither a
MusicBrainz fact nor a pure whim, and it is exactly what an LLM is good at
spotting.

### Direction (agreed by Joel, 2026-10-06)

1. **The engine finds the cases, with no AI.** It knows where things get
   stuck, deterministically:
   - **orphans**: artists the listener likes (liked tracks, plays) that
     nothing leads to;
   - **loops**: artists whose branches keep landing on the same small
     group.
2. **The agent proposes, among the listener's own catalog only.** For each
   orphan it is handed the liked artists that have a card, and picks one to
   three anchors, each with a closeness and a one-sentence reason — "Arthur
   Satàn → The Kinks, 3: sixties English pop, melodies, a Ray Davies
   irony". It invents no name, it matches: the agent drives, it does not
   choose in its head (above).
3. **The listener says yes in one key.** The proposals wait in a list (a
   "suggested" section of the `ac` modal, or a dedicated `:` command): ⏎
   accepts at the proposed closeness, `h`/`l` correct it first, `x`
   refuses, and **a refusal is remembered** so the proposal does not come
   back. Nothing to imagine, nothing written without a yes.
4. **The listener chooses where it goes, proposal by proposal** (Joel,
   2026-10-06): the agent proposes, the listener decides whether to accept
   it *and how*. Two destinations:
   - **a link in the card** — a typed link (`influence`, `similar`…) with
     its closeness and the agent's sentence as its `note`, committed into
     the fork. It is knowledge: it leaves with `Cp`, the review is the pull
     request (0025), and since a card's links feed the vectorized text, it
     also sharpens the orphan's vector. Written in the orphan's card alone
     is enough: the engine walks links both ways, so `arthur-satan →
     the-beatles` brings Arthur Satàn up from the Beatles too;
   - **an `ac`** — in `learned/`, like one drawn by hand: taste, never
     carried by `Cp`, invisible to the vectors.

   The agent may say which one it leans to (a typed kinship of sound reads
   like a card link), but it never settles it.
5. **On demand first** — `:suggest run`, which calls Claude Code from
   forkstify (2026-10-07, below; the first idea was a subcommand run from
   Claude Code, floor 1). In the background after a like, perhaps later.

### The mockup (Claude Design, 2026-10-06)

Project board **`Propositions.dc.html`**, from a brief that only described
the feature. Joel, 2026-10-07: "the mockup gives a graphic orientation;
you stay master of the changes to what it proposed". It shows:

- **1a** — the proposals as a section of the `ac` modal, between the
  connections drawn and the search. Its own note says the limit: nothing
  leads you to open `ac` on an orphan.
- **1b** — **`:suggest`, marked recommended**: one list for every artist,
  grouped by case (orphans, loops, audience links), with the date of the
  agent's run and "2 pending · 0 declined"; `:suggest run` launches Claude
  Code, and forkstify does not wait for it to stay playable.
- **2a** — the closeness: the `ac` question as it is, the agent's sentence
  above it, `▸` starting at the agent's closeness instead of 4.
- **2b** — the destination: `c` a card link, `a` a connection, each line
  saying what it entails; `▸` marks the agent's lean, nothing is chosen.
- **2c** — for a card link: the type, then the diff shown before writing,
  `y` writes and commits.
- **2d** — what a connection and a refusal write, and the echo after.

### Direction after the mockup (2026-10-07)

What is kept, and what is corrected against the code and the decisions:

1. **`:suggest` is the screen** (1b). The slice in the `ac` modal (1a) is a
   second door to the same list; it waits until the first one is used.
2. **`:suggest run` calls Claude Code directly** — floor 2 for this
   feature, without floor 1: no control socket, no JSON subcommands. It is
   simpler than floor 1, not heavier. forkstify:
   - computes the orphans and, for each, the candidates (below), with no
     AI;
   - runs `claude -p` **in the background**, the music going on, with
     `--tools ""` (no tool at all: no file, no command, no MCP),
     `--output-format json`, `--json-schema` for the shape of the answer,
     `--no-session-persistence`. Options checked on Claude Code 2.1.285;
   - reads the answer, **keeps only the proposals whose two slugs were in
     what it sent**, and files them.

   Text in, JSON out: the agent can name no artist the catalog does not
   hold, and it touches nothing. Claude Code uses its own login on the
   machine: no key in forkstify, no connection screen. Nothing is
   configured either: the command is `claude`; if it is not on the `PATH`,
   `:suggest run` says so. Another agent is a second use, and an
   abstraction then.
3. **What the agent receives**: for each orphan, its card (name, tags,
   origin, years, links); and the **liked artists that have a card**, each
   with its name, slug and tags. **What it returns**, per proposal: the
   orphan, the anchor, a closeness (1–5), **a type**, its lean (card link
   or connection), and the reason in one sentence.
4. **The agent proposes the type, and only a reading.** The mockup started
   `▸` on the first type of the chosen closeness — `family` at 3, which
   would commit "Arthur Satàn and the Kinks are family". `member`,
   `collab` and `family` are **facts**, and facts never come from a
   language model ([catalog.md](catalog.md)). The agent picks among
   `scene`, `influence` and `similar`; 2c offers those three, `▸` on the
   agent's.
5. **A card link is one line**, as 0010 has it — not the mockup's
   `[[links]]` block, and in `cards/`, not `artists/`:
   `{ to = "the-kinks", type = "influence", proximity = 3, note = "sixties English pop, melodies, a Ray Davies irony" }`.
   `proximity` is written only when it differs from the grid's.
6. **The trace.** A card link is an edit: its commit is an ordinary one
   (`Forkstify: edit`, 0017), and its body says "suggested by the agent,
   <date>". A connection keeps the shape of 0014 — `the-beatles = 3` in
   the artist's file —, so it carries no trace: once accepted, it is the
   listener's like any other.
7. **Where things live.** The **refusals** are remembered in `learned/`,
   in the orphan's own file, beside its connections — they follow the
   listener between machines and merge the same way. The **pending
   proposals** are local state (`$XDG_STATE_HOME/forkstify/`): a run is
   cheap, and a list waiting on one machine has no reason to travel.
8. **The orphans, and nothing else for now.** An orphan is an artist the
   listener likes that **no link, `audience` aside, ties to another artist
   they like** — card links and connections, both ways, since the engine
   walks both. No count, so no threshold to tune. "No link at all" was
   tried first and missed the point: on 2026-10-07 it gave 16 orphans and
   left out Calexico, Joel's most liked artist, whose three links lead to
   Giant Sand, Iron & Wine and Wilco — none of them liked, so Calexico
   never comes up from where he listens. The definition above gives 52.
   **The candidates are the liked artists that are not orphans**: an
   anchor has to be reachable, or two orphans only make an island. The
   loops come off the screen: since the draw of 2026-10-06 takes every
   link, the Beatles' loop may well be gone, and the agent receives no
   loop anyway. The audience section waits for its own feature, as the
   mockup has it.
9. **A ceiling per run**: the ten most liked orphans. The prompt stays
   short, the answer comes in under two minutes, and the next run takes the
   next ones.
10. **The keys**, from the mockup, local to the screen: `j`/`k` move, `⏎`
    accepts (closeness, then destination, then for a card link the type and
    the diff), `x` declines, `esc` goes back. In 2b, `c` card link, `a`
    connection; in 2c, `h`/`l` the type, `y` writes and commits — the `Cp`
    gesture. The mockup's `2` "play from here" goes: it plays nothing that
    the list is for.

### On one artist, on demand (2026-10-07)

Joel, after reading the full answers: "there are few proposals, because
of the artists targeted at first. I would like to ask for a suggest on a
targeted artist (on demand)". The orphans are only the cases the engine
finds; Expérience (tied to Michel Cloup alone, audience aside) or the
Beatles (fifteen links, the same few played) are not orphans, and are
exactly where the listener feels the lack.

- **`:suggest <artist>`** asks for one artist, any artist with a card,
  orphan or not; **`aS`** is its key (proposed: `S` for *suggest*, free in
  `a` since `aL` left), aimed by 0020's rule — the highlighted row,
  otherwise what plays. `:suggest` alone stays the list, `:suggest run`
  the orphans. The proposals of a targeted run join the same list, under
  the artist.
- **The prompt changes in three places** (text below): one TARGET instead
  of ORPHANS, its links both ways shown; **up to five** anchors instead of
  three; the candidates are the liked artists **not yet tied to it**
  (card links both ways and connections, `audience` aside). An orphan
  stays a candidate unless the target is an orphan too — tying an orphan
  to a reachable artist is what rescues it.
- **Tried on Expérience and the Beatles**, 10–14 s and about US$0.11
  each:

| Target | Anchor | Type, closeness | Lean | Reason |
|---|---|---|---|---|
| Expérience | Astéréotypie | similar, 4 | connection | Spoken French words over tense, swelling post-rock guitars. |
| Expérience | Miossec | scene, 4 | card | Raw, half-spoken French indie rock with bleak, confessional lyrics, 90s–2000s. |
| Expérience | Dominique A | scene, 3 | card | French indie scene that renewed chanson with minimal, noisy rock arrangements. |
| Expérience | Noir Désir | scene, 3 | card | Southwest French alternative rock with angry, literary, politically charged lyrics. |
| Expérience | La Dispute | similar, 3 | connection | Spoken-word vocals that build into loud, cathartic guitar climaxes. |
| The Beatles | Elliott Smith | influence, 4 | card | Beatlesque melodies, layered harmonies and McCartney-style chord turns in intimate songwriting. |
| The Beatles | Tame Impala | influence, 3 | card | Revolver-era psychedelic pop: phased vocals, swirling production, Lennon-like melodies. |
| The Beatles | Blur | influence, 3 | card | Britpop built on Beatles-style British melodic songwriting and studio playfulness. |
| The Beatles | Eric Clapton | scene, 3 | card | Same sixties British rock scene, blues-rooted guitar alongside British Invasion pop. |

  Arthur Satàn is rightly absent from the Beatles' list: Joel's `ac`
  already ties them.

The targeted instructions differ from the orphans' in their second
paragraph and their third rule:

```
The listener asked for suggestions around one TARGET artist. Its links today do not reach far enough into what they listen to. Choose up to five ANCHORS from the CANDIDATES list: artists the listener likes, not yet tied to the target, whose music is genuinely close to the target's. A good anchor is one a listener who loves the anchor would be glad to hear the target right after, and the other way round.
...
- Fewer is fine. If no candidate is genuinely close, return no suggestion. An empty answer is better than a weak one.
```

followed by `TARGET` (its card, its links, who links to it) and
`CANDIDATES`.

**Wired on 2026-10-07**: `aS` and `:suggest [artist] [again]` (`src/suggest.rs`),
the screen in the overlay block, the refusals in `learned/` (`declined`,
merged as a union), the waiting list in `$XDG_STATE_HOME/forkstify/suggestions.json`.
The general run followed the same day, on Joel's ask ("I want to be able
to launch a general `:suggest`, with no argument or with `all`"):
`:suggest` and `:suggest all` ask about the orphans — the most familiar
first, ten per call, any orphan already answered from the same card and
liked artists skipped, even when nothing was kept for it —, then show
everything that waits; `:suggest waiting` is the list alone. A refusal
travels in the orphan's line ("declined: …") and is filtered out of the
answer again. On the real catalog: Calexico, Flotation Toy Warning,
Nasser, Cheveu, Bad Bunny, La Ruda Salska, Kid Francescoli, Kevin Morby,
Soko, Spook and the Guay — 21 proposals in 25 s.

### What a call costs, and when not to make one (2026-10-07)

Joel: "aren't these calls included in the subscription? Shouldn't we
cache them, so as not to make them again every time?"

- **They are in the subscription.** Claude Code is logged in through
  claude.ai on Joel's machine (`claude auth status`: `"authMethod":
  "claude.ai"`, no `ANTHROPIC_API_KEY`): a `claude -p` call counts against
  the plan's usage limits like any session. The `total_cost_usd` it
  reports is an estimate at API prices, not a charge — only extra usage,
  if it is enabled on the account and the limits are passed, is paid on
  top. Not checked against the account page. A call still spends a share
  of the limits, which is the reason not to make it twice.
- **The waiting list is the cache.** Proposals received are kept; one
  accepted or declined never comes back. What is added is the rule for
  not calling again:
  1. **`aS` on an artist that still has proposals waiting shows them**,
     with no call.
  2. **The agent is called again only when** nothing is left waiting for
     that artist, or **what would be sent has changed** — the target's
     card or the list of liked artists. A fingerprint of the prompt says
     so, as the vector index already does for a card's text.
  3. **An explicit ask forces a new call** when the listener wants other
     ideas: `:suggest <artist> again`, or `r` on the screen.
  4. **What was declined or accepted leaves the candidates** of the next
     call. The answers vary from one run to the next (two runs on the same
     orphans shared about half their anchors), so a new call then brings
     what is new rather than what was already settled.
- The cache lives with the waiting list, in local state, and does not
  travel: on another machine it costs one call at most.

### To settle

1. **Thin cards** like the Beatles' (two links of their own): should the
   agent also propose links for them? That is the "review by an LLM" part
   below.
2. **When the list is seen**: only through `:suggest`, or does the home or
   the bar say "3 suggestions waiting"?
3. **The lean** came out "connection" for nearly everything in the trial,
   kinships of scene included: the instruction does not bite. Harmless —
   the listener decides —, but to rework or drop.

### The prompt, tried on 2026-10-07

Run on Joel's catalog, ten orphans, the 181 non-orphan liked artists as
candidates (about 24 kB of prompt):

```
claude -p --safe-mode --tools "" --output-format json \
  --json-schema "$(cat schema.json)" --no-session-persistence < prompt.txt
```

`--safe-mode` keeps the user's own `CLAUDE.md`, skills, hooks and MCP
servers out of the call while keeping the login — `--bare` would too, but
it only takes an API key. The answer is in `structured_output`. **About 28
seconds, US$0.14–0.17 as reported** (`total_cost_usd`, on the default
model, Opus 5.5). Every anchor of the two runs was in the list sent; the
second run, candidates without the orphans, gave 21 proposals and left
Brutus empty, as the prompt allows. A sample:

| Orphan | Anchor | Type, closeness | Reason |
|---|---|---|---|
| Calexico | Tindersticks | similar, 3 | Cinematic, brass-tinged slow songs with a dusky, widescreen melancholy. |
| Cheveu | Arthur Satàn | scene, 4 | Both come from France's 2000s lo-fi garage-punk underground. |
| Fishbach | Étienne Daho | influence, 3 | Dark, eighties-flavoured French synth-pop with dramatic chanson vocals. |
| Traband | Les Négresses Vertes | similar, 3 | Punk-rooted acoustic folk with brass, accordion and Balkan swing. |
| Godspeed You! Black Emperor | Arcade Fire | scene, 3 | Both came out of Montréal's indie scene with grand orchestral ambitions. |

The first run, candidates including the other orphans, tied Traband to
Vladimír Václavek — right, and useless: two orphans.

The instructions, followed in the prompt by `ORPHANS` (each card: name,
slug, tags, origin, years, current links) and `CANDIDATES` (name, slug,
tags, origin, years):

```
You suggest connections between artists for forkstify, a music player that plays by branches: from an artist, it moves to the artists linked to it.

Each ORPHAN below is an artist the listener likes, but no link leads from any other artist they like to it, so it hardly ever plays. For each orphan, choose one to three ANCHORS from the CANDIDATES list: artists the listener also likes and already reaches, whose music is genuinely close to the orphan's. A good anchor is one a listener who loves the anchor would be glad to hear the orphan right after.

Rules:
- Anchors must come from CANDIDATES, by their slug, exactly as written. Never name any other artist.
- Judge by the music: sound, songwriting, era, scene, lineage. The tags are hints from MusicBrainz and are sometimes wrong or missing; trust your knowledge of the artists over them. Ignore shared country or language alone.
- If no candidate is genuinely close, return no suggestion for that orphan. An empty answer is better than a weak one.
- type: "similar" (they sound alike, they go together), "scene" (same scene, same moment, same circle), or "influence" (one descends from the other). Never claim a fact (members, collaborations, family): those come from other sources.
- proximity, 1 to 5: 1 a distant echo, 2 an influence far back, 3 a family or a scene, 4 they go together, 5 almost the same universe.
- lean: "card" when the kinship is knowledge anyone could check, "connection" when it is more a matter of taste.
- reason: one short sentence in English, saying what the two share musically. No hedging, no filler, at most 15 words.
```

The schema:

```json
{"type":"object","additionalProperties":false,"required":["suggestions"],"properties":{"suggestions":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["orphan","anchor","type","proximity","lean","reason"],"properties":{"orphan":{"type":"string"},"anchor":{"type":"string"},"type":{"enum":["similar","scene","influence"]},"proximity":{"type":"integer","minimum":1,"maximum":5},"lean":{"enum":["card","connection"]},"reason":{"type":"string"}}}}}}
```

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
   trailer of the birth commit, since 0025 leaves no flag in the card).
6. **Whether floor 2 is necessary** — to decide after living with floor 1
   from Claude Code.
