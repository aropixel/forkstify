# 0005 — Cards carry a version number

**Date**: 2026-08-30 · **Status**: accepted

## Context

The card format is a public interface ([0002](0002-shared-forkable-catalog.md)):
catalogs written by different people, at different times, will be read by
different versions of the application ([0004](0004-two-repositories-targetable-catalog.md)).

## Decision

Every card carries a **format version number** (the card's schema), not a
version of its content — git already versions the content.

## Consequences

- The application knows whether it can read a card, and how to migrate it if
  it is older than what it expects.
- A field at the head of the card, `format: 1` for now; the exact name gets
  settled along with the format ([design/catalog.md](../design/catalog.md)).
