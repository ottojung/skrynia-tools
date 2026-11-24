FROM docker.io/library/rust:1.93-bookworm AS builder
WORKDIR /usr/src/build
COPY publish-provider/Cargo.toml publish-provider/Cargo.lock ./
COPY publish-provider/src ./src
RUN cargo build --locked --release --bin skrynia-publish-provider

FROM docker.io/library/debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/build/target/release/skrynia-publish-provider /usr/local/bin/skrynia-publish-provider
USER 65532:65532
ENV PUBLICATION_PROVIDER_BIND=127.0.0.1:8787
ENV PATH_TO_PUBLICATION_REQUESTS=/srv/publication-requests
ENV PATH_TO_SKRYNIA_DISK=/srv/skrynia-disk
ENV PATH_TO_NGINX_SKRYNIA_ROOT=/srv/nginx-skrynia
EXPOSE 8787
ENTRYPOINT ["/usr/local/bin/skrynia-publish-provider"]
