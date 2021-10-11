# syntax=docker/dockerfile:1

FROM rust:latest AS base

ARG DEBIAN_FRONTEND=noninteractive

RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y \
        libclang1 \
        sudo

FROM base AS build

# we depend upon:
# * >= 1.32 for uniform module paths
# * >= 1.36 for std::mem::MaybeUninit
# * >= 1.38 for std::ptr::cast
# * >= 1.52 for warn(rustdoc:all)
ARG TOOLCHAIN
ENV TOOLCHAIN=${TOOLCHAIN:-1.52}

RUN --mount=type=cache,target=/tmp/cache/cargo                  \
    --mount=type=cache,target=/tmp/cache/target,sharing=private \
    rustup self update                  && \
    rustup toolchain install $TOOLCHAIN && \
    rustup default           $TOOLCHAIN && \
    rustup component add clippy

ENV CARGO_HOME=/tmp/cache/cargo
ENV CARGO_TARGET_DIR=/tmp/cache/target

WORKDIR /srv/rust

FROM build AS sudo_pair-deps

RUN cargo new --lib sudo_plugin-sys
RUN cargo new --lib sudo_plugin
RUN cargo new --lib sudo_pair
RUN cargo new --lib examples/raw_plugin_api

COPY Cargo.toml                         .
COPY sudo_plugin-sys/Cargo.toml         ./sudo_plugin-sys
COPY sudo_plugin-sys/build.rs           ./sudo_plugin-sys
COPY sudo_plugin-sys/src/bindings       ./sudo_plugin-sys/src/bindings
COPY sudo_plugin/Cargo.toml             ./sudo_plugin
COPY sudo_pair/Cargo.toml               ./sudo_pair
COPY examples/raw_plugin_api/Cargo.toml ./examples/raw_plugin_api

RUN --mount=type=cache,target=${CARGO_HOME}       \
    --mount=type=cache,target=${CARGO_TARGET_DIR} \
    cargo build

FROM sudo_pair-deps AS sudo_pair

ARG CARGOFLAGS
ARG RUSTFLAGS="-A warnings -A unknown_lints --verbose"
ARG RUSTDOCFLAGS

# replace the dummy crates with the full project
COPY . .

FROM sudo_pair AS sudo_pair-build

RUN --mount=type=cache,target=${CARGO_HOME}       \
    --mount=type=cache,target=${CARGO_TARGET_DIR} \
    cargo build ${CARGOFLAGS}

FROM sudo_pair-build AS sudo_pair-test

RUN --mount=type=cache,target=${CARGO_HOME}       \
    --mount=type=cache,target=${CARGO_TARGET_DIR} \
    cargo test ${CARGOFLAGS}

FROM sudo_pair-build AS sudo_pair-lint

RUN --mount=type=cache,target=${CARGO_HOME}       \
    --mount=type=cache,target=${CARGO_TARGET_DIR} \
    cargo clippy ${CARGOFLAGS}

FROM sudo_pair AS sudo_pair-sample

RUN --mount=type=cache,target=/var/cache/apt \
    apt-get install -y  \
        busybox-syslogd \
        socat           \
        vim

# copy the cached cargo build stages into the container
RUN --mount=type=cache,target=${CARGO_HOME}       \
    --mount=type=cache,target=${CARGO_TARGET_DIR} \
    cp -a ${CARGO_HOME}       ~/.cargo && \
    cp -a ${CARGO_TARGET_DIR} .

ENV CARGO_HOME=~/.cargo
ENV CARGO_TARGET_DIR=/srv/rust/target

RUN make -C sample
RUN make -C sample prefix= exec_prefix=/usr install
