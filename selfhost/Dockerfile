# syntax=docker/dockerfile:1
#
# TeleStore headless API server image (Track B).
#
# Builds ONLY the `td-headless` binary. It still compiles the app_lib crate,
# which embeds the built frontend via Tauri's generate_context! macro, so the
# builder stage also builds the Vite frontend to produce app/dist.
#
# Build from the repository root:
#   docker build -f selfhost/Dockerfile -t telestore-headless .

# ─────────────────────────── Builder ───────────────────────────
FROM rust:1-bookworm AS builder

# Tauri/webkit build dependencies (app_lib links against them) + Node for the
# frontend build that generate_context! needs to embed.
RUN apt-get update && apt-get install -y --no-install-recommends \
        libwebkit2gtk-4.1-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libssl-dev \
        build-essential \
        curl \
        file \
        pkg-config \
        ca-certificates \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY app/ ./app/

# Build the frontend (produces app/dist, embedded by generate_context!).
WORKDIR /build/app
RUN npm ci || npm install
RUN npm run build

# Build the headless binary only.
WORKDIR /build/app/src-tauri
RUN cargo build --release --features headless --bin td-headless

# ─────────────────────────── Runtime ───────────────────────────
FROM debian:bookworm-slim AS runtime

# The binary links the Tauri/webkit stack even though it never opens a window,
# so the runtime libs must be present.
RUN apt-get update && apt-get install -y --no-install-recommends \
        libwebkit2gtk-4.1-0 \
        libgtk-3-0 \
        libayatana-appindicator3-1 \
        librsvg2-2 \
        libssl3 \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 telestore

COPY --from=builder /build/app/src-tauri/target/release/td-headless /usr/local/bin/td-headless

USER telestore
ENV TD_DATA_DIR=/data \
    TD_CACHE_DIR=/data/cache \
    TD_BIND=0.0.0.0 \
    TD_PORT=8550

VOLUME ["/data"]
EXPOSE 8550

ENTRYPOINT ["/usr/local/bin/td-headless"]
