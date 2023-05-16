FROM rust:latest as builder

WORKDIR /usr/src/health-check

COPY . .

RUN cargo fetch

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /runner

EXPOSE 50052

COPY --from=builder /usr/src/health-check/target/release/health-check health-check

RUN ["cargo", "run"]
