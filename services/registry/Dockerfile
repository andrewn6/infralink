FROM rust:latest as builder

WORKDIR /usr/src/registry 

COPY . .

RUN cargo build --release

FROM debian:bull-eye-slim 

WORKDIR /runner

COPY --from=builder /usr/src/registry/target/release/registry registry

EXPOSE 8083

RUN ["cargo", "run"]