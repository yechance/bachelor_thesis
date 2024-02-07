FROM rust:latest as builder

# Install build dependencies
RUN apt-get update && apt-get install -y cmake

WORKDIR /app
RUN USER=root cargo new --bin thesis_server
WORKDIR /app/thesis_server

# COPY RUST project files into the container
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

# Build the Rust project
RUN cargo build --release

FROM debian:buster-slim

# Set the working directory
WORKDIR /app

# COPY only the necessary files from the builder image
COPY --from=builder /app/thesis_server/target/release/server .

CMD ["./server"]
