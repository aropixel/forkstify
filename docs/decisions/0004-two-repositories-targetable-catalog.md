# 0004 — Two repositories; the application targets or imports a catalog of its choice

**Date**: 2026-08-30 · **Status**: accepted

## Context

The catalog is shared and forkable ([0002](0002-shared-forkable-catalog.md)).
Application and catalog have different life cycles and different
contributors: you do not fork an application to change your top tracks for
The Cure.

## Decision

- **Two repositories**: `forkstify` (the application) and a catalog
  repository (named on 2026-08-31: `forkstify-catalog`).
- The application **imports** a catalog repository — the reference catalog,
  your own fork, or somebody else's "because it looks cool". **Importing =
  cloning**, then declaring "this is the one I use".
- **One active catalog at a time.** You may have imported several and
  **switch** between them whenever you feel like it.

## Consequences

- The application holds no card of its own; it knows a list of locally
  cloned catalogs and which one is active.
- No merging between catalogs: switching means changing worlds. Which makes
  resolving cards a great deal simpler.
- This opens a question about the personal overlay of
  [0002](0002-shared-forkable-catalog.md): if the active catalog is a clone
  you edit and commit to, then the fork *is* the overlay. See
  [design/catalog.md](../design/catalog.md).
