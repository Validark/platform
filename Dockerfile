# syntax = docker/dockerfile:1.5

# Docker image for rs-drive-abci
#
# This image is divided multiple parts:
# - deps - includes all dependencies and some libraries
# - sources - includes full source code
# - build-* - actual build process of given image
# - drive-abci, dashmate, testsuite, dapi - final images
#
# The following build arguments can be provided using --build-arg:
# - CARGO_BUILD_PROFILE - set to `release` to build final binary, without debugging information
# - NODE_ENV - node.js environment name to use to build the library
# - SCCACHE_GHA_ENABLED, ACTIONS_CACHE_URL, ACTIONS_RUNTIME_TOKEN - store sccache caches inside github actions
# - SCCACHE_MEMCACHED - set to memcache server URI (eg. tcp://172.17.0.1:11211) to enable sccache memcached backend
# - ALPINE_VERSION - use different version of Alpine base image; requires also rust:apline...
#   image to be available
# - USERNAME, USER_UID, USER_GID - specification of user used to run the binary
#
ARG ALPINE_VERSION=3.16

#
# DEPS: INSTALL AND CACHE DEPENDENCIES
#
FROM rust:alpine${ALPINE_VERSION} as deps

#
# Install some dependencies
#
RUN apk add --no-cache \
        alpine-sdk \
        bash \
        binutils \
        ca-certificates \
        clang-static clang-dev \
        cmake \
        git \
        libc-dev \
        linux-headers \
        llvm-static llvm-dev  \
        nodejs \
        npm \
        openssl-dev \
        perl \
        python3 \
        unzip \
        wget \
        xz \
        zeromq-dev

SHELL ["/bin/bash", "-xc"]

ARG TARGETARCH

RUN rustup install stable && \
    rustup target add wasm32-unknown-unknown --toolchain stable

# Install protoc - protobuf compiler
# The one shipped with Alpine does not work
RUN if [[ "$TARGETARCH" == "arm64" ]] ; then export PROTOC_ARCH=aarch_64; else export PROTOC_ARCH=x86_64; fi; \
    curl -Ls https://github.com/protocolbuffers/protobuf/releases/download/v22.4/protoc-22.4-linux-${PROTOC_ARCH}.zip \
        -o /tmp/protoc.zip && \
    unzip -qd /opt/protoc /tmp/protoc.zip && \
    rm /tmp/protoc.zip && \
    ln -s /opt/protoc/bin/protoc /usr/bin/

# Install sccache for caching
RUN if [[ "$TARGETARCH" == "arm64" ]] ; then export SCC_ARCH=aarch64; else export SCC_ARCH=x86_64; fi; \
    curl -Ls \
        https://github.com/mozilla/sccache/releases/download/v0.4.1/sccache-v0.4.1-${SCC_ARCH}-unknown-linux-musl.tar.gz | \
        tar -C /tmp -xz && \
        mv /tmp/sccache-*/sccache /usr/bin/

# Install emcmake, dependency of bls-signatures -> bls-dash-sys
# TODO: Build ARM image to check if this is still needed
# RUN curl -Ls \
#         https://github.com/emscripten-core/emsdk/archive/refs/tags/3.1.36.tar.gz | \
#         tar -C /opt -xz && \
#         ln -s /opt/emsdk-* /opt/emsdk && \
#         /opt/emsdk/emsdk install latest && \
#         /opt/emsdk/emsdk activate latest

# Configure Node.js
RUN npm install -g npm@9.6.6 && \
    npm install -g corepack@latest && \
    corepack prepare yarn@3.3.0 --activate && \
    corepack enable

# Switch to clang
RUN rm /usr/bin/cc && ln -s /usr/bin/clang /usr/bin/cc

#
# Configure sccache
#
# Activate sccache for Rust code
ENV RUSTC_WRAPPER=/usr/bin/sccache
# Set args below to use Github Actions cache; see https://github.com/mozilla/sccache/blob/main/docs/GHA.md
ARG SCCACHE_GHA_ENABLED
ARG ACTIONS_CACHE_URL
ARG ACTIONS_RUNTIME_TOKEN
# Alternative solution is to use memcache
ARG SCCACHE_MEMCACHED

# Disable incremental buildings, not supported by sccache
ARG CARGO_INCREMENTAL=false

# Select whether we want dev or release
ARG CARGO_BUILD_PROFILE=dev
ENV CARGO_BUILD_PROFILE ${CARGO_BUILD_PROFILE}

ARG NODE_ENV=production
ENV NODE_ENV ${NODE_ENV}

