FROM rust:1.82-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler libprotobuf-dev pkg-config openssl libssl-dev

COPY rust/ /app/rust
COPY Cargo.toml /app/
COPY Cargo.lock /app/ 2>/dev/null || true

RUN cargo build --release -p clawstack-gateway -p clawstack-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl
COPY --from=builder /app/target/release/clawstack /usr/local/bin/
EXPOSE 8080 50051
ENTRYPOINT ["clawstack"]
CMD ["serve"]
