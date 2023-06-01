FROM rust:latest

WORKDIR /usr/src/registry

COPY Cargo.toml ./

RUN cargo build --release

COPY src ./src

EXPOSE 8083

CMD ["cargo", "run", "release"]