# Install wasm-bindgen-cli in the same profile as other components, to sacrifice some performance & disk space to gain
# better build caching
WORKDIR /platform
# TODO: refactor to not need the wasm-bindgen-cli and remove the copy below, as deps stage should be independent
COPY Cargo.lock .
RUN --mount=type=cache,sharing=shared,target=/usr/local/cargo/registry/index \
    --mount=type=cache,sharing=shared,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,sharing=shared,target=/usr/local/cargo/git/db \
    --mount=type=cache,sharing=shared,target=/platform/target \
    export CARGO_TARGET_DIR=/platform/target ; \
    ls -lha /usr/local/cargo/registry/index && \
    ls -lha /usr/local/cargo/registry/cache && \
    ls -lha /usr/local/cargo/git/db && \
    ls -lha /platform/target && \
    cargo install cargo-lock --features=cli --profile "$CARGO_BUILD_PROFILE" ; \
    WASM_BINDGEN_VERSION=$(cargo-lock list -p wasm-bindgen | egrep -o '[0-9.]+') ; \
    cargo install --profile "$CARGO_BUILD_PROFILE" "wasm-bindgen-cli@${WASM_BINDGEN_VERSION}"

RUN --mount=type=cache,sharing=shared,target=/usr/local/cargo/registry/index \
    --mount=type=cache,sharing=shared,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,sharing=shared,target=/usr/local/cargo/git/db \
    --mount=type=cache,sharing=shared,target=/platform/target \
    ls -lha /usr/local/cargo/registry/index && \
    ls -lha /usr/local/cargo/registry/cache && \
    ls -lha /usr/local/cargo/git/db && \
    ls -lha /platform/target

#
# EXECUTE BUILD
#
FROM deps as sources

# We run builds with extensive caching.
#
# Note:
# 1. All these --mount... are to cache reusable info between runs.
# See https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci
# 2. We add `--config net.git-fetch-with-cli=true` to address ARM build issue,
# see https://github.com/rust-lang/cargo/issues/10781#issuecomment-1441071052
# 3. Github Actions have shared networking configured, so we need to set a random
# SCCACHE_SERVER_PORT port to avoid conflicts in case of parallel compilation
# 4. We also set RUSTC to include exact toolchain name in compilation command, and
# include this in cache key

WORKDIR /platform

COPY . .

RUN yarn config set enableInlineBuilds true

#
# STAGE: BUILD RS-DRIVE-ABCI
#
# This will prebuild majority of dependencies
FROM sources AS build-drive-abci

RUN mkdir /artifacts

