FROM rust:latest as builder

# Install build dependencies
RUN apt-get update && apt-get install -y cmake

WORKDIR /usr/src/app
RUN USER=root cargo new --bin thesis_client
WORKDIR /usr/src/app/thesis_client

# COPY RUST project files into the container
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

# Build the Rust project
RUN cargo build --release

FROM debian:buster-slim

# Set the working directory
WORKDIR /usr/src/app

# COPY only the necessary files from the builder image
COPY --from=builder /usr/src/app/thesis_client/target/release/client .

# Expose the port that the application listens on.
EXPOSE 3000
# Args : message size (default : 100Mb)
ENV message_size = 100000000

CMD ["./client", "${message_size}"]
