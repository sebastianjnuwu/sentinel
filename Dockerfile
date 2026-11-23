FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock README.md ./
COPY src ./src
RUN cargo build --release --locked

FROM scratch
LABEL org.opencontainers.image.source="https://github.com/sebastianjnuwu/sentinel"
LABEL org.opencontainers.image.description="Fast, cross-platform Rust CLI for multi-ecosystem dependency, license, and OSV vulnerability scanning."
LABEL org.opencontainers.image.licenses="Apache-2.0"
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /src/target/release/sentinel /sentinel
WORKDIR /workspace
ENTRYPOINT ["/sentinel"]
