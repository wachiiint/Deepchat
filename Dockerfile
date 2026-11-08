# Deepchat server only. Clients are desktop apps and connect over raw TCP to port 7878.

# Stage 1: compile (the rust image already has OpenSSL headers and pkg-config)
FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY protocol protocol
COPY server server
# The Tauri client is not copied, so drop it from the workspace
RUN sed -i 's|, "client/src-tauri"||' Cargo.toml
RUN cargo build --release -p server

# Stage 2: run (only the binary, plus OpenSSL and CA certs for HTTPS calls to the AI API)
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/server /usr/local/bin/server
EXPOSE 7878
CMD ["server"]
