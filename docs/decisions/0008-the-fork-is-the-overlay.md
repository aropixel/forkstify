# 0008 — No separate overlay: the fork is the overlay

**Date**: 2026-08-30 · **Status**: accepted · **Supersedes** the
"two layers" part of [0002](0002-shared-forkable-catalog.md)

## Context

[0002](0002-shared-forkable-catalog.md) called for two layers: a shared base
and a personal overlay applied on top. Then
[0004](0004-two-repositories-targetable-catalog.md) settled the import
model: you clone a catalog, you declare it active, one active at a time.

With that model, my changes naturally live in the clone: my top tracks for
The Cure are a commit on my fork. A separate overlay would keep my tops when
I switch to somebody else's catalog — the very opposite of "this is the one
I use".

## Decision

**There is no separate overlay. The active catalog is a git clone, and
personal changes are commits inside it.** Making the catalog your own means
forking it in the literal sense.

## Consequences

- A single resolution rule: the card from the active catalog, full stop. No
  field-by-field merging.
- The application helps **commit** (a card edited or generated) and **pull
  upstream updates** (`git pull` from the origin repository, conflict
  resolution). That is where the complexity we removed elsewhere goes.
- Switching catalogs changes everything, personal tops included. That is
  intended.
