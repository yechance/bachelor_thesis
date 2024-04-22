FROM rust:latest as builder

RUN apt-get update && apt-get install -y cmake

WORKDIR /app
RUN USER=root cargo new --bin client
WORKDIR /app/client

# COPY RUST project files into the container
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

# Build the Rust project
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y iproute2 && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app
RUN mkdir /app/data

# COPY only the necessary files from the builder image
COPY --from=builder /app/client/target/release/client .
COPY ./start.sh .

RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]
