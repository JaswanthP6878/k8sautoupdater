FROM rust:slim-bookworm as builder
WORKDIR /app
# RUN apt update && apt install libsqlite3-dev -y
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# RUN apt update && apt install libsqlite3-0 -y
USER 1000:1000
COPY --from=builder --chown=1000:1000 /app/target/release/k8sautoupdater /k8sautoupdater
# ENV PORT=8080
EXPOSE 3000
ENTRYPOINT ["/k8sautoupdater"]