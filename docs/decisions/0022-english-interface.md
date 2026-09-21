# 0022 — The interface is in English

- **Date**: 2026-09-10
- **Status**: accepted — the "documentation stays in French" part is
  superseded by [0024](0024-everything-in-english.md)

## Context

Joel wants to publish a first version of forkstify soon, and wants it
**reachable and usable by as many people as possible**. The code, the card
format, the paths and the application's commits were already in English
([0017](0017-syncing-the-learned.md), AGENTS.md); only the interface —
screens, toasts, hints, command-line output, Omarchy widget — was still in
French.

## Decision

1. **Everything the user sees is in English**: the TUI (home, session,
   discography, search, key tables), messages and errors, command-line
   output, the configuration file written on first run, the bar widget and
   its install script. The engine's short labels (sources `liked` · `tail` ·
   `non-top` · `off-catalog`, reasons `shared members` · `family ties` ·
   `same scene` · `close to the branch's center`) are part of it.
2. **The subcommands follow**: `parcours` becomes `journey`, `ecouter`
   becomes `listen`; `check`, `import`, `vectors` and `merge-learned` do not
   change.
3. **The `[catalogue]` section of the configuration file becomes
   `[catalog]`**; the old name is still read (serde alias), nothing to
   change on existing machines.
4. **The notes the application writes into the cards** (`set while
   listening, <date>`, `linked while listening, <date>`) are in English,
   like its commits: it is the application speaking, not Joel.
5. **The vectorized text does not move** (`src/embed.rs`: "Genres: … Pays:
   …"): it is not an interface, and changing it would invalidate the index
   ([0019](0019-the-application-vectorizes.md)). Neither do the spikes in
   `src/bin/`: those are development tools.

## Consequences

- The documentation, the decisions, the human commits and the design notes
  stay in French; the "Language" line of AGENTS.md is updated.
  *(Superseded on 2026-09-21 by [0024](0024-everything-in-english.md): they
  are in English too.)*
- One single language on screen: the interface's English vocabulary (card,
  catalog, learned, seed, journey, branch, fork, tail, comfort zone, cocoon
  → exploration) becomes the reference for any new visible text.
