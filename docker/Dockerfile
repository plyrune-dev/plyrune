# syntax=docker/dockerfile:1
# Plyrune's tool image: one image for every gate, so that nothing nests Docker.
#
# Base: the public tauten proofs toolchain image, pinned by the digest resolved
# and recorded in docs/ADOPTION.md (Lean v4.35.0, Aeneas nightly-2026.11.07-c5eb4a2,
# Charon 8cae8fd on nightly-2026-09-17, mathlib as in proofs/lake-manifest.json).
#
# Added on top: the pinned Rust toolchain (clippy, rustfmt) and cargo-tauten built
# from the pinned tauten revision with --locked, as tauten's own tool image does
# (docker/Dockerfile at bac9fdd, MIT).
#
# The image's default Rust toolchain stays Charon's nightly, which the proofs gate
# translates with; every Plyrune gate names 1.98.0 explicitly.
FROM ghcr.io/tauten-dev/proofs-toolchain@sha256:5c1fd168f399e028db85e4a76c8e742abc7bdf93207e5270e54b36636b40a2d8

ARG TAUTEN_REPOSITORY=https://github.com/tauten-dev/tauten
ARG TAUTEN_REVISION=bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42
ARG RUST_TOOLCHAIN=1.98.0

RUN --mount=type=cache,id=plyrune-apt,target=/var/cache/apt,sharing=locked \
    apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/

RUN --mount=type=cache,id=plyrune-rustup,target=/opt/rustup/downloads,sharing=locked \
    rustup toolchain install ${RUST_TOOLCHAIN} --profile minimal \
      --component clippy --component rustfmt

RUN --mount=type=cache,id=plyrune-tool-registry,target=/opt/cargo/registry,sharing=locked \
    --mount=type=cache,id=plyrune-tool-git,target=/opt/cargo/git,sharing=locked \
    --mount=type=cache,id=plyrune-tool-target,target=/tmp/tool-target \
    CARGO_TARGET_DIR=/tmp/tool-target cargo +${RUST_TOOLCHAIN} install --locked \
      --git ${TAUTEN_REPOSITORY} --rev ${TAUTEN_REVISION} cargo-tauten

# Charon's full-MIR sysroot, built once here. The tauten template builds it on
# first use as root; the Plyrune gate services run as the checkout's owner
# (docker/entry.sh) and could not write it into the toolchain.
RUN charon print-sysroot > /dev/null

ENV CARGO_TARGET_DIR=/cache/target \
    CARGO_TERM_COLOR=always \
    GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=safe.directory GIT_CONFIG_VALUE_0=/work/plyrune
WORKDIR /work/plyrune
