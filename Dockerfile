FROM rust:1.76 as build
WORKDIR /rust-reverse-shell-system
COPY . .
RUN cargo build --release --bin server

FROM debian:bookworm-slim
COPY --from=build /rust-reverse-shell-system/target/release/server .
EXPOSE 9000
CMD ["./server"]
