# Build environment for forkstify. The binary embeds librespot, whose
# audio backend (rodio → alsa) links libasound at build time, so the
# builder needs the alsa headers and pkg-config on top of the Rust image;
# fastembed links ONNX Runtime (C++) statically, which needs g++. git is
# there for the tests of fork.rs, which drive real repositories.
#
#   docker build -t forkstify-build .
#   docker run --rm -v "$PWD":/app -w /app \
#     -v forkstify-cargo:/usr/local/cargo/registry forkstify-build \
#     cargo build --release
#
# The produced binary runs on the host and only needs libasound.so.2 and
# libstdc++.so.6 at runtime (present on any desktop Linux).
FROM rust:1-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libasound2-dev g++ git \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add clippy rustfmt
