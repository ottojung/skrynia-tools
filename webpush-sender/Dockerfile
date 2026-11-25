# Disposable qualification image only. Not a deployment or production base.
# Build stage fetches crates over the network; runtime is fully offline.
FROM rust:1.93-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked --bin webpush-sender

FROM debian:bookworm-slim AS runtime
RUN useradd --uid 10001 --create-home --shell /usr/sbin/nologin sender
COPY --from=build /src/target/release/webpush-sender /usr/local/bin/webpush-sender
USER sender
WORKDIR /home/sender
ENTRYPOINT ["/usr/local/bin/webpush-sender"]