RUN --mount=type=cache,sharing=shared,target=/root/.cache/sccache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    --mount=type=cache,sharing=shared,target=/platform/target \
    export SCCACHE_SERVER_PORT=$((RANDOM+1025)) && \
    if [[ -z "${SCCACHE_MEMCACHED}" ]] ; then unset SCCACHE_MEMCACHED ; fi ; \
    cargo build \
        --profile "$CARGO_BUILD_PROFILE" \
        -p drive-abci \
       --config net.git-fetch-with-cli=true && \
    cp /platform/target/*/drive-abci /artifacts/drive-abci && \
    sccache --show-stats

#
# STAGE: BUILD JAVASCRIPT INTERMEDIATE IMAGE
#
FROM sources AS build-js

RUN mkdir /artifacts

RUN --mount=type=cache,sharing=shared,target=/root/.cache/sccache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    --mount=type=cache,sharing=shared,id=wasm_dpp_target,target=/platform/target \
    --mount=type=cache,target=/tmp/unplugged \
    cp -R /tmp/unplugged /platform/.yarn/ && \
    export SCCACHE_SERVER_PORT=$((RANDOM+1025)) && \
    if [[ -z "${SCCACHE_MEMCACHED}" ]] ; then unset SCCACHE_MEMCACHED ; fi ; \
    export SKIP_GRPC_PROTO_BUILD=1 && \
    yarn install && \
    yarn build && \
    cp -R /platform/.yarn/unplugged /tmp/ && \
    sccache --show-stats

#
# STAGE: FINAL DRIVE-ABCI IMAGE
#
FROM alpine:${ALPINE_VERSION} AS drive-abci

LABEL maintainer="Dash Developers <dev@dash.org>"
LABEL description="Drive ABCI Rust"

WORKDIR /var/lib/dash

RUN apk add --no-cache libgcc libstdc++

COPY --from=build-drive-abci /artifacts/drive-abci /usr/bin/drive-abci
COPY --from=build-drive-abci /platform/packages/rs-drive-abci/.env.example /var/lib/dash/rs-drive-abci/.env

# Double-check that we don't have missing deps
RUN ldd /usr/bin/drive-abci

# Create a volume
VOLUME /var/lib/dash

ENV DB_PATH=/var/lib/dash/rs-drive-abci/db

#
# Create new non-root user
#
ARG USERNAME=dash
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN addgroup -g $USER_GID $USERNAME && \
    adduser -D -u $USER_UID -G $USERNAME -h /var/lib/dash/rs-drive-abci $USERNAME && \
    chown -R $USER_UID:$USER_GID /var/lib/dash/rs-drive-abci

USER $USERNAME

ENV RUST_BACKTRACE=1
WORKDIR /var/lib/dash/rs-drive-abci
ENTRYPOINT ["/usr/bin/drive-abci"]
CMD ["-vvvv", "start"]

EXPOSE 26658

#
# STAGE: DASHMATE BUILD
#
FROM build-js AS build-dashmate

# Install Test Suite specific dependencies using previous
# node_modules directory to reuse built binaries
RUN --mount=type=cache,sharing=shared,target=/root/.cache/sccache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    --mount=type=cache,sharing=shared,target=/platform/target \
    --mount=type=cache,target=/tmp/unplugged \
    cp -R /tmp/unplugged /platform/.yarn/ && \
    yarn workspaces focus --production dashmate && \
    cp -R /platform/.yarn/unplugged /tmp/

RUN rm -fr ./packages/rs-*
# TODO: Clean all other files not needed by dapi

#
#  STAGE: FINAL DASHMATE IMAGE
#
FROM node:16-alpine${ALPINE_VERSION} AS dashmate

RUN apk add --no-cache docker-cli docker-cli-compose curl

LABEL maintainer="Dash Developers <dev@dash.org>"
LABEL description="Dashmate Helper Node.JS"

WORKDIR /platform

COPY --from=build-dashmate /platform /platform

USER node
ENTRYPOINT ["/platform/packages/dashmate/docker/entrypoint.sh"]


#
# STAGE: TEST SUITE BUILD
#
FROM build-js AS build-testsuite

# Install Test Suite specific dependencies using previous
# node_modules directory to reuse built binaries
RUN --mount=type=cache,sharing=shared,target=/root/.cache/sccache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    --mount=type=cache,sharing=shared,id=wasm_dpp_target,target=/platform/target \
    --mount=type=cache,target=/tmp/unplugged \
    cp -R /tmp/unplugged /platform/.yarn/ && \
    yarn workspaces focus --production @dashevo/platform-test-suite && \
    cp -R /platform/.yarn/unplugged /tmp/

# Remove Rust sources
RUN rm -fr ./packages/rs-*
# TODO: Clean all other files not needed by dapi


#
#  STAGE: FINAL TEST SUITE IMAGE
#
FROM node:16-alpine${ALPINE_VERSION} AS testsuite

RUN apk add --no-cache bash

LABEL maintainer="Dash Developers <dev@dash.org>"
LABEL description="Dash Platform test suite"

WORKDIR /platform

COPY --from=build-testsuite /platform /platform

RUN cp /platform/packages/platform-test-suite/.env.example /platform/packages/platform-test-suite/.env

EXPOSE 2500 2501 2510
USER node
ENTRYPOINT ["/platform/packages/platform-test-suite/bin/test.sh"]

#
# STAGE: DAPI BUILD
#
FROM build-js AS build-dapi

# Install Test Suite specific dependencies using previous
# node_modules directory to reuse built binaries
RUN --mount=type=cache,sharing=shared,target=/root/.cache/sccache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    --mount=type=cache,sharing=shared,id=wasm_dpp_target,target=/platform/target \
    --mount=type=cache,target=/tmp/unplugged \
    cp -R /tmp/unplugged /platform/.yarn/ && \
    yarn workspaces focus --production @dashevo/dapi && \
    cp -R /platform/.yarn/unplugged /tmp/

# Remove Rust sources
RUN rm -fr ./packages/rs-*
# TODO: Clean all other files not needed by dapi

#
# STAGE: FINAL DAPI IMAGE
#
FROM node:16-alpine3.16 AS dapi

LABEL maintainer="Dash Developers <dev@dash.org>"
LABEL description="DAPI Node.JS"

# Install ZMQ shared library
RUN apk add --no-cache zeromq-dev

WORKDIR /platform

COPY --from=build-dapi /platform /platform

RUN cp /platform/packages/dapi/.env.example /platform/packages/dapi/.env

EXPOSE 2500 2501 2510
USER node