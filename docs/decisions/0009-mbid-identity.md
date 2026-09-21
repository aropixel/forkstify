# 0009 — An artist's identity is their MBID; Spotify is one implementation

**Date**: 2026-08-30 · **Status**: accepted

## Context

The catalog is a **free** artist database that has to outlive the streaming
services ("take back the algorithm"). MusicBrainz is the reference free music
database: stable identifiers (MBID), CC0/ODbL data, and every entry already
links the Spotify, Deezer, Apple Music, Tidal and Qobuz identifiers (checked
on The Cure, 2026-08-30).

## Decision

- **An artist's identity in the catalog is their MBID.**
- **Spotify is one "implementation" among others** — the first pipe plugged
  in, not the foundation. Deezer or others can come later, without touching
  the catalog.

## Consequences

- The card carries the MBID as its key and service identifiers as
  implementation fields (`spotify = "…"`, one day `deezer = "…"`).
- The code separates the engine (which knows only the catalog) from the
  playback and library implementations (which know a service).
- The facts in a generated card are anchored on MusicBrainz / Wikidata,
  which also supply the service identifiers.
