# 0024 — Everything the repository holds is in English

- **Date**: 2026-09-21
- **Status**: accepted · **Supersedes** the "Language" part of
  [0022](0022-english-interface.md)

## Context

[0022](0022-english-interface.md) drew the line between what the user sees —
English — and what the project says to itself — French: documentation,
decisions, design notes, human commits. That line held as long as the
repository was Joel's workbench.

It stops holding at publication. Someone who arrives through the README
reads English, then opens `docs/` and hits a wall: the decisions that
explain *why* the engine works the way it does, the design notes that a
contributor needs before touching anything, the progress note that says
where things stand. The part of the repository that is hardest to
rediscover was the part nobody outside could read. A repository aimed at
open source cannot keep its memory in a language its readers do not have.

Joel, 2026-09-21: "convert every document of the project into English, and
state in AGENTS.md that everything generated must be generated in English".

## Decision

1. **Everything the repository holds is in English.** Prose included: the
   documentation, the decisions, the design notes, `AGENTS.md` itself. No
   surface of the repository is in French any more.
2. **Everything generated is generated in English** — by the application as
   [0017](0017-syncing-the-learned.md) already had it, and by the agent:
   documents, decisions, notes, code, and the commit messages it writes.
   What is produced here is produced in English, with no exception to ask
   about.
3. **File names follow.** `docs/conception/` becomes `docs/design/`,
   `docs/avancement.md` becomes `docs/progress.md`, `docs/atouts.md`
   becomes `docs/strengths.md`, `docs/reglages.md` becomes
   `docs/tuning.md`, and every decision keeps its number with an English
   slug. Paths are a public interface, as
   [0014](0014-shape-of-the-learned.md) already said of the ones on disk.
4. **Translating a decision does not revise it.** A decision is immutable in
   substance: translation changes the language, never the content, the date
   or the status. Names on disk quoted in an old decision keep the spelling
   they had that day — `catalogue.toml`, `fiches/`, `usage/` — since they
   are dated facts; the decision that renamed them says so.
5. **French remains the language of the exchange between Joel and the
   agent**, in the terminal. It is what is written down that goes into
   English.

## Consequences

- The 44 documents of the repository are translated in one pass, and the
  cross-links are rewritten onto the new paths. `git mv` keeps the history:
  `git log --follow` still reads on every file.
- The vocabulary of [`docs/vision.md`](../vision.md) gains its English
  terms — seed, branch, fork point, segment, journey, comfort zone, card,
  top, door, link, base / mine / learned, promotion — which were already
  the ones on disk and on screen. "Poncer" survives as the French slang
  behind the `e` key, and nothing else.
- The AGENTS.md rule is one line: everything generated is in English. The
  "Language" bullet of 0022 is no longer to be read as the rule.
- What this does not change: the vectorized text (`src/embed.rs`) stays as
  it is, for the same reason as in 0022 §5 — changing it would invalidate
  the index ([0019](0019-the-application-vectorizes.md)).
