FROM rust:1.71-bullseye as builder
WORKDIR /usr/src/trin-testground

# Cache dependencies between test runs,
# See https://blog.mgattozzi.dev/caching-rust-docker-builds/
# And https://github.com/rust-lang/cargo/issues/2644

RUN mkdir -p ./plan/src/
RUN echo "fn main() { println!(\"If you see this message, you may want to clean up the target directory or the Docker build cache.\") }" > ./plan/src/main.rs
COPY ./plan/Cargo.toml ./plan/
RUN cd ./plan/ && cargo build

COPY . .

# This is in order to make sure `main.rs`s mtime timestamp is updated to avoid the dummy `main`
# remaining in the release binary.
# https://github.com/rust-lang/cargo/issues/9598
RUN touch ./plan/src/main.rs

RUN cd plan && cargo build --bin=trin-testground && mv /usr/src/trin-testground/plan/target/debug/trin-testground /usr/local/bin/trin-testground

FROM debian:bullseye-slim
COPY --from=builder /usr/local/bin/trin-testground /usr/local/bin/trin-testground
EXPOSE 6060
ENTRYPOINT ["trin-testground"]