FROM rust:latest as builder

WORKDIR /usr/src/health-check

COPY . .

COPY ../../models /usr/src/health-check/models

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /runner

EXPOSE 50052

COPY --from=builder /usr/src/health-check/target/release/health-check health-check

RUN ["cargo", "run"]