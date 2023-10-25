FROM rust:1.71-bullseye as builder
RUN USER=root cargo new --bin portal-testground
WORKDIR /portal-testground

# Cache dependencies between test runs,
# See https://blog.mgattozzi.dev/caching-rust-docker-builds/
# And https://github.com/rust-lang/cargo/issues/2644
RUN apt-get update && apt-get install clang -y
COPY ./plan/Cargo.toml ./Cargo.toml
COPY ./plan/src ./src
RUN cargo build --release

FROM ubuntu:23.04
# remove iputils-ping soon
RUN apt-get update && apt-get install curl jq iputils-ping nodejs musl-dev -y && ln -s /usr/lib/x86_64-linux-musl/libc.so /lib/libc.musl-x86_64.so.1
COPY --from=builder /portal-testground/target/release/portal-testground .
COPY --from=builder /portal-testground/src/clients ./src/clients
ENV RUST_LOG=debug
# port for testground
EXPOSE 6060
# Export ports used by portal nodes to allow outside access to the node
EXPOSE 8545 9009/udp 9000/udp
ENTRYPOINT ["./portal-testground"]