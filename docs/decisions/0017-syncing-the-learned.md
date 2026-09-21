# 0017 — The learned layer syncs itself: commit, push, pull, merge by counter

- **Date**: 2026-09-07
- **Status**: accepted

## Context

Joel listened for a whole day on one machine, `learned/` filled up, then he
moved to the other machine: nothing had been committed, and the learned data
stayed behind. On the other machine, ten files in `learned/artists/` were
waiting the same way, never committed. Two machines, two diverging learned
layers, neither one pushed.

That was half intended: [0014](0014-shape-of-the-learned.md) makes the
learned layer a **silent** one, written without confirmation; only card
edits commit ([0013](0013-keyboard-tuning-measure-or-edit.md)). Feedback
item no. 10 had reserved `:sync`, `:push` and `:pull` without settling the
frequency or, above all, the **merge**: two machines writing
`odezenne.toml` end in a textual conflict, and a `git merge` makes no sense
at all on decayed counters. The machine has moreover **no backup**: learned
data that is not pushed is learned data at risk.

## Decision

1. **The application commits and pushes the learned layer itself.** Pull on
   start (after committing what was learned here, so the rebase has
   something to merge), an autosave commit **every ten minutes** of
   listening if `learned/` moved, commit and push **on exit**, and `:sync`
   on demand. The push happens in the background; offline, listening carries
   on and says so, and the push waits for next time. Network calls have a
   short timeout (`ConnectTimeout=5`) so the screen never hangs.
2. **Merging happens by counter, not by line.** A git merge driver
   (`merge=learned` on `learned/artists/*.toml`, driven by
   `forkstify merge-learned <base> <ours> <theirs>`) does a three-way merge:
   the plays on each side since the common ancestor **add up**, decayed to
   the day of the merge; `last` takes the more recent; a ban or a like set
   on either side **holds**; the weight follows the side that moved it; a
   top new on one side comes in as is. The driver is declared by the
   application in the clone's config (never version controlled) and the
   attribute in the reference repository's `.gitattributes`, with
   `.git/info/attributes` as a fallback for a clone that lacks it.
3. **Learned files are written sorted.** Hash map ordering produced spurious
   diffs on every write; tops now live in a `BTreeMap`.
4. **The commit messages the application produces are in English**, like the
   card format and the vocabulary on disk: they are a public interface of
   the repository (Joel, 2026-09-07). The project's human commits stay in
   French. Subject: `learned: 3 artists`, `import: 12 cards from <remote>`;
   an edit keeps the displayed sentence as its subject.
5. **Every commit from the application carries a trailer** `Forkstify: <kind>
   <version>` (`learned`, `edit`, `import`). That is what will make it
   possible to count usage across public forks: GitHub's commit search
   indexes the messages of public repositories —
   `gh api search/commits -f q='"Forkstify:"' --jq .total_count` — and the
   reference repository's fork count counts the users.

## Consequences

- 0014 is not revised: the learned layer stays silent when written and never
  enters a PR; it is simply **pushed** to the user's fork, which makes it
  portable from one machine to the next.
- `:sync` and `:push` are wired (the same gesture); `:pull` has no reason to
  exist on its own, the pull happens on start.
- A rejected push (the other machine pushed in the meantime) waits for the
  next pull, which rebases and merges. Nothing is ever lost, nothing is ever
  overwritten.
- The very first reconciliation between Joel's two machines happens by
  launching forkstify on each: the first commits and pushes, the second
  commits, rebases and merges through the driver.
- Known limits: GitHub's commit search only sees the default branches of
  **public** repositories; a private fork is not counted. The trailer says
  nothing more than "forkstify wrote this", not who.
