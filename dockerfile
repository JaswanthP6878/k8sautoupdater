# Use Rust slim image for building
FROM rust:slim-bookworm AS builder

# Set work directory
WORKDIR /app

RUN apt update && apt install -y libssl-dev pkg-config

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo fetch
RUN cargo build --release

COPY src/ src/
COPY log4rs.yaml ./
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

USER 1000:1000

COPY --from=builder --chown=1000:1000 /app/target/release/k8sautoupdater /k8sautoupdater

EXPOSE 3000

ENTRYPOINT ["/k8sautoupdater"]
