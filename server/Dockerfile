FROM rust:latest as builder

RUN apt-get update && apt-get install -y cmake

WORKDIR /app
RUN USER=root cargo new --bin server
WORKDIR /app/server

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

# COPY only the necessary files from the builder image
COPY --from=builder /app/server/target/release/server .
COPY ./start.sh .
COPY ./cert.crt ./cert.crt
COPY ./cert.key ./cert.key

RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]
