# syntax=docker/dockerfile:1.7

FROM rust:1.82-slim-bookworm AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    sqlite3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

# sqlx::query! macros verify SQL against a real database at compile time.
# Build a throwaway SQLite with the migrations applied so the macros pass.
RUN mkdir -p /tmp/db && \
    for f in migrations/*.sql; do sqlite3 /tmp/db/build.db < "$f"; done && \
    DATABASE_URL=sqlite:/tmp/db/build.db cargo build --release && \
    rm -rf /tmp/db

# ---

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 1000 -m -d /home/app app

WORKDIR /data
COPY --from=build /app/target/release/centralenv-server /usr/local/bin/centralenv-server

ENV BIND_ADDR=0.0.0.0:3001 \
    DATABASE_URL=sqlite:///data/centralenv.db \
    RUST_LOG=centralenv_server=info,tower_http=info

USER app
EXPOSE 3001

CMD ["centralenv-server"]
