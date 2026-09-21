# 0002 — The catalog is a set of text files, shared and forkable

**Date**: 2026-08-30 · **Status**: accepted — the "two layers" part is
superseded by [0008](0008-the-fork-is-the-overlay.md)

## Context

Branches need to know which tracks an artist has and which artists are
close. That knowledge could come from an API (Spotify, whose recommendation
endpoints have been closed to new applications since late 2024), from an
opaque local database, or from text files.

The project addresses the Linux ecosystem, where people are used to picking
up configuration files, forking them and adapting them.

## Decision

The catalog is **a set of version-controlled text files**, one per artist,
meant to be **shared and forked**. Everyone takes the catalog, makes it
their own — their top tracks for The Cure are *A Forest* and *10:15 Saturday
Night*, not *Boys Don't Cry* — and can contribute their cards back.

## Consequences

- **The card format is a public interface.** It must be stable, documented,
  readable and editable by hand. See [design/catalog.md](../design/catalog.md).
- Two layers: a shared **base** and a personal **overlay** that always wins.
- The catalog is portable from one installation to the next: cloning it is
  enough.
- Anything derived from the cards (vectors, indexes) is a regenerable cache,
  never a source of truth.
- The catalog and the application have different life cycles; they could
  live in two separate repositories (to be decided).
