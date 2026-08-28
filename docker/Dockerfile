# ProRT-IP WarScan Production Docker Image
# Multi-stage build for minimal image size
#
# Build:
#   docker build -t prtip:latest .
#   docker build --build-arg VERSION=1.0.0 -t prtip:1.0.0 .
#
# Run:
#   docker run --rm --net=host --cap-add=NET_RAW prtip -sS -p 80,443 192.168.1.1
#   docker run -it --rm --net=host --cap-add=NET_RAW prtip --tui

ARG RUST_VERSION=1.85
ARG VERSION=1.0.0

# ==============================================================================
# Stage 1: Build (Debian-based for maximum compatibility)
# ==============================================================================
FROM rust:${RUST_VERSION}-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpcap-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/prtip-core/Cargo.toml crates/prtip-core/
COPY crates/prtip-network/Cargo.toml crates/prtip-network/
COPY crates/prtip-scanner/Cargo.toml crates/prtip-scanner/
COPY crates/prtip-cli/Cargo.toml crates/prtip-cli/
COPY crates/prtip-tui/Cargo.toml crates/prtip-tui/

# Create dummy source files to cache dependencies
RUN mkdir -p crates/prtip-core/src crates/prtip-network/src \
    crates/prtip-scanner/src crates/prtip-cli/src crates/prtip-tui/src \
    && echo "fn main() {}" > crates/prtip-cli/src/main.rs \
    && echo "pub fn dummy() {}" > crates/prtip-core/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/prtip-network/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/prtip-scanner/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/prtip-tui/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release --locked 2>/dev/null || true

# Copy actual source code
# Note: data/ directory is inside crates/prtip-core/data/ and is embedded via include_str!
COPY crates/ crates/

# Touch sources to invalidate cargo cache for actual build
RUN touch crates/*/src/lib.rs crates/prtip-cli/src/main.rs 2>/dev/null || true

# Build release binary
RUN cargo build --release --locked

# Strip binary for smaller size
RUN strip /build/target/release/prtip

# ==============================================================================
# Stage 2: Runtime (Debian slim for libpcap compatibility)
# ==============================================================================
FROM debian:bookworm-slim AS runtime

ARG VERSION
LABEL maintainer="ProRT-IP Contributors <noreply@github.com>"
LABEL description="ProRT-IP WarScan - Modern high-performance network scanner"
LABEL version="${VERSION}"
LABEL org.opencontainers.image.source="https://github.com/doublegate/ProRT-IP"
LABEL org.opencontainers.image.license="GPL-3.0"

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpcap0.8 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd -m -s /bin/bash prtip

# Copy binary from builder
COPY --from=builder /build/target/release/prtip /usr/local/bin/prtip

# Copy documentation
COPY README.md /usr/share/doc/prtip/
COPY LICENSE /usr/share/doc/prtip/
COPY CHANGELOG.md /usr/share/doc/prtip/

# Set capabilities on binary (requires --cap-add=NET_RAW at runtime)
# Note: setcap doesn't persist in Docker images, capabilities must be granted at runtime

# Default user (can be overridden with --user root for raw packet access)
USER prtip
WORKDIR /home/prtip

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD prtip --version || exit 1

# Default entrypoint
ENTRYPOINT ["prtip"]
CMD ["--help"]

# ==============================================================================
# Stage 3: Alpine variant (smaller image, musl libc)
# ==============================================================================
FROM alpine:3.24 AS alpine

ARG VERSION
LABEL maintainer="ProRT-IP Contributors <noreply@github.com>"
LABEL description="ProRT-IP WarScan - Modern high-performance network scanner (Alpine)"
LABEL version="${VERSION}"

# Install runtime dependencies
RUN apk add --no-cache \
    libpcap \
    ca-certificates \
    libgcc

# Create non-root user
RUN adduser -D -s /bin/sh prtip

# Note: For Alpine, you need to build with musl target
# This stage expects a pre-built musl binary
# COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/prtip /usr/local/bin/prtip

# For now, use the glibc binary (requires compatibility layer or rebuild)
COPY --from=builder /build/target/release/prtip /usr/local/bin/prtip

COPY README.md /usr/share/doc/prtip/
COPY LICENSE /usr/share/doc/prtip/

USER prtip
WORKDIR /home/prtip

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD prtip --version || exit 1

ENTRYPOINT ["prtip"]
CMD ["--help"]
