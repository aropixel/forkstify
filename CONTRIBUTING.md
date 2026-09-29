# Contributing to forkstify

**Open an issue, not a pull request.** forkstify works on an *issue-first*
model: contributions arrive as **well-described issues**, a maintainer
triages them, and the change is then implemented — by an agent, most of
the time — and **reviewed by a human** before it lands.

Pull requests opened from outside the maintainers are closed
automatically, with a pointer back here. That is not a judgement on the
code: it is that a patch is the *expensive* half of a contribution and
the *cheap* half is the one that is missing.

## Why

An unsolicited patch costs a maintainer more than it saves. It has to be
read, held against decisions it could not know about, rebased, argued
over, and often rewritten — while the thing that was actually scarce was
the **understanding**: what breaks, where, under which conditions, and
what it ought to do instead.

Writing code stopped being the bottleneck. Knowing precisely what to
write never did. So forkstify asks for the part that is still hard.

An issue also survives. It is where the discussion lives, where the
reason is written down, and where the next person looks. A merged patch
with no issue behind it leaves the reasoning in a diff, which nobody
reads twice.

## What a good issue looks like

**Use your own agent to prepare it.** Point Claude Code, or whatever you
use, at your clone; let it reproduce the problem, read the surrounding
code, and write the issue with you. Precision is the whole contribution
here, and an agent is good at precision.

Then, before you post it:

- **Say what you verified yourself.** An issue written by an agent and
  never checked is noise, and it costs more to triage than it saves. Run
  it. Read it. Say which parts you confirmed and which you did not.
- **Name the files and functions** you believe are involved, with paths,
  and say how sure you are.
- **Give the smallest reproduction** you can. For the player, that means
  the keys pressed, the state of the queue, and what the screen said.
- **Say what should happen instead**, and why — in one sentence if you
  can. Every automatic decision in forkstify has to be explainable in one
  sentence and changeable in one commit; a change to one is held to the
  same bar.
- **Propose an approach** if you have one, and say what you would not do.
  A rejected approach, argued, is worth as much as a chosen one.

You are welcome to attach a patch or a branch **as a reference** inside
the issue — a link, or a diff in a code block. It will be read as
evidence of what you mean, not as something to merge.

## Before opening one

- Read [`docs/vision.md`](docs/vision.md) for what the product is, and
  [`docs/decisions/`](docs/decisions/) for what is already settled. A
  decision is never rewritten: to go back on one, a new decision
  supersedes it. If your issue contradicts a decision, say which one and
  why it should be superseded — that is a legitimate issue, and a good
  one.
- Check [`docs/progress.md`](docs/progress.md): it may already be known,
  or already done and waiting for a release.
- Search the open issues.

## The catalog is a different repository

Cards, links, tags, tops — anything about **which artists exist and how
they connect** — belong to
[forkstify-catalog](https://github.com/aropixel/forkstify-catalog), and
that repository takes **pull requests**: a card is data you can read in
full, and the review *is* the pull request
([0025](docs/decisions/0025-the-review-is-the-pull-request.md)). Only this
repository, the application, is issue-first.

## Security

Do not open a public issue for a vulnerability. Write to the address in
the repository's profile instead, and say what you found and what it
lets someone do.

## Maintainers

Maintainers — the repository's owner, the organization's members, and
anyone given write access — open pull requests as usual. The rule above
is about unsolicited patches from outside, nothing else.
