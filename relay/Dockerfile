# syntax=docker/dockerfile:1
# Nomi Relay's server (relay-server). Build context: relay/.

FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
RUN cargo build --release --bin relay-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/relay-server /usr/local/bin/relay-server

# Configuration comes from the environment, never the image: RELAY_DATABASE_URL,
# RELAY_SHARED_DEV_URL, RELAY_SHARED_PRODUCTION_URL, RELAY_CONTROL_KEY, RELAY_ENCRYPTION_KEY,
# RELAY_PUBLIC_DOMAIN and optionally RELAY_BIND (see crates/relay-core/src/config.rs).
EXPOSE 8080
CMD ["relay-server"]
