# 0007 — Cards are written in TOML

**Date**: 2026-08-30 · **Status**: accepted

## Context

The card format is a public interface
([0002](0002-shared-forkable-catalog.md)): it has to be readable and
editable by hand by people who did not write the application. Candidates:
YAML, TOML, Markdown with front matter.

In Rust ([0006](0006-rust.md)), TOML is first class (`serde` + `toml`);
`serde_yaml` is no longer maintained by its author. Against TOML: lists of
objects (doors, links) are written as `[[table]]`, more verbose than in
YAML.

## Decision

**TOML.** One card = one `.toml` file, with `format = 1` at the head.

## Consequences

- No indentation ambiguity and no implicit typing (the `no` that turns into
  `false` in YAML) — a good thing for cards written by hand by strangers.
- Doors and links are written as `[[doors]]` / `[[links]]`. See the sketch
  in [design/catalog.md](../design/catalog.md).
- Parsing with `serde` + `toml`, no exotic dependency.
