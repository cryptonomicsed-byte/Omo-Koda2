# Ọmọ Kọ́dà — Èṣù kernel (Rust)
# Multi-stage: build the workspace, then ship a slim runtime image.
# The `omokoda` binary (omokoda-cli) exposes `serve` on the chosen port.

FROM rust:1-slim-bookworm AS builder
WORKDIR /build

# System deps: git (workspace pulls the ifascript git dependency) + TLS + build tools.
RUN apt-get update && apt-get install -y --no-install-recommends \
        git pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the whole workspace and build the CLI (which bundles omokoda-core::server).
COPY . .
RUN cargo build --release --package omokoda-cli

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 esu

COPY --from=builder /build/target/release/omokoda /usr/local/bin/omokoda

# Sovereign vault lives on a mounted volume so souls survive restarts.
ENV VAULT_BASE=/app/vault
RUN mkdir -p /app/vault && chown -R esu:esu /app
USER esu

EXPOSE 7777
# 0.0.0.0:7777 — the /v1/* kernel API (Axiom dashboard wires to this).
ENTRYPOINT ["omokoda"]
CMD ["serve", "--port", "7777"]
