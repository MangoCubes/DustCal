FROM rust:1.94-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates openssl

WORKDIR /app
COPY --from=builder /app/target/release/dustcal /app/dustcal

EXPOSE 3000

ENTRYPOINT ["/app/dustcal"]
