FROM rust:1.88-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release --target x86_64-unknown-linux-musl && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.21

RUN apk add --no-cache ca-certificates curl

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/hhh-rs /usr/local/bin/hhh-rs

RUN mkdir -p /data

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

HEALTHCHECK --interval=5s --timeout=3s --start-period=10s --retries=5 \
  CMD curl -f http://localhost:3000/ready || exit 1

CMD ["hhh-rs"]
