# 0019 — The application does its own vectorizing

- **Date**: 2026-09-09
- **Status**: accepted

## Context

[0016](0016-broad-base-and-on-the-fly-generation.md) made on-the-fly
generation the main path by which a catalog grows, and left the fate of a
generated card's **vector** open. It was born without one: navigable
through its links and tags, but deaf to everything that goes through
meaning — `vector_neighbors`, a branch's centroid, the adventurous branch.
The damage was silent and cumulative: the more you listen, the less the
vector space covers the catalog.

Two ways out were compared in
[design/on-the-fly-generation.md](../design/on-the-fly-generation.md): a
command launching the Python container in the background, or computing
inside the application with `fastembed` in Rust. The first is a chore you
forget, and automating it would make docker and python runtime
dependencies of a music player. The second was measured by a trial on
2026-09-09: cosine ≥ 0.999999 against the Python index over the 316 cards,
provided the input is truncated to 128 tokens; cold build 26 s; binary
+35 MB; `g++` in the build image; a 241 MB model downloaded on first use.
The trial also revealed that the Python index was **not normalized** (norms
from 2.5 to 3.6), which the engine's centroid was suffering from.

## Decision

- **The application computes the vectors itself** (`embed.rs`): same model
  (`paraphrase-multilingual-MiniLM-L12-v2`, quantized, 384 dimensions, mean
  pooling), same text composed from the card's structure as
  `tools/vectoriser.py`, **truncation at 128 tokens**, **normalized**
  vectors.
- **A generated card is born with its vector, in the same commit**
  `Forkstify: edit`. If the model cannot be reached, the card comes in
  anyway, the screen says "without vector", and `forkstify vectors` catches
  up.
- **`forkstify vectors` regenerates the whole index** from the cards and
  writes `vectors/meta.toml`. It replaces `tools/vectoriser.py`: the
  reference and the forks vectorize with the same binary, which is the
  strongest possible form of "the same vectors everywhere"
  ([catalog.md](../design/catalog.md)).
- **An import regenerates the index within its commit**, with no docker.

## Consequences

- The build image carries `g++` (ONNX Runtime, statically linked); the
  binary depends on `libstdc++.so.6` at runtime, present everywhere.
  `fastembed`'s default features pull OpenSSL in: we take the `rustls`
  variants, like the rest of the binary.
- The model lives in `$XDG_CACHE_HOME/forkstify/fastembed`, never in the
  catalog. Downloading it the first time needs the network, which
  generation requires anyway, and the screen says so beforehand.
- The reference index was regenerated once by the application
  (2026-09-09): same directions, norms at 1.
- **Still to do**: editing a link in session (`aL`) changes the text of both
  cards and does not recompute their vectors yet — an edit only counts for
  the engine at the next launch anyway; `forkstify vectors` covers the case
  in the meantime.
