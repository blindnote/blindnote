# syntax=docker/dockerfile:1.7

# BlindNote is one self-contained binary: the Rust server embeds the built
# client (crates/server/build.rs reads ../../client/dist). So the image is a
# two-step build -- client first, then the binary that bakes it in -- landing
# in a distroless runtime that carries nothing but the binary and glibc.
#
# Reproducibility (CLAUDE.md section 3, section 9) is the point of this project,
# so every base image is pinned by digest and nothing here reaches the network
# except the package managers during build. The exact Rust is also fixed by
# rust-toolchain.toml; SOURCE_DATE_EPOCH is set by the release workflow.

# ---- 1. client: Vite build -> /client/dist ----------------------------------
FROM node:22-bookworm-slim@sha256:4b150b48b4116e2a22f2c39cc27025fbf4f153e2078fd3b14d159fdb5c7279da AS client
WORKDIR /client
# Install deps from the lockfile alone first, so this layer caches across
# source-only changes.
COPY client/package.json client/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY client/ ./
RUN npm run build

# ---- 2. server: cargo build, embedding the client ---------------------------
FROM rust:1.99.1-bookworm@sha256:9181b56c2a21ab1d5e73cffe04fc762426d30aa7fac3c91141dd033f49a7fc8b AS build
WORKDIR /src
# The toolchain file pins the compiler; copy it first so a mismatch fails early.
COPY rust-toolchain.toml ./
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
# build.rs resolves the client at ../../client/dist from crates/server.
COPY --from=client /client/dist/ client/dist/
# The release profile in Cargo.toml already sets one codegen unit and fat LTO.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p blindnote-server --bin blindnote \
    && cp /src/target/release/blindnote /usr/local/bin/blindnote

# ---- 3. runtime: distroless, non-root ---------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
LABEL org.opencontainers.image.source="https://github.com/blindnote/blindnote" \
      org.opencontainers.image.description="One-time-read, end-to-end encrypted, self-destructing notes." \
      org.opencontainers.image.licenses="PolyForm-Shield-1.0.0"
COPY --from=build /usr/local/bin/blindnote /usr/local/bin/blindnote
# nonroot (uid 65532). Data is a mounted volume owned by this uid on the host.
USER nonroot:nonroot
ENV BLINDNOTE_BIND=0.0.0.0:8080 \
    BLINDNOTE_DATA_DIR=/data
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/blindnote"]
