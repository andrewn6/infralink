FROM rust:latest as builder

WORKDIR /usr/src/principal

COPY . .

RUN cargo fetch

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /runner

COPY --from=builder /usr/src/principal/target/release/principal principal

EXPOSE 7000

RUN ["cargo", "run"]