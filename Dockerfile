FROM rust:1.85-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/hhh-rs /usr/local/bin/hhh-rs

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

CMD ["hhh-rs"]
