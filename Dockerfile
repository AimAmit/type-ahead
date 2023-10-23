# Leveraging the pre-built Docker images with
# cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest-rust-1.65.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM rust:1.65-slim AS template-rust
COPY --from=builder /app/target/release/type-ahead /usr/local/bin
EXPOSE 5050
ENTRYPOINT ["/usr/local/bin/type-ahead"]