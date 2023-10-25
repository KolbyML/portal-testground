FROM rust:1.71-bullseye as builder
RUN USER=root cargo new --bin trin-testground
WORKDIR /trin-testground

# Cache dependencies between test runs,
# See https://blog.mgattozzi.dev/caching-rust-docker-builds/
# And https://github.com/rust-lang/cargo/issues/2644
RUN pwd
RUN apt-get update && apt-get install clang -y && apt-get -qy full-upgrade && apt-get install -qy curl && apt-get install -qy curl && curl -sSL https://get.docker.com/ | sh
RUN pwd
COPY ./plan/Cargo.toml ./Cargo.toml
COPY ./plan/src ./src
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /trin-testground/target/release/trin-testground .
COPY --from=builder /trin-testground/src/clients ./src/clients
RUN pwd
RUN ls
RUN ls ./src/clients/
VOLUME /var/run/docker.sock
ENV RUST_LOG=debug
EXPOSE 6060
ENTRYPOINT ["./trin-testground"]