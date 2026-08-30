# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --bin nomi-orchestrator

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/nomi-orchestrator /usr/local/bin/nomi-orchestrator

# Runtime configuration (DATABASE_URL, JWT_SECRET, SETTINGS_ENCRYPTION_KEY,
# MQTT_BROKER_HOST, MQTT_BROKER_PORT, RUN_WORKER_INLINE) is supplied by the
# deployment environment, never baked into the image — see
# crates/nomi-server/src/main.rs for the full list this binary reads.
EXPOSE 8080
CMD ["nomi-orchestrator"]
