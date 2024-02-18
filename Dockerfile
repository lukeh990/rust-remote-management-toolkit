FROM rust:1.76 as build

# create a new empty shell project
RUN USER=root cargo new --lib rust-reverse-shell-system
WORKDIR /rust-reverse-shell-system

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/rrss_lib*
RUN cargo build --release

# our final base
FROM debian:bookworm-slim

RUN apt-get update

WORKDIR /rust-reverse-shell-system

# copy the build artifact from the build stage
COPY --from=build /rust-reverse-shell-system/target/release/server .

# set the startup command to run your binary
CMD ["./server"]
