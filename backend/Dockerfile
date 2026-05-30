# syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS build

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

ENV SQLX_OFFLINE=true

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --locked --release --bin backend && \
    cp target/release/backend /bin/server


FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates libssl3 ffmpeg && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ARG UID=10001
RUN useradd --system --uid ${UID} --no-create-home --shell /usr/sbin/nologin appuser

COPY --from=build /bin/server /usr/local/bin/backend
COPY --chown=appuser:appuser migrations /app/migrations

USER appuser
WORKDIR /app

ENV RUST_LOG=info

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/backend"]
