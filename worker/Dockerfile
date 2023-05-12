FROM rust:latest as builder

WORKDIR /usr/src/worker

COPY . .

RUN cargo fetch

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /runner

COPY --from=builder /usr/src/worker/target/release/worker worker

EXPOSE 50051

RUN ["cargo", "run"]