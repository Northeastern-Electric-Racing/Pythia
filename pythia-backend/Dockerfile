# The backend is self-contained; build with this directory as the context:
#   docker build -t pythia-backend pythia-backend/
FROM rust:1-bookworm AS builder

# diesel's sqlite backend links the system libsqlite3 at build time
RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/pythia
COPY . .
RUN cargo install --path . --locked

FROM debian:bookworm-slim

# libsqlite3 is loaded dynamically at runtime
RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/pythia-backend /usr/local/bin/pythia-backend

# DB self-generates from migrations on first start; keep it on a volume
ENV DATABASE_URL=/data/pythia.db
VOLUME ["/data"]
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/pythia-backend"]
